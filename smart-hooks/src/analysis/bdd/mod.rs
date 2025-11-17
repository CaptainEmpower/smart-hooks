pub mod feature_discovery;
pub mod file_analyzer;
pub mod static_selector;
/// BDD feature selection module
/// Refactored into SRP-compliant submodules
pub mod types;

// Re-export main types for external use
pub use feature_discovery::BddFeatureDiscovery;
pub use file_analyzer::BddFileAnalyzer;
pub use static_selector::StaticBddSelector;
pub use types::{BddFeatureSelection, BddTestContext, FileChange, SelectedFeature};

use anyhow::Result;
use std::path::Path;

/// Main interface for BDD feature selection
pub struct BddFeatureSelector;

impl BddFeatureSelector {
    /// Should run a BDD feature for a specific file (simple heuristic)
    pub fn should_run_feature_for_file(file_path: &str, _feature_file: &str) -> bool {
        // Core business logic files
        if file_path.contains("core/")
            || file_path.contains("lib.rs")
            || file_path.contains("main.rs")
        {
            return true;
        }

        // API/handler files
        if file_path.contains("api/")
            || file_path.contains("handler/")
            || file_path.contains("controller/")
        {
            return true;
        }

        // Skip utility, test, and documentation files
        if file_path.contains("utils/")
            || file_path.contains("test/")
            || file_path.ends_with("README.md")
            || file_path.contains("doc/")
        {
            return false;
        }

        // Default to running for Rust source files
        file_path.ends_with(".rs")
    }

    /// Select BDD features using static analysis
    pub fn select_features_static(
        changed_files: &[FileChange],
        feature_files: &[String],
    ) -> Result<BddFeatureSelection> {
        StaticBddSelector::select_features_for_files(feature_files, changed_files)
    }

    /// Create BDD test context for a project
    pub fn create_test_context<P: AsRef<Path>>(
        project_root: P,
        changed_files: Vec<FileChange>,
    ) -> Result<BddTestContext> {
        BddFeatureDiscovery::create_bdd_context(project_root, changed_files)
    }

    /// Discover BDD features in a project
    pub fn discover_features<P: AsRef<Path>>(project_root: P) -> Result<Vec<String>> {
        BddFeatureDiscovery::discover_bdd_features(project_root)
    }

    /// Analyze staged files for BDD context
    pub fn analyze_staged_files() -> Result<Vec<FileChange>> {
        BddFileAnalyzer::analyze_staged_files()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::config::TestSelectorConfig;

    fn should_run_feature_for_file_with_config(
        file_path: &str,
        _feature_file: &str,
        config: &TestSelectorConfig,
    ) -> bool {
        // Check against behavioral patterns from config
        for pattern in &config.bdd_test_patterns {
            if file_path.contains(pattern) {
                return true;
            }
        }

        // Fallback to simple heuristic
        BddFeatureSelector::should_run_feature_for_file(file_path, _feature_file)
    }

    #[test]
    fn test_should_run_feature_for_file() {
        assert!(BddFeatureSelector::should_run_feature_for_file(
            "src/core/business_logic.rs",
            "any.feature"
        ));
        assert!(BddFeatureSelector::should_run_feature_for_file(
            "src/lib.rs",
            "any.feature"
        ));
        assert!(BddFeatureSelector::should_run_feature_for_file(
            "src/api/handlers.rs",
            "any.feature"
        ));
        assert!(!BddFeatureSelector::should_run_feature_for_file(
            "src/utils/helpers.rs",
            "any.feature"
        ));
        assert!(!BddFeatureSelector::should_run_feature_for_file(
            "README.md",
            "any.feature"
        ));
    }

    #[test]
    fn test_discover_bdd_features() {
        // Test with current directory (should work even if no features found)
        let result = BddFeatureSelector::discover_features(".");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hybrid_selection() {
        let config = TestSelectorConfig::default();

        assert!(should_run_feature_for_file_with_config(
            "src/core/business_logic.rs",
            "any.feature",
            &config
        ));
        assert!(should_run_feature_for_file_with_config(
            "src/api/controller.rs",
            "any.feature",
            &config
        ));
        assert!(!should_run_feature_for_file_with_config(
            "README.md",
            "any.feature",
            &config
        ));
        assert!(!should_run_feature_for_file_with_config(
            "src/utils/helpers.rs",
            "any.feature",
            &config
        ));
    }

    #[test]
    fn test_static_feature_selection() {
        let staged_files = vec![
            FileChange {
                file_path: "src/core/move_validator.rs".to_string(),
                change_type: "modified".to_string(),
                diff_summary: None,
                function_changes: vec!["validate_move".to_string()],
                line_count: 50,
            },
            FileChange {
                file_path: "src/apply/strategy.rs".to_string(),
                change_type: "added".to_string(),
                diff_summary: None,
                function_changes: vec!["apply_strategy".to_string()],
                line_count: 75,
            },
        ];

        // Use empty feature list since we don't have actual feature files
        let available_features = vec![];

        let selection =
            BddFeatureSelector::select_features_static(&staged_files, &available_features);

        // Should handle empty features gracefully
        assert!(selection.is_ok());
        let selection = selection.unwrap();
        assert!(!selection.should_run_bdd); // No features to run
        assert_eq!(selection.selected_features.len(), 0);
        assert!(selection.confidence >= 0.0);
    }
}