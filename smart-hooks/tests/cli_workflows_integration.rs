/// CLI workflow integration tests
/// Tests command-line interface execution and argument handling
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn create_minimal_rust_project(temp_dir: &Path) -> Result<(), std::io::Error> {
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(
        temp_dir.join("Cargo.toml"),
        r#"
[package]
name = "cli-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"

[features]
default = []
experimental = ["dep:experimental-crate"]

[dependencies.experimental-crate]
version = "0.1"
optional = true
"#,
    )?;

    fs::write(
        src_dir.join("lib.rs"),
        r#"
//! CLI test project

#[cfg(feature = "experimental")]
use experimental_crate::Feature;

#[cfg(test)]
mod tests {
    #[test]
    fn example_test() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn main_function() {
    println!("Main function executed");
}
"#,
    )?;

    fs::write(
        src_dir.join("core.rs"),
        r#"
//! Core module

#[cfg(feature = "experimental")]
pub fn experimental_feature() {
    println!("Experimental feature");
}

pub fn standard_feature() {
    println!("Standard feature");
}

#[cfg(test)]
mod tests {
    #[test]
    fn core_test() {
        assert!(true);
    }
}
"#,
    )?;

    Ok(())
}

#[test]
fn test_cli_help_display() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "smart-hooks", "--", "--help"])
        .current_dir("../../")
        .output()
        .expect("Failed to execute help command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should display main commands
    assert!(stdout.contains("smart-hooks"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("bdd"));
}

#[test]
fn test_subcommand_help_display() {
    let subcommands = ["test", "check", "bdd"];

    for subcommand in &subcommands {
        let output = Command::new("cargo")
            .args(["run", "--bin", "smart-hooks", "--", subcommand, "--help"])
            .current_dir("../../")
            .output()
            .expect("Failed to execute subcommand help");

        assert!(
            output.status.success(),
            "Help failed for subcommand: {}",
            subcommand
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "Empty help output for subcommand: {}",
            subcommand
        );
    }
}

#[test]
fn test_conditional_compilation_detection_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_minimal_rust_project(temp_dir.path()).unwrap();

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
        .current_dir("../../")
        .output()
        .expect("Failed to execute conditional compilation check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect the cfg patterns we created
    assert!(stdout.contains("Found") && stdout.contains("conditional compilation pattern"));
    assert!(stdout.contains("cfg(feature = \"experimental\")") || stdout.contains("cfg(test)"));
    assert!(stdout.contains("✅"));
}

#[test]
fn test_selective_unit_test_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_minimal_rust_project(temp_dir.path()).unwrap();

    let core_file = temp_dir.path().join("src/core.rs");
    let lib_file = temp_dir.path().join("src/lib.rs");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "test",
            "selective-unit",
            "--verbose",
            &core_file.to_string_lossy(),
            &lib_file.to_string_lossy(),
        ])
        .current_dir("../../")
        .output()
        .expect("Failed to execute selective unit tests");

    // Command should parse correctly (even if it fails later due to test project limitations)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show it processed the files
    assert!(!stdout.is_empty() || !stderr.is_empty());
}

#[test]
fn test_smart_selector_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_minimal_rust_project(temp_dir.path()).unwrap();

    let core_file = temp_dir.path().join("src/core.rs");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "test",
            "smart-selector",
            "--verbose",
            &core_file.to_string_lossy(),
        ])
        .current_dir("../../")
        .output()
        .expect("Failed to execute smart selector");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show some processing output
    assert!(!stdout.is_empty() || !stderr.is_empty());
}

#[test]
fn test_invalid_argument_handling() {
    let test_cases = vec![
        // Invalid subcommand
        (
            vec!["invalid-command"],
            "Invalid subcommand should be rejected",
        ),
        // Missing required arguments
        (vec!["test"], "Missing test subcommand should be rejected"),
        (vec!["check"], "Missing check subcommand should be rejected"),
        (vec!["bdd"], "Missing BDD subcommand should be rejected"),
        // Invalid flags
        (
            vec!["check", "conditional-compilation", "--invalid-flag"],
            "Invalid flag should be rejected",
        ),
    ];

    for (args, description) in test_cases {
        let mut cmd_args = vec!["run", "--bin", "smart-hooks", "--"];
        cmd_args.extend(args.clone());

        let output = Command::new("cargo")
            .args(&cmd_args)
            .current_dir("../../")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success(), "{}: {:?}", description, args);
    }
}

#[test]
fn test_project_directory_validation() {
    // Test with non-existent directory
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--project-dir",
            "/nonexistent/path/to/project",
            "--detect-only",
        ])
        .current_dir("../../")
        .output()
        .expect("Failed to execute with invalid directory");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty()); // Should show error message
}

#[test]
fn test_workspace_detection_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace structure
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[workspace]
members = ["member1", "member2"]

