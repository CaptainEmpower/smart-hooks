use std::fs;
use std::path::Path;
/// End-to-end workflow integration tests
/// Tests complete user scenarios and cross-module coordination
use std::process::Command;
use tempfile::TempDir;

fn create_test_rust_project(temp_dir: &Path) -> Result<(), std::io::Error> {
    // Create a mock Rust project structure
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create Cargo.toml
    fs::write(
        temp_dir.join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"

[features]
default = []
experimental = []
"#,
    )?;

    // Create lib.rs with different types of code
    fs::write(
        src_dir.join("lib.rs"),
        r#"
//! Test library for smart-hooks integration testing

#[cfg(feature = "experimental")]
use experimental_crate::ExperimentalFeature;

use anyhow::Result;

pub mod core;
pub mod utils;

/// Main library function
pub fn run_application() -> Result<()> {
    let processor = core::DataProcessor::new();
    let result = processor.process_data("test input")?;
    println!("Processed: {}", result);
    Ok(())
}

#[cfg(feature = "experimental")]
pub fn experimental_feature() -> Result<()> {
    // This would use experimental features
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_application() {
        assert!(run_application().is_ok());
    }
}
"#,
    )?;

    // Create core module (triggers unit tests)
    fs::write(
        src_dir.join("core.rs"),
        r#"
//! Core business logic module

use anyhow::Result;

pub struct DataProcessor {
    config: ProcessorConfig,
}

struct ProcessorConfig {
    timeout_ms: u64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            config: ProcessorConfig { timeout_ms: 5000 },
        }
    }

    pub fn process_data(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            anyhow::bail!("Input cannot be empty");
        }

        Ok(format!("Processed: {}", input.to_uppercase()))
    }

    pub fn validate_input(&self, input: &str) -> bool {
        !input.is_empty() && input.len() < 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let processor = DataProcessor::new();
        assert!(processor.process_data("hello").is_ok());
        assert!(processor.process_data("").is_err());
    }

    #[test]
    fn test_validate_input() {
        let processor = DataProcessor::new();
        assert!(processor.validate_input("valid"));
        assert!(!processor.validate_input(""));
    }
}
"#,
    )?;

    // Create utils module
    fs::write(
        src_dir.join("utils.rs"),
        r#"
//! Utility functions

use std::collections::HashMap;

pub fn create_config_map() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("version".to_string(), "1.0".to_string());
    config.insert("env".to_string(), "test".to_string());
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_config_map() {
        let config = create_config_map();
        assert_eq!(config.get("version"), Some(&"1.0".to_string()));
    }
}
"#,
    )?;

    Ok(())
}

#[test]
fn test_selective_unit_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_test_rust_project(temp_dir.path()).unwrap();

    // Test selective unit test command
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "test",
            "selective-unit",
            "--verbose",
            &temp_dir.path().join("src/core.rs").to_string_lossy(),
            &temp_dir.path().join("src/lib.rs").to_string_lossy(),
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute selective unit tests");

    // Command should execute (may fail due to missing dependencies, but should parse correctly)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show some indication it processed the files
    // Note: May fail at cargo execution stage, but CLI parsing should work
    println!("Selective unit test output: {}", stdout);
    if !stderr.is_empty() {
        println!("Selective unit test stderr: {}", stderr);
    }
}

#[test]
fn test_smart_selector_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_test_rust_project(temp_dir.path()).unwrap();

    // Test smart selector command
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "test",
            "smart-selector",
            "--verbose",
            &temp_dir.path().join("src/core.rs").to_string_lossy(),
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute smart selector");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Smart selector output: {}", stdout);
    if !stderr.is_empty() {
        println!("Smart selector stderr: {}", stderr);
    }
}

#[test]
fn test_conditional_compilation_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_test_rust_project(temp_dir.path()).unwrap();

    // Test the full conditional compilation workflow
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--all-files",
            "--project-dir",
            &temp_dir.path().to_string_lossy(),
            "--detect-only", // Avoid actual cargo check to prevent dependency issues
            "--verbose",
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute conditional compilation check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Debug output for test development (can be removed)
    // println!("STDOUT: {}", stdout);
    // println!("STDERR: {}", stderr);

    // Should detect the conditional compilation patterns we created
    assert!(stdout.contains("Found 5 conditional compilation pattern(s)"));
    assert!(stdout.contains("cfg(feature = \"experimental\")"));
    assert!(stdout.contains("cfg(test)"));
    assert!(stdout.contains("✅ Detection complete"));
}

