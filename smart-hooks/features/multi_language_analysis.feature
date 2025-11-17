Feature: Multi-Language Project Analysis
  As a developer working on a polyglot project
  I want smart-hooks to analyze all my languages intelligently
  So that changes in one language don't break dependent code in other languages

  Scenario: Auto-detection of project languages
    Given I have a project with these files:
      | file_path                    | language   | purpose                |
      | src/main.rs                  | rust       | backend service        |
      | frontend/src/app.tsx         | typescript | react frontend         |
      | scripts/deploy.py            | python     | deployment script      |
      | api/routes/users.php         | php        | legacy API endpoint    |
      | package.json                 | config     | npm configuration      |
      | Cargo.toml                   | config     | rust configuration     |
    When smart-hooks analyzes the project
    Then it should detect Rust as the primary language
    And it should identify TypeScript, Python, and PHP as secondary languages
    And it should recognize the appropriate package managers for each
    And it should suggest the most relevant test frameworks

  Scenario: Cross-language dependency tracking
    Given I have a Rust backend that exposes a JSON API
    And I have TypeScript frontend that consumes this API
    And the API contract is defined in "api-schema.json"
    When I modify the Rust API endpoint structure
    Then smart-hooks should detect the TypeScript code that depends on this API
    And it should recommend running frontend integration tests
    And it should flag any TypeScript types that may need updating

  Scenario: Language-specific tool integration
    Given I have multi-language formatting configured
    When smart-hooks runs formatting on the entire project
    Then it should use cargo fmt for Rust files
    And it should use prettier for TypeScript files
    And it should use black for Python files
    And it should use php-cs-fixer for PHP files
    And it should handle tool failures gracefully with clear error messages

  Scenario: Unified test strategy across languages
    Given I have tests in multiple languages:
      | language   | test_type    | location                     |
      | rust       | unit         | src/*/tests.rs               |
      | rust       | integration  | tests/*.rs                   |
      | typescript | unit         | src/**/*.test.ts             |
      | typescript | e2e          | e2e/**/*.spec.ts             |
      | python     | unit         | test_*.py                    |
    When I modify a file that affects multiple languages
    Then smart-hooks should create a unified test plan
    And it should execute tests in the optimal order
    And it should aggregate results across all languages
    And it should provide a consolidated success/failure report

  Scenario: Language-specific complexity analysis
    Given I want to assess project health across languages
    When I run "smart-hooks summary --by-language"
    Then I should see complexity metrics for each language:
      | metric                    | rust | typescript | python | php |
      | lines_of_code            | yes  | yes        | yes    | yes |
      | cyclomatic_complexity    | yes  | yes        | yes    | yes |
      | dependency_count         | yes  | yes        | yes    | yes |
      | test_coverage           | yes  | yes        | yes    | yes |
    And I should see recommendations specific to each language's best practices

  Scenario: Incremental analysis for large polyglot projects
    Given I have a large project with 1000+ files across 5 languages
    When smart-hooks performs analysis
    Then it should cache language detection results
    And it should only re-analyze files that have changed
    And it should complete analysis in under 10 seconds for typical changes
    And it should provide progress indicators for long-running operations