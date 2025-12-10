use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Log level: "error", "warn", "info", "debug", "trace"
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Enable file logging
    #[serde(default = "default_true")]
    pub file_logging: bool,

    /// Log file directory
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,

    /// Max log file size in MB (0 = unlimited)
    #[serde(default = "default_max_log_size")]
    pub max_log_size_mb: u32,

    /// Number of rotated log files to keep
    #[serde(default = "default_log_files_keep")]
    pub log_files_keep: u32,

    /// Enable console output
    #[serde(default = "default_true")]
    pub console_output: bool,

    /// Enable structured JSON logging
    #[serde(default = "default_false")]
    pub json_format: bool,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Export metrics to file
    #[serde(default = "default_false")]
    pub export_to_file: bool,

    /// Metrics export path
    #[serde(default = "default_metrics_dir")]
    pub export_dir: PathBuf,

    /// Metrics export interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub export_interval_secs: u32,
}

// Default value functions
fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_log_dir() -> PathBuf {
    dirs::cache_dir()
        .map(|d| d.join("lucastra").join("logs"))
        .unwrap_or_else(|| PathBuf::from("./logs"))
}

fn default_metrics_dir() -> PathBuf {
    dirs::cache_dir()
        .map(|d| d.join("lucastra").join("metrics"))
        .unwrap_or_else(|| PathBuf::from("./metrics"))
}

fn default_max_log_size() -> u32 {
    10 // 10 MB
}

fn default_log_files_keep() -> u32 {
    5
}

fn default_metrics_interval() -> u32 {
    60 // 60 seconds
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file_logging: true,
            log_dir: default_log_dir(),
            max_log_size_mb: default_max_log_size(),
            log_files_keep: default_log_files_keep(),
            console_output: true,
            json_format: false,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_to_file: false,
            export_dir: default_metrics_dir(),
            export_interval_secs: default_metrics_interval(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.file_logging);
        assert!(config.console_output);
        assert!(!config.json_format);
    }

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert!(!config.export_to_file);
        assert_eq!(config.export_interval_secs, 60);
    }

    #[test]
    fn test_tracing_config_serialization() {
        let config = TracingConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: TracingConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.level, config.level);
    }
}
