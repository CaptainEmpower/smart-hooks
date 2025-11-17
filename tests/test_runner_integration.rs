use smart_hooks::execution::test_runner;

/// Integration tests for test runner functionality
/// Tests actual cargo command execution

#[test]
fn test_run_cargo_version() {
    // Simple test that should always work
    let result = test_runner::run_cargo_command(&["--version"], None);
    assert!(result.is_ok());
    assert!(result.unwrap()); // cargo --version should succeed
}

#[test]
fn test_run_invalid_cargo_command() {
    // Test with invalid command
    let result = test_runner::run_cargo_command(&["invalid-command"], None);
    assert!(result.is_ok()); // Command runs but returns false
    assert!(!result.unwrap()); // Should fail
}

#[test]
fn test_run_cargo_help() {
    // Another command that should work
    let result = test_runner::run_cargo_command(&["help"], None);
    assert!(result.is_ok());
    assert!(result.unwrap()); // cargo help should succeed
}