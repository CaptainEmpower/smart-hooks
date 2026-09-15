//! smart-hooks — intelligent test selection for Rust projects.
//!
//! Modular architecture following the Single Responsibility Principle.
pub mod analysis;
pub mod dependency;
pub mod execution;
pub mod project;
pub mod utilities;

// Re-export commonly used types and functions
pub use analysis::config::TestSelectorConfig;
pub use analysis::dependency_mapper::{create_test_plan, create_test_plan_with_config, TestPlan};
pub use analysis::file_analyzer::{analyze_file_content, FunctionalityFlags};
pub use execution::plan_executor::execute_test_plan;
pub use execution::test_runner::run_cargo_command;
pub use utilities::file_utils::{is_rust_file, read_file_content};
pub use utilities::impact_analyzer::{determine_impact_level, ImpactLevel};
pub use utilities::module_utils::extract_module_name;

// Legacy compatibility - keep the utils module for existing binaries
pub mod utils {
    pub use crate::execution::test_runner::run_cargo_command;
    pub use crate::utilities::file_utils::is_rust_file;
    pub use crate::utilities::impact_analyzer::determine_impact_level;
    pub use crate::utilities::module_utils::extract_module_name;
}
