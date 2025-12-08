//! Scenario-based examples for smart-hooks
//! 
//! This module provides real-world usage scenarios and workflow examples,
//! helping users understand how to integrate smart-hooks into their development process.
//! Follows SRP by handling only scenario-based examples.

use serde_json::{json, Value};

/// Generate git integration workflow examples
pub fn generate_git_integration_examples() -> Value {
    json!([
        {
            "scenario": "Pre-commit Hook Setup",
            "description": "Set up smart-hooks for git pre-commit validation",
            "steps": [
                "smart-hooks install pre-commit",
                "git add .",
                "git commit -m 'Add feature'",
                "# smart-hooks automatically runs via prek or fallback"
            ],
            "explanation": "Demonstrates automatic execution during git commit process"
        },
        {
            "scenario": "Selective Testing on Commit",
            "description": "Run only tests affected by changes in commit",
            "steps": [
                "git add src/auth.rs src/models/user.rs",
                "smart-hooks test $(git diff --cached --name-only)",
                "git commit -m 'Update authentication logic'"
            ],
            "explanation": "Tests only what's necessary based on actual changes"
        },
        {
            "scenario": "Branch Analysis Workflow",
            "description": "Analyze changes in a feature branch",
            "steps": [
                "git checkout -b feature/new-api",
                "# Make changes...",
                "smart-hooks test $(git diff main --name-only)",
                "smart-hooks analyze ."
            ],
            "explanation": "Comprehensive analysis of all changes in feature branch"
        }
    ])
}

/// Generate automation and CI/CD integration examples
pub fn generate_automation_examples() -> Value {
    json!([
        {
            "scenario": "GitHub Actions Integration",
            "description": "Use smart-hooks in GitHub Actions workflow",
            "workflow": {
                "name": "Smart Testing",
                "on": ["push", "pull_request"],
                "jobs": {
                    "test": {
                        "runs-on": "ubuntu-latest",
                        "steps": [
                            {
                                "uses": "actions/checkout@v2"
                            },
                            {
                                "name": "Install smart-hooks",
                                "run": "cargo install smart-hooks"
                            },
                            {
                                "name": "Run intelligent tests",
                                "run": "smart-hooks test --json $(git diff HEAD^ --name-only) > test-results.json"
                            }
                        ]
                    }
                }
            },
            "explanation": "Integrates smart test selection into CI pipeline"
        },
        {
            "scenario": "Jenkins Pipeline Integration",
            "description": "Use smart-hooks in Jenkins declarative pipeline",
            "pipeline": {
                "agent": "any",
                "stages": [
                    {
                        "name": "Analysis",
                        "steps": [
                            "sh 'cargo install smart-hooks'",
                            "sh 'smart-hooks analyze . --json > analysis.json'",
                            "sh 'smart-hooks test --json $(git diff HEAD~1 --name-only) > tests.json'"
                        ]
                    }
                ]
            },
            "explanation": "Demonstrates Jenkins integration with JSON output processing"
        },
        {
            "scenario": "Docker Container Testing",
            "description": "Run smart-hooks analysis in Docker environment",
            "steps": [
                "docker run --rm -v $(pwd):/workspace rust:latest sh -c 'cd /workspace && cargo install smart-hooks && smart-hooks test --json .'",
                "# Process JSON output for test results"
            ],
            "explanation": "Containerized testing with smart-hooks for consistent environments"
        }
    ])
}

/// Generate basic workflow examples for new users
pub fn generate_basic_usage_examples() -> Value {
    json!([
        {
            "scenario": "First Time Setup",
            "description": "Getting started with smart-hooks",
            "steps": [
                "cargo install smart-hooks",
                "cd /path/to/your/project",
                "smart-hooks status",
                "smart-hooks capabilities"
            ],
            "explanation": "Initial setup and system capability discovery"
        },
        {
            "scenario": "Daily Development Workflow",
            "description": "Typical usage during development",
            "steps": [
                "# After making changes to code",
                "smart-hooks test src/modified_file.rs",
                "# Before committing",
                "smart-hooks run pre-commit",
                "git commit -m 'Fix bug in authentication'"
            ],
            "explanation": "Common development cycle with smart test selection"
        },
        {
            "scenario": "Project Analysis",
            "description": "Understanding your project structure",
            "steps": [
                "smart-hooks analyze .",
                "smart-hooks capabilities integration",
                "smart-hooks check"
            ],
            "explanation": "Comprehensive project analysis and readiness check"
        }
    ])
}

