/// Integration tests for BDD feature analysis modules
/// Tests BDD feature discovery, selection, and file analysis workflows
use smart_hooks::analysis::bdd::{
    types::FileChange, BddFeatureDiscovery, BddFeatureSelector, StaticBddSelector,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_project_with_bdd_features(temp_dir: &Path) -> Result<(), std::io::Error> {
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create Cargo.toml
    fs::write(
        temp_dir.join("Cargo.toml"),
        r#"
[package]
name = "bdd-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
"#,
    )?;

    // Create main source files
    fs::write(
        src_dir.join("lib.rs"),
        r#"
//! BDD test project library

pub mod user_management;
pub mod authentication;
pub mod data_processing;

pub use user_management::UserService;
pub use authentication::AuthService;
"#,
    )?;

    fs::write(
        src_dir.join("user_management.rs"),
        r#"
//! User management module

use anyhow::Result;

pub struct UserService {
    users: Vec<User>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub is_active: bool,
}

impl UserService {
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }

    pub fn create_user(&mut self, email: String) -> Result<u64> {
        let id = self.users.len() as u64 + 1;
        let user = User {
            id,
            email,
            is_active: true,
        };
        self.users.push(user);
        Ok(id)
    }

    pub fn deactivate_user(&mut self, user_id: u64) -> Result<()> {
        if let Some(user) = self.users.iter_mut().find(|u| u.id == user_id) {
            user.is_active = false;
            Ok(())
        } else {
            anyhow::bail!("User not found");
        }
    }

    pub fn get_user(&self, user_id: u64) -> Option<&User> {
        self.users.iter().find(|u| u.id == user_id)
    }
}
"#,
    )?;

    fs::write(
        src_dir.join("authentication.rs"),
        r#"
//! Authentication service

use anyhow::Result;

pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }

    pub fn authenticate_user(&self, email: &str, password: &str) -> Result<bool> {
        // Mock authentication logic
        if email.is_empty() || password.len() < 6 {
            return Ok(false);
        }
        Ok(email.contains("@") && password == "validpassword")
    }

    pub fn generate_token(&self, user_id: u64) -> Result<String> {
        Ok(format!("token_for_user_{}", user_id))
    }
}
"#,
    )?;

    fs::write(
        src_dir.join("data_processing.rs"),
        r#"
//! Data processing functionality

use anyhow::Result;

pub struct DataProcessor {
    batch_size: usize,
}

impl DataProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    pub fn process_batch(&self, items: &[String]) -> Result<Vec<String>> {
        if items.len() > self.batch_size {
            anyhow::bail!("Batch size exceeded");
        }

        let processed: Vec<String> = items.iter()
            .map(|item| format!("processed_{}", item))
            .collect();

        Ok(processed)
    }

    pub fn validate_data(&self, data: &str) -> bool {
        !data.is_empty() && data.len() < 1000
    }
}
"#,
    )?;

    // Create features directory structure
    let features_dir = temp_dir.join("features");
    fs::create_dir_all(&features_dir)?;

    // User management feature
    fs::write(
        features_dir.join("user_management.feature"),
        r#"
Feature: User Management
    As a system administrator
    I want to manage users in the system
    So that I can control access and maintain user accounts

    Scenario: Create a new user
        Given the user service is initialized
        When I create a user with email "test@example.com"
        Then the user should be created successfully
        And the user should be active

    Scenario: Deactivate an existing user
        Given the user service has a user with id 1
        When I deactivate the user with id 1
        Then the user should be deactivated
        And the user should not be active

    Scenario: Get user information
        Given the user service has a user with id 1
        When I request user information for id 1
        Then I should receive the user details
        And the user details should be correct

    Scenario: Handle invalid user operations
        Given the user service is initialized
        When I try to deactivate a non-existent user
        Then I should receive an error
        And no users should be affected
"#,
    )?;

    // Authentication feature
    fs::write(
        features_dir.join("authentication.feature"),
        r#"
Feature: User Authentication
    As a user
    I want to authenticate with the system
    So that I can access protected resources

    Scenario: Successful authentication
        Given the authentication service is available
        When I authenticate with email "user@example.com" and password "validpassword"
        Then the authentication should succeed
        And I should receive a valid token

    Scenario: Failed authentication with invalid email
        Given the authentication service is available
        When I authenticate with email "invalid-email" and password "validpassword"
        Then the authentication should fail
        And no token should be generated

    Scenario: Failed authentication with short password
        Given the authentication service is available
        When I authenticate with email "user@example.com" and password "123"
        Then the authentication should fail
        And no token should be generated

    Scenario: Token generation for authenticated user
        Given a user is successfully authenticated
        When I request a token for user id 42
        Then I should receive a token
        And the token should contain the user id
"#,
    )?;

    // Data processing feature
    fs::write(
        features_dir.join("data_processing.feature"),
        r#"
Feature: Data Processing
    As a data analyst
    I want to process data in batches
    So that I can efficiently handle large datasets

    Scenario: Process valid data batch
        Given a data processor with batch size 5
        When I process a batch with 3 items
        Then the processing should succeed
        And all items should be processed correctly

    Scenario: Reject oversized batch
        Given a data processor with batch size 5
        When I process a batch with 10 items
        Then the processing should fail
        And an error should indicate batch size exceeded

    Scenario: Validate data quality
        Given a data processor is initialized
        When I validate data "valid input"
        Then the validation should pass
        When I validate empty data ""
        Then the validation should fail

    Scenario: Handle edge cases
        Given a data processor with batch size 1
        When I process exactly 1 item
        Then the processing should succeed
        And the item should be processed correctly
"#,
    )?;

    // Create step definitions directory
    let steps_dir = features_dir.join("step_definitions");
    fs::create_dir_all(&steps_dir)?;

    fs::write(
        steps_dir.join("user_management_steps.rs"),
        r#"
// Step definitions for user management features
use cucumber::{given, when, then, World};
use bdd_test_project::{UserService, User};

#[derive(Debug, Default, World)]
pub struct UserWorld {
    pub user_service: Option<UserService>,
    pub created_user_id: Option<u64>,
    pub error_message: Option<String>,
    pub retrieved_user: Option<User>,
}

#[given("the user service is initialized")]
fn given_user_service_initialized(world: &mut UserWorld) {
    world.user_service = Some(UserService::new());
}

#[when(regex = r"^I create a user with email \"(.+)\"$")]
fn when_create_user(world: &mut UserWorld, email: String) {
    if let Some(ref mut service) = world.user_service {
        match service.create_user(email) {
            Ok(id) => world.created_user_id = Some(id),
            Err(e) => world.error_message = Some(e.to_string()),
        }
    }
}

#[then("the user should be created successfully")]
fn then_user_created_successfully(world: &UserWorld) {
    assert!(world.created_user_id.is_some());
}
"#,
    )?;

    Ok(())
}

