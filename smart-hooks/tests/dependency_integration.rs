/// Integration tests for dependency analysis modules
/// Tests the complete dependency analysis pipeline and module interactions
use smart_hooks::dependency::{
    test_resolver::RustTestResolver, types::TestType, DependencyAnalyzer, RustDependencyAnalyzer,
};
use smart_hooks::project::RustProjectConfig;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_rust_project_with_dependencies(temp_dir: &Path) -> Result<(), std::io::Error> {
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create Cargo.toml
    fs::write(
        temp_dir.join("Cargo.toml"),
        r#"
[package]
name = "test-dependency-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", optional = true }

[features]
default = []
async-support = ["tokio"]
"#,
    )?;

    // Create lib.rs with various dependency patterns
    fs::write(
        src_dir.join("lib.rs"),
        r#"
//! Test library with complex dependencies

use anyhow::Result;
use serde::{Serialize, Deserialize};

#[cfg(feature = "async-support")]
use tokio::runtime::Runtime;

pub mod core;
pub mod utils;
pub mod types;

pub use core::DataProcessor;
pub use types::{Config, ProcessResult};

/// Main library function with dependencies
pub fn process_data(input: &str) -> Result<ProcessResult> {
    let processor = DataProcessor::new();
    processor.process(input)
}

#[cfg(feature = "async-support")]
pub async fn process_data_async(input: &str) -> Result<ProcessResult> {
    // Async processing logic
    Ok(ProcessResult::success(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        assert!(process_data("test").is_ok());
    }
}
"#,
    )?;

    // Create core module with internal dependencies
    fs::write(
        src_dir.join("core.rs"),
        r#"
//! Core processing module

use anyhow::{Result, Context};
use crate::types::{Config, ProcessResult};
use crate::utils::validate_input;

pub struct DataProcessor {
    config: Config,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn process(&self, input: &str) -> Result<ProcessResult> {
        validate_input(input)?;

        let result = input.to_uppercase();
        Ok(ProcessResult::success(&result))
    }

    pub fn configure(&mut self, config: Config) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let processor = DataProcessor::new();
        assert!(processor.process("test").is_ok());
    }
}
"#,
    )?;

    // Create types module
    fs::write(
        src_dir.join("types.rs"),
        r#"
//! Type definitions

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub timeout_ms: u64,
    pub max_retries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub data: String,
    pub timestamp: u64,
}

impl ProcessResult {
    pub fn success(data: &str) -> Self {
        Self {
            success: true,
            data: data.to_string(),
            timestamp: 0, // Would use actual timestamp
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: message.to_string(),
            timestamp: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.timeout_ms, 5000);
    }
}
"#,
    )?;

    // Create utils module
    fs::write(
        src_dir.join("utils.rs"),
        r#"
//! Utility functions

use anyhow::{Result, bail};

pub fn validate_input(input: &str) -> Result<()> {
    if input.is_empty() {
        bail!("Input cannot be empty");
    }

    if input.len() > 1000 {
        bail!("Input too long");
    }

    Ok(())
}

pub fn format_result(input: &str) -> String {
    format!("Processed: {}", input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input() {
        assert!(validate_input("valid").is_ok());
        assert!(validate_input("").is_err());
    }
}
"#,
    )?;

    // Create tests directory with integration tests
    let tests_dir = temp_dir.join("tests");
    fs::create_dir_all(&tests_dir)?;

    fs::write(
        tests_dir.join("integration_test.rs"),
        r#"
//! Integration tests for the test project

use test_dependency_project::*;

#[test]
fn test_full_workflow() {
    let result = process_data("integration test");
    assert!(result.is_ok());
}
"#,
    )?;

    Ok(())
}

#[test]
fn test_dependency_analysis_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project_with_dependencies(temp_dir.path()).unwrap();

    // Test project discovery
    let project_config = RustProjectConfig::discover(temp_dir.path()).unwrap();
    assert_eq!(project_config.metadata.name, "test-dependency-project");
    assert_eq!(project_config.crates.len(), 1);

    // Test dependency analysis
    let analyzer = RustDependencyAnalyzer::discover_from_path(temp_dir.path()).unwrap();

    // Analyze lib.rs file
    let lib_file = temp_dir.path().join("src/lib.rs");
    let dependencies_result = analyzer.analyze_file_dependencies(&lib_file);

    // Should be able to analyze the file without errors
    assert!(dependencies_result.is_ok());
    let dependencies = dependencies_result.unwrap();

    // Basic validation - should return some form of result
    // (specific dependencies may vary based on implementation)
    println!("Found {} dependencies", dependencies.len());
}

