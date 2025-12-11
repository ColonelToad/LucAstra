//! LLM service for LucAstra.
//!
//! This module provides integration with various LLM providers (OpenAI, Anthropic, llamafile, etc.)
//! with async/await support, streaming responses, and embeddings generation.

pub mod client;
pub mod conversation;
pub mod inference;
pub mod providers;

pub use client::LlamafileClient;
pub use conversation::{Conversation, ConversationError, Message, Role};
pub use inference::{InferenceRequest, InferenceResponse, LLMService};
pub use providers::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, LLMProvider,
    ProviderConfig, ProviderError, ProviderResult, StopReason,
};

use lucastra_core::Result;

/// Initialize the LLM service.
pub fn init(endpoint: Option<String>) -> Result<LLMService> {
    let url = endpoint.unwrap_or_else(|| "http://localhost:8000".to_string());
    tracing::info!("Initializing LLM service at {}", url);
    Ok(LLMService::new(url))
}