/// Generate advanced usage scenarios
pub fn generate_advanced_usage_examples() -> Value {
    json!([
        {
            "scenario": "Multi-language Project Analysis",
            "description": "Analyze complex multi-language projects",
            "steps": [
                "smart-hooks analyze . --verbose",
                "smart-hooks test --json $(find . -name '*.rs' -o -name '*.ts' -o -name '*.py')",
                "smart-hooks capabilities analysis"
            ],
            "explanation": "Handles projects with multiple programming languages"
        },
        {
            "scenario": "Custom Hook Integration",
            "description": "Integrate with existing custom git hooks",
            "steps": [
                "# Add to existing .git/hooks/pre-commit",
                "#!/bin/bash",
                "smart-hooks test $(git diff --cached --name-only) --json > /tmp/test-plan.json",
                "# Process test plan and run selected tests",
                "echo 'Smart hooks analysis completed'"
            ],
            "explanation": "Custom integration preserving existing git hook workflows"
        },
        {
            "scenario": "BDD Feature Development",
            "description": "Use BDD analysis for behavior-driven development (requires claude-ai)",
            "steps": [
                "cargo install smart-hooks --features claude-ai",
                "smart-hooks bdd src/user_stories/",
                "smart-hooks bdd --json features/ > bdd-analysis.json"
            ],
            "explanation": "AI-powered BDD feature analysis and test discovery"
        }
    ])
}

/// Generate CI/CD specific integration examples
pub fn generate_ci_cd_examples() -> Value {
    json!([
        {
            "scenario": "GitLab CI Integration",
            "description": "Smart-hooks in GitLab CI pipeline",
            "config": {
                "image": "rust:latest",
                "stages": ["test"],
                "test": {
                    "stage": "test",
                    "script": [
                        "cargo install smart-hooks",
                        "smart-hooks test --json $(git diff $CI_COMMIT_BEFORE_SHA --name-only) | tee test-results.json",
                        "smart-hooks analyze . --json > analysis.json"
                    ],
                    "artifacts": {
                        "reports": {
                            "junit": "test-results.json"
                        }
                    }
                }
            },
            "explanation": "GitLab CI with artifact collection and intelligent test selection"
        },
        {
            "scenario": "Azure DevOps Integration",
            "description": "Use smart-hooks in Azure DevOps pipeline",
            "steps": [
                "- task: Bash@3",
                "  inputs:",
                "    targetType: 'inline'",
                "    script: |",
                "      cargo install smart-hooks",
                "      smart-hooks test --json $(git diff HEAD~1 --name-only) > $(Agent.TempDirectory)/test-plan.json"
            ],
            "explanation": "Azure DevOps integration with test plan output"
        }
    ])
}

/// Generate development workflow examples
pub fn generate_development_examples() -> Value {
    json!([
        {
            "scenario": "Feature Branch Workflow",
            "description": "Smart testing for feature development",
            "steps": [
                "git checkout -b feature/payment-processing",
                "# Develop payment features...",
                "smart-hooks test src/payment/",
                "smart-hooks analyze . --verbose",
                "git add . && git commit -m 'Add payment processing'",
                "smart-hooks run pre-push"
            ],
            "explanation": "Complete feature development with intelligent testing"
        },
        {
            "scenario": "Refactoring Workflow",
            "description": "Safe refactoring with comprehensive testing",
            "steps": [
                "# Before refactoring",
                "smart-hooks test . --json > before-refactor.json",
                "# Perform refactoring...",
                "smart-hooks test $(git diff --name-only) --json > after-refactor.json",
                "# Compare test coverage and results"
            ],
            "explanation": "Ensures refactoring doesn't break existing functionality"
        },
        {
            "scenario": "Code Review Preparation", 
            "description": "Prepare comprehensive analysis for code review",
            "steps": [
                "smart-hooks analyze . --json > code-analysis.json",
                "smart-hooks test $(git diff main --name-only) --json > test-coverage.json",
                "smart-hooks capabilities > capabilities.json",
                "# Include all JSON files in PR description"
            ],
            "explanation": "Provides reviewers with comprehensive code impact analysis"
        }
    ])
}

