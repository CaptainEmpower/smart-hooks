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

    /// File patterns that should trigger BDD tests
    pub bdd_test_patterns: Vec<String>,

    /// Generic structural patterns for BDD test selection (domain-agnostic)
    pub bdd_structural_patterns: Option<BddStructuralPatterns>,

    /// Pattern-to-Tag mapping for cucumber BDD tests
    /// Maps file patterns to cucumber tags for semantic test selection
    pub bdd_pattern_tags: Option<HashMap<String, Vec<String>>>,

    /// Whether to enable content-based analysis (requires file reading)
    pub enable_content_analysis: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BddStructuralPatterns {
    /// Core business logic directories
    pub core_business_logic: Vec<String>,

    /// Application logic and strategy patterns
    pub application_logic: Vec<String>,

    /// Error handling and validation
    pub error_handling: Vec<String>,

    /// API endpoints and controllers
    pub api_interfaces: Vec<String>,

    /// Additional behavioral patterns (DDD, CQRS, etc.)
    pub behavioral_patterns: Vec<String>,
}

impl Default for BddStructuralPatterns {
    fn default() -> Self {
        Self {
            core_business_logic: vec![
                "/core/".to_string(),
                "/service/".to_string(),
                "/domain/".to_string(),
                "/business/".to_string(),
                "/logic/".to_string(),
            ],
            application_logic: vec![
                "/apply/".to_string(),
                "/strategy/".to_string(),
                "/handler/".to_string(),
                "/processor/".to_string(),
                "/workflow/".to_string(),
                "/pipeline/".to_string(),
            ],
            error_handling: vec![
                "/error".to_string(),
                "/validate".to_string(),
                "/types".to_string(),
                "/validation/".to_string(),
                "/exception/".to_string(),
            ],
            api_interfaces: vec![
                "/api/".to_string(),
                "/controller/".to_string(),
                "/endpoint/".to_string(),
                "/route/".to_string(),
                "/web/".to_string(),
                "/http/".to_string(),
            ],
            behavioral_patterns: vec![
                "/command/".to_string(),
                "/event/".to_string(),
                "/aggregate/".to_string(),
                "/saga/".to_string(),
                "/policy/".to_string(),
            ],
        }
    }
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
            bdd_test_patterns: vec![
                "/apply/".to_string(),
                "/strategy/".to_string(),
                "/fast_export/".to_string(),
                "file_mover_service.rs".to_string(),
                "service.rs".to_string(),
            ],
            bdd_structural_patterns: Some(BddStructuralPatterns::default()),
            bdd_pattern_tags: Some(Self::default_pattern_tags()),
            enable_content_analysis: false, // Disabled by default for safety
        }
    }
}

impl TestSelectorConfig {
    /// Default pattern-to-tag mapping for cucumber BDD tests
    fn default_pattern_tags() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();

        // Core business logic
        map.insert(
            "/core/".to_string(),
            vec!["@business-logic".to_string(), "@critical".to_string()],
        );
        map.insert(
            "/service/".to_string(),
            vec!["@business-logic".to_string(), "@integration".to_string()],
        );
        map.insert(
            "/domain/".to_string(),
            vec!["@business-logic".to_string(), "@domain-rules".to_string()],
        );

        // Application logic
        map.insert(
            "/apply/".to_string(),
            vec!["@workflow".to_string(), "@process".to_string()],
        );
        map.insert(
            "/strategy/".to_string(),
            vec!["@workflow".to_string(), "@decision".to_string()],
        );
        map.insert(
            "/handler/".to_string(),
            vec!["@workflow".to_string(), "@command".to_string()],
        );

        // Error handling
        map.insert(
            "/error".to_string(),
            vec!["@error-handling".to_string(), "@robustness".to_string()],
        );
        map.insert(
            "/validate".to_string(),
            vec!["@error-handling".to_string(), "@validation".to_string()],
        );
        map.insert(
            "/types".to_string(),
            vec!["@error-handling".to_string(), "@data-integrity".to_string()],
        );

        // API interfaces
        map.insert(
            "/api/".to_string(),
            vec!["@api".to_string(), "@external".to_string()],
        );
        map.insert(
            "/controller/".to_string(),
            vec!["@api".to_string(), "@web".to_string()],
        );

