//! Llamafile provider implementation.

use super::{
    CompletionRequest, CompletionResponse, LLMProvider,
    ProviderError, ProviderResult, StopReason,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize)]
struct LlamafileCompletionRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlamafileCompletionResponse {
    content: String,
    #[serde(default)]
    stop: bool,
}

/// Llamafile provider for local LLM inference.
pub struct LlamafileProvider {
    endpoint: String,
    client: Client,
}

impl LlamafileProvider {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[async_trait]
impl LLMProvider for LlamafileProvider {
    fn name(&self) -> &str {
        "llamafile"
    }

    fn default_model(&self) -> &str {
        "llamafile-7b"
    }

    async fn health_check(&self) -> ProviderResult<bool> {
        let url = format!("{}/health", self.endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let llamafile_req = LlamafileCompletionRequest {
            prompt: request.prompt.clone(),
            n_predict: request.max_tokens.map(|t| t as i32),
            temperature: request.temperature,
            top_p: request.top_p,
        };

        let url = format!("{}/v1/completions", self.endpoint);
        debug!("Sending completion request to {}", url);

        let resp = self
            .client
            .post(&url)
            .json(&llamafile_req)
            .send()
            .await
            .map_err(|e| ProviderError::RequestError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ProviderError::RequestError(format!(
                "Server returned status {}",
                resp.status()
            )));
        }

        let llamafile_resp: LlamafileCompletionResponse = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        Ok(CompletionResponse {
            content: llamafile_resp.content,
            stop_reason: if llamafile_resp.stop {
                StopReason::Stop
            } else {
                StopReason::Complete
            },
            tokens_used: None,
            model: Some(self.default_model().to_string()),
        })
    }

    fn supports_streaming(&self) -> bool {
        false // Will add SSE support in later iteration
    }

    fn supports_embeddings(&self) -> bool {
        false // Llamafile doesn't expose embeddings endpoint by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = LlamafileProvider::new("http://localhost:8000".to_string());
        assert_eq!(provider.name(), "llamafile");
        assert!(!provider.supports_streaming());
        assert!(!provider.supports_embeddings());
    }

    #[tokio::test]
    async fn test_health_check_unreachable() {
        let provider = LlamafileProvider::new("http://localhost:9999".to_string());
        let result = provider.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}