#[test]
fn test_bdd_feature_discovery() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_bdd_features(temp_dir.path()).unwrap();

    // Test feature discovery
    let features = BddFeatureDiscovery::discover_bdd_features(temp_dir.path()).unwrap();

    assert_eq!(features.len(), 3);
    assert!(features
        .iter()
        .any(|f| f.ends_with("user_management.feature")));
    assert!(features
        .iter()
        .any(|f| f.ends_with("authentication.feature")));
    assert!(features
        .iter()
        .any(|f| f.ends_with("data_processing.feature")));
}

#[test]
fn test_bdd_feature_discovery_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create project without features
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "no-bdd-project"
version = "0.1.0"
"#,
    )
    .unwrap();

    let features = BddFeatureDiscovery::discover_bdd_features(temp_dir.path()).unwrap();
    assert_eq!(features.len(), 0);
}

#[test]
fn test_static_bdd_feature_selection() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_bdd_features(temp_dir.path()).unwrap();

    let features = BddFeatureDiscovery::discover_bdd_features(temp_dir.path()).unwrap();

    // Create file changes that should trigger BDD features
    let file_changes = vec![
        FileChange {
            file_path: "src/user_management.rs".to_string(),
            change_type: "modified".to_string(),
            diff_summary: None,
            function_changes: vec!["create_user".to_string(), "deactivate_user".to_string()],
            line_count: 25,
        },
        FileChange {
            file_path: "src/authentication.rs".to_string(),
            change_type: "modified".to_string(),
            diff_summary: None,
            function_changes: vec!["authenticate_user".to_string()],
            line_count: 15,
        },
    ];

    let selection = StaticBddSelector::select_features_for_files(&features, &file_changes).unwrap();

    assert!(selection.should_run_bdd);
    assert!(selection.confidence > 0.0);
    assert!(!selection.selected_features.is_empty());

    // Should select relevant features based on file names
    assert!(selection
        .selected_features
        .iter()
        .any(|f| f.feature_file.contains("user_management.feature")));
    assert!(selection
        .selected_features
        .iter()
        .any(|f| f.feature_file.contains("authentication.feature")));
}

