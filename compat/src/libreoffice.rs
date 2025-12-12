//! LibreOffice launcher via relibc sandbox.
//!
//! Provides a minimal launcher for LibreOffice documents within the LucAstra
//! compatibility layer, with file I/O sandboxed through HostFileAccess.

use lucastra_core::Result;
use std::path::PathBuf;

/// LibreOffice launch configuration.
#[derive(Debug, Clone)]
pub struct LibreOfficeConfig {
    /// Path to LibreOffice executable (e.g., ~/.lucastra/data/apps/libreoffice/soffice)
    pub executable_path: PathBuf,
    /// Document file to open
    pub document_path: PathBuf,
    /// Enable sandbox mode (restrict file access to allowed dirs)
    pub sandbox_enabled: bool,
}

impl LibreOfficeConfig {
    pub fn new(executable: PathBuf, document: PathBuf) -> Self {
        Self {
            executable_path: executable,
            document_path: document,
            sandbox_enabled: true,
        }
    }

    /// Set sandbox mode.
    pub fn with_sandbox(mut self, enabled: bool) -> Self {
        self.sandbox_enabled = enabled;
        self
    }

    /// Validate that executable and document paths are accessible.
    pub fn validate(&self) -> Result<()> {
        if !self.executable_path.exists() {
            return Err(lucastra_core::LuCastraError::SyscallError(format!(
                "LibreOffice executable not found: {}",
                self.executable_path.display()
            )));
        }

        if !self.document_path.exists() {
            tracing::warn!(
                "Document path does not exist: {}",
                self.document_path.display()
            );
            // Allow non-existent paths (e.g., new document)
        }

        Ok(())
    }
}

/// LibreOffice launcher.
pub struct LibreOfficeLauncher {
    config: LibreOfficeConfig,
}

impl LibreOfficeLauncher {
    pub fn new(config: LibreOfficeConfig) -> Self {
        Self { config }
    }

    /// Launch LibreOffice with the configured document.
    pub fn launch(&self) -> Result<()> {
        self.config.validate()?;

        tracing::info!(
            "Launching LibreOffice: {} (document: {}, sandbox: {})",
            self.config.executable_path.display(),
            self.config.document_path.display(),
            self.config.sandbox_enabled
        );

        // Placeholder: in a real implementation, this would:
        // 1. Set up a syscall handler with HostFileAccess delegation
        // 2. Load the ELF binary (LibreOffice executable)
        // 3. Pass document path as argv[1]
        // 4. Jump to entry point with sandbox context

        tracing::info!("LibreOffice launch requested (syscall handler integration pending)");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libreoffice_config_creation() {
        let config = LibreOfficeConfig::new(
            PathBuf::from("/app/soffice"),
            PathBuf::from("/home/user/document.odt"),
        );

        assert_eq!(config.executable_path, PathBuf::from("/app/soffice"));
        assert_eq!(
            config.document_path,
            PathBuf::from("/home/user/document.odt")
        );
        assert!(config.sandbox_enabled);
    }

    #[test]
    fn test_libreoffice_sandbox_toggle() {
        let config = LibreOfficeConfig::new(
            PathBuf::from("/app/soffice"),
            PathBuf::from("/home/user/document.odt"),
        )
        .with_sandbox(false);

        assert!(!config.sandbox_enabled);
    }

    #[test]
    fn test_libreoffice_validate_missing_executable() {
        let config = LibreOfficeConfig::new(
            PathBuf::from("/nonexistent/soffice"),
            PathBuf::from("/home/user/document.odt"),
        );

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
