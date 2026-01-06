use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
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
        if matches!(
            operation,
            FileOperation::Write
                | FileOperation::Move
                | FileOperation::Delete
                | FileOperation::Copy
        ) && !self.allow_host_write
        {
            return Err(FileAccessError::PermissionDenied);
        }

        // Reject read operations if disabled
        if matches!(operation, FileOperation::Read | FileOperation::List) && !self.allow_host_read {
            return Err(FileAccessError::PermissionDenied);
        }

        // Check if path is in allowed dirs
        let path = path
            .canonicalize()
            .map_err(|_| FileAccessError::InvalidPath(path.display().to_string()))?;

        let is_allowed = self
            .allowed_dirs
            .iter()
            .any(|allowed| path.starts_with(allowed));

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
            ('D'..='Z').contains(&drive)
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

/// Performs host file operations with validation and audit logging
pub struct FileAccessTool {
    validator: FileAccessValidator,
    audit_path: PathBuf,
}

impl FileAccessTool {
    pub fn new(validator: FileAccessValidator, audit_path: PathBuf) -> Self {
        Self {
            validator,
            audit_path,
        }
    }

    pub fn execute(
        &self,
        operation: FileOperation,
        path: &Path,
        dest_path: Option<&Path>,
    ) -> crate::ToolResult {
        let mut audit = AuditEntry {
            timestamp: format!("{:?}", SystemTime::now()),
            operation,
            source_path: path.display().to_string(),
            dest_path: dest_path.map(|p| p.display().to_string()),
            success: false,
            error_msg: None,
            user_approved: true,
        };

        let result = self.perform(operation, path, dest_path);

        match &result {
            Ok(msg) => {
                audit.success = true;
                self.append_audit(audit);
                crate::ToolResult::success("host_file_access", msg.clone())
            }
            Err(err) => {
                audit.success = false;
                audit.error_msg = Some(err.to_string());
                self.append_audit(audit);
                crate::ToolResult::failure("host_file_access", err.to_string())
            }
        }
    }

    fn perform(
        &self,
        operation: FileOperation,
        path: &Path,
        dest_path: Option<&Path>,
    ) -> FileAccessResult<String> {
        // Validate source
        self.validator.validate_path(path, operation)?;

        // Validate destination if needed
        if matches!(
            operation,
            FileOperation::Move | FileOperation::Copy | FileOperation::Write
        ) {
            if let Some(dest) = dest_path {
                if dest.exists() {
                    self.validator.validate_path(dest, operation)?;
                } else if let Some(parent) = dest.parent() {
                    self.validator.validate_path(parent, operation)?;
                } else {
                    return Err(FileAccessError::InvalidPath(
                        "Destination path has no parent".to_string(),
                    ));
                }
            } else {
                return Err(FileAccessError::InvalidPath(
                    "Destination path required".to_string(),
                ));
            }
        }

        match operation {
            FileOperation::Read => {
                let contents = fs::read_to_string(path)
                    .map_err(|e| FileAccessError::OperationFailed(e.to_string()))?;
                Ok(contents)
            }
            FileOperation::List => {
                let entries = fs::read_dir(path)
                    .map_err(|e| FileAccessError::OperationFailed(e.to_string()))?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path().display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(entries)
            }
            FileOperation::Copy => {
                let dest = dest_path.unwrap();
                fs::copy(path, dest)
                    .map_err(|e| FileAccessError::OperationFailed(e.to_string()))?;
                Ok(format!("copied to {}", dest.display()))
            }
            FileOperation::Move => {
                let dest = dest_path.unwrap();
                fs::rename(path, dest)
                    .map_err(|e| FileAccessError::OperationFailed(e.to_string()))?;
                Ok(format!("moved to {}", dest.display()))
            }
            FileOperation::Delete => {
                if path.is_dir() {
                    fs::remove_dir_all(path)
                        .map_err(|e| FileAccessError::OperationFailed(e.to_string()))?;
                } else {
                    fs::remove_file(path)
                        .map_err(|e| FileAccessError::OperationFailed(e.to_string()))?;
                }
                Ok("deleted".to_string())
            }
            FileOperation::Write => Err(FileAccessError::OperationFailed(
                "Write operation requires content (not implemented)".to_string(),
            )),
        }
    }