#[test]
fn test_bdd_feature_selection_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_bdd_features(temp_dir.path()).unwrap();

    let features = BddFeatureDiscovery::discover_bdd_features(temp_dir.path()).unwrap();

    // Create file changes that don't match any features
    let file_changes = vec![
        FileChange {
            file_path: "src/unrelated_module.rs".to_string(),
            change_type: "added".to_string(),
            diff_summary: None,
            function_changes: vec!["some_unrelated_function".to_string()],
            line_count: 10,
        },
        FileChange {
            file_path: "README.md".to_string(),
            change_type: "modified".to_string(),
            diff_summary: None,
            function_changes: vec![],
            line_count: 5,
        },
    ];

    let selection = StaticBddSelector::select_features_for_files(&features, &file_changes).unwrap();

    assert!(!selection.should_run_bdd);
    assert_eq!(selection.selected_features.len(), 0);
    assert_eq!(selection.confidence, 0.0);
}

#[test]
fn test_bdd_test_context_creation() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_bdd_features(temp_dir.path()).unwrap();

    let file_changes = vec![FileChange {
        file_path: "src/user_management.rs".to_string(),
        change_type: "modified".to_string(),
        diff_summary: Some("Modified user creation logic".to_string()),
        function_changes: vec!["create_user".to_string()],
        line_count: 20,
    }];

    let context = BddFeatureDiscovery::create_bdd_context(temp_dir.path(), file_changes).unwrap();

    assert!(!context.feature_files.is_empty());
    assert!(!context.changed_files.is_empty());
    assert_eq!(context.changed_files.len(), 1);
}

#[test]
fn test_bdd_file_pattern_matching() {
    let temp_dir = TempDir::new().unwrap();

    // Test basic file extension pattern matching
    let test_cases = vec![
        "features/user.feature",
        "tests/features/auth.feature",
        "spec/user_behavior.feature",
    ];

    for file_path in test_cases {
        let full_path = temp_dir.path().join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, "").unwrap();

        // Just verify the file exists and has .feature extension
        assert!(full_path.exists());
        assert!(file_path.ends_with(".feature"));
    }
}

#[test]
fn test_bdd_integration_workflow() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_bdd_features(temp_dir.path()).unwrap();

    // Full workflow: discover features → analyze changes → select features

    // Step 1: Discover available features
    let available_features = BddFeatureSelector::discover_features(temp_dir.path()).unwrap();
    assert!(!available_features.is_empty());

    // Step 2: Simulate file changes
    let file_changes = vec![
        FileChange {
            file_path: "src/user_management.rs".to_string(),
            change_type: "modified".to_string(),
            diff_summary: Some("Updated user validation".to_string()),
            function_changes: vec!["create_user".to_string(), "deactivate_user".to_string()],
            line_count: 35,
        },
        FileChange {
            file_path: "src/data_processing.rs".to_string(),
            change_type: "modified".to_string(),
            diff_summary: Some("Fixed batch size validation".to_string()),
            function_changes: vec!["process_batch".to_string()],
            line_count: 15,
        },
    ];

    // Step 3: Select relevant features
    let selection =
        BddFeatureSelector::select_features_static(&file_changes, &available_features).unwrap();

    // Step 4: Create test context
    let context = BddFeatureSelector::create_test_context(temp_dir.path(), file_changes).unwrap();

    // Verify results
    assert!(selection.should_run_bdd);
    assert!(!selection.selected_features.is_empty());
    assert!(selection.confidence > 0.0);

    assert_eq!(context.changed_files.len(), 2);
    assert_eq!(context.feature_files.len(), available_features.len());

    // Should recommend user management and data processing features
    assert!(selection
        .selected_features
        .iter()
        .any(|f| f.feature_file.contains("user_management")));
    assert!(selection
        .selected_features
        .iter()
        .any(|f| f.feature_file.contains("data_processing")));
}

#[test]
fn test_bdd_feature_scenario_extraction() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_bdd_features(temp_dir.path()).unwrap();

    let features = BddFeatureDiscovery::discover_bdd_features(temp_dir.path()).unwrap();
    let user_feature = features
        .iter()
        .find(|f| f.contains("user_management.feature"))
        .expect("User management feature not found");

    // Test scenario extraction from feature file
    let file_changes = vec![FileChange {
        file_path: "src/user_management.rs".to_string(),
        change_type: "modified".to_string(),
        diff_summary: None,
        function_changes: vec!["create_user".to_string()],
        line_count: 10,
    }];

    let selection =
        StaticBddSelector::select_features_for_files(&[user_feature.clone()], &file_changes)
            .unwrap();

    let selected_feature = &selection.selected_features[0];
    assert!(!selected_feature.scenarios.is_empty());

    // Should extract scenario names from the feature file
    assert!(selected_feature
        .scenarios
        .iter()
        .any(|s| s.contains("Create a new user")));
    assert!(selected_feature
        .scenarios
        .iter()
        .any(|s| s.contains("Deactivate an existing user")));
}