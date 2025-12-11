//! Streaming response support for real-time LLM output.

use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug)]
pub enum StreamError {
    Error(String),
    ConnectionClosed,
    ParseError(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::Error(msg) => write!(f, "stream error: {}", msg),
            StreamError::ConnectionClosed => write!(f, "connection closed"),
            StreamError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

impl std::error::Error for StreamError {}

pub type StreamResult<T> = Result<T, StreamError>;

/// Chunk of a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

/// Trait for streaming completions.
#[async_trait::async_trait]
pub trait StreamableProvider {
    /// Stream completion chunks as they arrive.
    async fn stream_complete(
        &self,
        request: super::CompletionRequest,
    ) -> StreamResult<Pin<Box<dyn Stream<Item = StreamResult<StreamChunk>> + Send>>>;
}

// TODO: Implement for OpenAI (SSE parsing)
// TODO: Implement for llamafile (SSE parsing)
