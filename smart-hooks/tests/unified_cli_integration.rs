use std::fs;
/// Integration tests for the unified smart-hooks CLI
/// Tests end-to-end command execution and workflows
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_cli_help_commands() {
    // Test main help
    let output = Command::new("cargo")
        .args(["run", "--bin", "smart-hooks", "--", "--help"])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute smart-hooks");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("smart-hooks"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("bdd"));
    assert!(stdout.contains("check"));
}

#[test]
fn test_subcommand_help() {
    // Test test subcommand help
    let output = Command::new("cargo")
        .args(["run", "--bin", "smart-hooks", "--", "test", "--help"])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute smart-hooks test help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("selective-unit"));
    assert!(stdout.contains("integration"));
    assert!(stdout.contains("smart-selector"));

    // Test check subcommand help
    let output = Command::new("cargo")
        .args(["run", "--bin", "smart-hooks", "--", "check", "--help"])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute smart-hooks check help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("conditional-compilation"));

    // Test bdd subcommand help
    let output = Command::new("cargo")
        .args(["run", "--bin", "smart-hooks", "--", "bdd", "--help"])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute smart-hooks bdd help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("claude-selector"));
}

#[test]
fn test_conditional_compilation_detect_only() {
    let temp_dir = TempDir::new().unwrap();

    // Create a Rust file with conditional compilation
    let test_file = temp_dir.path().join("test.rs");
    fs::write(
        &test_file,
        r#"
#[cfg(feature = "test-feature")]
use some_crate::TestFeature;

#[cfg(not(test))]
use another_crate::Production;

fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    // Test detect-only mode
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
        .expect("Failed to execute conditional compilation check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect the conditional patterns
    assert!(stdout.contains("Found 2 conditional compilation pattern(s)"));
    assert!(stdout.contains("cfg(feature = \"test-feature\")"));
    assert!(stdout.contains("cfg(not(test))"));
    assert!(stdout.contains("✅ Detection complete"));
}

#[test]
fn test_conditional_compilation_no_patterns() {
    let temp_dir = TempDir::new().unwrap();

    // Create a Rust file without conditional compilation
    let test_file = temp_dir.path().join("simple.rs");
    fs::write(
        &test_file,
        r#"
use std::collections::HashMap;

fn main() {
    let map: HashMap<String, i32> = HashMap::new();
    println!("Map created: {:?}", map);
}
"#,
    )
    .unwrap();

    // Test detect-only mode
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
        .expect("Failed to execute conditional compilation check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find no patterns
    assert!(stdout.contains("⏭️  No conditional compilation patterns found"));
}

#[test]
fn test_legacy_binary_compatibility() {
    // Ensure legacy binaries still work during transition
    let binaries = [
        "selective-unit-tests",
        "integration-tests-if-needed",
        "smart-test-selector",
        "conditional-compilation-checker",
    ];

    for binary in binaries {
        let output = Command::new("cargo")
            .args(["run", "--bin", binary, "--", "--help"])
            .current_dir("../../") // Go to workspace root
            .output();

        // Should either succeed or fail gracefully
        match output {
            Ok(output) => {
                // If it runs, it should show help
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    stdout.contains("help") || stderr.contains("Usage") || stderr.contains("USAGE"),
                    "Binary {} should show help information",
                    binary
                );
            }
            Err(e) => {
                println!("Binary {} failed to run (acceptable): {}", binary, e);
                // This is acceptable as some binaries may not be built
            }
        }
    }
}

#[test]
fn test_invalid_subcommand() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "smart-hooks", "--", "invalid-command"])
        .current_dir("../../") // Go to workspace root
        .output()
        .expect("Failed to execute smart-hooks with invalid command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show error about unrecognized subcommand
    assert!(
        stderr.contains("unexpected argument")
            || stderr.contains("unrecognized subcommand")
            || stderr.contains("invalid")
    );
}

#[test]
fn test_missing_required_args() {
    // Test commands that require arguments but don't get them
    let test_cases = [
        // No subcommand after "test"
        vec!["run", "--bin", "smart-hooks", "--", "test"],
        // No subcommand after "check"
        vec!["run", "--bin", "smart-hooks", "--", "check"],
        // No subcommand after "bdd"
        vec!["run", "--bin", "smart-hooks", "--", "bdd"],
    ];

    for args in test_cases {
        let output = Command::new("cargo")
            .args(args)
            .current_dir("../../") // Go to workspace root
            .output()
            .expect("Failed to execute smart-hooks");

        assert!(
            !output.status.success(),
            "Command should fail without subcommand"
        );
    }
}

#[test]
#[ignore = "Requires actual git repository setup"]
fn test_staged_files_integration() {
    // This test would require setting up a real git repository
    // and staging files to test the git integration
    // Marked as ignore to avoid CI issues but can be run manually

    let _temp_dir = TempDir::new().unwrap();

    // Would set up git repo, create files, stage them
    // Then test that conditional compilation checker finds staged files

    println!("Integration test with git staging requires manual setup");
}

#[test]
fn test_workspace_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a mock workspace
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"
[workspace]
members = ["crate1", "crate2"]

[package]
name = "test-workspace"
version = "0.1.0"
"#,
    )
    .unwrap();

    // Create member directories and Cargo.toml files
    let crate1_dir = temp_dir.path().join("crate1");
    fs::create_dir_all(&crate1_dir).unwrap();
    let crate1_toml = crate1_dir.join("Cargo.toml");
    fs::write(
        &crate1_toml,
        r#"
[package]
name = "crate1"
version = "0.1.0"
"#,
    )
    .unwrap();

    let crate2_dir = temp_dir.path().join("crate2");
    fs::create_dir_all(&crate2_dir).unwrap();
    let crate2_toml = crate2_dir.join("Cargo.toml");
    fs::write(
        &crate2_toml,
        r#"
[package]
name = "crate2"
version = "0.1.0"
"#,
    )
    .unwrap();

    // Create a Rust file with conditional compilation in crate1
    let crate1_src = crate1_dir.join("src");
    fs::create_dir_all(&crate1_src).unwrap();
    let lib_rs = crate1_src.join("lib.rs");
    fs::write(
        &lib_rs,
        r#"
#[cfg(feature = "workspace-feature")]
use some_crate::WorkspaceFeature;

pub fn hello() {
    println!("Hello from crate1");
}
"#,
    )
    .unwrap();

    // Test workspace detection (detect-only to avoid cargo errors)
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
        .expect("Failed to execute conditional compilation check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect patterns and mention workspace checking
    assert!(stdout.contains("Found 1 conditional compilation pattern"));
    assert!(stdout.contains("cfg(feature = \"workspace-feature\")"));
}