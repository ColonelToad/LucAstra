//! Anthropic Claude API provider implementation.

use super::*;
use reqwest::Client;
use serde_json::{json, Value};

/// Anthropic Claude API provider.
#[derive(Clone)]
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
        }
    }

    /// Create a provider with a custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Create a provider with a custom model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn health_check(&self) -> ProviderResult<bool> {
        // Anthropic doesn't have a dedicated health endpoint, so we check if we can reach the API
        let response = self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse> {
        let body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens.unwrap_or(1024),
            "messages": [{
                "role": "user",
                "content": request.prompt
            }],
            "temperature": request.temperature,
            "top_p": request.top_p,
            "stop_sequences": request.stop_sequences,
        });

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::RequestError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::RequestError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let content = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| ProviderError::InvalidResponse("Missing content.text".to_string()))?
            .to_string();

        let stop_reason = match json["stop_reason"].as_str() {
            Some("end_turn") => StopReason::Complete,
            Some("max_tokens") => StopReason::Length,
            Some("stop_sequence") => StopReason::Stop,
            _ => StopReason::Error,
        };

        Ok(CompletionResponse {
            content,
            stop_reason,
            tokens_used: json["usage"]["output_tokens"].as_u64().map(|t| t as usize),
            model: Some(self.model.clone()),
        })
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_embeddings(&self) -> bool {
        false
    }

    fn default_model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key");
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.default_model(), "claude-3-5-sonnet-20241022");
        assert!(provider.supports_streaming());
        assert!(!provider.supports_embeddings());
    }

    #[test]
    fn test_custom_base_url() {
        let provider = AnthropicProvider::new("test-key")
            .with_base_url("https://custom.anthropic.com");
        assert_eq!(provider.base_url, "https://custom.anthropic.com");
    }

    #[test]
    fn test_custom_model() {
        let provider = AnthropicProvider::new("test-key")
            .with_model("claude-3-opus-20240229");
        assert_eq!(provider.default_model(), "claude-3-opus-20240229");
    }
}
