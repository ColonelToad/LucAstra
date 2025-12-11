//! LLM inference and prompt management.

use crate::client::LlamafileClient;
use lucastra_core::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub context: Option<Vec<String>>, // Retrieved context snippets for RAG
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub stop_reason: String,
}

/// LLM service that wraps the provider interface.
pub struct LLMService {
    client: LlamafileClient, // Legacy client for backward compatibility
    system_prompt: String,
}

impl LLMService {
    pub fn new(endpoint: String) -> Self {
        Self {
            client: LlamafileClient::new(endpoint),
            system_prompt: "You are a helpful assistant embedded in an OS. Answer questions concisely and accurately.".to_string(),
        }
    }

    /// Check if the LLM server is online (blocking for backward compatibility).
    pub fn health_check(&self) -> Result<bool> {
        self.client
            .health_check()
            .map_err(|e| lucastra_core::LuCastraError::ServiceError(e.to_string()))
    }

    /// Perform inference with optional RAG context.
    pub fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let prompt = self.build_prompt(&request.prompt, request.context.clone());

        info!("LLM inference request: {} chars", prompt.len());

        let max_tokens = request.max_tokens.unwrap_or(256) as i32;
        let temperature = request.temperature.unwrap_or(0.7);

        // Try to call the LLM server, fall back to mock if unavailable
        match self.client.complete(&prompt, Some(max_tokens), Some(temperature)) {
            Ok(text) => Ok(InferenceResponse {
                text,
                stop_reason: "complete".to_string(),
            }),
            Err(e) => {
                info!("LLM server unavailable, using mock response: {}", e);
                let mock_response = format!(
                    "Mock response to: {}{}",
                    request.prompt,
                    if request.context.is_some() {
                        " [with retrieved context]"
                    } else {
                        ""
                    }
                );
                Ok(InferenceResponse {
                    text: mock_response,
                    stop_reason: "mock".to_string(),
                })
            }
        }
    }

    /// Build a prompt with optional RAG context.
    fn build_prompt(&self, query: &str, context: Option<Vec<String>>) -> String {
        let mut prompt = format!("{}\n\n", self.system_prompt);

        if let Some(docs) = context {
            prompt.push_str("## Context\n");
            for (i, doc) in docs.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, doc));
            }
            prompt.push_str("\n");
        }

        prompt.push_str(&format!("## User Query\n{}\n\n## Answer", query));
        prompt
    }

    /// Set custom system prompt.
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }
}
