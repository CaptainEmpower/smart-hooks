//! Command-specific examples for smart-hooks
//! 
//! This module provides specific usage examples for individual commands,
//! helping users understand how to use each command effectively.
//! Follows SRP by handling only command-specific examples.

use serde_json::{json, Value};

/// Generate examples for the test command
pub fn generate_test_examples() -> Value {
    json!([
        {
            "title": "Basic Test Selection",
            "description": "Run intelligent test selection on specific files",
            "command": "smart-hooks test src/main.rs",
            "explanation": "Analyzes main.rs and selects relevant tests to run"
        },
        {
            "title": "Multiple File Analysis",
            "description": "Analyze multiple files for test impact",
            "command": "smart-hooks test src/lib.rs src/utils.rs",
            "explanation": "Analyzes both files and creates comprehensive test plan"
        },
        {
            "title": "JSON Output for CI/CD",
            "description": "Get structured JSON output for automation",
            "command": "smart-hooks test --json src/",
            "explanation": "Returns test plan in JSON format for CI/CD integration"
        },
        {
            "title": "Test All Changed Files",
            "description": "Analyze all staged files in git repository",
            "command": "smart-hooks test $(git diff --cached --name-only)",
            "explanation": "Uses git to get staged files and analyze their test impact"
        }
    ])
}

/// Generate examples for the analyze command
pub fn generate_analyze_examples() -> Value {
    json!([
        {
            "title": "Current Directory Analysis",
            "description": "Analyze the current directory for project structure",
            "command": "smart-hooks analyze .",
            "explanation": "Discovers and analyzes all crates in the current workspace"
        },
        {
            "title": "Specific Project Analysis",
            "description": "Analyze a specific project directory",
            "command": "smart-hooks analyze /path/to/project",
            "explanation": "Analyzes multi-language project at specified path"
        },
        {
            "title": "Verbose Analysis",
            "description": "Get detailed analysis output",
            "command": "smart-hooks analyze . --verbose",
            "explanation": "Provides detailed information about discovered crates and dependencies"
        },
        {
            "title": "JSON Analysis for Tools",
            "description": "Get analysis results in JSON format",
            "command": "smart-hooks analyze . --json",
            "explanation": "Returns structured analysis data for tool integration"
        }
    ])
}

/// Generate examples for the run command (prek integration)
pub fn generate_run_examples() -> Value {
    json!([
        {
            "title": "Run Specific Hooks",
            "description": "Execute specific pre-commit hooks via prek",
            "command": "smart-hooks run pre-commit pre-push",
            "explanation": "Delegates to prek to run specified hooks, falls back to smart analysis"
        },
        {
            "title": "Run on All Files",
            "description": "Execute hooks on all files in repository",
            "command": "smart-hooks run --all-files pre-commit",
            "explanation": "Runs pre-commit hooks on entire repository via prek delegation"
        },
        {
            "title": "Run on Specific Files",
            "description": "Execute hooks on selected files only",
            "command": "smart-hooks run --files src/main.rs src/lib.rs pre-commit",
            "explanation": "Runs hooks on specified files only through prek integration"
        },
        {
            "title": "JSON Output for CI",
            "description": "Get structured output from hook execution",
            "command": "smart-hooks run --json pre-commit",
            "explanation": "Returns execution results in JSON format for CI/CD processing"
        }
    ])
}

/// Generate examples for the install command
pub fn generate_install_examples() -> Value {
    json!([
        {
            "title": "Install Pre-commit Hooks",
            "description": "Install git pre-commit hooks",
            "command": "smart-hooks install pre-commit",
            "explanation": "Uses prek to install hooks, provides smart-hooks guidance as fallback"
        },
        {
            "title": "Install Pre-push Hooks", 
            "description": "Install git pre-push hooks",
            "command": "smart-hooks install pre-push",
            "explanation": "Installs pre-push hooks via prek with intelligent fallback"
        },
        {
            "title": "Install with JSON Feedback",
            "description": "Get installation status in JSON format",
            "command": "smart-hooks install --json pre-commit",
            "explanation": "Returns installation progress and results in structured format"
        }
    ])
}

/// Generate examples for the capabilities command
pub fn generate_capabilities_examples() -> Value {
    json!([
        {
            "title": "System Capabilities Overview",
            "description": "Get complete system capabilities information",
            "command": "smart-hooks capabilities",
            "explanation": "Shows all available features, integrations, and command capabilities"
        },
        {
            "title": "Analysis Domain Capabilities",
            "description": "Get capabilities for analysis domain",
            "command": "smart-hooks capabilities analysis",
            "explanation": "Shows intelligent test selection and dependency analysis capabilities"
        },
        {
            "title": "Integration Capabilities",
            "description": "Check integration tool availability",
            "command": "smart-hooks capabilities integration", 
            "explanation": "Shows prek integration status and git hooks compatibility"
        },
        {
            "title": "AI Capabilities Check",
            "description": "Verify AI-enhanced features availability",
            "command": "smart-hooks capabilities ai",
            "explanation": "Shows Claude AI BDD analysis and semantic capabilities"
        }
    ])
}