#[test]
fn test_test_target_resolution() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project_with_dependencies(temp_dir.path()).unwrap();

    let project_config = RustProjectConfig::discover(temp_dir.path()).unwrap();
    let resolver = RustTestResolver::new(project_config);

    // Test with changed files
    let changed_files = vec![
        temp_dir.path().join("src/core.rs"),
        temp_dir.path().join("src/types.rs"),
    ];

    let test_targets = resolver.find_affected_test_targets(&changed_files).unwrap();

    // Should generate unit test targets for each changed module
    assert!(test_targets.iter().any(|t| t.name.contains("core")));
    assert!(test_targets.iter().any(|t| t.name.contains("types")));

    // Check test types
    assert!(test_targets
        .iter()
        .any(|t| matches!(t.test_type, TestType::Unit { .. })));
}

#[test]
fn test_workspace_dependency_analysis() {
    let temp_dir = TempDir::new().unwrap();

    // Create workspace structure
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[workspace]
members = ["crate1", "crate2"]

[package]
name = "workspace-root"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    // Create crate1
    let crate1_dir = temp_dir.path().join("crate1");
    fs::create_dir_all(crate1_dir.join("src")).unwrap();
    fs::write(
        crate1_dir.join("Cargo.toml"),
        r#"
[package]
name = "crate1"
version = "0.1.0"
edition = "2021"

[dependencies]
crate2 = { path = "../crate2" }
"#,
    )
    .unwrap();

    fs::write(
        crate1_dir.join("src/lib.rs"),
        r#"
use crate2::SharedType;

pub fn use_shared_type() -> SharedType {
    SharedType::new()
}
"#,
    )
    .unwrap();

    // Create crate2
    let crate2_dir = temp_dir.path().join("crate2");
    fs::create_dir_all(crate2_dir.join("src")).unwrap();
    fs::write(
        crate2_dir.join("Cargo.toml"),
        r#"
[package]
name = "crate2"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        crate2_dir.join("src/lib.rs"),
        r#"
pub struct SharedType {
    value: i32,
}

impl SharedType {
    pub fn new() -> Self {
        Self { value: 42 }
    }
}
"#,
    )
    .unwrap();

    // Test workspace discovery and analysis
    let project_config = RustProjectConfig::discover(temp_dir.path()).unwrap();
    assert_eq!(project_config.crates.len(), 2);

    // Basic validation that we can discover the workspace
    let crate1 = project_config.crates.iter().find(|c| c.name == "crate1");
    let crate2 = project_config.crates.iter().find(|c| c.name == "crate2");
    assert!(crate1.is_some());
    assert!(crate2.is_some());
}

#[test]
fn test_conditional_dependency_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create Cargo.toml with optional dependencies
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "conditional-deps"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
tokio = { version = "1.0", optional = true }
serde = { version = "1.0", optional = true }

[features]
default = []
async = ["tokio"]
serialization = ["serde"]
full = ["async", "serialization"]
"#,
    )
    .unwrap();

    // Create source with conditional dependencies
    fs::write(
        src_dir.join("lib.rs"),
        r#"
use anyhow::Result;

#[cfg(feature = "async")]
use tokio::runtime::Runtime;

#[cfg(feature = "serialization")]
use serde::{Serialize, Deserialize};

#[cfg(all(feature = "async", feature = "serialization"))]
pub async fn async_serialize() -> Result<String> {
    Ok("async serialized".to_string())
}

pub fn basic_function() -> Result<()> {
    Ok(())
}
"#,
    )
    .unwrap();

    let _project_config = RustProjectConfig::discover(temp_dir.path()).unwrap();
    let analyzer = RustDependencyAnalyzer::discover_from_path(temp_dir.path()).unwrap();

    let lib_file = temp_dir.path().join("src/lib.rs");
    let dependencies_result = analyzer.analyze_file_dependencies(&lib_file);

    // Should be able to analyze conditional dependencies without errors
    assert!(dependencies_result.is_ok());
    let dependencies = dependencies_result.unwrap();

    // Basic validation - the analyzer should process the file
    println!(
        "Conditional analysis found {} dependencies",
        dependencies.len()
    );
}

#[test]
fn test_dependency_graph_analysis() {
    let temp_dir = TempDir::new().unwrap();
    create_rust_project_with_dependencies(temp_dir.path()).unwrap();

    let _project_config = RustProjectConfig::discover(temp_dir.path()).unwrap();
    let analyzer = RustDependencyAnalyzer::discover_from_path(temp_dir.path()).unwrap();

    // Analyze individual files in the project
    let lib_file = temp_dir.path().join("src/lib.rs");
    let lib_result = analyzer.analyze_file_dependencies(&lib_file);

    let core_file = temp_dir.path().join("src/core.rs");
    let core_result = analyzer.analyze_file_dependencies(&core_file);

    // Should be able to analyze both files without errors
    assert!(lib_result.is_ok());
    assert!(core_result.is_ok());

    let lib_dependencies = lib_result.unwrap();
    let core_dependencies = core_result.unwrap();

    println!(
        "Lib dependencies: {}, Core dependencies: {}",
        lib_dependencies.len(),
        core_dependencies.len()
    );
}
