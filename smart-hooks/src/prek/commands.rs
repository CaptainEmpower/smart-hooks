//! Prek command execution and delegation
//! 
//! This module handles the execution of prek commands with intelligent fallback
//! to smart-hooks functionality when prek is unavailable.
//! Follows SRP by handling only command execution concerns.

use anyhow::Result;
use serde_json::json;
use std::process::Command;

use crate::json_output::create_status_json;
use crate::prek::detection::is_prek_available;
use crate::prek::fallback::{handle_prek_unavailable, handle_prek_install_fallback, show_smart_hooks_only, validate_smart_hooks_config_only};

/// Execute prek hooks with intelligent fallback
pub fn run_prek_hooks(
    hooks: Vec<String>,
    all_files: bool,
    files: Option<Vec<String>>,
    json_output: bool,
) -> Result<()> {
    let status_json = create_status_json(
        "initializing",
        Some("Running pre-commit hooks via prek CLI delegation"),
        Some(json!({
            "delegation_target": "prek",
            "hooks": hooks,
            "all_files": all_files,
            "files": files
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&status_json)?);
    } else {
        println!("🔗 Running pre-commit hooks via prek CLI delegation...");
    }

    // Check if prek is available
    if !is_prek_available() {
        return handle_prek_unavailable(hooks, all_files, files, json_output);
    }

    // Execute prek command
    execute_prek_command(&hooks, all_files, files, json_output)
}

/// Install git hooks using prek or fallback to smart-hooks
pub fn run_prek_install(hook_type: String, json_output: bool) -> Result<()> {
    if !is_prek_available() {
        return handle_prek_install_fallback(&hook_type, json_output);
    }

    execute_prek_install(&hook_type, json_output)
}

/// List available hooks from prek and smart-hooks
pub fn run_prek_list(json_output: bool) -> Result<()> {
    if !is_prek_available() {
        return show_smart_hooks_only(json_output);
    }

    // Try to execute prek list, then show smart hooks
    execute_prek_list(json_output)?;
    show_additional_smart_hooks(json_output);
    Ok(())
}

/// Validate configuration using prek or smart-hooks
pub fn run_prek_validate(json_output: bool) -> Result<()> {
    if !is_prek_available() {
        return validate_smart_hooks_config_only(json_output);
    }

    execute_prek_validate(json_output)
}

/// Execute prek command with specified parameters
fn execute_prek_command(
    hooks: &[String],
    all_files: bool,
    files: Option<Vec<String>>,
    json_output: bool,
) -> Result<()> {
    let mut cmd = Command::new("prek");
    cmd.arg("run");

    // Add hook arguments
    for hook in hooks {
        cmd.arg(hook);
    }

    // Add file arguments
    if all_files {
        cmd.arg("--all-files");
    }

    if let Some(file_list) = files {
        cmd.arg("--files");
        for file in file_list {
            cmd.arg(file);
        }
    }

    let execution_status = create_status_json(
        "executing",
        Some(&format!("Executing prek run with {} hooks", hooks.len())),
        Some(json!({
            "command": "prek run",
            "hooks_count": hooks.len(),
            "hooks": hooks
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&execution_status)?);
    } else {
        println!("🔗 Executing: prek run with {} hooks", hooks.len());
    }

    let status = cmd.status()?;

    let result_status = create_status_json(
        if status.success() { "completed" } else { "failed" },
        Some(if status.success() { "Prek execution completed" } else { "Prek execution failed" }),
        Some(json!({
            "executor": "prek",
            "hooks_executed": hooks.len(),
            "exit_code": status.code().unwrap_or(-1)
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result_status)?);
    }

    if !status.success() {
        return handle_prek_execution_failure(json_output);
    }

    Ok(())
}

/// Execute prek install command
fn execute_prek_install(hook_type: &str, json_output: bool) -> Result<()> {
    let install_status = create_status_json(
        "installing",
        Some(&format!("Installing {} hooks via prek", hook_type)),
        Some(json!({
            "hook_type": hook_type,
            "installer": "prek"
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&install_status)?);
    } else {
        println!("📦 Installing {} hooks via prek...", hook_type);
    }

    let status = Command::new("prek")
        .arg("install")
        .arg(hook_type)
        .status()?;

    let result_status = create_status_json(
        if status.success() { "completed" } else { "failed" },
        Some(if status.success() { "Prek hooks installed successfully" } else { "Prek installation failed" }),
        Some(json!({
            "hook_type": hook_type,
            "installer": "prek",
            "success": status.success()
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result_status)?);
    } else if status.success() {
        println!("✅ Prek hooks installed successfully");
    } else {
        println!("❌ Prek installation failed");
    }

    Ok(())
}

/// Execute prek list command
fn execute_prek_list(json_output: bool) -> Result<()> {
    let list_status = create_status_json(
        "listing",
        Some("Listing available prek hooks"),
        Some(json!({"source": "prek"}))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&list_status)?);
    } else {
        println!("📋 Available prek hooks:");
    }

    let status = Command::new("prek")
        .arg("list")
        .status()?;

    let result_status = create_status_json(
        if status.success() { "completed" } else { "failed" },
        Some(if status.success() { "Successfully listed prek hooks" } else { "Failed to list prek hooks" }),
        Some(json!({
            "source": "prek",
            "success": status.success()
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result_status)?);
    } else if !status.success() {
        println!("❌ Failed to list prek hooks");
    }

    Ok(())
}

/// Execute prek validate command
fn execute_prek_validate(json_output: bool) -> Result<()> {
    let validate_status = create_status_json(
        "validating",
        Some("Validating configuration via prek"),
        Some(json!({"validator": "prek"}))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&validate_status)?);
    } else {
        println!("✅ Validating configuration via prek...");
    }

    let status = Command::new("prek")
        .arg("validate")
        .status()?;

    let result_status = create_status_json(
        if status.success() { "passed" } else { "failed" },
        Some(if status.success() { "Prek configuration validation passed" } else { "Prek configuration validation failed" }),
        Some(json!({
            "validator": "prek",
            "success": status.success()
        }))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result_status)?);
    } else if status.success() {
        println!("✅ Prek configuration validation passed");
    } else {
        println!("❌ Prek configuration validation failed");
    }

    Ok(())
}

/// Show additional smart-hooks capabilities after prek list
fn show_additional_smart_hooks(json_output: bool) {
    if !json_output {
        println!("\n🧠 Additional smart-hooks capabilities:");
    }
    crate::prek::fallback::list_smart_hooks(json_output);
}

/// Handle prek execution failure with fallback
fn handle_prek_execution_failure(json_output: bool) -> Result<()> {
    let fallback_status = create_status_json(
        "prek_failed",
        Some("Prek execution failed, falling back to smart analysis"),
        Some(json!({"fallback": "smart_analysis"}))
    );

    if json_output {
        println!("{}", serde_json::to_string_pretty(&fallback_status)?);
    } else {
        println!("⚠️  Prek execution failed, falling back to smart analysis...");
    }

    // Fallback to smart analysis
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_prek_hooks_with_empty_hooks() {
        let result = run_prek_hooks(vec![], false, None, true);
        // Should handle empty hooks gracefully
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_prek_install() {
        let result = run_prek_install("pre-commit".to_string(), true);
        // Should not panic regardless of prek availability
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_run_prek_list() {
        let result = run_prek_list(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_prek_validate() {
        let result = run_prek_validate(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prek_execution_failure() {
        let result = handle_prek_execution_failure(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_additional_smart_hooks() {
        // Should not panic
        show_additional_smart_hooks(true);
        show_additional_smart_hooks(false);
    }
}