/// Generate examples for the list command
pub fn generate_list_examples() -> Value {
    json!([
        {
            "title": "List All Available Hooks",
            "description": "Show hooks from prek and smart-hooks capabilities",
            "command": "smart-hooks list",
            "explanation": "Lists prek hooks if available, plus smart-hooks capabilities"
        },
        {
            "title": "JSON Hook List",
            "description": "Get hook list in structured format",
            "command": "smart-hooks list --json",
            "explanation": "Returns available hooks and capabilities in JSON format"
        }
    ])
}

/// Generate examples for the validate command
pub fn generate_validate_examples() -> Value {
    json!([
        {
            "title": "Validate Configuration",
            "description": "Check prek and smart-hooks configuration",
            "command": "smart-hooks validate",
            "explanation": "Validates prek config if available, otherwise checks smart-hooks setup"
        },
        {
            "title": "JSON Validation Results",
            "description": "Get validation results in structured format", 
            "command": "smart-hooks validate --json",
            "explanation": "Returns detailed validation results and recommendations"
        }
    ])
}

/// Generate examples for the status command
pub fn generate_status_examples() -> Value {
    json!([
        {
            "title": "System Status Check",
            "description": "Get comprehensive system status",
            "command": "smart-hooks status",
            "explanation": "Shows environment, integrations, and readiness status"
        },
        {
            "title": "JSON Status for Monitoring",
            "description": "Get status in machine-readable format",
            "command": "smart-hooks status --json",
            "explanation": "Returns complete status information for monitoring systems"
        }
    ])
}

/// Generate examples for the schema command
pub fn generate_schema_examples() -> Value {
    json!([
        {
            "title": "Get Command Schema",
            "description": "Get JSON schema for a specific command",
            "command": "smart-hooks schema test",
            "explanation": "Returns JSON schema for test command validation"
        },
        {
            "title": "All Commands Schema",
            "description": "Get schemas for all available commands",
            "command": "smart-hooks schema --all",
            "explanation": "Returns comprehensive schema collection for API integration"
        },
        {
            "title": "Schema in Different Format",
            "description": "Get schema in specified format",
            "command": "smart-hooks schema test --format json",
            "explanation": "Explicitly request schema in JSON format"
        }
    ])
}

/// Generate examples for the check command
pub fn generate_check_examples() -> Value {
    json!([
        {
            "title": "Check Current Directory",
            "description": "Check conditional compilation in current location",
            "command": "smart-hooks check",
            "explanation": "Performs conditional compilation checks on current directory"
        },
        {
            "title": "Check Specific Path",
            "description": "Check conditional compilation at specific path",
            "command": "smart-hooks check ./src",
            "explanation": "Checks conditional compilation configuration in src directory"
        },
        {
            "title": "JSON Check Results",
            "description": "Get check results in structured format",
            "command": "smart-hooks check --json",
            "explanation": "Returns compilation check results for automated processing"
        }
    ])
}

#[cfg(feature = "claude-ai")]
/// Generate examples for the bdd command (requires claude-ai feature)
pub fn generate_bdd_examples() -> Value {
    json!([
        {
            "title": "BDD Feature Analysis",
            "description": "Analyze BDD features for specific files",
            "command": "smart-hooks bdd src/user_auth.rs",
            "explanation": "Uses Claude AI to discover relevant BDD features for authentication code"
        },
        {
            "title": "Multiple File BDD Analysis",
            "description": "Analyze BDD features for multiple files",
            "command": "smart-hooks bdd src/api/ src/models/",
            "explanation": "Discovers BDD features related to API and model changes"
        },
        {
            "title": "JSON BDD Output",
            "description": "Get BDD analysis in structured format",
            "command": "smart-hooks bdd --json src/",
            "explanation": "Returns BDD feature analysis for CI/CD integration"
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_examples() {
        let examples = generate_test_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_generate_analyze_examples() {
        let examples = generate_analyze_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_generate_run_examples() {
        let examples = generate_run_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_generate_install_examples() {
        let examples = generate_install_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_generate_capabilities_examples() {
        let examples = generate_capabilities_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_all_examples_have_required_fields() {
        let test_examples = generate_test_examples();
        for example in test_examples.as_array().unwrap() {
            assert!(example["title"].is_string());
            assert!(example["description"].is_string());
            assert!(example["command"].is_string());
            assert!(example["explanation"].is_string());
        }
    }

    #[test]
    fn test_examples_contain_smart_hooks_commands() {
        let test_examples = generate_test_examples();
        let first_example = &test_examples.as_array().unwrap()[0];
        let command = first_example["command"].as_str().unwrap();
        assert!(command.starts_with("smart-hooks"));
    }

    #[test] 
    fn test_json_examples_include_json_flag() {
        let test_examples = generate_test_examples();
        let json_example = test_examples.as_array().unwrap()
            .iter()
            .find(|ex| ex["command"].as_str().unwrap().contains("--json"))
            .unwrap();
        
        assert!(json_example["command"].as_str().unwrap().contains("--json"));
        assert!(json_example["explanation"].as_str().unwrap().to_lowercase().contains("json"));
    }

    #[cfg(feature = "claude-ai")]
    #[test]
    fn test_generate_bdd_examples() {
        let examples = generate_bdd_examples();
        assert!(examples.is_array());
        assert!(examples.as_array().unwrap().len() > 0);
        
        // Check that BDD examples mention Claude AI
        let first_example = &examples.as_array().unwrap()[0];
        let explanation = first_example["explanation"].as_str().unwrap();
        assert!(explanation.to_lowercase().contains("claude"));
    }
}