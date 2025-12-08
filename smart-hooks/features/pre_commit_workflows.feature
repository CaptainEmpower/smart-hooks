Feature: Pre-commit Workflow Integration
  As a developer using pre-commit hooks
  I want smart-hooks to integrate seamlessly with my existing workflow
  So that I get intelligent analysis without changing my development process

  Scenario: Installing smart-hooks as pre-commit hooks
    Given I have an existing project with pre-commit configured
    When I add smart-hooks to my .pre-commit-config.yaml:
      """
      repos:
        - repo: https://github.com/your-org/smart-hooks
          rev: v0.2.0
          hooks:
            - id: smart-test-selector
            - id: smart-hooks-format
            - id: dependency-impact-analysis
      """
    Then pre-commit should successfully install smart-hooks
    And all smart-hooks commands should be available
    And the hooks should execute without errors on sample files

  Scenario: Smart test selector in pre-commit workflow
    Given I have smart-test-selector configured as a pre-commit hook
    When I commit changes to "src/core/processor.rs"
    Then the pre-commit hook should run automatically
    And it should only execute tests related to the processor module
    And it should complete significantly faster than running all tests
    And it should fail the commit if any selected tests fail
    And it should provide clear feedback about which tests were run

  Scenario: Format hook with multi-language support
    Given I have smart-hooks-format configured as a pre-commit hook
    When I commit changes to files in multiple languages:
      | file_path               | language   |
      | src/backend/api.rs      | rust       |
      | frontend/components.tsx | typescript |
      | scripts/build.py        | python     |
    Then the format hook should process all files appropriately
    And it should apply language-specific formatting rules
    And it should either fix formatting issues or fail with clear messages
    And it should only touch files that actually need formatting

  Scenario: Dependency impact analysis hook
    Given I have dependency-impact-analysis configured as a pre-commit hook
    When I commit changes that affect multiple modules
    Then the hook should analyze the full impact of my changes
    And it should warn about potential breaking changes
    And it should suggest additional tests that should be run
    And it should provide a summary of affected components

  Scenario: Hook failure handling and recovery
    Given smart-hooks hooks are configured in pre-commit
    When a smart-hooks command fails due to:
      | failure_type           | cause                              |
      | missing_dependencies   | cargo not installed                |
      | invalid_project       | not a valid Rust/multi-lang project |
      | analysis_timeout      | project too large for quick analysis |
    Then the hook should fail with a clear error message
    And it should provide specific instructions for resolution
    And it should not leave the git repository in an inconsistent state
    And subsequent runs should work after resolving the issue

  Scenario: Performance optimization for large commits
    Given I'm committing a large changeset with 50+ files
    When the smart-hooks pre-commit hooks execute
    Then analysis should complete within reasonable time (< 30 seconds)
    And memory usage should remain reasonable (< 500MB)
    And the hooks should provide progress indicators for long operations
    And users should be able to configure timeout limits

  Scenario: Integration with existing pre-commit ecosystem
    Given I have other pre-commit hooks configured:
      | hook_name           | purpose                    |
      | trailing-whitespace | remove trailing whitespace |
      | check-yaml         | validate YAML files       |
      | black              | Python code formatting     |
      | eslint             | JavaScript/TypeScript linting |
    When I run pre-commit with smart-hooks added
    Then smart-hooks should work alongside existing hooks
    And there should be no conflicts or interference
    And the execution order should be logical and predictable
    And smart-hooks should complement rather than duplicate existing functionality