use crate::{InstallMethod, Result, ToolResult};
use std::process::{Command, Stdio};
use tracing::info;

/// Install program tool implementation
pub struct InstallTool;

impl InstallTool {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, program: &str, method: &InstallMethod) -> Result<ToolResult> {
        info!(
            "Executing install tool: program='{}', method={:?}",
            program, method
        );

        match method {
            InstallMethod::Command { cmd, args } => self.execute_command(program, cmd, args),
            InstallMethod::Download {
                url,
                installer_args,
            } => self.download_and_install(program, url, installer_args),
        }
    }

    fn execute_command(&self, program: &str, cmd: &str, args: &[String]) -> Result<ToolResult> {
        info!("Running command: {} {:?}", cmd, args);

        let output = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = format!(
                "Successfully installed '{}'\n\nOutput:\n{}",
                program, stdout
            );
            Ok(ToolResult::success("install", result))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error = format!(
                "Failed to install '{}': {}\n\nError output:\n{}",
                program, output.status, stderr
            );
            Ok(ToolResult::failure("install", error))
        }
    }

    fn download_and_install(
        &self,
        program: &str,
        url: &str,
        installer_args: &[String],
    ) -> Result<ToolResult> {
        info!("Downloading from: {}", url);

        // For MVP, we'll use PowerShell's Invoke-WebRequest
        let temp_file = format!("C:\\Users\\Public\\Downloads\\{}_installer.exe", program);

        // Download
        let download_output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", url, temp_file),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !download_output.status.success() {
            let stderr = String::from_utf8_lossy(&download_output.stderr);
            return Ok(ToolResult::failure(
                "install",
                format!("Download failed: {}", stderr),
            ));
        }

        // Install
        info!("Installing from: {}", temp_file);
        let install_output = Command::new(&temp_file)
            .args(installer_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if install_output.status.success() {
            let result = format!("Successfully downloaded and installed '{}'", program);
            Ok(ToolResult::success("install", result))
        } else {
            let stderr = String::from_utf8_lossy(&install_output.stderr);
            Ok(ToolResult::failure(
                "install",
                format!("Installation failed: {}", stderr),
            ))
        }
    }
}

impl Default for InstallTool {
    fn default() -> Self {
        Self::new()
    }
}
