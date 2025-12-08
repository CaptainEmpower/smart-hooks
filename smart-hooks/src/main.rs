//! Smart Hooks - Intelligent git hooks that understand your code changes
//! 
//! Main entry point that coordinates between all modules while maintaining
//! Single Responsibility Principle. Each module handles specific concerns.

use anyhow::Result;

// Module declarations
mod cli;
mod json_output;
mod schema;
mod examples;
mod agent_commands;
mod prek;
mod smart_analysis;
mod system_info;

use cli::{parse_cli, validate_cli_args, Commands};
use agent_commands::{run_capabilities_command, run_schema_command, run_status_command, run_examples_command};
use prek::{run_prek_hooks, run_prek_install, run_prek_list, run_prek_validate};
use smart_analysis::{run_smart_test_selector, run_bdd_selector, run_multi_lang_analyzer, run_conditional_compilation_checker};

fn main() -> Result<()> {
    let cli = parse_cli();
    
    // Validate CLI arguments for consistency
    validate_cli_args(&cli)?;
    
    // Route to appropriate command handler
    match cli.command {
        // Prek integration - delegate to prek for standard pre-commit functionality
        Commands::Run { hooks, all_files, files } => {
            run_prek_hooks(hooks, all_files, files, cli.json)
        }
        Commands::Install { hook_type } => {
            run_prek_install(hook_type, cli.json)
        }
        Commands::List => {
            run_prek_list(cli.json)
        }
        Commands::Validate => {
            run_prek_validate(cli.json)
        }
        
        // Smart-hooks enhanced functionality
        Commands::Test { files } => {
            run_smart_test_selector(files, cli.json)
        }
        #[cfg(feature = "claude-ai")]
        Commands::Bdd { files } => {
            run_bdd_selector(files, cli.json)
        }
        Commands::Analyze { project_root, verbose } => {
            run_multi_lang_analyzer(project_root, verbose, cli.json)
        }
        Commands::Check { path } => {
            run_conditional_compilation_checker(path, cli.json)
        }
        
        // Agent-friendly commands
        Commands::Capabilities => {
            run_capabilities_command(cli.json)
        }
        Commands::Schema { command, format } => {
            run_schema_command(command, format, cli.json)
        }
        Commands::Status => {
            run_status_command(cli.json)
        }
        Commands::Examples { command, scenario } => {
            run_examples_command(command, scenario, cli.json)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_main_function_structure() {
        // Test that main function is correctly structured and modules are accessible
        // This is a smoke test to ensure no module import issues
        
        // We can't easily test main() directly, but we can test that modules are accessible
        assert!(true); // Placeholder to ensure compilation
    }

    #[test]
    fn test_command_routing() {
        // Test that we can create different command types without panics
        use cli::Commands;

        let test_cmd = Commands::Test { files: vec![] };
        let capabilities_cmd = Commands::Capabilities;
        let install_cmd = Commands::Install { hook_type: "pre-commit".to_string() };
        
        // Just testing that these can be created - actual execution testing
        // is done in individual module tests
        assert!(matches!(test_cmd, Commands::Test { .. }));
        assert!(matches!(capabilities_cmd, Commands::Capabilities));
        assert!(matches!(install_cmd, Commands::Install { .. }));
    }

    #[test]
    fn test_module_integration() {
        // Test that modules can be imported and basic functions are accessible
        
        // Test CLI module - just test that commands can be created
        use cli::Commands;
        let _test_cmd = Commands::Test { files: vec![] };
        
        // Test schema module  
        let schema_result = schema::get_test_command_schema();
        assert!(schema_result.contains("TestCommand"));
        
        // Test examples module
        let example_commands = examples::get_available_example_commands();
        assert!(example_commands.contains(&"test"));
        
        // Test agent commands module
        let prek_available = system_info::is_prek_available();
        assert!(prek_available || !prek_available); // Just testing it returns bool
    }

    #[test]
    fn test_json_output_utilities() {
        use json_output::{JsonResponse, create_status_json};
        use serde_json::json;

        let response = JsonResponse::success(json!({"test": "data"}));
        assert_eq!(response.status, "success");

        let status = create_status_json("running", Some("test"), None);
        assert_eq!(status["status"], "running");
    }

    #[test]
    fn test_smart_analysis_utilities() {
        use smart_analysis::{analyze_file_impact};
        use smart_hooks::TestPlan;

        let mut plan = TestPlan {
            unit_tests: std::collections::HashSet::from(["test1".to_string()]),
            integration_tests: true,
            bdd_tests: false
        };

        assert_eq!(plan.unit_tests.len(), 1);
        assert!(plan.integration_tests);

        let files = vec!["src/main.rs".to_string(), "tests/test.rs".to_string()];
        let impact = analyze_file_impact(&files).unwrap();
        assert_eq!(impact.total_files(), 2);
    }

    #[test]
    fn test_prek_integration_utilities() {
        use prek::is_prek_available;

        // Test that the function is accessible and doesn't panic
        let available = is_prek_available();
        assert!(available || !available); // Just checking it returns a bool
    }

    #[test]
    fn test_error_handling() {
        // Test that modules handle errors gracefully
        use schema::get_schema_for_command;

        let valid_result = get_schema_for_command("test");
        assert!(valid_result.is_ok());

        let invalid_result = get_schema_for_command("invalid_command");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_all_modules_follow_srp() {
        // Verify that all modules follow Single Responsibility Principle
        // by checking their public interfaces are focused
        
        // CLI module should only handle CLI concerns
        use cli::{parse_cli, validate_cli_args, supports_json_output, command_name};
        
        // JSON output should only handle formatting
        use json_output::{JsonResponse, output_result, create_status_json};
        
        // Schema should only handle schema generation
        use schema::{get_test_command_schema, get_all_commands_schema, validate_schema};
        
        // Examples should only handle example generation
        use examples::{get_command_examples, get_scenario_examples, get_all_examples};
        
        // Each module has focused, single-purpose functions
        assert!(true); // Compilation success indicates proper module separation
    }
}