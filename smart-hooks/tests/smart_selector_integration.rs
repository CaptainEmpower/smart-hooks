use smart_hooks::{create_test_plan_with_config, TestSelectorConfig};

/// Integration tests for the smart test selector functionality
/// Tests the full pipeline from file analysis to test plan creation

#[test]
fn test_empty_file_list() {
    let config = TestSelectorConfig::path_based_only();
    let files = vec![];
    let plan = create_test_plan_with_config(&files, &config).unwrap();

    assert_eq!(plan.unit_tests.len(), 0);
    assert!(!plan.integration_tests);
}

#[test]
fn test_non_rust_files_ignored() {
    let config = TestSelectorConfig::path_based_only();
    let files = vec![
        "README.md".to_string(),
        "Cargo.toml".to_string(),
        "config.json".to_string(),
    ];
    let plan = create_test_plan_with_config(&files, &config).unwrap();

    assert_eq!(plan.unit_tests.len(), 0);
    assert!(!plan.integration_tests);
}

#[test]
fn test_core_validator_file_path_based() {
    // Test path-based matching only (no file content analysis)
    let config = TestSelectorConfig::path_based_only();
    let files = vec!["some/project/src/core/move_validator.rs".to_string()];
    let plan = create_test_plan_with_config(&files, &config).unwrap();

    println!("Unit tests found: {:?}", plan.unit_tests);
    assert!(plan.unit_tests.contains("core::move_validator"));
    assert!(plan.integration_tests);
}

#[test]
fn test_core_type_files_path_based() {
    // Test path-based matching for core modules
    let config = TestSelectorConfig::path_based_only();
    let files = vec![
        "some/project/src/types.rs".to_string(),
        "some/project/src/error.rs".to_string(),
    ];
    let plan = create_test_plan_with_config(&files, &config).unwrap();

    // These should trigger integration tests via module classification
    assert!(plan.integration_tests);
}

#[test]
fn test_mixed_file_types_path_based() {
    let config = TestSelectorConfig::path_based_only();
    let files = vec![
        "some/project/src/core/move_validator.rs".to_string(),
        "some/project/src/apply/strategy.rs".to_string(),
        "some/project/src/types.rs".to_string(),
        "README.md".to_string(), // Should be ignored
    ];
    let plan = create_test_plan_with_config(&files, &config).unwrap();

    // Should trigger all test types via path matching
    assert!(plan.unit_tests.contains("core::move_validator"));
    assert!(plan.integration_tests);
}
