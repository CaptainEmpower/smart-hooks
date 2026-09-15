/// Integration tests for smart-hooks end-to-end workflows
use anyhow::Result;
use smart_hooks::analysis::{config::TestSelectorConfig, dependency_mapper};
use smart_hooks::project::discovery::ProjectDiscovery;
use smart_hooks::utilities::{file_utils, impact_analyzer};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test the complete workflow of project discovery -> dependency analysis -> test planning
#[test]
fn test_end_to_end_workflow() -> Result<()> {
    // Create a temporary Rust project structure
    let temp_dir = TempDir::new()?;
    let project_root = temp_dir.path();

    // Create Cargo.toml
    fs::write(
        project_root.join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
    )?;

    // Create src directory with main.rs
    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(
        src_dir.join("main.rs"),
        r#"
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
}

fn main() {
    let user = User {
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
    };
    println!("User: {}", user.name);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let user = User {
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
        };
        assert_eq!(user.name, "Test");
    }
}
"#,
    )?;

    fs::write(
        src_dir.join("lib.rs"),
        r#"
pub mod utils;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#,
    )?;

    fs::write(
        src_dir.join("utils.rs"),
        r#"
pub fn format_string(input: &str) -> String {
    format!("Formatted: {}", input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_string() {
        let result = format_string("test");
        assert_eq!(result, "Formatted: test");
    }
}
"#,
    )?;

    // Test 1: Project Discovery
    let project_config = ProjectDiscovery::discover(project_root)?;
    assert_eq!(project_config.crates.len(), 1);

    // Test 2: File Analysis
    let main_file = src_dir.join("main.rs");
    let main_file_str = main_file.to_str().unwrap();
    assert!(file_utils::is_rust_file(main_file_str));
    assert!(file_utils::is_core_functionality_file(&main_file));

    // Test 3: Impact Analysis
    let changed_files = vec![main_file_str.to_string()];
    let impact_level = impact_analyzer::determine_impact_level(&changed_files);
    assert!(impact_level != smart_hooks::utilities::impact_analyzer::ImpactLevel::None);

    // Test 4: Test Planning. `main.rs` is a crate root, so it has no module of
    // its own; a sibling module is what produces a unit-test selection.
    let module_file = src_dir.join("calculator.rs");
    fs::write(
        &module_file,
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )?;
    let test_plan =
        dependency_mapper::create_test_plan(&[module_file.to_string_lossy().to_string()])?;
    assert!(
        test_plan.unit_tests.contains("calculator"),
        "expected `calculator` in {:?}",
        test_plan.unit_tests
    );

    Ok(())
}

/// Test configuration loading and validation
#[test]
fn test_configuration_workflow() {
    let config = TestSelectorConfig::default();

    // Test default configuration has sensible values
    assert!(!config.bdd_test_patterns.is_empty());

    // Behavioural paths select scenario coverage; ordinary modules do not.
    assert!(config.should_run_bdd_tests("src/strategy/adaptive.rs"));
    assert!(!config.should_run_bdd_tests("src/calculator.rs"));
}

/// Test multi-file change analysis
#[test]
fn test_multi_file_analysis() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;

    // Create multiple related files
    fs::write(
        src_dir.join("main.rs"),
        "fn main() { println!(\"Hello\"); }",
    )?;
    fs::write(src_dir.join("lib.rs"), "pub mod utils;")?;
    fs::write(src_dir.join("utils.rs"), "pub fn helper() -> i32 { 42 }")?;

    let changed_files = vec![
        src_dir.join("main.rs").to_string_lossy().to_string(),
        src_dir.join("utils.rs").to_string_lossy().to_string(),
    ];

    // Test impact analysis with multiple files
    let impact = impact_analyzer::determine_impact_level(&changed_files);
    assert!(
        impact == smart_hooks::utilities::impact_analyzer::ImpactLevel::High
            || impact == smart_hooks::utilities::impact_analyzer::ImpactLevel::Medium
    );

    // Test test planning with multiple files
    let test_plan = dependency_mapper::create_test_plan(&changed_files)?;

    // Should have recommendations for multiple files
    assert!(!test_plan.unit_tests.is_empty());

    Ok(())
}