    fn append_audit(&self, entry: AuditEntry) {
        if let Some(dir) = self.audit_path.parent() {
            let _ = fs::create_dir_all(dir);
        }

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_path)
        {
            if let Ok(line) = serde_json::to_string(&entry) {
                let _ = writeln!(file, "{}", line);
            }
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

    fn temp_base(tag: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("lucastra_host_file_access_{}_{}", tag, unique));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_execute_read_logs_success_audit() {
        let base = temp_base("read");
        let file_path = base.join("file.txt");
        fs::write(&file_path, "hello world").unwrap();

        let allowed = vec![base.canonicalize().unwrap()];
        let validator = FileAccessValidator::new(allowed, true, false, false);
        let audit_path = base.join("audit.log");
        let tool = FileAccessTool::new(validator, audit_path.clone());

        let result = tool.execute(FileOperation::Read, &file_path, None);

        assert!(result.success);
        assert_eq!(result.output, "hello world");

        let audit_contents = fs::read_to_string(&audit_path).unwrap();
        let last_line = audit_contents.lines().last().unwrap();
        let entry: AuditEntry = serde_json::from_str(last_line).unwrap();

        assert!(entry.success);
        assert_eq!(entry.operation, FileOperation::Read);
        assert_eq!(entry.source_path, file_path.display().to_string());
    }

    #[test]
    fn test_execute_copy_denied_when_write_disabled() {
        let base = temp_base("copy_deny");
        let src = base.join("src.txt");
        let dest = base.join("dest.txt");
        fs::write(&src, "copy me").unwrap();

        let allowed = vec![base.canonicalize().unwrap()];
        let validator = FileAccessValidator::new(allowed, true, false, false);
        let audit_path = base.join("audit.log");
        let tool = FileAccessTool::new(validator, audit_path.clone());

        let result = tool.execute(FileOperation::Copy, &src, Some(dest.as_path()));

        assert!(!result.success);
        assert!(result.output.contains("Permission denied"));

        let audit_contents = fs::read_to_string(&audit_path).unwrap();
        let last_line = audit_contents.lines().last().unwrap();
        let entry: AuditEntry = serde_json::from_str(last_line).unwrap();

        assert!(!entry.success);
        assert!(entry
            .error_msg
            .unwrap_or_default()
            .contains("Permission denied"));
    }

    #[test]
    fn test_execute_copy_success_when_allowed() {
        let base = temp_base("copy_ok");
        let src = base.join("src.txt");
        let dest = base.join("dest.txt");
        fs::write(&src, "copy me").unwrap();

        let allowed = vec![base.canonicalize().unwrap()];
        let validator = FileAccessValidator::new(allowed, true, true, false);
        let audit_path = base.join("audit.log");
        let tool = FileAccessTool::new(validator, audit_path.clone());

        let result = tool.execute(FileOperation::Copy, &src, Some(dest.as_path()));

        assert!(result.success);
        assert!(dest.exists());

        let audit_contents = fs::read_to_string(&audit_path).unwrap();
        let last_line = audit_contents.lines().last().unwrap();
        let entry: AuditEntry = serde_json::from_str(last_line).unwrap();

        assert!(entry.success);
        assert_eq!(entry.operation, FileOperation::Copy);
        assert_eq!(entry.dest_path, Some(dest.display().to_string()));
    }

    #[test]
    fn test_execute_list_denied_when_read_disabled() {
        let base = temp_base("list_deny");
        let allowed = vec![base.canonicalize().unwrap()];
        let validator = FileAccessValidator::new(allowed, false, true, false);
        let audit_path = base.join("audit.log");
        let tool = FileAccessTool::new(validator, audit_path.clone());

        let result = tool.execute(FileOperation::List, &base, None);

        assert!(!result.success);
        assert!(result.output.contains("Permission denied"));

        let audit_contents = fs::read_to_string(&audit_path).unwrap();
        let last_line = audit_contents.lines().last().unwrap();
        let entry: AuditEntry = serde_json::from_str(last_line).unwrap();

        assert!(!entry.success);
        assert_eq!(entry.operation, FileOperation::List);
        assert!(entry
            .error_msg
            .unwrap_or_default()
            .contains("Permission denied"));
    }
}
