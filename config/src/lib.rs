use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use thiserror::Error;

pub mod observability;
pub use observability::{MetricsConfig, TracingConfig};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Read(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("Config directory not found")]
    NoConfigDir,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main configuration structure for LucAstra
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub llm: LlmConfig,

    #[serde(default)]
    pub storage: StorageConfig,

    #[serde(default)]
    pub search: SearchConfig,

    #[serde(default)]
    pub gui: GuiConfig,

    #[serde(default)]
    pub security: SecurityConfig,

    #[serde(default)]
    pub advanced: AdvancedConfig,

    #[serde(default)]
    pub tracing: TracingConfig,

    #[serde(default)]
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LLM server URL (default: http://localhost:8000)
    #[serde(default = "default_llm_url")]
    pub server_url: String,

    /// Auto-start embedded LLM server
    #[serde(default = "default_true")]
    pub auto_start: bool,

    /// Model size: "7b", "13b", "70b"
    #[serde(default = "default_model_size")]
    pub model_size: String,

    /// Download model on first run if missing
    #[serde(default = "default_true")]
    pub auto_download: bool,

    /// Use GPU acceleration if available
    #[serde(default = "default_true")]
    pub use_gpu: bool,

    /// Quantization level: "none", "4bit", "8bit"
    #[serde(default = "default_quantization")]
    pub quantization: String,

    /// Enable streaming responses
    #[serde(default = "default_true")]
    pub streaming: bool,

    /// Max tokens per response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature (0.0-2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Root data directory (default: ~/.lucastra/data)
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Use host filesystem (true) or virtual (false)
    #[serde(default = "default_true")]
    pub use_host_fs: bool,

    /// Max cache size in MB
    #[serde(default = "default_cache_size")]
    pub cache_size_mb: u64,

    /// Enable file watching for auto-indexing
    #[serde(default = "default_true")]
    pub auto_index: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Use vector search (requires LanceDB)
    #[serde(default = "default_false")]
    pub use_vector_search: bool,

    /// BM25 k1 parameter
    #[serde(default = "default_bm25_k1")]
    pub bm25_k1: f32,

    /// BM25 b parameter
    #[serde(default = "default_bm25_b")]
    pub bm25_b: f32,

    /// Maximum search results
    #[serde(default = "default_max_results")]
    pub max_results: usize,

    /// Embedding model name
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: u32,

    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: u32,

    /// Theme: "dark", "light", "auto"
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: u16,

    /// Enable animations
    #[serde(default = "default_true")]
    pub animations: bool,

    /// Message history limit
    #[serde(default = "default_message_history")]
    pub message_history_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable RBAC permission system
    #[serde(default = "default_true")]
    pub enable_rbac: bool,

    /// Enable tool sandboxing
    #[serde(default = "default_true")]
    pub enable_sandboxing: bool,

    /// Require authentication
    #[serde(default = "default_false")]
    pub require_auth: bool,

    /// OAuth providers: "github", "google", "microsoft"
    #[serde(default)]
    pub oauth_providers: Vec<String>,

    /// Enable hardware authentication (Windows Hello, Touch ID)
    #[serde(default = "default_false")]
    pub enable_biometrics: bool,

    /// Allow reading from host directories
    #[serde(default = "default_true")]
    pub allow_host_read: bool,

    /// Allow writing to host directories
    #[serde(default = "default_false")]
    pub allow_host_write: bool,

    /// Allow USB/removable media access
    #[serde(default = "default_false")]
    pub allow_usb: bool,

    /// Auto-sync documents to host (if false, explicit copy only)
    #[serde(default = "default_false")]
    pub auto_sync_documents: bool,

    /// Allowed host directories (expand ~ and env vars)
    #[serde(default = "default_allowed_dirs")]
    pub allowed_host_dirs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// Enable telemetry (opt-in)
    #[serde(default = "default_false")]
    pub telemetry: bool,

    /// Log level: "error", "warn", "info", "debug", "trace"
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable crash reporting
    #[serde(default = "default_false")]
    pub crash_reporting: bool,

    /// Beta channel updates
    #[serde(default = "default_false")]
    pub beta_channel: bool,

    /// Worker threads (0 = auto)
    #[serde(default)]
    pub worker_threads: usize,
}

// Default value functions
fn default_llm_url() -> String {
    "http://localhost:8000".to_string()
}

fn default_model_size() -> String {
    "7b".to_string()
}

fn default_quantization() -> String {
    "4bit".to_string()
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_temperature() -> f32 {
    0.7
}

fn fallback_config_dir() -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".lucastra")
}

fn resolve_config_dir() -> PathBuf {
    if let Ok(dir) = env::var("LUCASTRA_CONFIG_HOME") {
        return PathBuf::from(dir);
    }

    dirs::config_dir()
        .map(|p| p.join("lucastra"))
        .unwrap_or_else(fallback_config_dir)
}

fn default_data_dir() -> PathBuf {
    resolve_config_dir().join("data")
}

fn default_cache_size() -> u64 {
    1024 // 1GB
}

fn default_bm25_k1() -> f32 {
    1.2
}

fn default_bm25_b() -> f32 {
    0.75
}

fn default_max_results() -> usize {
    10
}

fn default_embedding_model() -> String {
    "bge-small-en-v1.5".to_string()
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    800
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_font_size() -> u16 {
    16
}

fn default_message_history() -> usize {
    1000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_allowed_dirs() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            "~/Documents".to_string(),
            "~/Downloads".to_string(),
            "~/Desktop".to_string(),
        ]
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec![
            "~/Documents".to_string(),
            "~/Downloads".to_string(),
            "~/Desktop".to_string(),
        ]
    }
}

