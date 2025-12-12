use lucastra_core::{command::FileEntry, Result};
use lucastra_hal::FileSystemDriver;
use std::collections::HashMap;
use tracing::info;

/// Filesystem manager: handles mounting, file I/O, and virtual filesystem abstractions.
pub struct FilesystemManager {
    mount_points: HashMap<String, Box<dyn FileSystemDriver + Send>>,
}

impl FilesystemManager {
    pub fn new() -> Self {
        Self {
            mount_points: HashMap::new(),
        }
    }

    /// Mount a filesystem at a path.
    pub fn mount<T: FileSystemDriver + Send + 'static>(
        &mut self,
        mount_point: &str,
        driver: T,
    ) -> Result<()> {
        let mut driver = Box::new(driver);
        driver.mount(mount_point)?;
        self.mount_points.insert(mount_point.to_string(), driver);
        info!("Filesystem mounted at {}", mount_point);
        Ok(())
    }

    /// Unmount a filesystem.
    pub fn unmount(&mut self, mount_point: &str) -> Result<()> {
        if let Some(mut driver) = self.mount_points.remove(mount_point) {
            driver.unmount()?;
            info!("Filesystem unmounted from {}", mount_point);
            Ok(())
        } else {
            Err(lucastra_core::LuCastraError::FilesystemError(format!(
                "Mount point not found: {}",
                mount_point
            )))
        }
    }

    /// Resolve a path to the appropriate filesystem driver.
    fn resolve_driver(&self, path: &str) -> Result<&dyn FileSystemDriver> {
        for (mount_point, driver) in &self.mount_points {
            if path.starts_with(mount_point) && driver.is_mounted() {
                return Ok(driver.as_ref());
            }
        }
        Err(lucastra_core::LuCastraError::FilesystemError(format!(
            "No filesystem mounted for path: {}",
            path
        )))
    }

    /// Resolve a path mutably to the appropriate filesystem driver.
    fn resolve_driver_mut(&mut self, path: &str) -> Result<&mut dyn FileSystemDriver> {
        for (mount_point, driver) in &mut self.mount_points {
            if path.starts_with(mount_point) && driver.is_mounted() {
                return Ok(driver.as_mut());
            }
        }
        Err(lucastra_core::LuCastraError::FilesystemError(format!(
            "No filesystem mounted for path: {}",
            path
        )))
    }

    /// List files in a directory.
    pub fn list_files(&self, path: &str) -> Result<Vec<FileEntry>> {
        let driver = self.resolve_driver(path)?;
        let files = driver.list_files(path)?;
        Ok(files
            .into_iter()
            .map(|f| FileEntry {
                path: f,
                is_dir: false,
                size: 0,
            })
            .collect())
    }

    /// Read file contents.
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let driver = self.resolve_driver(path)?;
        driver.read_file(path)
    }

    /// Write file contents.
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<()> {
        let driver = self.resolve_driver_mut(path)?;
        driver.write_file(path, content)
    }
}

impl Default for FilesystemManager {
    fn default() -> Self {
        Self::new()
    }
}
