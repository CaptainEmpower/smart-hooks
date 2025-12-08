//! Prek integration module organization
//! 
//! This module organizes prek integration into focused sub-modules
//! following Single Responsibility Principle.

pub mod commands;
pub mod detection;
pub mod fallback;

// Re-export main functions for backward compatibility
pub use commands::{run_prek_hooks, run_prek_install, run_prek_list, run_prek_validate};
pub use detection::is_prek_available;