/// Generate troubleshooting and debugging examples
pub fn generate_troubleshooting_examples() -> Value {
    json!([
        {
            "scenario": "Debugging Test Selection",
            "description": "Understand why specific tests were selected",
            "steps": [
                "smart-hooks test --json src/problematic_file.rs > debug.json",
                "cat debug.json | jq '.test_plan'",
                "smart-hooks analyze . --verbose"
            ],
            "explanation": "Provides detailed insight into test selection logic"
        },
        {
            "scenario": "Integration Issues",
            "description": "Diagnose prek integration problems",
            "steps": [
                "smart-hooks status --json",
                "smart-hooks capabilities integration",
                "smart-hooks validate --json"
            ],
            "explanation": "Comprehensive system diagnosis for integration issues"
        },
        {
            "scenario": "Performance Analysis",
            "description": "Analyze smart-hooks performance impact",
            "steps": [
                "time smart-hooks test $(git diff --cached --name-only)",
                "smart-hooks analyze . --json | jq '.project_analysis.crates_count'",
                "smart-hooks capabilities > system-info.json"
            ],
            "explanation": "Performance monitoring and optimization analysis"
        }
    ])
}

/// Get available scenario categories
pub fn get_available_scenarios() -> Vec<&'static str> {
    vec![
        "git_integration",
        "automation", 
        "basic_usage",
        "advanced_usage",
        "ci_cd",
        "development",
        "troubleshooting"
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_git_integration_examples() {
        let examples = generate_git_integration_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
        
        let first_scenario = &examples.as_array().unwrap()[0];
        assert!(first_scenario["scenario"].is_string());
        assert!(first_scenario["description"].is_string());
        assert!(first_scenario["steps"].is_array());
    }

    #[test]
    fn test_generate_automation_examples() {
        let examples = generate_automation_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_generate_basic_usage_examples() {
        let examples = generate_basic_usage_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_get_available_scenarios() {
        let scenarios = get_available_scenarios();
        assert!(scenarios.len() > 0);
        assert!(scenarios.contains(&"git_integration"));
        assert!(scenarios.contains(&"automation"));
        assert!(scenarios.contains(&"basic_usage"));
    }

    #[test]
    fn test_scenarios_have_required_fields() {
        let git_examples = generate_git_integration_examples();
        for example in git_examples.as_array().unwrap() {
            assert!(example["scenario"].is_string());
            assert!(example["description"].is_string());
            
            // Either steps or workflow/pipeline should be present
            assert!(example["steps"].is_array() || 
                    example["workflow"].is_object() ||
                    example["pipeline"].is_object() ||
                    example["config"].is_object());
        }
    }

    #[test]
    fn test_automation_examples_include_ci_systems() {
        let automation = generate_automation_examples();
        let scenarios_text = serde_json::to_string(&automation).unwrap();
        
        // Check that major CI systems are covered
        assert!(scenarios_text.contains("GitHub Actions") || scenarios_text.contains("github"));
        assert!(scenarios_text.contains("Jenkins") || scenarios_text.contains("jenkins"));
    }

    #[test]
    fn test_examples_contain_smart_hooks_commands() {
        let basic_usage = generate_basic_usage_examples();
        let steps_text = serde_json::to_string(&basic_usage).unwrap();
        assert!(steps_text.contains("smart-hooks"));
    }

    #[test]
    fn test_troubleshooting_includes_json_analysis() {
        let troubleshooting = generate_troubleshooting_examples();
        let content = serde_json::to_string(&troubleshooting).unwrap();
        assert!(content.contains("--json"));
        assert!(content.contains("jq") || content.contains("JSON"));
    }
}