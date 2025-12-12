//! HTTP client for llamafile server communication.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("request error: {0}")]
    RequestError(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("server error: {0}")]
    ServerError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub n_predict: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    #[serde(default)]
    pub stop: bool,
}

/// HTTP client for llamafile API (OpenAI-compatible endpoint).
pub struct LlamafileClient {
    endpoint: String,
    client: reqwest::Client,
}

impl LlamafileClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    /// Check if the server is reachable (blocking).
    pub fn health_check(&self) -> Result<bool, ClientError> {
        let runtime =
            tokio::runtime::Runtime::new().map_err(|e| ClientError::RequestError(e.to_string()))?;
        runtime.block_on(self.health_check_async())
    }

    async fn health_check_async(&self) -> Result<bool, ClientError> {
        let url = format!("{}/health", self.endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Send a completion request to the llamafile server (blocking).
    pub fn complete(
        &self,
        prompt: &str,
        n_predict: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<String, ClientError> {
        let runtime =
            tokio::runtime::Runtime::new().map_err(|e| ClientError::RequestError(e.to_string()))?;
        runtime.block_on(self.complete_async(prompt, n_predict, temperature))
    }

    async fn complete_async(
        &self,
        prompt: &str,
        n_predict: Option<i32>,
        temperature: Option<f32>,
    ) -> Result<String, ClientError> {
        let req = CompletionRequest {
            prompt: prompt.to_string(),
            n_predict,
            temperature,
            top_p: None,
        };

        let url = format!("{}/v1/completions", self.endpoint);
        debug!("Sending completion request to {}", url);

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| ClientError::RequestError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ClientError::ServerError(format!(
                "Server returned {}",
                resp.status()
            )));
        }

        let body = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| ClientError::ParseError(e.to_string()))?;

        let content = body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| ClientError::ParseError("No text in response".to_string()))?;

        Ok(content.to_string())
    }
}