/// Test error handling in various scenarios
#[test]
fn test_error_handling() {
    // Test with non-existent directory
    let result = ProjectDiscovery::discover(Path::new("/nonexistent/path"));
    assert!(result.is_err());

    // Test impact analysis with empty file list
    let impact = impact_analyzer::determine_impact_level(&[]);
    assert_eq!(
        impact,
        smart_hooks::utilities::impact_analyzer::ImpactLevel::None
    );

    // Test test planning with empty files
    let empty_files = vec![];
    let test_plan = dependency_mapper::create_test_plan(&empty_files);
    assert!(test_plan.is_ok()); // Should handle empty gracefully
}

/// Test file utility functions comprehensively
#[test]
fn test_file_utilities() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn test() {}")?;

    // Test file type detection
    assert!(file_utils::is_rust_file(test_file.to_str().unwrap()));

    // Test with different file extensions
    let js_file = temp_dir.path().join("script.js");
    fs::write(&js_file, "console.log('hello');")?;
    assert!(!file_utils::is_rust_file(js_file.to_str().unwrap()));

    // Test core functionality detection
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;
    let main_file = src_dir.join("main.rs");
    fs::write(&main_file, "fn main() {}")?;
    assert!(file_utils::is_core_functionality_file(&main_file));

    // Test relative path calculation
    let relative = file_utils::get_relative_path(
        main_file.to_str().unwrap(),
        temp_dir.path().to_str().unwrap(),
    );
    assert!(relative.is_some());
    assert_eq!(relative.unwrap(), "/src/main.rs");

    Ok(())
}

/// Test the complete smart test selection workflow
#[test]
fn test_smart_test_selection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_root = temp_dir.path();

    // Set up a complete Rust project
    fs::write(
        project_root.join("Cargo.toml"),
        r#"
[package]
name = "integration-test-project"
version = "0.1.0"
edition = "2021"
"#,
    )?;

    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir)?;

    // Core business logic file
    fs::write(
        src_dir.join("business.rs"),
        r#"
pub struct Calculator;

impl Calculator {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    pub fn multiply(a: i32, b: i32) -> i32 {
        a * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(Calculator::add(2, 3), 5);
    }
    
    #[test]
    fn test_multiply() {
        assert_eq!(Calculator::multiply(2, 3), 6);
    }
}
"#,
    )?;

    // Utility file
    fs::write(
        src_dir.join("utils.rs"),
        r#"
pub fn format_output(value: i32) -> String {
    format!("Result: {}", value)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_output() {
        assert_eq!(format_output(42), "Result: 42");
    }
}
"#,
    )?;

    // Main application file
    fs::write(
        src_dir.join("main.rs"),
        r#"
mod business;
mod utils;

use business::Calculator;
use utils::format_output;

fn main() {
    let result = Calculator::add(5, 3);
    println!("{}", format_output(result));
}
"#,
    )?;

    // Test complete workflow
    let project_config = ProjectDiscovery::discover(project_root)?;
    assert!(!project_config.crates.is_empty());

    // Simulate changes to the business logic file
    let changed_files = vec![src_dir.join("business.rs").to_string_lossy().to_string()];

    // Analyze impact
    let impact = impact_analyzer::determine_impact_level(&changed_files);
    assert!(impact != smart_hooks::utilities::impact_analyzer::ImpactLevel::None);

    // Create test plan
    let test_plan = dependency_mapper::create_test_plan(&changed_files)?;

    // Verify test plan includes relevant tests
    assert!(!test_plan.unit_tests.is_empty() || test_plan.integration_tests || test_plan.bdd_tests);

    // Check that the test plan contains business-related tests
    let has_business_test = test_plan
        .unit_tests
        .iter()
        .any(|test| test.contains("business") || test.contains("calculator"));
    assert!(has_business_test || test_plan.integration_tests);

    Ok(())
}
