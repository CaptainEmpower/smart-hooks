use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Test execution utilities
/// Focused on running cargo commands safely
/// Run a cargo command with specified arguments and working directory
pub fn run_cargo_command(args: &[&str], cwd: Option<&Path>) -> Result<bool> {
    let mut command = Command::new("cargo");
    command.args(args);

    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let output = command.output()?;
    Ok(output.status.success())
}

/// Run unit test for specific module
pub fn run_unit_test(module_name: &str) -> Result<bool> {
    run_cargo_command(
        &["test", module_name, "--lib", "--quiet"],
        None, // Use current directory since we're now standalone
    )
}

/// Run every integration-test target the project defines.
///
/// `--tests` rather than `--test <name>`: naming one target made the hook fail
/// in any project that does not happen to define a target by that name, which
/// is the same defect that `--test cucumber_tests` had. With `--tests`, a
/// project with no integration targets is a no-op success.
pub fn run_integration_tests() -> Result<bool> {
    run_cargo_command(
        &["test", "--tests", "--quiet"],
        None, // Use current directory since we're now standalone
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_cargo_command_help() {
        // Test with a simple cargo command that should always work
        let result = run_cargo_command(&["--version"], None);
        assert!(result.is_ok());
        assert!(result.unwrap()); // cargo --version should succeed
    }

    // Note: Other tests would require actual test environment
    // Integration tests will cover the full functionality
}
