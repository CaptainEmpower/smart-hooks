//! Examples module organization
//! 
//! This module organizes smart-hooks examples into focused sub-modules
//! following Single Responsibility Principle.

pub mod commands;
pub mod scenarios;

// Re-export main functions for backward compatibility
pub use commands::{
    generate_test_examples, generate_analyze_examples, generate_run_examples,
    generate_install_examples, generate_capabilities_examples, generate_list_examples,
    generate_validate_examples, generate_status_examples, generate_schema_examples,
    generate_check_examples,
};

#[cfg(feature = "claude-ai")]
pub use commands::generate_bdd_examples;

pub use scenarios::{
    generate_git_integration_examples, generate_automation_examples, generate_basic_usage_examples,
    generate_advanced_usage_examples, generate_ci_cd_examples, generate_development_examples,
    generate_troubleshooting_examples, get_available_scenarios
};

/// Get examples for a specific command
pub fn get_command_examples(command: &str) -> String {
    let examples = match command {
        "test" => generate_test_examples(),
        "analyze" => generate_analyze_examples(),
        "run" => generate_run_examples(), 
        "install" => generate_install_examples(),
        "capabilities" => generate_capabilities_examples(),
        "list" => generate_list_examples(),
        "validate" => generate_validate_examples(),
        "status" => generate_status_examples(),
        "schema" => generate_schema_examples(),
        "check" => generate_check_examples(),
        #[cfg(feature = "claude-ai")]
        "bdd" => generate_bdd_examples(),
        _ => return format!("No examples available for command: {}", command),
    };
    
    serde_json::to_string_pretty(&examples)
        .unwrap_or_else(|_| format!("Error formatting examples for: {}", command))
}

/// Get examples for a specific scenario
pub fn get_scenario_examples(scenario: &str) -> String {
    let examples = match scenario {
        "git_integration" => generate_git_integration_examples(),
        "automation" => generate_automation_examples(),
        "basic_usage" => generate_basic_usage_examples(),
        "advanced_usage" => generate_advanced_usage_examples(),
        "ci_cd" => generate_ci_cd_examples(),
        "development" => generate_development_examples(),
        "troubleshooting" => generate_troubleshooting_examples(),
        _ => return format!("No examples available for scenario: {}", scenario),
    };
    
    serde_json::to_string_pretty(&examples)
        .unwrap_or_else(|_| format!("Error formatting examples for: {}", scenario))
}

/// Get all available examples organized by category
pub fn get_all_examples() -> String {
    let all_examples = serde_json::json!({
        "commands": {
            "test": generate_test_examples(),
            "analyze": generate_analyze_examples(),
            "run": generate_run_examples(),
            "install": generate_install_examples(),
            "capabilities": generate_capabilities_examples(),
            "list": generate_list_examples(),
            "validate": generate_validate_examples(),
            "status": generate_status_examples(),
            "schema": generate_schema_examples(),
            "check": generate_check_examples(),
        },
        "scenarios": {
            "git_integration": generate_git_integration_examples(),
            "automation": generate_automation_examples(),
            "basic_usage": generate_basic_usage_examples(),
            "advanced_usage": generate_advanced_usage_examples(),
            "ci_cd": generate_ci_cd_examples(),
            "development": generate_development_examples(),
            "troubleshooting": generate_troubleshooting_examples(),
        }
    });

    serde_json::to_string_pretty(&all_examples)
        .unwrap_or_else(|_| "Error formatting all examples".to_string())
}

/// Get available example commands
pub fn get_available_example_commands() -> Vec<&'static str> {
    let mut commands = vec![
        "test", "analyze", "run", "install", "capabilities", 
        "list", "validate", "status", "schema", "check"
    ];
    
    #[cfg(feature = "claude-ai")]
    commands.push("bdd");
    
    commands
}

/// Get available example scenarios
pub fn get_available_example_scenarios() -> Vec<&'static str> {
    get_available_scenarios()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_examples_valid() {
        let examples = get_command_examples("test");
        assert!(!examples.contains("No examples available"));
        assert!(examples.contains("smart-hooks"));
    }

    #[test]
    fn test_get_command_examples_invalid() {
        let examples = get_command_examples("nonexistent");
        assert!(examples.contains("No examples available for command: nonexistent"));
    }

    #[test]
    fn test_get_scenario_examples_valid() {
        let examples = get_scenario_examples("git_integration");
        assert!(!examples.contains("No examples available"));
        assert!(examples.contains("scenario"));
    }

    #[test]
    fn test_get_scenario_examples_invalid() {
        let examples = get_scenario_examples("nonexistent");
        assert!(examples.contains("No examples available for scenario: nonexistent"));
    }

    #[test]
    fn test_get_all_examples() {
        let all_examples = get_all_examples();
        assert!(all_examples.contains("commands"));
        assert!(all_examples.contains("scenarios"));
        assert!(all_examples.contains("test"));
        assert!(all_examples.contains("git_integration"));
    }

    #[test]
    fn test_get_available_example_commands() {
        let commands = get_available_example_commands();
        assert!(commands.contains(&"test"));
        assert!(commands.contains(&"analyze"));
        assert!(commands.contains(&"capabilities"));
        assert!(commands.len() > 5);
    }

    #[test]
    fn test_get_available_example_scenarios() {
        let scenarios = get_available_example_scenarios();
        assert!(scenarios.contains(&"git_integration"));
        assert!(scenarios.contains(&"automation"));
        assert!(scenarios.len() > 3);
    }

    #[test]
    fn test_all_commands_have_examples() {
        let commands = get_available_example_commands();
        for command in commands {
            let examples = get_command_examples(command);
            assert!(!examples.contains("No examples available"));
            assert!(examples.len() > 50); // Should be substantial JSON content
        }
    }

    #[test]
    fn test_all_scenarios_have_examples() {
        let scenarios = get_available_example_scenarios();
        for scenario in scenarios {
            let examples = get_scenario_examples(scenario);
            assert!(!examples.contains("No examples available"));
            assert!(examples.len() > 50); // Should be substantial JSON content
        }
    }

    #[cfg(feature = "claude-ai")]
    #[test]
    fn test_bdd_examples_available_with_feature() {
        let commands = get_available_example_commands();
        assert!(commands.contains(&"bdd"));
        
        let bdd_examples = get_command_examples("bdd");
        assert!(!bdd_examples.contains("No examples available"));
        assert!(bdd_examples.contains("Claude AI"));
    }
}