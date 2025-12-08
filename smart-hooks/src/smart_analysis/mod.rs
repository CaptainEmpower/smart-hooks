//! Smart analysis module organization
//! 
//! This module organizes smart analysis functionality into focused sub-modules
//! following Single Responsibility Principle.

pub mod test_selection;
pub mod project_analysis;
pub mod file_impact;

// Re-export main functions for backward compatibility
pub use test_selection::{
    run_smart_test_selector, run_bdd_selector
};

pub use project_analysis::{
    run_multi_lang_analyzer, run_conditional_compilation_checker
};

pub use file_impact::{
    analyze_file_impact
};