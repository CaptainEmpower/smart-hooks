pub mod bdd; // Refactored from bdd_feature_selector
/// Analysis module for smart test selection
///
/// This module provides various analyzers and detectors that help determine which tests to run
/// based on the nature of file changes, risk assessment, and intelligent mapping.
///
/// All submodules follow SRP with maximum 300 LOC per file.
pub mod bdd_detector;
pub mod claude_bdd_detector;
pub mod config;
pub mod dependency_mapper;
pub mod file_analyzer;