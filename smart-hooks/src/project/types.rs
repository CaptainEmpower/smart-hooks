/// Core types for project configuration
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustProjectConfig {
    /// Root of the workspace or single-crate project
    pub workspace_root: PathBuf,
    /// Information about each crate in the project
    pub crates: Vec<CrateInfo>,
    /// Test execution strategy configuration
    pub test_strategy: TestStrategy,
    /// Project metadata
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    /// Name of the crate
    pub name: String,
    /// Path to the crate directory
    pub path: PathBuf,
    /// Path to the Cargo.toml file
    pub manifest_path: PathBuf,
    /// Whether this is a binary or library crate
    pub crate_type: CrateType,
    /// Dependencies of this crate
    pub dependencies: Vec<String>,
    /// Features available in this crate
    pub features: Vec<String>,
    /// Test configuration for this crate
    pub test_config: CrateTestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrateType {
    /// Library crate (lib.rs)
    Library,
    /// Binary crate (main.rs)
    Binary,
    /// Both library and binary
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateTestConfig {
    /// Patterns for unit test files
    pub unit_test_patterns: Vec<String>,
    /// Directories containing integration tests
    pub integration_test_dirs: Vec<PathBuf>,
    /// Whether doc tests are enabled
    pub doc_tests_enabled: bool,
    /// Custom test commands or flags
    pub custom_test_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStrategy {
    /// How aggressive to be with test selection
    pub selection_mode: TestSelectionMode,
    /// Whether to run integration tests automatically
    pub auto_integration_tests: bool,
    /// Whether to run doc tests
    pub run_doc_tests: bool,
    /// Maximum time to spend on test analysis (seconds)
    pub max_analysis_time: u64,
    /// AI-powered test selection configuration
    pub ai_config: Option<AITestConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestSelectionMode {
    /// Run all tests
    All,
    /// Run tests for changed modules only
    Conservative,
    /// Use dependency analysis for smart selection
    Smart,
    /// Use AI analysis for intelligent selection
    Intelligent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITestConfig {
    /// Confidence threshold for AI recommendations
    pub confidence_threshold: f32,
    /// Context to provide to AI for analysis
    pub project_context: String,
    /// Whether to use AI for BDD test selection
    pub enable_bdd_selection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Rust edition (2018, 2021, etc.)
    pub edition: String,
    /// Project description
    pub description: Option<String>,
    /// Project repository URL
    pub repository: Option<String>,
}

impl Default for TestStrategy {
    fn default() -> Self {
        TestStrategy {
            selection_mode: TestSelectionMode::Smart,
            auto_integration_tests: true,
            run_doc_tests: true,
            max_analysis_time: 30,
            ai_config: None,
        }
    }
}

impl Default for CrateTestConfig {
    fn default() -> Self {
        CrateTestConfig {
            unit_test_patterns: vec!["src/**/*.rs".to_string()],
            integration_test_dirs: Vec::new(),
            doc_tests_enabled: true,
            custom_test_commands: Vec::new(),
        }
    }
}