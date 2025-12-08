/// Test configuration builder for crates
use anyhow::Result;
use std::path::Path;

use crate::project::types::{CrateTestConfig, CrateType};

/// Builds test configuration for individual crates
pub struct TestConfigBuilder;

impl TestConfigBuilder {
    /// Create test configuration for a crate
    pub fn create_test_config(
        crate_path: &Path,
        crate_type: &CrateType,
    ) -> Result<CrateTestConfig> {
        let mut unit_test_patterns =
            vec!["src/**/*.rs".to_string(), "tests/unit/**/*.rs".to_string()];

        let mut integration_test_dirs = Vec::new();
        let tests_dir = crate_path.join("tests");
        if tests_dir.exists() {
            integration_test_dirs.push(tests_dir);
        }

        // Add more patterns based on crate type
        match crate_type {
            CrateType::Library => {
                unit_test_patterns.push("src/lib.rs".to_string());
            }
            CrateType::Binary => {
                unit_test_patterns.push("src/main.rs".to_string());
            }
            CrateType::Mixed => {
                unit_test_patterns.push("src/lib.rs".to_string());
                unit_test_patterns.push("src/main.rs".to_string());
            }
        }

        Ok(CrateTestConfig {
            unit_test_patterns,
            integration_test_dirs,
            doc_tests_enabled: true,
            custom_test_commands: Vec::new(),
        })
    }
}