        map
    }

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

    /// Check if a file path should trigger BDD tests
    pub fn should_run_bdd_tests(&self, file_path: &str) -> bool {
        self.bdd_test_patterns
            .iter()
            .any(|pattern| file_path.contains(pattern))
    }

    /// Check if a file path matches any structural pattern for BDD testing
    /// This is domain-agnostic and based on software architecture patterns
    pub fn matches_structural_pattern(&self, file_path: &str) -> bool {
        if let Some(patterns) = &self.bdd_structural_patterns {
            // Check all pattern categories
            patterns
                .core_business_logic
                .iter()
                .chain(patterns.application_logic.iter())
                .chain(patterns.error_handling.iter())
                .chain(patterns.api_interfaces.iter())
                .chain(patterns.behavioral_patterns.iter())
                .any(|pattern| file_path.contains(pattern))
        } else {
            // Fallback to legacy bdd_test_patterns if structural patterns not configured
            self.should_run_bdd_tests(file_path)
        }
    }

    /// Get cucumber tags for a file path based on pattern mapping
    /// Returns empty vector if no patterns match
    pub fn get_cucumber_tags_for_file(&self, file_path: &str) -> Vec<String> {
        if let Some(pattern_tags) = &self.bdd_pattern_tags {
            let mut all_tags = Vec::new();

            for (pattern, tags) in pattern_tags {
                if file_path.contains(pattern) {
                    all_tags.extend(tags.clone());
                }
            }

            // Remove duplicates while preserving order
            let mut unique_tags = Vec::new();
            for tag in all_tags {
                if !unique_tags.contains(&tag) {
                    unique_tags.push(tag);
                }
            }

            unique_tags
        } else {
            Vec::new()
        }
    }

    /// Get cucumber command for running tests with specific tags
    pub fn get_cucumber_command_for_file(&self, file_path: &str) -> Option<String> {
        let tags = self.get_cucumber_tags_for_file(file_path);

        if tags.is_empty() {
            None
        } else {
            // Build cucumber command with OR-ed tags: cucumber --tags "@tag1 or @tag2"
            Some(format!("cucumber --tags \"{}\"", tags.join(" or ")))
        }
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
        assert!(!config.bdd_test_patterns.is_empty());
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
    fn test_bdd_test_patterns() {
        let config = TestSelectorConfig::default();

        assert!(config.should_run_bdd_tests("src/apply/strategy.rs"));
        assert!(config.should_run_bdd_tests("src/file_mover_service.rs"));
        assert!(!config.should_run_bdd_tests("src/types.rs"));
    }

    #[test]
    fn test_structural_patterns() {
        let config = TestSelectorConfig::default();

        // Test core business logic patterns
        assert!(config.matches_structural_pattern("src/core/processor.rs"));
        assert!(config.matches_structural_pattern("src/service/handler.rs"));
        assert!(config.matches_structural_pattern("src/domain/entity.rs"));

        // Test application logic patterns
        assert!(config.matches_structural_pattern("src/apply/workflow.rs"));
        assert!(config.matches_structural_pattern("src/strategy/selector.rs"));
        assert!(config.matches_structural_pattern("src/handler/command.rs"));

        // Test API patterns
        assert!(config.matches_structural_pattern("src/api/endpoint.rs"));
        assert!(config.matches_structural_pattern("src/controller/user.rs"));

        // Test error handling patterns
        assert!(config.matches_structural_pattern("src/error.rs"));
        assert!(config.matches_structural_pattern("src/validation/validator.rs"));

        // Test non-matching patterns
        assert!(!config.matches_structural_pattern("README.md"));
        assert!(!config.matches_structural_pattern("src/utils/helpers.rs"));
        assert!(!config.matches_structural_pattern("tests/fixtures/data.rs"));
    }

    #[test]
    fn test_cucumber_tags_for_file() {
        let config = TestSelectorConfig::default();

        // Test core business logic tags
        let tags = config.get_cucumber_tags_for_file("src/core/processor.rs");
        assert!(tags.contains(&"@business-logic".to_string()));
        assert!(tags.contains(&"@critical".to_string()));

        // Test API tags
        let tags = config.get_cucumber_tags_for_file("src/api/user_controller.rs");
        assert!(tags.contains(&"@api".to_string()));
        assert!(tags.contains(&"@external".to_string()));

        // Test workflow tags
        let tags = config.get_cucumber_tags_for_file("src/strategy/selector.rs");
        assert!(tags.contains(&"@workflow".to_string()));
        assert!(tags.contains(&"@decision".to_string()));

        // Test no tags for non-matching patterns
        let tags = config.get_cucumber_tags_for_file("README.md");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_cucumber_command_generation() {
        let config = TestSelectorConfig::default();

        // Test command generation for core files
        let cmd = config.get_cucumber_command_for_file("src/core/processor.rs");
        assert!(cmd.is_some());
        let cmd_str = cmd.unwrap();
        assert!(cmd_str.contains("cucumber --tags"));
        assert!(cmd_str.contains("@business-logic"));

        // Test no command for non-matching files
        let cmd = config.get_cucumber_command_for_file("README.md");
        assert!(cmd.is_none());
    }
}