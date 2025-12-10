use lucastra_core::Result;

/// Filesystem driver abstraction (FAT32, ext4, etc.).
pub trait FileSystemDriver {
    fn mount(&mut self, path: &str) -> Result<()>;
    fn unmount(&mut self) -> Result<()>;
    fn list_files(&self, path: &str) -> Result<Vec<String>>;
    fn read_file(&self, path: &str) -> Result<Vec<u8>>;
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()>;
    fn is_mounted(&self) -> bool;
}

/// Mock filesystem for testing.
pub struct MockFileSystem {
    mounted: bool,
    files: std::collections::HashMap<String, Vec<u8>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            mounted: false,
            files: std::collections::HashMap::new(),
        }
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemDriver for MockFileSystem {
    fn mount(&mut self, path: &str) -> Result<()> {
        tracing::info!("Mounting mock filesystem at {}", path);
        self.mounted = true;
        Ok(())
    }

    fn unmount(&mut self) -> Result<()> {
        tracing::info!("Unmounting mock filesystem");
        self.mounted = false;
        Ok(())
    }

    fn list_files(&self, _path: &str) -> Result<Vec<String>> {
        Ok(self.files.keys().cloned().collect())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| lucastra_core::LuCastraError::FilesystemError(format!("File not found: {}", path)))
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()> {
        self.files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    fn is_mounted(&self) -> bool {
        self.mounted
    }
}
