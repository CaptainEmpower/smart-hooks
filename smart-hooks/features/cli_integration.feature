Feature: CLI Command Integration
  As a developer using smart-hooks from the command line
  I want consistent and predictable behavior
  So that I can integrate it reliably in my CI/CD pipeline

  Scenario: Summary command provides project health insights
    Given I have a multi-language project with Rust, TypeScript, and Python
    When I run "smart-hooks summary --verbose"
    Then I should see a project overview with language breakdown
    And I should see complexity assessment for each language
    And I should see dependency analysis with external/internal counts
    And I should see actionable recommendations for improvement
    And the output should be properly formatted and readable

  Scenario: Format command handles multiple languages
    Given I have files with different formatting issues:
      | language   | file_path                 | issue                    |
      | rust       | src/core.rs              | incorrect indentation    |
      | typescript | src/frontend/api.ts      | missing semicolons       |
      | python     | scripts/data_processor.py | inconsistent spacing     |
    When I run "smart-hooks format auto --verbose"
    Then it should detect all three languages automatically
    And it should apply the appropriate formatter for each language
    And it should report the number of files formatted per language
    And all formatting issues should be resolved

  Scenario: Analyze command with dependency graphs
    Given I have a complex project with multiple modules
    When I run "smart-hooks analyze dependencies --graph --verbose"
    Then I should see a dependency graph visualization
    And I should see external vs internal dependency breakdown
    And I should see circular dependency warnings if any exist
    And I should see recommendations for reducing coupling

  Scenario: CLI argument validation and error handling
    Given smart-hooks is available on my system
    When I run invalid commands like:
      | command                              | expected_error                    |
      | smart-hooks test selective           | missing file arguments            |
      | smart-hooks format auto --invalid    | unrecognized option               |
      | smart-hooks analyze --non-existent  | invalid subcommand                |
    Then each command should fail with a clear error message
    And it should suggest the correct usage
    And it should exit with appropriate error codes

  Scenario: JSON output for CI integration
    Given I need to integrate smart-hooks with CI tools
    When I run commands with "--format json":
      | command                          |
      | smart-hooks summary --format json |
      | smart-hooks analyze dependencies --format json |
    Then the output should be valid JSON
    And it should contain all relevant data in structured format
    And it should be suitable for programmatic processing
    And error cases should also return structured JSON

  Scenario: Verbose vs quiet output modes
    Given I want to control output verbosity
    When I run the same command with different verbosity:
      | verbosity | command                        |
      | default   | smart-hooks test selective file.rs |
      | verbose   | smart-hooks test selective file.rs --verbose |
      | quiet     | smart-hooks test selective file.rs --quiet   |
    Then verbose mode should show detailed analysis steps
    And default mode should show summary information
    And quiet mode should show only essential output
    And all modes should preserve the same functionality