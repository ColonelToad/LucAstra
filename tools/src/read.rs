use crate::{Result, ToolError, ToolResult};
use lucastra_fs::FilesystemManager;
use tracing::info;

/// Read file tool implementation
pub struct ReadTool<'a> {
    filesystem: &'a FilesystemManager,
}

impl<'a> ReadTool<'a> {
    pub fn new(filesystem: &'a FilesystemManager) -> Self {
        Self { filesystem }
    }
    
    pub fn execute(&self, path: &str) -> Result<ToolResult> {
        info!("Executing read tool: path='{}'", path);
        
        match self.filesystem.read_file(path) {
            Ok(bytes) => {
                let content = String::from_utf8_lossy(&bytes).to_string();
                Ok(ToolResult::success("read", content))
            }
            Err(e) => {
                let error = format!("Failed to read file '{}': {}", path, e);
                Ok(ToolResult::failure("read", error))
            }
        }
    }
}
