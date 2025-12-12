//! LLM provider abstraction layer.
//!
//! This module defines a common interface for different LLM providers (OpenAI, Anthropic, llamafile, etc.)
//! enabling runtime provider switching and multi-provider support.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod anthropic;
pub mod llamafile;
pub mod openai;

use crate::streaming::{StreamChunk, StreamResult};
use futures::Stream;
use std::pin::Pin;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("request failed: {0}")]
    RequestError(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("authentication failed: {0}")]
    AuthError(String),
    #[error("rate limit exceeded: {0}")]
    RateLimitError(String),
    #[error("provider not supported: {0}")]
    UnsupportedError(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

/// Common request format for LLM completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            max_tokens: Some(256),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop_sequences: None,
            stream: false,
        }
    }
}

/// Common response format for LLM completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub stop_reason: StopReason,
    pub tokens_used: Option<usize>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Complete,
    Length,
    Stop,
    Error,
}

/// Embedding request for generating vector representations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
    pub model: Option<String>,
}

/// Embedding response with vector representations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub dimensions: usize,
}

/// Common trait for all LLM providers.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Get the provider name (e.g., "openai", "anthropic", "llamafile").
    fn name(&self) -> &str;

    /// Check if the provider is available and responsive.
    async fn health_check(&self) -> ProviderResult<bool>;

    /// Generate a completion (non-streaming).
    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse>;

    /// Generate a streaming completion (if supported).
    async fn complete_stream(
        &self,
        _request: CompletionRequest,
    ) -> ProviderResult<Pin<Box<dyn Stream<Item = StreamResult<StreamChunk>> + Send>>> {
        Err(ProviderError::UnsupportedError(format!(
            "{} does not support streaming",
            self.name()
        )))
    }

    /// Generate embeddings for the given texts.
    /// Returns error with UnsupportedError if provider doesn't support embeddings.
    async fn embed(&self, _request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        Err(ProviderError::UnsupportedError(format!(
            "{} does not support embeddings",
            self.name()
        )))
    }

    /// Check if provider supports streaming responses.
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if provider supports embeddings.
    fn supports_embeddings(&self) -> bool {
        false
    }

    /// Get the default model name for this provider.
    fn default_model(&self) -> &str;
}

/// Provider configuration from config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub timeout_secs: Option<u64>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: "llamafile".to_string(),
            api_key: None,
            endpoint: Some("http://localhost:8000".to_string()),
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(256),
            timeout_secs: Some(30),
        }
    }
}

/// Factory function to create a provider from config.
pub async fn create_provider(config: ProviderConfig) -> ProviderResult<Box<dyn LLMProvider>> {
    match config.provider.as_str() {
        "llamafile" => {
            let endpoint = config
                .endpoint
                .unwrap_or_else(|| "http://localhost:8000".to_string());
            Ok(Box::new(llamafile::LlamafileProvider::new(endpoint)))
        }
        "openai" => {
            let api_key = config.api_key.ok_or_else(|| {
                ProviderError::AuthError("OpenAI requires api_key in config".to_string())
            })?;
            let mut provider = openai::OpenAIProvider::new(api_key, config.model)?;
            if let Some(endpoint) = config.endpoint {
                provider = provider.with_base_url(endpoint);
            }
            Ok(Box::new(provider))
        }
        "anthropic" => {
            let api_key = config.api_key.ok_or_else(|| {
                ProviderError::AuthError("Anthropic requires api_key in config".to_string())
            })?;
            let mut provider = anthropic::AnthropicProvider::new(api_key);
            if let Some(endpoint) = config.endpoint {
                provider = provider.with_base_url(endpoint);
            }
            if let Some(model) = config.model {
                provider = provider.with_model(model);
            }
            Ok(Box::new(provider))
        }
        _ => Err(ProviderError::UnsupportedError(format!(
            "Unknown provider: {}",
            config.provider
        ))),
    }
}
