use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub payload: CommandPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandPayload {
    /// List devices (USB, input devices)
    ListDevices,

    /// Mount a device by path (e.g., "/dev/usb0")
    Mount { device_path: String, mount_point: String },

    /// Unmount a device
    Unmount { mount_point: String },

    /// List files in a directory
    ListFiles { path: String },

    /// Read file contents
    ReadFile { path: String },

    /// Write file contents
    WriteFile { path: String, content: Vec<u8> },

    /// Search filesystem (BM25)
    Search { query: String },

    /// Query the LLM (with optional search context)
    Query { text: String, use_rag: Option<bool> },

    /// Get system status
    Status,

    /// Shutdown system
    Shutdown,

    /// Echo for testing
    Echo { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub command_id: String,
    pub payload: ResponsePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponsePayload {
    Devices(Vec<String>),
    Files(Vec<FileEntry>),
    Content(Vec<u8>),
    SearchResults(Vec<SearchResult>),
    Status(String),
    Success(String),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub score: f32,
    pub snippet: String,
}
