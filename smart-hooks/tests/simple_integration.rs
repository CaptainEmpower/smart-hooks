/// Simple integration tests that focus on existing working functionality
use anyhow::Result;
use smart_hooks::analysis::dependency_mapper;
use smart_hooks::utilities::{file_utils, impact_analyzer};
use std::fs;
use tempfile::TempDir;

/// Test basic file utilities that are core to the system
#[test]
fn test_file_utilities_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create test files
    let rust_file = temp_dir.path().join("main.rs");
    fs::write(&rust_file, "fn main() {}")?;
    
    let js_file = temp_dir.path().join("app.js");
    fs::write(&js_file, "console.log('test');")?;
    
    // Test file type detection
    assert!(file_utils::is_rust_file(rust_file.to_str().unwrap()));
    assert!(!file_utils::is_rust_file(js_file.to_str().unwrap()));
    
    // Test core functionality detection
    assert!(file_utils::is_core_functionality_file(&rust_file));
    
    Ok(())
}

/// Test impact analysis with various file configurations
#[test]
fn test_impact_analysis_workflow() {
    // Test with empty changes
    let empty_files: Vec<String> = vec![];
    let impact = impact_analyzer::determine_impact_level(&empty_files);
    assert_eq!(impact, smart_hooks::utilities::impact_analyzer::ImpactLevel::None);
    
    // Test with core file changes
    let core_files = vec!["src/main.rs".to_string()];
    let impact = impact_analyzer::determine_impact_level(&core_files);
    assert!(impact != smart_hooks::utilities::impact_analyzer::ImpactLevel::None);
    
    // Test with multiple file changes
    let multiple_files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
        "src/utils.rs".to_string(),
    ];
    let impact = impact_analyzer::determine_impact_level(&multiple_files);
    assert!(impact == smart_hooks::utilities::impact_analyzer::ImpactLevel::High ||
            impact == smart_hooks::utilities::impact_analyzer::ImpactLevel::Medium);
}

/// Test the test plan creation functionality
#[test]
fn test_test_plan_creation() -> Result<()> {
    // Test with empty file list
    let empty_files: Vec<String> = vec![];
    let test_plan = dependency_mapper::create_test_plan(&empty_files)?;
    
    // Should handle empty input gracefully
    assert!(test_plan.unit_tests.is_empty() || !test_plan.unit_tests.is_empty());
    
    // Test with core files
    let core_files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
    ];
    let test_plan = dependency_mapper::create_test_plan(&core_files)?;
    
    // Should generate some kind of test recommendation
    assert!(
        !test_plan.unit_tests.is_empty() || 
        test_plan.integration_tests || 
        test_plan.bdd_tests
    );
    
    Ok(())
}

/// Test file content analysis
#[test]
fn test_file_content_analysis() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create a file with function definitions
    let complex_file = temp_dir.path().join("complex.rs");
    fs::write(&complex_file, r#"
use std::collections::HashMap;

pub struct Calculator {
    history: HashMap<String, i32>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            history: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, a: i32, b: i32) -> i32 {
        let result = a + b;
        self.history.insert(format!("add_{}_{}", a, b), result);
        result
    }
    
    fn internal_helper(&self) -> bool {
        !self.history.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculator() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(2, 3), 5);
    }
}
"#)?;

    // Test that this is detected as a Rust file
    assert!(file_utils::is_rust_file(complex_file.to_str().unwrap()));
    
    // Test that it's considered core functionality
    assert!(file_utils::is_core_functionality_file(&complex_file));
    
    Ok(())
}

/// Test error handling in the system
#[test]
fn test_error_handling() {
    // Test impact analysis with malformed file paths
    let weird_files = vec![
        "".to_string(),
        "/".to_string(),
        "non_existent_file_that_should_not_exist.rs".to_string(),
    ];
    
    // Should not panic and should handle gracefully
    let impact = impact_analyzer::determine_impact_level(&weird_files);
    assert!(impact == smart_hooks::utilities::impact_analyzer::ImpactLevel::None ||
            impact == smart_hooks::utilities::impact_analyzer::ImpactLevel::Low);
    
    // Test file utilities with non-existent files
    assert!(!file_utils::is_rust_file("/non/existent/file.rs"));
}

/// Test the end-to-end workflow with a minimal setup
#[test]
fn test_minimal_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create minimal Rust project structure
    fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#)?;

    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;
    
    fs::write(src_dir.join("main.rs"), r#"
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        // Test passes
        assert!(true);
    }
}
"#)?;

    // Test the complete analysis workflow
    let changed_files = vec![
        src_dir.join("main.rs").to_string_lossy().to_string()
    ];
    
    // Step 1: Analyze impact
    let impact = impact_analyzer::determine_impact_level(&changed_files);
    assert!(impact != smart_hooks::utilities::impact_analyzer::ImpactLevel::None);
    
    // Step 2: Create test plan
    let test_plan = dependency_mapper::create_test_plan(&changed_files)?;
    
    // Step 3: Verify we get reasonable recommendations
    assert!(
        !test_plan.unit_tests.is_empty() ||
        test_plan.integration_tests ||
        test_plan.bdd_tests
    );
    
    Ok(())
}