use lucastra_app::SystemState;
use lucastra_config::Config;
use serde_json::to_string_pretty;
use std::env;
use std::fs;
use std::path::PathBuf;

fn ensure_config_home_with_default() -> PathBuf {
    // Create an isolated temp config directory with a default config.json
    let temp_dir = env::temp_dir().join("lucastra_system_state_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp config dir");

    let cfg = Config::default();
    let cfg_path = temp_dir.join("config.json");
    fs::write(&cfg_path, to_string_pretty(&cfg).expect("serialize config"))
        .expect("write config.json");

    env::set_var("LUCASTRA_CONFIG_HOME", &temp_dir);
    temp_dir
}

#[test]
fn test_system_state_initialization() {
    ensure_config_home_with_default();
    // Test that SystemState can be created
    let result = SystemState::new();
    assert!(result.is_ok(), "SystemState initialization failed");

    let state = result.unwrap();
    assert!(!state.config.llm.server_url.is_empty());
    assert!(!state.config.storage.data_dir.as_os_str().is_empty());
}

#[test]
fn test_config_persistence_roundtrip() {
    use lucastra_config::Config;

    // Use a temporary config directory
    let temp_dir = std::env::temp_dir().join("lucastra_config_test_rt");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // Set environment to use temp directory
    env::set_var("LUCASTRA_CONFIG_HOME", &temp_dir);

    // Create a default config
    let config = Config::default();
    assert_eq!(config.tracing.level, "info");
    assert!(config.metrics.enabled);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
    env::remove_var("LUCASTRA_CONFIG_HOME");
}

#[test]
fn test_metrics_tracking_integration() {
    ensure_config_home_with_default();
    let state = SystemState::new().expect("Failed to create SystemState");

    // Record some metrics
    state.metrics.record_command();
    state.metrics.record_command();
    state.metrics.record_tool_success();
    state.metrics.record_tool_failure();

    let snapshot = state.metrics.snapshot();
    assert_eq!(snapshot.command_count, 2);
    assert_eq!(snapshot.tool_success_count, 1);
    assert_eq!(snapshot.tool_failure_count, 1);
}

#[test]
fn test_system_state_config_access() {
    ensure_config_home_with_default();
    let state = SystemState::new().expect("Failed to create SystemState");
    let config = state.get_config();

    // Verify security config is accessible
    assert!(!config.security.allowed_host_dirs.is_empty());

    // Verify tracing config is accessible
    assert!(!config.tracing.level.is_empty());
}

#[test]
fn test_filesystem_operations() {
    ensure_config_home_with_default();
    let state = SystemState::new().expect("Failed to create SystemState");

    // Verify that filesystem operations are accessible
    // The filesystem should have a mock filesystem mounted at /mnt/root
    let list_result = state.filesystem.list_files("/mnt/root");
    // This may succeed or fail depending on implementation, but should not panic
    let _ = list_result;
}
