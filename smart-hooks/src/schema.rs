//! JSON schema generation for API validation
//! 
//! This module generates JSON schemas for smart-hooks commands,
//! enabling AI agents and tools to validate requests before execution.
//! Follows SRP by handling only schema generation concerns.

use anyhow::Result;
use serde_json::{json, Value};

/// Generate schema for the test command
pub fn get_test_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "TestCommand",
        "type": "object",
        "required": ["files"],
        "properties": {
            "files": {
                "type": "array",
                "items": {"type": "string"},
                "description": "List of files to analyze for test selection",
                "minItems": 0
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schema for the analyze command
pub fn get_analyze_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "AnalyzeCommand",
        "type": "object",
        "properties": {
            "project_root": {
                "type": "string",
                "description": "Project root directory",
                "default": ".",
                "pattern": "^[^\\0]+$"
            },
            "verbose": {
                "type": "boolean",
                "description": "Enable verbose output",
                "default": false
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schema for the run command
pub fn get_run_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "RunCommand",
        "type": "object",
        "properties": {
            "hooks": {
                "type": "array",
                "items": {
                    "type": "string",
                    "minLength": 1
                },
                "description": "Hook IDs to run",
                "uniqueItems": true
            },
            "all_files": {
                "type": "boolean",
                "description": "Run on all files in the repo",
                "default": false
            },
            "files": {
                "type": "array",
                "items": {
                    "type": "string",
                    "pattern": "^[^\\0]+$"
                },
                "description": "Specific files to run against",
                "uniqueItems": true
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schema for the install command
pub fn get_install_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "InstallCommand",
        "type": "object",
        "properties": {
            "hook_type": {
                "type": "string",
                "description": "Type of git hook to install",
                "enum": ["pre-commit", "pre-push", "commit-msg", "pre-rebase", "post-checkout"],
                "default": "pre-commit"
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schema for the capabilities command
pub fn get_capabilities_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "CapabilitiesCommand",
        "type": "object",
        "properties": {
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schema for the status command
pub fn get_status_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "StatusCommand",
        "type": "object",
        "properties": {
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schema for the check command
pub fn get_check_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "CheckCommand",
        "type": "object",
        "properties": {
            "path": {
                "type": ["string", "null"],
                "description": "Path to check (defaults to current directory)",
                "pattern": "^[^\\0]*$"
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        },
        "additionalProperties": false
    })).unwrap()
}

/// Generate schemas for all commands
pub fn get_all_commands_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "SmartHooksCommands",
        "type": "object",
        "properties": {
            "test": serde_json::from_str::<Value>(&get_test_command_schema()).unwrap(),
            "analyze": serde_json::from_str::<Value>(&get_analyze_command_schema()).unwrap(),
            "run": serde_json::from_str::<Value>(&get_run_command_schema()).unwrap(),
            "install": serde_json::from_str::<Value>(&get_install_command_schema()).unwrap(),
            "capabilities": serde_json::from_str::<Value>(&get_capabilities_command_schema()).unwrap(),
            "status": serde_json::from_str::<Value>(&get_status_command_schema()).unwrap(),
            "check": serde_json::from_str::<Value>(&get_check_command_schema()).unwrap()
        },
        "additionalProperties": false
    })).unwrap()
}

/// Get schema for a specific command by name
pub fn get_schema_for_command(command: &str) -> Result<String> {
    let schema = match command {
        "test" => get_test_command_schema(),
        "analyze" => get_analyze_command_schema(),
        "run" => get_run_command_schema(),
        "install" => get_install_command_schema(),
        "capabilities" => get_capabilities_command_schema(),
        "status" => get_status_command_schema(),
        "check" => get_check_command_schema(),
        "all" => get_all_commands_schema(),
        _ => {
            return Err(anyhow::anyhow!("Unknown command for schema: {}", command));
        }
    };
    Ok(schema)
}

/// Validate that a schema is valid JSON
pub fn validate_schema(schema_str: &str) -> Result<()> {
    serde_json::from_str::<Value>(schema_str)?;
    Ok(())
}

/// Get list of available schema commands
pub fn get_available_schema_commands() -> Vec<&'static str> {
    vec![
        "test", "analyze", "run", "install", 
        "capabilities", "status", "check", "all"
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_test_command_schema() {
        let schema = get_test_command_schema();
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(parsed["title"], "TestCommand");
        assert_eq!(parsed["type"], "object");
        
        let required = parsed["required"].as_array().unwrap();
        assert!(required.contains(&json!("files")));
    }

    #[test]
    fn test_analyze_command_schema() {
        let schema = get_analyze_command_schema();
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["title"], "AnalyzeCommand");
        assert!(parsed["properties"]["project_root"]["default"] == ".");
        assert!(parsed["properties"]["verbose"]["type"] == "boolean");
    }

    #[test]
    fn test_run_command_schema() {
        let schema = get_run_command_schema();
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["title"], "RunCommand");
        assert!(parsed["properties"]["hooks"]["type"] == "array");
        assert!(parsed["properties"]["all_files"]["type"] == "boolean");
    }

    #[test]
    fn test_install_command_schema() {
        let schema = get_install_command_schema();
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["title"], "InstallCommand");
        let hook_type_enum = parsed["properties"]["hook_type"]["enum"].as_array().unwrap();
        assert!(hook_type_enum.contains(&json!("pre-commit")));
        assert!(hook_type_enum.contains(&json!("pre-push")));
    }

    #[test]
    fn test_capabilities_command_schema() {
        let schema = get_capabilities_command_schema();
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["title"], "CapabilitiesCommand");
        assert!(parsed["properties"]["json"]["type"] == "boolean");
    }

    #[test]
    fn test_all_commands_schema() {
        let schema = get_all_commands_schema();
        let parsed: Value = serde_json::from_str(&schema).unwrap();

        assert_eq!(parsed["title"], "SmartHooksCommands");
        assert!(parsed["properties"]["test"].is_object());
        assert!(parsed["properties"]["analyze"].is_object());
        assert!(parsed["properties"]["run"].is_object());
        assert!(parsed["properties"]["install"].is_object());
    }

    #[test]
    fn test_get_schema_for_command() {
        let test_schema = get_schema_for_command("test").unwrap();
        let parsed: Value = serde_json::from_str(&test_schema).unwrap();
        assert_eq!(parsed["title"], "TestCommand");

        let invalid_result = get_schema_for_command("invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_validate_schema() {
        let valid_schema = r#"{"type": "object", "properties": {}}"#;
        assert!(validate_schema(valid_schema).is_ok());

        let invalid_schema = r#"{"type": "object", "properties": {"#;
        assert!(validate_schema(invalid_schema).is_err());
    }

    #[test]
    fn test_get_available_schema_commands() {
        let commands = get_available_schema_commands();
        assert!(commands.contains(&"test"));
        assert!(commands.contains(&"analyze"));
        assert!(commands.contains(&"run"));
        assert!(commands.contains(&"install"));
        assert!(commands.contains(&"all"));
    }

    #[test]
    fn test_all_schemas_are_valid() {
        for command in get_available_schema_commands() {
            let schema = get_schema_for_command(command).unwrap();
            validate_schema(&schema).unwrap();
        }
    }

    #[test]
    fn test_schema_has_required_fields() {
        let commands = ["test", "analyze", "run", "install"];
        
        for cmd in &commands {
            let schema = get_schema_for_command(cmd).unwrap();
            let parsed: Value = serde_json::from_str(&schema).unwrap();
            
            // All schemas should have these required fields
            assert!(parsed["$schema"].is_string());
            assert!(parsed["title"].is_string());
            assert!(parsed["type"] == "object");
            assert!(parsed["properties"].is_object());
        }
    }
}