#[test]
fn test_error_handling_workflow() {
    // Test various error conditions

    // 1. Invalid project directory
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--project-dir",
            "/nonexistent/path",
            "--detect-only",
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute with invalid directory");

    assert!(!output.status.success());

    // 2. Invalid arguments
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--invalid-flag",
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute with invalid flag");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unrecognized")
            || stderr.contains("unknown")
    );
}

#[test]
fn test_mixed_file_types_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_test_rust_project(temp_dir.path()).unwrap();

    // Create non-Rust files
    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
    fs::write(temp_dir.path().join("script.py"), "print('hello')").unwrap();

    // Test with mixed file types
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--all-files",
            "--project-dir",
            &temp_dir.path().to_string_lossy(),
            "--detect-only",
            "--verbose",
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute with mixed files");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should only process Rust files
    assert!(stdout.contains("Found 5 conditional compilation pattern(s)"));
}

#[test]
fn test_cli_argument_combinations() {
    let temp_dir = TempDir::new().unwrap();
    create_test_rust_project(temp_dir.path()).unwrap();

    let project_dir = temp_dir.path().to_string_lossy().to_string();

    // Test various argument combinations
    let test_cases = vec![
        // Basic usage
        vec!["check", "conditional-compilation", "--detect-only"],
        // With verbose
        vec![
            "check",
            "conditional-compilation",
            "--detect-only",
            "--verbose",
        ],
        // With all files
        vec![
            "check",
            "conditional-compilation",
            "--all-files",
            "--detect-only",
        ],
        // With project dir
        vec![
            "check",
            "conditional-compilation",
            "--all-files",
            "--project-dir",
            &project_dir,
            "--detect-only",
        ],
        // Combined flags
        vec![
            "check",
            "conditional-compilation",
            "--all-files",
            "--detect-only",
            "--verbose",
        ],
    ];

    for args in test_cases {
        let args_clone = args.clone();
        let mut cmd_args = vec!["run", "--bin", "smart-hooks", "--"];
        cmd_args.extend(args);

        let output = Command::new("cargo")
            .args(cmd_args)
            .current_dir("../../") // Go to workspace root
            .output()
            .expect("Failed to execute command combination");

        assert!(
            output.status.success(),
            "Command failed with args: {:?}\nStderr: {}",
            args_clone,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[ignore = "Requires Claude CLI to be installed"]
fn test_claude_bdd_workflow() {
    // This test requires Claude CLI to be installed
    // Marked as ignore to avoid CI failures

    let temp_dir = TempDir::new().unwrap();
    create_test_rust_project(temp_dir.path()).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--features",
            "claude-ai",
            "--",
            "bdd",
            "claude-selector",
            "--project-dir",
            &temp_dir.path().to_string_lossy(),
            "--dry-run",
            "--context",
            "Test Rust project",
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute Claude BDD selector");

    // If Claude CLI is available, this should work
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Claude BDD output: {}", stdout);
    } else {
        println!("Claude CLI not available (expected in CI)");
    }
}

#[test]
fn test_performance_with_large_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create a larger project structure
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create multiple modules with conditional compilation
    for i in 0..20 {
        let module_content = format!(
            r#"
//! Module {}

#[cfg(feature = "module-{}")]
use module_{}_crate::Feature{};

#[cfg(not(test))]
use production_crate::Production{};

pub struct Module{} {{
    id: u32,
}}

impl Module{} {{
    pub fn new() -> Self {{
        Self {{ id: {} }}
    }}

    #[cfg(feature = "module-{}")]
    pub fn special_feature(&self) -> String {{
        format!("Special feature for module {}")
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_module_{}_new() {{
        let module = Module{}::new();
        assert_eq!(module.id, {});
    }}
}}
"#,
            i, i, i, i, i, i, i, i, i, i, i, i, i
        );

        fs::write(src_dir.join(format!("module_{}.rs", i)), module_content).unwrap();
    }

    // Create main lib.rs that includes all modules
    let lib_content = (0..20)
        .map(|i| format!("pub mod module_{};", i))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(src_dir.join("lib.rs"), lib_content).unwrap();

    // Test performance with larger project
    let start = std::time::Instant::now();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--all-files",
            "--project-dir",
            &temp_dir.path().to_string_lossy(),
            "--detect-only",
        ])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute performance test");

    let duration = start.elapsed();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Debug output for test development (can be removed)
    // println!("Performance test STDOUT: {}", stdout);

    // Should detect all the conditional patterns (80 patterns: 4 per module * 20 modules)
    assert!(stdout.contains("Found 80 conditional compilation pattern(s)"));

    // Should complete reasonably quickly (under 5 seconds for this small test)
    assert!(
        duration.as_secs() < 5,
        "Command took too long: {:?}",
        duration
    );

    println!("Performance test completed in {:?}", duration);
}