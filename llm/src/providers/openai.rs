//! OpenAI provider implementation.

use super::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, LLMProvider,
    ProviderError, ProviderResult, StopReason,
};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize)]
struct OpenAICompletionRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAICompletionResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIChoice {
    text: String,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIUsage {
    total_tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

/// OpenAI provider for GPT models and embeddings.
pub struct OpenAIProvider {
    _api_key: String,
    model: String,
    embedding_model: String,
    client: Client,
    pub(crate) base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: Option<String>) -> ProviderResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| ProviderError::AuthError(e.to_string()))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| ProviderError::RequestError(e.to_string()))?;

        Ok(Self {
            _api_key: api_key,
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            embedding_model: "text-embedding-3-small".to_string(),
            client,
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn default_model(&self) -> &str {
        &self.model
    }

    async fn health_check(&self) -> ProviderResult<bool> {
        let url = format!("{}/models", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let openai_req = OpenAICompletionRequest {
            model: self.model.clone(),
            prompt: request.prompt.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop_sequences,
        };

        let url = format!("{}/completions", self.base_url);
        debug!("Sending OpenAI completion request to {}", url);

        let resp = self
            .client
            .post(&url)
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    ProviderError::AuthError("Invalid API key".to_string())
                } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    ProviderError::RateLimitError("Rate limit exceeded".to_string())
                } else {
                    ProviderError::RequestError(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::RequestError(format!(
                "OpenAI API returned {}: {}",
                status, body
            )));
        }

        let openai_resp: OpenAICompletionResponse = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let choice = openai_resp
            .choices
            .first()
            .ok_or_else(|| ProviderError::InvalidResponse("No choices in response".to_string()))?;

        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => StopReason::Stop,
            Some("length") => StopReason::Length,
            _ => StopReason::Complete,
        };

        Ok(CompletionResponse {
            content: choice.text.clone(),
            stop_reason,
            tokens_used: openai_resp.usage.map(|u| u.total_tokens),
            model: Some(self.model.clone()),
        })
    }

    async fn embed(&self, request: EmbeddingRequest) -> ProviderResult<EmbeddingResponse> {
        let model = request
            .model
            .unwrap_or_else(|| self.embedding_model.clone());

        let openai_req = OpenAIEmbeddingRequest {
            model: model.clone(),
            input: request.texts,
        };

        let url = format!("{}/embeddings", self.base_url);
        debug!("Sending OpenAI embedding request to {}", url);

        let resp = self
            .client
            .post(&url)
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    ProviderError::AuthError("Invalid API key".to_string())
                } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    ProviderError::RateLimitError("Rate limit exceeded".to_string())
                } else {
                    ProviderError::RequestError(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::RequestError(format!(
                "OpenAI API returned {}: {}",
                status, body
            )));
        }

        let openai_resp: OpenAIEmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let embeddings: Vec<Vec<f32>> = openai_resp.data.into_iter().map(|d| d.embedding).collect();

        let dimensions = embeddings.first().map(|e| e.len()).ok_or_else(|| {
            ProviderError::InvalidResponse("No embeddings in response".to_string())
        })?;

        Ok(EmbeddingResponse {
            embeddings,
            model: openai_resp.model,
            dimensions,
        })
    }

    fn supports_streaming(&self) -> bool {
        true // OpenAI supports SSE streaming, will implement later
    }

    fn supports_embeddings(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAIProvider::new("test-key".to_string(), None).unwrap();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.default_model(), "gpt-4o-mini");
        assert!(provider.supports_streaming());
        assert!(provider.supports_embeddings());
    }

    #[test]
    fn test_custom_model() {
        let provider =
            OpenAIProvider::new("test-key".to_string(), Some("gpt-4".to_string())).unwrap();
        assert_eq!(provider.default_model(), "gpt-4");
    }

    #[test]
    fn test_custom_base_url() {
        let provider = OpenAIProvider::new("test-key".to_string(), None)
            .unwrap()
            .with_base_url("https://custom.openai.com/v1".to_string());
        assert_eq!(provider.base_url, "https://custom.openai.com/v1");
    }
}
