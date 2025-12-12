use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod file_access;
pub mod install;
pub mod read;
pub mod search;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Search error: {0}")]
    Search(String),

    #[error("Read error: {0}")]
    Read(String),

    #[error("Install error: {0}")]
    Install(String),

    #[error("Core error: {0}")]
    Core(#[from] lucastra_core::LuCastraError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ToolError>;

/// Tool abstraction for agentic tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool", content = "params")]
pub enum Tool {
    /// Search the filesystem using BM25
    Search { query: String, top_k: Option<usize> },

    /// Read file contents
    Read { path: String },

    /// Install a program (via terminal command)
    Install {
        program: String,
        method: InstallMethod,
    },

    /// Access host filesystem with validation and auditing
    HostFileAccess {
        operation: file_access::FileOperation,
        path: String,
        dest_path: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Run a shell command
    Command { cmd: String, args: Vec<String> },

    /// Download and install from URL
    Download {
        url: String,
        installer_args: Vec<String>,
    },
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub output: String,
}

impl ToolResult {
    pub fn success(tool: &str, output: String) -> Self {
        Self {
            tool: tool.to_string(),
            success: true,
            output,
        }
    }

    pub fn failure(tool: &str, error: String) -> Self {
        Self {
            tool: tool.to_string(),
            success: false,
            output: error,
        }
    }
}
