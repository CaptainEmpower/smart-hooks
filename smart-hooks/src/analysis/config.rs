use serde::{Deserialize, Serialize};
/// Test selection configuration
/// Makes the test selector configurable rather than hard-coded
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestSelectorConfig {
    /// File patterns that should trigger unit tests for specific modules
    pub unit_test_patterns: HashMap<String, String>,

    /// File patterns that should trigger integration tests
    pub integration_test_patterns: Vec<String>,

    /// Whether to enable content-based analysis (requires file reading)
    pub enable_content_analysis: bool,
}

impl Default for TestSelectorConfig {
    fn default() -> Self {
        let mut unit_test_patterns = HashMap::new();

        // Default patterns for common Rust project structures
        unit_test_patterns.insert(
            "move_validator.rs".to_string(),
            "move_validator".to_string(),
        );
        unit_test_patterns.insert(
            "history_processor.rs".to_string(),
            "history_processor".to_string(),
        );
        unit_test_patterns.insert(
            "file_mover_service.rs".to_string(),
            "file_mover_service".to_string(),
        );

        Self {
            unit_test_patterns,
            integration_test_patterns: vec![
                "/core/".to_string(),
                "/types.rs".to_string(),
                "/error.rs".to_string(),
                "/lib.rs".to_string(),
                "move_validator.rs".to_string(),
            ],
            enable_content_analysis: false, // Disabled by default for safety
        }
    }
}

impl TestSelectorConfig {
    /// Create config with content analysis enabled
    pub fn with_content_analysis() -> Self {
        Self {
            enable_content_analysis: true,
            ..Default::default()
        }
    }

    /// Create minimal config for path-based analysis only
    pub fn path_based_only() -> Self {
        Self {
            enable_content_analysis: false,
            ..Default::default()
        }
    }

    /// Load configuration from a file (if it exists)
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Check if a file path matches any unit test pattern
    pub fn matches_unit_test_pattern(&self, file_path: &str) -> Option<String> {
        for (pattern, module) in &self.unit_test_patterns {
            if file_path.contains(pattern) {
                return Some(module.clone());
            }
        }
        None
    }

    /// Check if a file path should trigger integration tests
    pub fn should_run_integration_tests(&self, file_path: &str) -> bool {
        self.integration_test_patterns
            .iter()
            .any(|pattern| file_path.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TestSelectorConfig::default();
        assert!(!config.enable_content_analysis);
        assert!(!config.unit_test_patterns.is_empty());
        assert!(!config.integration_test_patterns.is_empty());
    }

    #[test]
    fn test_unit_test_pattern_matching() {
        let config = TestSelectorConfig::default();

        assert_eq!(
            config.matches_unit_test_pattern("src/core/move_validator.rs"),
            Some("move_validator".to_string())
        );
        assert_eq!(config.matches_unit_test_pattern("src/other.rs"), None);
    }

    #[test]
    fn test_integration_test_patterns() {
        let config = TestSelectorConfig::default();

        assert!(config.should_run_integration_tests("src/core/mover.rs"));
        assert!(config.should_run_integration_tests("src/types.rs"));
        assert!(!config.should_run_integration_tests("src/apply/strategy.rs"));
    }

    #[test]
    fn a_config_without_optional_patterns_terminates() {
        // Regression for the review on #8: `should_run_bdd_tests` and
        // `matches_structural_pattern` called each other, so a config with
        // `bdd_structural_patterns: None` recursed until the stack overflowed
        // (SIGABRT) while the planner processed an ordinary Rust file. Both
        // functions are gone; this pins the behaviour that replaced them.
        let config = TestSelectorConfig {
            enable_content_analysis: false,
            ..Default::default()
        };

        assert!(!config.should_run_integration_tests("src/thing.rs"));
        assert_eq!(config.matches_unit_test_pattern("src/thing.rs"), None);
    }
}
