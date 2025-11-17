/// Smart Hooks Library
/// Modular architecture following Single Responsibility Principle (SRP)
/// Supports intelligent git hooks for Rust and multi-language projects
///
/// All modules follow SRP with maximum 300 LOC per file
/// Phase 2: Extended with multi-language project support
/// Phase 3: Hot reload system for cache-first execution
pub mod analysis;
pub mod dependency;
pub mod execution;
pub mod project;
pub mod utilities;

// Hot reload system (Phase 3)
#[cfg(feature = "hotreload")]
pub mod hotreload;

// Re-export commonly used types and functions
pub use analysis::bdd::{BddFeatureSelection, BddFeatureSelector, FileChange as BddFileChange};
pub use analysis::config::TestSelectorConfig;
pub use analysis::dependency_mapper::{create_test_plan, create_test_plan_with_config, TestPlan};
pub use analysis::file_analyzer::{analyze_file_content, FunctionalityFlags};
pub use execution::plan_executor::execute_test_plan;
pub use execution::test_runner::run_cargo_command;
pub use utilities::file_utils::{is_rust_file, read_file_content};
pub use utilities::impact_analyzer::{determine_impact_level, ImpactLevel};
pub use utilities::module_utils::extract_module_name;

// Re-export project discovery types
pub use project::{CrateInfo, ProjectMetadata, RustProjectConfig, TestStrategy};

// Multi-language project support
pub use project::{
    BuildConfig, BuildTool, Language, LanguageCommands, LanguageConfig, MultiLangProjectConfig,
    MultiLangProjectDiscovery, PackageManager, TestFramework,
};

// Re-export dependency analysis types
pub use dependency::{
    Dependency, DependencyAnalyzer, DependencyGraph, RustDependencyAnalyzer, TestTarget,
};

// Multi-language dependency analysis
pub use dependency::{
    LanguageDependencyAnalyzer, MultiLangDependencyAnalyzer, PhpDependencyAnalyzer,
    PythonDependencyAnalyzer, TypeScriptDependencyAnalyzer,
};

// Hot reload system exports
#[cfg(feature = "hotreload")]
pub use hotreload::{
    BackgroundWarmingService, CacheEntry, CacheKey, CacheStorage, DiskLruCache, FileChangeTracker,
    HookResult, HookResults, HookType, HotReloadConfig, HotReloadEngine,
};

// Legacy compatibility - keep the utils module for existing binaries
pub mod utils {
    pub use crate::execution::test_runner::run_cargo_command;
    pub use crate::utilities::file_utils::is_rust_file;
    pub use crate::utilities::impact_analyzer::determine_impact_level;
    pub use crate::utilities::module_utils::extract_module_name;
}