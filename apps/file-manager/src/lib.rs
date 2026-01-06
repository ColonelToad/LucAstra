//! LucAstra File Manager - Native file browser and operations
//!
//! Provides file listing, opening, copying, moving, and deletion with
//! safety confirmations and audit logging.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileOpError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub type FileOpResult<T> = Result<T, FileOpError>;

/// File entry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> FileOpResult<Self> {
        let metadata = std::fs::metadata(path).map_err(|e| FileOpError::IoError(e))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();

        let modified = format!(
            "{:?}",
            metadata.modified().unwrap_or(std::time::SystemTime::now())
        );

        Ok(Self {
            path: path.to_path_buf(),
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        })
    }
}

/// File Manager state
#[derive(Debug, Clone, Default)]
pub struct FileManager {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub history: Vec<PathBuf>,
}

impl FileManager {
    /// Create a new file manager starting at a directory
    pub fn new(path: PathBuf) -> FileOpResult<Self> {
        if !path.is_dir() {
            return Err(FileOpError::NotFound(format!(
                "{} is not a directory",
                path.display()
            )));
        }

        let mut fm = Self {
            current_dir: path,
            entries: Vec::new(),
            history: Vec::new(),
        };

        fm.refresh()?;
        Ok(fm)
    }

    /// Refresh the file listing
    pub fn refresh(&mut self) -> FileOpResult<()> {
        self.entries.clear();

        // Add parent directory link if not at root
        if self.current_dir.parent().is_some() {
            self.entries.push(FileEntry {
                path: self.current_dir.parent().unwrap().to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
                size: 0,
                modified: String::new(),
            });
        }

        // removed invalid `path` usage; read_dir on current_dir directly
        let entries = std::fs::read_dir(&self.current_dir).map_err(FileOpError::IoError)?;

        for entry in entries.flatten() {
            if let Ok(file_entry) = FileEntry::from_path(&entry.path()) {
                self.entries.push(file_entry);
            }
        }

        // Sort: directories first, then by name
        self.entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.cmp(&b.name)
            }
        });

        Ok(())
    }

    /// Navigate to a directory
    pub fn navigate(&mut self, path: &Path) -> FileOpResult<()> {
        if !path.is_dir() {
            return Err(FileOpError::NotFound(format!(
                "{} is not a directory",
                path.display()
            )));
        }

        self.history.push(self.current_dir.clone());
        self.current_dir = path.to_path_buf();
        self.refresh()?;
        Ok(())
    }

    /// Go back to previous directory
    pub fn back(&mut self) -> FileOpResult<()> {
        if let Some(prev) = self.history.pop() {
            self.current_dir = prev;
            self.refresh()?;
            Ok(())
        } else {
            Err(FileOpError::OperationFailed(
                "No previous directory".to_string(),
            ))
        }
    }

    /// Copy a file
    pub fn copy(&self, src: &Path, dest: &Path) -> FileOpResult<()> {
        if !src.exists() {
            return Err(FileOpError::NotFound(src.display().to_string()));
        }

        std::fs::copy(src, dest).map_err(FileOpError::IoError)?;

        tracing::info!("Copied {} to {}", src.display(), dest.display());
        Ok(())
    }

    /// Move a file
    pub fn move_file(&mut self, src: &Path, dest: &Path) -> FileOpResult<()> {
        if !src.exists() {
            return Err(FileOpError::NotFound(src.display().to_string()));
        }

        std::fs::rename(src, dest).map_err(FileOpError::IoError)?;

        tracing::info!("Moved {} to {}", src.display(), dest.display());
        self.refresh()?;
        Ok(())
    }

    /// Delete a file or directory
    pub fn delete(&mut self, path: &Path) -> FileOpResult<()> {
        if !path.exists() {
            return Err(FileOpError::NotFound(path.display().to_string()));
        }

        if path.is_dir() {
            std::fs::remove_dir_all(path).map_err(FileOpError::IoError)?;
        } else {
            std::fs::remove_file(path).map_err(FileOpError::IoError)?;
        }

        tracing::info!("Deleted {}", path.display());
        self.refresh()?;
        Ok(())
    }

    /// List current directory entries
    pub fn list(&self) -> &[FileEntry] {
        &self.entries
    }

    /// Get file entry by index
    pub fn get_entry(&self, index: usize) -> Option<&FileEntry> {
        self.entries.get(index)
    }

    /// Get full path for an entry by index
    pub fn get_path(&self, index: usize) -> Option<PathBuf> {
        self.entries.get(index).map(|e| e.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("lucastra_fm_test_{}", timestamp));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_file_manager_creation() {
        let dir = temp_dir();
        let fm = FileManager::new(dir.clone()).unwrap();
        assert_eq!(fm.current_dir, dir);
    }

    #[test]
    fn test_file_listing() {
        let dir = temp_dir();
        fs::write(dir.join("test1.txt"), "content1").unwrap();
        fs::write(dir.join("test2.txt"), "content2").unwrap();

        let fm = FileManager::new(dir.clone()).unwrap();
        assert!(fm.list().len() >= 2);
    }

    #[test]
    fn test_file_copy() {
        let dir = temp_dir();
        let src = dir.join("original.txt");
        let dst = dir.join("copy.txt");

        fs::write(&src, "original content").unwrap();

        let fm = FileManager::new(dir).unwrap();
        fm.copy(&src, &dst).unwrap();

        assert!(dst.exists());
        let content = fs::read_to_string(&dst).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_file_delete() {
        let dir = temp_dir();
        let file = dir.join("deleteme.txt");
        fs::write(&file, "to delete").unwrap();

        let mut fm = FileManager::new(dir).unwrap();
        fm.delete(&file).unwrap();

        assert!(!file.exists());
    }

    #[test]
    fn test_navigation() {
        let dir = temp_dir();
        let subdir = dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let mut fm = FileManager::new(dir).unwrap();
        fm.navigate(&subdir).unwrap();

        assert_eq!(fm.current_dir, subdir);
        assert!(fm.back().is_ok());
    }
}