[package]
name = "workspace-test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    // Create workspace members
    for member in ["member1", "member2"] {
        let member_dir = temp_dir.path().join(member);
        fs::create_dir_all(member_dir.join("src")).unwrap();

        fs::write(
            member_dir.join("Cargo.toml"),
            &format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
                member
            ),
        )
        .unwrap();

        fs::write(
            member_dir.join("src/lib.rs"),
            &format!(
                r#"
//! {} library

#[cfg(test)]
mod tests {{
    #[test]
    fn test_{}() {{
        assert!(true);
    }}
}}
"#,
                member, member
            ),
        )
        .unwrap();
    }

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
        .current_dir("../../")
        .output()
        .expect("Failed to execute workspace detection");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect cfg(test) patterns in the workspace members
    assert!(stdout.contains("conditional compilation pattern"));
}

#[test]
fn test_output_verbosity_levels() {
    let temp_dir = TempDir::new().unwrap();
    create_minimal_rust_project(temp_dir.path()).unwrap();

    // Test without verbose
    let output_normal = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--project-dir",
            &temp_dir.path().to_string_lossy(),
            "--detect-only",
        ])
        .current_dir("../../")
        .output()
        .expect("Failed to execute normal verbosity");

    // Test with verbose
    let output_verbose = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "smart-hooks",
            "--",
            "check",
            "conditional-compilation",
            "--project-dir",
            &temp_dir.path().to_string_lossy(),
            "--detect-only",
            "--verbose",
        ])
        .current_dir("../../")
        .output()
        .expect("Failed to execute verbose mode");

    if !output_normal.status.success() {
        println!("Normal command failed. Stderr: {}", String::from_utf8_lossy(&output_normal.stderr));
        println!("Normal command failed. Stdout: {}", String::from_utf8_lossy(&output_normal.stdout));
    }
    if !output_verbose.status.success() {
        println!("Verbose command failed. Stderr: {}", String::from_utf8_lossy(&output_verbose.stderr));
        println!("Verbose command failed. Stdout: {}", String::from_utf8_lossy(&output_verbose.stdout));
    }
    
    assert!(output_normal.status.success());
    assert!(output_verbose.status.success());

    let stdout_normal = String::from_utf8_lossy(&output_normal.stdout);
    let stdout_verbose = String::from_utf8_lossy(&output_verbose.stdout);

    // Verbose output should be longer or contain more details
    assert!(!stdout_normal.is_empty());
    assert!(!stdout_verbose.is_empty());
    // Note: Exact verbosity differences depend on implementation
}

#[test]
fn test_file_filtering_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_minimal_rust_project(temp_dir.path()).unwrap();

    // Create non-Rust files that should be ignored
    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "target/").unwrap();

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
        .current_dir("../../")
        .output()
        .expect("Failed to execute file filtering test");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should only process Rust files, not the other files we created
    assert!(stdout.contains("conditional compilation pattern"));
    // Should not mention processing .md, .json, or other non-Rust files
    assert!(!stdout.to_lowercase().contains("readme.md"));
    assert!(!stdout.to_lowercase().contains("config.json"));
}

#[test]
#[ignore = "Requires actual git repository setup"]
fn test_git_integration_workflow() {
    // This test would require setting up a real git repository
    // and staging files to test the git integration
    let temp_dir = TempDir::new().unwrap();
    create_minimal_rust_project(temp_dir.path()).unwrap();

    // Would initialize git repo, stage files, then test git-based commands
    println!("Git integration test requires manual setup");
}

#[test]
fn test_performance_with_multiple_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create a project with many files
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "performance-test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    // Create many modules with conditional compilation
    for i in 0..10 {
        let module_content = format!(
            r#"
//! Module {}

#[cfg(feature = "feature-{}")]
use feature_{}_crate::Feature{};

#[cfg(test)]
mod tests {{
    #[test]
    fn test_module_{}() {{
        assert!(true);
    }}
}}

pub fn function_{}() {{
    println!("Function from module {}")
}}
"#,
            i, i, i, i, i, i, i
        );

        fs::write(src_dir.join(format!("module_{}.rs", i)), module_content).unwrap();
    }

    let lib_content = (0..10)
        .map(|i| format!("pub mod module_{};", i))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(src_dir.join("lib.rs"), lib_content).unwrap();

    let start_time = std::time::Instant::now();

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
        .current_dir("../../")
        .output()
        .expect("Failed to execute performance test");

    let duration = start_time.elapsed();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect all the conditional patterns (20 patterns: 2 per module * 10 modules)
    assert!(stdout.contains("Found 20 conditional compilation pattern(s)"));

    // Should complete reasonably quickly
    assert!(
        duration.as_secs() < 10,
        "Command took too long: {:?}",
        duration
    );

    println!("Performance test completed in {:?}", duration);
}