//! Agent-friendly command implementations
//! 
//! This module implements commands specifically designed for AI agents and automation,
//! providing structured JSON output and comprehensive system information.
//! Follows SRP by handling only agent-oriented command execution.

use anyhow::Result;

use crate::json_output::output_result;
use crate::schema::get_schema_for_command;
use crate::examples::{get_command_examples, get_scenario_examples, get_all_examples};
use crate::system_info::{generate_capabilities_info, generate_status_info};

/// Execute the capabilities discovery command
pub fn run_capabilities_command(json_output: bool) -> Result<()> {
    let capabilities = generate_capabilities_info();

    output_result(
        json_output,
        capabilities.clone(),
        &format!("🎯 Smart-Hooks Capabilities\n{}", serde_json::to_string_pretty(&capabilities)?)
    )
}

/// Execute the schema generation command
pub fn run_schema_command(command: Option<String>, format: String, json_output: bool) -> Result<()> {
    let command_name = command.as_deref().unwrap_or("all");
    let schema = get_schema_for_command(command_name)?;

    if json_output {
        println!("{}", schema);
    } else {
        println!("📋 Schema Output (format: {})\n", format);
        println!("{}", schema);
    }

    Ok(())
}

/// Execute the system status check command
pub fn run_status_command(json_output: bool) -> Result<()> {
    let status = generate_status_info();

    output_result(
        json_output,
        status.clone(),
        &format!("🔍 Smart-Hooks System Status\n{}", serde_json::to_string_pretty(&status)?)
    )
}

/// Execute the examples command
pub fn run_examples_command(command: Option<String>, scenario: Option<String>, json_output: bool) -> Result<()> {
    let examples_json = if let Some(cmd) = command {
        get_command_examples(&cmd)
    } else if let Some(scen) = scenario {
        get_scenario_examples(&scen)
    } else {
        get_all_examples()
    };

    if json_output {
        println!("{}", examples_json);
    } else {
        println!("📖 Smart-Hooks Usage Examples\n");
        println!("{}", examples_json);
    }

    Ok(())
}

// Functions moved to system_info module for better SRP compliance

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_capabilities_command_json() {
        // Test that JSON output doesn't panic
        let result = run_capabilities_command(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_schema_command() {
        let result = run_schema_command(Some("test".to_string()), "json".to_string(), true);
        assert!(result.is_ok());

        let result = run_schema_command(None, "json".to_string(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_status_command() {
        let result = run_status_command(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_examples_command() {
        let result = run_examples_command(Some("test".to_string()), None, true);
        assert!(result.is_ok());

        let result = run_examples_command(None, Some("automation".to_string()), true);
        assert!(result.is_ok());

        let result = run_examples_command(None, None, true);
        assert!(result.is_ok());
    }
}