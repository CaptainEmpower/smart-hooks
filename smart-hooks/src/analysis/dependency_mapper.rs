use crate::analysis::config::TestSelectorConfig;
use crate::analysis::file_analyzer::analyze_file_content;
use crate::utilities::{file_utils, module_utils};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

/// Maps file changes to required test types
/// Focused on determining which tests to run based on change analysis

#[derive(Debug, Default, PartialEq)]
pub struct TestPlan {
    pub unit_tests: HashSet<String>,
    pub integration_tests: bool,
    pub bdd_tests: bool,
}

/// Create test plan based on list of changed files
pub fn create_test_plan(files: &[String]) -> Result<TestPlan> {
    create_test_plan_with_config(files, &TestSelectorConfig::default())
}

/// Create test plan with custom configuration
pub fn create_test_plan_with_config(
    files: &[String],
    config: &TestSelectorConfig,
) -> Result<TestPlan> {
    let mut plan = TestPlan::default();

    for file in files {
        if !file_utils::is_rust_file(file) {
            continue;
        }

        let path = Path::new(file);

        // Add configuration-based test mappings
        add_config_based_tests(file, &mut plan, config);

        // Add content-based test mappings only if enabled and safe
        if config.enable_content_analysis && file_utils::is_core_functionality_file(path) {
            add_content_based_tests(path, &mut plan)?;
        }
    }

    Ok(plan)
}

fn add_config_based_tests(file_path: &str, plan: &mut TestPlan, config: &TestSelectorConfig) {
    // Add unit tests based on configuration patterns
    if let Some(module_name) = config.matches_unit_test_pattern(file_path) {
        plan.unit_tests.insert(module_name);
    }

    // Also try to extract module name from path structure
    if let Some(module_name) = module_utils::extract_module_name(file_path) {
        plan.unit_tests.insert(module_name);
    }

    // Check configuration-based test triggers
    if config.should_run_integration_tests(file_path) {
        plan.integration_tests = true;
    }

    if config.should_run_bdd_tests(file_path) {
        plan.bdd_tests = true;
    }
}

fn add_content_based_tests(path: &Path, plan: &mut TestPlan) -> Result<()> {
    let flags = analyze_file_content(path)?;

    if flags.has_move_operations || flags.has_conflict_resolution {
        plan.bdd_tests = true;
    }

    if flags.has_core_types || flags.has_error_handling {
        plan.integration_tests = true;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_plan_empty() {
        let plan = create_test_plan(&[]).unwrap();
        assert_eq!(plan, TestPlan::default());
    }

    #[test]
    fn test_create_test_plan_non_rust_files() {
        let files = vec!["README.md".to_string(), "Cargo.toml".to_string()];
        let plan = create_test_plan(&files).unwrap();
        assert_eq!(plan, TestPlan::default());
    }

    #[test]
    fn test_config_based_core_validator() {
        let mut plan = TestPlan::default();
        let config = TestSelectorConfig::default();
        add_config_based_tests("any-project/src/core/move_validator.rs", &mut plan, &config);

        assert!(plan.unit_tests.contains("core::move_validator"));
        assert!(plan.integration_tests);
    }

    #[test]
    fn test_config_based_behavioral_module() {
        let mut plan = TestPlan::default();
        let config = TestSelectorConfig::default();
        add_config_based_tests("project/src/apply/strategy.rs", &mut plan, &config);

        assert!(plan.unit_tests.contains("apply::strategy"));
        assert!(plan.bdd_tests);
    }

    #[test]
    fn test_config_based_core_module() {
        let mut plan = TestPlan::default();
        let config = TestSelectorConfig::default();
        add_config_based_tests("crate/src/types.rs", &mut plan, &config);

        assert!(plan.unit_tests.contains("types"));
        assert!(plan.integration_tests);
    }
}
