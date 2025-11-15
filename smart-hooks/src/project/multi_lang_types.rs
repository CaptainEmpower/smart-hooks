/// Multi-language project support types
/// Provides a unified interface for different programming languages
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unified project configuration supporting multiple languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLangProjectConfig {
    /// Root of the project
    pub project_root: PathBuf,
    /// Primary language of the project
    pub primary_language: Language,
    /// All languages detected in the project
    pub languages: Vec<LanguageConfig>,
    /// Project metadata
    pub metadata: ProjectMetadata,
    /// Global test strategy
    pub test_strategy: TestStrategy,
    /// Build tool configuration
    pub build_config: BuildConfig,
}

/// Supported programming languages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    PHP,
    Go,
    Java,
    CSharp,
    Unknown(String),
}

/// Language-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Programming language
    pub language: Language,
    /// Language-specific source directories
    pub source_dirs: Vec<PathBuf>,
    /// File patterns for this language
    pub file_patterns: Vec<String>,
    /// Package/dependency management
    pub package_manager: PackageManager,
    /// Testing framework configuration
    pub test_framework: TestFramework,
    /// Build and run commands
    pub commands: LanguageCommands,
    /// Language-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Package management systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageManager {
    // Rust
    Cargo,
    // Node.js ecosystem
    Npm,
    Yarn,
    Pnpm,
    // Python ecosystem
    Pip,
    Poetry,
    Conda,
    Pipenv,
    // PHP ecosystem
    Composer,
    // Go ecosystem
    GoMod,
    // Java ecosystem
    Maven,
    Gradle,
    // .NET ecosystem
    NuGet,
    // Generic/Unknown
    Unknown(String),
}

/// Testing frameworks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestFramework {
    // Rust
    RustTest,
    // JavaScript/TypeScript
    Jest,
    Mocha,
    Vitest,
    Cypress,
    // Python
    Pytest,
    Unittest,
    // PHP
    PHPUnit,
    Pest,
    // Go
    GoTest,
    // Java
    JUnit,
    TestNG,
    // .NET
    MSTest,
    NUnit,
    XUnit,
    // BDD frameworks
    Cucumber,
    // Generic/Unknown
    Unknown(String),
}

/// Language-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageCommands {
    /// Command to run tests
    pub test_command: Vec<String>,
    /// Command to build the project
    pub build_command: Option<Vec<String>>,
    /// Command to run linter
    pub lint_command: Option<Vec<String>>,
    /// Command to format code
    pub format_command: Option<Vec<String>>,
    /// Command to install dependencies
    pub install_command: Option<Vec<String>>,
}

/// Build system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Primary build tool
    pub build_tool: BuildTool,
    /// Build targets/configurations
    pub targets: Vec<String>,
    /// Environment variables needed for build
    pub environment: HashMap<String, String>,
}

/// Build tools and systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildTool {
    // Rust
    Cargo,
    // Node.js
    WebpackJS,
    Vite,
    Rollup,
    // Python
    SetupPy,
    Wheel,
    // PHP
    ComposerPhp,
    // Go
    GoBuild,
    // Java
    Maven,
    Gradle,
    // .NET
    DotNet,
    MSBuild,
    // Multi-language
    Makefile,
    BazelBuild,
    // Generic
    Unknown(String),
}

/// Unified project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Project description
    pub description: Option<String>,
    /// Project repository URL
    pub repository: Option<String>,
    /// Project license
    pub license: Option<String>,
    /// Project authors
    pub authors: Vec<String>,
}

/// Test strategy for multi-language projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStrategy {
    /// How to select tests to run
    pub selection_mode: TestSelectionMode,
    /// Whether to run tests for all languages or just primary
    pub cross_language_testing: bool,
    /// Whether to run integration tests
    pub run_integration_tests: bool,
    /// Whether to run end-to-end tests
    pub run_e2e_tests: bool,
    /// Maximum time for test analysis (seconds)
    pub max_analysis_time: u64,
    /// AI configuration for intelligent test selection
    pub ai_config: Option<AITestConfig>,
}

/// Test selection strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestSelectionMode {
    /// Run all available tests
    All,
    /// Run tests only for changed files
    Conservative,
    /// Use dependency analysis for smart selection
    Smart,
    /// Use AI for intelligent test selection
    Intelligent,
}

/// AI configuration for test selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITestConfig {
    /// Confidence threshold for AI recommendations
    pub confidence_threshold: f32,
    /// Project context for AI analysis
    pub project_context: String,
    /// Enable BDD test selection
    pub enable_bdd_selection: bool,
    /// Enable cross-language impact analysis
    pub cross_language_analysis: bool,
}

impl Language {
    /// Get file extensions for this language
    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Rust => vec!["rs"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::JavaScript => vec!["js", "jsx", "mjs"],
            Language::Python => vec!["py", "pyx", "pyi"],
            Language::PHP => vec!["php", "phtml", "php3", "php4", "php5"],
            Language::Go => vec!["go"],
            Language::Java => vec!["java"],
            Language::CSharp => vec!["cs"],
            Language::Unknown(_) => vec![],
        }
    }

    /// Get common source directories for this language
    pub fn common_source_dirs(&self) -> Vec<&'static str> {
        match self {
            Language::Rust => vec!["src", "tests"],
            Language::TypeScript | Language::JavaScript => {
                vec!["src", "lib", "app", "pages", "components"]
            }
            Language::Python => vec!["src", "lib", "tests", "test"],
            Language::PHP => vec!["src", "lib", "app", "public"],
            Language::Go => vec!["cmd", "pkg", "internal"],
            Language::Java => vec!["src/main", "src/test"],
            Language::CSharp => vec!["src", "lib", "app"],
            Language::Unknown(_) => vec![],
        }
    }

    /// Get default package manager for this language
    pub fn default_package_manager(&self) -> PackageManager {
        match self {
            Language::Rust => PackageManager::Cargo,
            Language::TypeScript | Language::JavaScript => PackageManager::Npm,
            Language::Python => PackageManager::Pip,
            Language::PHP => PackageManager::Composer,
            Language::Go => PackageManager::GoMod,
            Language::Java => PackageManager::Maven,
            Language::CSharp => PackageManager::NuGet,
            Language::Unknown(_) => PackageManager::Unknown("unknown".to_string()),
        }
    }
}

impl Default for TestStrategy {
    fn default() -> Self {
        TestStrategy {
            selection_mode: TestSelectionMode::Smart,
            cross_language_testing: false,
            run_integration_tests: true,
            run_e2e_tests: false,
            max_analysis_time: 30,
            ai_config: None,
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        BuildConfig {
            build_tool: BuildTool::Unknown("unknown".to_string()),
            targets: vec!["default".to_string()],
            environment: HashMap::new(),
        }
    }
}