//! LLM service for LucAstra.
//!
//! This module provides integration with a llamafile-based 7B model running as an HTTP server.
//! The model is assumed to be running locally (default: http://localhost:8000).

pub mod client;
pub mod inference;

pub use client::LlamafileClient;
pub use inference::{InferenceRequest, InferenceResponse, LLMService};

use lucastra_core::Result;

/// Initialize the LLM service.
pub fn init(endpoint: Option<String>) -> Result<LLMService> {
    let url = endpoint.unwrap_or_else(|| "http://localhost:8000".to_string());
    tracing::info!("Initializing LLM service at {}", url);
    Ok(LLMService::new(url))
}
