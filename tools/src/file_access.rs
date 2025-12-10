use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileAccessError {
    #[error("Path not in whitelist: {0}")]
    NotWhitelisted(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("USB not allowed")]
    UsbNotAllowed,
}

pub type FileAccessResult<T> = Result<T, FileAccessError>;

/// File operation types for audit logging and validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Move,
    Copy,
    Delete,
    List,
}

impl std::fmt::Display for FileOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperation::Read => write!(f, "read"),
            FileOperation::Write => write!(f, "write"),
            FileOperation::Move => write!(f, "move"),
            FileOperation::Copy => write!(f, "copy"),
            FileOperation::Delete => write!(f, "delete"),
            FileOperation::List => write!(f, "list"),
        }
    }
}

/// Audit log entry for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    #[serde(rename = "op")]
    pub operation: FileOperation,
    pub source_path: String,
    pub dest_path: Option<String>,
    pub success: bool,
    pub error_msg: Option<String>,
    pub user_approved: bool,
}

/// Host file access request (user confirmation required on first access)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostFileAccessRequest {
    pub operation: FileOperation,
    pub path: PathBuf,
    pub dest_path: Option<PathBuf>,
    pub requires_approval: bool,
}

/// Validator for file access against whitelist
pub struct FileAccessValidator {
    allowed_dirs: Vec<PathBuf>,
    allow_host_read: bool,
    allow_host_write: bool,
    allow_usb: bool,
}

impl FileAccessValidator {
    pub fn new(
        allowed_dirs: Vec<PathBuf>,
        allow_host_read: bool,
        allow_host_write: bool,
        allow_usb: bool,
    ) -> Self {
        Self {
            allowed_dirs,
            allow_host_read,
            allow_host_write,
            allow_usb,
        }
    }

    /// Validate a path against whitelist
    pub fn validate_path(&self, path: &Path, operation: FileOperation) -> FileAccessResult<()> {
        // Reject write operations if disabled
        if matches!(operation, FileOperation::Write | FileOperation::Move | FileOperation::Delete | FileOperation::Copy) && !self.allow_host_write {
            return Err(FileAccessError::PermissionDenied);
        }

        // Reject read operations if disabled
        if matches!(operation, FileOperation::Read | FileOperation::List) && !self.allow_host_read {
            return Err(FileAccessError::PermissionDenied);
        }

        // Check if path is in allowed dirs
        let path = path.canonicalize()
            .map_err(|_| FileAccessError::InvalidPath(path.display().to_string()))?;

        let is_allowed = self.allowed_dirs.iter().any(|allowed| {
            path.starts_with(allowed)
        });

        if !is_allowed {
            return Err(FileAccessError::NotWhitelisted(path.display().to_string()));
        }

        // Check for USB (removable media)
        if self.is_removable_media(&path) && !self.allow_usb {
            return Err(FileAccessError::UsbNotAllowed);
        }

        Ok(())
    }

    /// Check if path is on removable media (USB)
    #[cfg(target_os = "windows")]
    fn is_removable_media(&self, path: &Path) -> bool {
        // Simplified: check if drive letter is in certain ranges or use Windows API
        // For now, detect common USB mount points
        if let Some(drive) = path.to_string_lossy().chars().next() {
            // D-Z typically USB, C is main drive
            drive >= 'D' && drive <= 'Z'
        } else {
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn is_removable_media(&self, path: &Path) -> bool {
        // Linux: check /media or /mnt
        let path_str = path.to_string_lossy();
        path_str.contains("/media/") || path_str.contains("/mnt/")
    }

    /// Get list of allowed directories
    pub fn allowed_dirs(&self) -> &[PathBuf] {
        &self.allowed_dirs
    }

    /// Update allowed directories
    pub fn set_allowed_dirs(&mut self, dirs: Vec<PathBuf>) {
        self.allowed_dirs = dirs;
    }

    /// Add a new allowed directory
    pub fn add_allowed_dir(&mut self, dir: PathBuf) {
        if !self.allowed_dirs.contains(&dir) {
            self.allowed_dirs.push(dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_rejects_write_when_disabled() {
        let validator = FileAccessValidator::new(vec![], false, false, false);
        let path = PathBuf::from("/tmp/test.txt");
        
        let result = validator.validate_path(&path, FileOperation::Write);
        assert!(matches!(result, Err(FileAccessError::PermissionDenied)));
    }

    #[test]
    fn test_validator_rejects_unlisted_path() {
        let temp = std::env::temp_dir();
        let allowed = vec![temp.join("lucastra_test_allowed")];
        let validator = FileAccessValidator::new(allowed, true, false, false);
        let unlisted = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        // unlisted should not be in allowed dirs
        let result = validator.validate_path(&unlisted, FileOperation::Read);
        assert!(matches!(result, Err(FileAccessError::NotWhitelisted(_))));
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: "2024-12-10T10:00:00Z".to_string(),
            operation: FileOperation::Read,
            source_path: "/home/user/file.txt".to_string(),
            dest_path: None,
            success: true,
            error_msg: None,
            user_approved: true,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("read") || json.contains("Read"));
        assert!(json.contains("file.txt"));
    }
}
