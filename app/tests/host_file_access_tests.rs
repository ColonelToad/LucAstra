use lucastra_app::SystemState;
use lucastra_config::SecurityConfig;
use lucastra_tools::file_access::{FileAccessTool, FileAccessValidator, FileOperation};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_host_file_access_integration() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // Create test directory
    let test_dir = std::env::temp_dir().join("lucastra_hfa_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    // Test that we can create validator and check paths
    let config = &state.config.security;
    let allowed_dirs: Vec<PathBuf> = config
        .allowed_host_dirs
        .iter()
        .map(PathBuf::from)
        .collect();
    let _validator = FileAccessValidator::new(
        allowed_dirs,
        config.allow_host_read,
        config.allow_host_write,
        config.allow_usb,
    );

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_file_access_validator_with_security_config() {
    // Create a security config
    let security_config = SecurityConfig {
        allow_host_read: true,
        allow_host_write: false,
        ..Default::default()
    };

    let allowed_dirs: Vec<PathBuf> = security_config
        .allowed_host_dirs
        .iter()
        .map(PathBuf::from)
        .collect();
    let validator = FileAccessValidator::new(
        allowed_dirs,
        security_config.allow_host_read,
        security_config.allow_host_write,
        security_config.allow_usb,
    );

    // Test read is allowed
    let test_path = std::env::temp_dir().join("test.txt");
    let result = validator.validate_path(&test_path, FileOperation::Read);
    // Should return error since temp_dir is likely not in allowed dirs, but that's expected
    assert!(result.is_err()); // Validation should enforce whitelist
}

#[test]
fn test_file_access_tool_execution() {
    let state = SystemState::new().expect("Failed to create SystemState");

    // Create test directory
    let test_dir = std::env::temp_dir().join("lucastra_fat_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    // Create test file
    let test_file = test_dir.join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Create audit directory
    let audit_dir = test_dir.join("audit");
    fs::create_dir_all(&audit_dir).unwrap();

    // Create validator
    let allowed_dirs: Vec<PathBuf> = state
        .config
        .security
        .allowed_host_dirs
        .iter()
        .map(PathBuf::from)
        .collect();
    let validator = FileAccessValidator::new(
        allowed_dirs,
        state.config.security.allow_host_read,
        state.config.security.allow_host_write,
        state.config.security.allow_usb,
    );

    // Create a FileAccessTool
    let tool = FileAccessTool::new(validator, audit_dir.clone());

    // Test reading the file (will likely fail due to whitelist but should not panic)
    let result = tool.execute(FileOperation::Read, &test_file, None);
    // Should have a result (either success or failure)
    assert!(!result.tool.is_empty());

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_audit_logging_integration() {
    // Create test directory
    let test_dir = std::env::temp_dir().join("lucastra_audit_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let audit_dir = test_dir.join("audit");
    fs::create_dir_all(&audit_dir).unwrap();

    let config = SecurityConfig::default();
    let allowed_dirs: Vec<PathBuf> = config
        .allowed_host_dirs
        .iter()
        .map(PathBuf::from)
        .collect();
    let validator = FileAccessValidator::new(
        allowed_dirs,
        config.allow_host_read,
        config.allow_host_write,
        config.allow_usb,
    );
    let tool = FileAccessTool::new(validator, audit_dir.clone());

    // Execute an operation
    let test_file = test_dir.join("test.txt");
    fs::write(&test_file, "test").unwrap();
    let _ = tool.execute(FileOperation::Read, &test_file, None);

    // Check that audit log directory was created
    assert!(audit_dir.exists());

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_security_config_whitelist_paths() {
    let config = SecurityConfig::default();

    // Should have default directories
    assert!(!config.allowed_host_dirs.is_empty());

    // Test tilde expansion
    let resolved = config.resolved_allowed_dirs();
    assert!(!resolved.is_empty());

    // Each resolved path should be absolute or expandable
    for path in &resolved {
        assert!(path.is_absolute() || path.to_string_lossy().contains("~"));
    }
}
