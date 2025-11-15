/// Integration tests for test execution and plan execution modules
/// Tests the complete test execution pipeline and cross-module coordination
use smart_hooks::{create_test_plan_with_config, execution::test_runner, TestSelectorConfig};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_executable_rust_project(temp_dir: &Path) -> Result<(), std::io::Error> {
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create Cargo.toml
    fs::write(
        temp_dir.join("Cargo.toml"),
        r#"
[package]
name = "execution-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.0"
"#,
    )?;

    // Create lib.rs with tests
    fs::write(
        src_dir.join("lib.rs"),
        r#"
//! Execution test project

use anyhow::Result;

pub mod calculator;
pub mod validator;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn process_input(input: &str) -> Result<String> {
    if input.is_empty() {
        anyhow::bail!("Empty input");
    }
    Ok(format!("Processed: {}", input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_process_input() {
        assert!(process_input("hello").is_ok());
        assert!(process_input("").is_err());
    }

    #[test]
    fn test_integration_workflow() {
        let input = "test data";
        let result = process_input(input).unwrap();
        assert!(result.contains("test data"));
    }
}
"#,
    )?;

    // Create calculator module
    fs::write(
        src_dir.join("calculator.rs"),
        r#"
//! Calculator functionality

pub struct Calculator {
    precision: usize,
}

impl Calculator {
    pub fn new(precision: usize) -> Self {
        Self { precision }
    }

    pub fn multiply(&self, a: f64, b: f64) -> f64 {
        let result = a * b;
        let factor = 10_f64.powi(self.precision as i32);
        (result * factor).round() / factor
    }

    pub fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            return Err("Division by zero".to_string());
        }
        Ok(self.multiply(a, 1.0 / b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_multiply() {
        let calc = Calculator::new(2);
        assert_eq!(calc.multiply(2.5, 4.0), 10.0);
    }

    #[test]
    fn test_calculator_divide() {
        let calc = Calculator::new(2);
        assert_eq!(calc.divide(10.0, 2.0).unwrap(), 5.0);
        assert!(calc.divide(10.0, 0.0).is_err());
    }

    #[test]
    fn test_precision() {
        let calc = Calculator::new(3);
        let result = calc.multiply(1.0 / 3.0, 3.0);
        assert!((result - 1.0).abs() < 0.001);
    }
}
"#,
    )?;

    // Create validator module
    fs::write(
        src_dir.join("validator.rs"),
        r#"
//! Input validation functionality

use anyhow::{Result, bail};

pub struct Validator {
    max_length: usize,
}

impl Validator {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }

    pub fn validate_email(&self, email: &str) -> Result<()> {
        if email.is_empty() {
            bail!("Email cannot be empty");
        }

        if email.len() > self.max_length {
            bail!("Email too long");
        }

        if !email.contains('@') {
            bail!("Invalid email format");
        }

        Ok(())
    }

    pub fn validate_password(&self, password: &str) -> Result<()> {
        if password.len() < 6 {
            bail!("Password too short");
        }

        if password.len() > self.max_length {
            bail!("Password too long");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new(50);

        assert!(validator.validate_email("test@example.com").is_ok());
        assert!(validator.validate_email("").is_err());
        assert!(validator.validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_password_validation() {
        let validator = Validator::new(50);

        assert!(validator.validate_password("validpassword").is_ok());
        assert!(validator.validate_password("123").is_err()); // Too short
    }

    #[test]
    fn test_length_limits() {
        let validator = Validator::new(10);

        assert!(validator.validate_email("test@x.com").is_ok());
        assert!(validator.validate_email("very-long-email@example.com").is_err());
    }
}
"#,
    )?;

    // Create integration tests
    let tests_dir = temp_dir.join("tests");
    fs::create_dir_all(&tests_dir)?;

    fs::write(
        tests_dir.join("integration_test.rs"),
        r#"
//! Integration tests

use execution_test_project::{add, process_input};
use execution_test_project::calculator::Calculator;
use execution_test_project::validator::Validator;

#[test]
fn test_full_workflow() {
    // Test basic math
    let result = add(5, 3);
    assert_eq!(result, 8);

    // Test processing
    let processed = process_input("integration test").unwrap();
    assert!(processed.contains("integration test"));
}

#[test]
fn test_calculator_integration() {
    let calc = Calculator::new(2);
    let result = calc.multiply(2.5, 4.0);
    assert_eq!(result, 10.0);
}

#[test]
fn test_validator_integration() {
    let validator = Validator::new(100);
    assert!(validator.validate_email("integration@test.com").is_ok());
    assert!(validator.validate_password("integration_password").is_ok());
}
"#,
    )?;

    Ok(())
}

#[test]
fn test_test_plan_creation() {
    let temp_dir = TempDir::new().unwrap();
    create_executable_rust_project(temp_dir.path()).unwrap();

    // Test files that should trigger tests
    let changed_files = vec![
        "src/calculator.rs".to_string(),
        "src/validator.rs".to_string(),
    ];

    let config = TestSelectorConfig::path_based_only();
    let plan = create_test_plan_with_config(&changed_files, &config).unwrap();

    // Verify plan creation
    assert!(!plan.unit_tests.is_empty());
    assert!(plan
        .unit_tests
        .iter()
        .any(|test| test.contains("calculator")));
    assert!(plan
        .unit_tests
        .iter()
        .any(|test| test.contains("validator")));
}

#[test]
fn test_cargo_command_execution() {
    // Test basic cargo commands that should always work
    let test_cases = vec![
        (vec!["--version"], "Cargo version should work"),
        (vec!["help"], "Cargo help should work"),
        (vec!["help", "test"], "Cargo help test should work"),
    ];

    for (args, description) in test_cases {
        let result = test_runner::run_cargo_command(&args, None);
        assert!(result.is_ok(), "{}: {:?}", description, args);
        assert!(result.unwrap(), "{}: command should succeed", description);
    }
}

#[test]
fn test_cargo_command_with_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_executable_rust_project(temp_dir.path()).unwrap();

    // Test cargo command in specific directory
    let result = test_runner::run_cargo_command(&["check"], Some(temp_dir.path()));

    // Should attempt to run cargo check in the test directory
    assert!(result.is_ok());
    // Note: May fail if dependencies aren't available, but the command execution should work
}

#[test]
fn test_invalid_cargo_commands() {
    let invalid_commands = vec![
        vec!["nonexistent-command"],
        vec!["test", "--invalid-flag-that-does-not-exist"],
        vec!["build", "--target", "invalid-target-triple"],
    ];

    for args in invalid_commands {
        let result = test_runner::run_cargo_command(&args, None);
        assert!(result.is_ok(), "Should not panic on invalid commands");
        // Should return false indicating failure
        assert!(!result.unwrap(), "Invalid command should fail: {:?}", args);
    }
}

#[test]
fn test_test_plan_with_different_configurations() {
    let test_files = vec![
        "src/core/business_logic.rs".to_string(),
        "src/api/handlers.rs".to_string(),
        "src/utils/helpers.rs".to_string(),
    ];

    // Test with path-based configuration
    let path_config = TestSelectorConfig::path_based_only();
    let path_plan = create_test_plan_with_config(&test_files, &path_config).unwrap();

    // Test with different configuration
    let custom_config = TestSelectorConfig::default();

    let custom_plan = create_test_plan_with_config(&test_files, &custom_config).unwrap();

    // Plans should be different based on configuration
    assert!(!path_plan.unit_tests.is_empty());
    assert!(!custom_plan.unit_tests.is_empty());

    // Custom config should trigger BDD for API files
    assert!(custom_plan.bdd_tests);
}

#[test]
fn test_test_runner_with_workspace() {
    let temp_dir = TempDir::new().unwrap();

    // Create simple workspace
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[workspace]
members = ["member"]

[package]
name = "workspace-test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let member_dir = temp_dir.path().join("member");
    fs::create_dir_all(member_dir.join("src")).unwrap();

    fs::write(
        member_dir.join("Cargo.toml"),
        r#"
[package]
name = "member"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        member_dir.join("src/lib.rs"),
        r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_workspace_member() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    )
    .unwrap();

    // Test running cargo commands in workspace
    let result = test_runner::run_cargo_command(&["check"], Some(temp_dir.path()));
    assert!(result.is_ok());

    // Test workspace-specific commands
    let result = test_runner::run_cargo_command(
        &["metadata", "--format-version", "1"],
        Some(temp_dir.path()),
    );
    assert!(result.is_ok());
}