fn expand_allowed_dir(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(path)
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            server_url: default_llm_url(),
            auto_start: true,
            model_size: default_model_size(),
            auto_download: true,
            use_gpu: true,
            quantization: default_quantization(),
            streaming: true,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            use_host_fs: true,
            cache_size_mb: default_cache_size(),
            auto_index: true,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            use_vector_search: false,
            bm25_k1: default_bm25_k1(),
            bm25_b: default_bm25_b(),
            max_results: default_max_results(),
            embedding_model: default_embedding_model(),
        }
    }
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            window_width: default_window_width(),
            window_height: default_window_height(),
            theme: default_theme(),
            font_size: default_font_size(),
            animations: true,
            message_history_limit: default_message_history(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_rbac: true,
            enable_sandboxing: true,
            require_auth: false,
            oauth_providers: vec![],
            enable_biometrics: false,
            allow_host_read: true,
            allow_host_write: false,
            allow_usb: false,
            auto_sync_documents: false,
            allowed_host_dirs: default_allowed_dirs(),
        }
    }
}

impl SecurityConfig {
    /// Resolve allowed host directories, expanding ~ to the home directory
    pub fn resolved_allowed_dirs(&self) -> Vec<PathBuf> {
        self.allowed_host_dirs
            .iter()
            .map(|d| expand_allowed_dir(d))
            .collect()
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            telemetry: false,
            log_level: default_log_level(),
            crash_reporting: false,
            beta_channel: false,
            worker_threads: 0,
        }
    }
}

impl Config {
    /// Load configuration from file, or create default if not found
    pub fn load() -> Result<Self> {
        ensure_base_dirs()?;
        let config_path = get_config_file_path()?;

        if config_path.exists() {
            tracing::info!("Loading config from: {}", config_path.display());
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            tracing::info!(
                "Config not found, creating default at: {}",
                config_path.display()
            );
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        tracing::info!("Config saved to: {}", config_path.display());
        Ok(())
    }

    /// Reload configuration from file
    pub fn reload(&mut self) -> Result<()> {
        *self = Self::load()?;
        Ok(())
    }
}

/// Get the configuration directory (~/.lucastra)
pub fn get_config_dir() -> Result<PathBuf> {
    Ok(resolve_config_dir())
}

/// Get the configuration file path (~/.lucastra/config.toml)
pub fn get_config_file_path() -> Result<PathBuf> {
    Ok(resolve_config_dir().join("config.toml"))
}

/// Get the data directory (~/.lucastra/data)
pub fn get_data_dir() -> Result<PathBuf> {
    Ok(resolve_config_dir().join("data"))
}

/// Get the logs directory (~/.lucastra/logs)
pub fn get_logs_dir() -> Result<PathBuf> {
    Ok(resolve_config_dir().join("logs"))
}

/// Get the models directory (~/.lucastra/models)
pub fn get_models_dir() -> Result<PathBuf> {
    Ok(resolve_config_dir().join("models"))
}

/// Ensure the config root and common subdirectories exist
pub fn ensure_base_dirs() -> Result<()> {
    let base = resolve_config_dir();

    for dir in [
        base.clone(),
        base.join("data"),
        base.join("logs"),
        base.join("models"),
    ] {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.llm.server_url, "http://localhost:8000");
        assert_eq!(config.llm.model_size, "7b");
        assert!(config.llm.auto_start);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[llm]"));
        assert!(toml_str.contains("[storage]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [llm]
            server_url = "http://localhost:9000"
            model_size = "13b"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.llm.server_url, "http://localhost:9000");
        assert_eq!(config.llm.model_size, "13b");
    }

    #[test]
    fn test_env_override_config_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        env::set_var("LUCASTRA_CONFIG_HOME", temp.path());

        let dir = get_config_dir().unwrap();
        assert_eq!(dir, temp.path());
        assert_eq!(get_logs_dir().unwrap(), temp.path().join("logs"));

        env::remove_var("LUCASTRA_CONFIG_HOME");
    }

    #[test]
    fn test_ensure_base_dirs_creates_structure() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        env::set_var("LUCASTRA_CONFIG_HOME", temp.path());

        ensure_base_dirs().unwrap();

        assert!(temp.path().is_dir());
        assert!(temp.path().join("logs").is_dir());
        assert!(temp.path().join("data").is_dir());
        assert!(temp.path().join("models").is_dir());

        env::remove_var("LUCASTRA_CONFIG_HOME");
    }

    #[test]
    fn test_load_creates_config_file_in_custom_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        env::set_var("LUCASTRA_CONFIG_HOME", temp.path());

        let config = Config::load().unwrap();
        let config_file = temp.path().join("config.toml");

        assert!(config_file.exists());
        assert_eq!(config.llm.server_url, "http://localhost:8000");
        assert_eq!(config.llm.model_size, "7b");

        env::remove_var("LUCASTRA_CONFIG_HOME");
        std::fs::remove_file(config_file).ok();
        std::fs::remove_dir_all(temp.path().join("logs")).ok();
        std::fs::remove_dir_all(temp.path().join("data")).ok();
        std::fs::remove_dir_all(temp.path().join("models")).ok();
    }

    #[test]
    fn test_resolved_allowed_dirs_expands_tilde() {
        let cfg = SecurityConfig {
            allowed_host_dirs: vec!["~/Documents".to_string()],
            ..SecurityConfig::default()
        };

        let dirs = cfg.resolved_allowed_dirs();
        let expected_prefix = dirs::home_dir().unwrap();

        assert!(dirs[0].starts_with(expected_prefix));
    }
}
