Feature: Smart Test Selection
  As a developer using smart-hooks
  I want only relevant tests to run when I modify files
  So that I get faster feedback without running unnecessary tests

  Background:
    Given I have a Rust project with smart-hooks configured
    And the project has a comprehensive test suite

  Scenario: Modifying a core module triggers its tests
    Given I have a file "src/core/validator.rs" with unit tests
    When I modify "src/core/validator.rs"
    Then the smart-hooks test selector should run validator tests
    And it should run integration tests that depend on validator
    And it should skip unrelated module tests
    And the execution time should be significantly faster than running all tests

  Scenario: Changing a utility module affects dependent modules
    Given I have a utility module "src/utils/helpers.rs"
    And I have modules that depend on helpers
    When I modify "src/utils/helpers.rs"
    Then smart-hooks should identify all dependent modules
    And it should run tests for all affected modules
    And it should provide a clear reason for each selected test

  Scenario: Documentation changes don't trigger code tests
    Given I have documentation files like "README.md" and "docs/api.md"
    When I modify only documentation files
    Then smart-hooks should detect no code impact
    And it should skip all code tests
    And it should only run documentation-related checks if configured

  Scenario: Multi-file changes are analyzed comprehensively
    Given I modify multiple files: "src/core/processor.rs", "src/api/handlers.rs", "tests/integration/workflow.rs"
    When I run smart-hooks test selection
    Then it should analyze the combined impact of all changes
    And it should select the union of all relevant tests
    And it should avoid duplicate test execution
    And it should provide a comprehensive execution plan

  Scenario: Test plan confidence scoring
    Given I modify "src/experimental/new_feature.rs"
    When smart-hooks analyzes the change impact
    Then it should provide confidence scores for each selected test
    And high-confidence tests should be marked as critical
    And low-confidence tests should be marked as optional
    And the user should be able to filter by confidence level

  Scenario: Cross-language impact detection
    Given I have a polyglot project with Rust and TypeScript
    And the Rust code exposes an API consumed by TypeScript
    When I modify the Rust API interface
    Then smart-hooks should detect TypeScript tests that need to run
    And it should run both Rust and TypeScript test suites
    And it should explain the cross-language dependency reasoning