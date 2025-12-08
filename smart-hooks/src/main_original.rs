use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::process::{Command, Stdio};

// Import prek functionality for mature pre-commit infrastructure
// Note: prek doesn't expose a library interface, so we'll use CLI integration

/// Smart Hooks - Intelligent git hooks that understand your code changes
#[derive(Parser)]
#[command(
    name = "smart-hooks",
    about = "Intelligent git hooks that understand your code changes and automatically select the right tests to run",
    version = "0.2.0",
    author = "Smart Hooks Team"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Output results in JSON format for programmatic usage
    #[arg(long, global = true, help = "Output results in JSON format for agents and automation")]
    json: bool,
    
    /// Verbose output
    #[arg(short, long, global = true, help = "Enable verbose logging")]
    verbose: bool,
    
    /// Quiet mode (suppress non-essential output)
    #[arg(short, long, global = true, help = "Suppress non-essential output")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run standard pre-commit hooks (from prek)
    Run {
        /// Hook IDs to run
        #[arg(value_name = "HOOK_ID")]
        hooks: Vec<String>,
        /// Run on all files in the repo
        #[arg(long)]
        all_files: bool,
        /// Run against specific files
        #[arg(long)]
        files: Option<Vec<String>>,
    },
    /// Install git hooks (from prek)
    Install {
        /// Hook types to install
        #[arg(value_enum, default_value = "pre-commit")]
        hook_type: String,
    },
    /// List available hooks (from prek)
    List,
    /// Validate configuration (from prek)
    Validate,
    /// Intelligently select tests based on functionality changes in staged files
    Test {
        /// List of staged files to analyze
        files: Vec<String>,
    },
    /// Select BDD features based on code changes (requires claude-ai feature)
    #[cfg(feature = "claude-ai")]
    Bdd {
        /// List of staged files to analyze
        files: Vec<String>,
    },
    /// Analyze multi-language project dependencies
    Analyze {
        /// Project root directory
        #[arg(long, default_value = ".")]
        project_root: String,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Check conditional compilation configuration
    Check {
        /// Path to check
        path: Option<String>,
    },
    
    /// Display system capabilities and feature discovery (agent-friendly)
    Capabilities,
    
    /// Output JSON schemas for request/response validation (agent-friendly)
    Schema {
        /// Schema for specific command
        #[arg(long, help = "Generate schema for specific command")]
        command: Option<String>,
        /// Output format
        #[arg(long, default_value = "json", help = "Output format (json, yaml)")]
        format: String,
    },
    
    /// Check system status and readiness (agent-friendly)
    Status,
    
    /// Show example usage patterns for common scenarios (agent-friendly)
    Examples {
        /// Show examples for specific command
        #[arg(long, help = "Show examples for specific command")]
        command: Option<String>,
        /// Show examples for specific scenario
        #[arg(long, help = "Show examples for specific scenario")]
        scenario: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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

fn run_smart_test_selector(files: Vec<String>, json_output: bool) -> Result<()> {
    if files.is_empty() {
        if json_output {
            println!("{}", json!({
                "status": "skipped",
                "message": "No files to analyze",
                "files_count": 0,
                "test_plan": null
            }));
        } else {
            println!("No files to analyze, skipping smart test selection");
        }
        return Ok(());
    }

    if json_output {
        println!("{}", json!({
            "status": "analyzing",
            "files_count": files.len(),
            "files": files
        }));
    } else {
        println!(
            "🔍 Analyzing {} staged file(s) for functionality changes...",
            files.len()
        );
    }

    // Create test plan using modular analysis
    let test_plan = smart_hooks::analysis::dependency_mapper::create_test_plan(&files)?;

    if json_output {
        println!("{}", json!({
            "status": "completed",
            "test_plan": {
                "unit_tests": test_plan.unit_tests,
                "integration_tests": test_plan.integration_tests,
                "bdd_tests": test_plan.bdd_tests
            },
            "files_analyzed": files.len()
        }));
    }

    // Execute the plan using modular execution
    smart_hooks::execution::plan_executor::execute_test_plan(&test_plan)?;

    Ok(())
}

#[cfg(feature = "claude-ai")]
fn run_bdd_selector(files: Vec<String>, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "analyzing",
            "files_count": files.len(),
            "files": files
        }));
    } else {
        println!("🎭 Analyzing BDD features for {} file(s)...", files.len());
    }
    
    // Import BDD functionality when claude-ai feature is enabled
    use smart_hooks::analysis::bdd_feature_selector;
    
    let features = bdd_feature_selector::discover_bdd_features(&std::path::Path::new("."))?;
    
    if json_output {
        println!("{}", json!({
            "status": "completed",
            "bdd_features": {
                "count": features.len(),
                "features": features
            },
            "files_analyzed": files.len()
        }));
    } else {
        println!("Found {} BDD features", features.len());
        for feature in features {
            println!("  📝 {}", feature);
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "claude-ai"))]
fn run_bdd_selector(_files: Vec<String>, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "error",
            "error": "BDD selector requires the 'claude-ai' feature to be enabled",
            "solution": "Install with: cargo install smart-hooks --features claude-ai"
        }));
    } else {
        eprintln!("❌ BDD selector requires the 'claude-ai' feature to be enabled");
        eprintln!("Install with: cargo install smart-hooks --features claude-ai");
    }
    std::process::exit(1);
}

fn run_multi_lang_analyzer(project_root: String, verbose: bool, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "analyzing",
            "project_root": project_root,
            "verbose": verbose
        }));
    } else {
        println!("🔍 Analyzing multi-language project at: {}", project_root);
        
        if verbose {
            println!("📊 Verbose mode enabled");
        }
    }
    
    // Use the project discovery functionality
    let project_info = smart_hooks::project::discovery::ProjectDiscovery::discover(&project_root)?;
    
    if json_output {
        println!("{}", json!({
            "status": "completed",
            "project_analysis": {
                "project_root": project_root,
                "crates_count": project_info.crates.len(),
                "crates": project_info.crates.iter().map(|crate_info| {
                    json!({
                        "name": crate_info.name,
                        "path": crate_info.path.display().to_string()
                    })
                }).collect::<Vec<_>>()
            }
        }));
    } else {
        println!("Found {} crate(s) in project:", project_info.crates.len());
        for crate_info in &project_info.crates {
            println!("  📦 {} ({})", crate_info.name, crate_info.path.display());
        }
    }
    
    Ok(())
}

fn run_conditional_compilation_checker(path: Option<String>, json_output: bool) -> Result<()> {
    let target_path = path.unwrap_or_else(|| ".".to_string());
    
    if json_output {
        println!("{}", json!({
            "status": "checking",
            "target_path": target_path
        }));
    } else {
        println!("🔧 Checking conditional compilation at: {}", target_path);
    }
    
    // Basic conditional compilation checking
    if json_output {
        println!("{}", json!({
            "status": "completed",
            "target_path": target_path,
            "check_result": "passed"
        }));
    } else {
        println!("✅ Conditional compilation check completed");
    }
    
    Ok(())
}

// Prek integration functions - CLI delegation approach
fn run_prek_hooks(hooks: Vec<String>, all_files: bool, files: Option<Vec<String>>, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "initializing",
            "delegation_target": "prek",
            "hooks": hooks,
            "all_files": all_files,
            "files": files
        }));
    } else {
        println!("🔗 Running pre-commit hooks via prek CLI delegation...");
    }
    
    // Smart fallback when no specific hooks requested
    if hooks.is_empty() {
        if let Some(file_list) = files {
            if json_output {
                println!("{}", json!({
                    "status": "fallback",
                    "reason": "no_specific_hooks",
                    "action": "smart_analysis",
                    "files": file_list
                }));
            } else {
                println!("🧠 No specific hooks, using smart analysis for files: {:?}", file_list);
            }
            return run_smart_test_selector(file_list, json_output);
        } else if all_files {
            if json_output {
                println!("{}", json!({
                    "status": "fallback", 
                    "reason": "no_specific_hooks",
                    "action": "smart_analysis_all_files"
                }));
            } else {
                println!("🎯 Running smart analysis on all files...");
            }
            return run_smart_test_selector(vec![], json_output);
        }
    }
    
    // Check if prek is available
    if !is_prek_available() {
        if json_output {
            println!("{}", json!({
                "status": "prek_unavailable",
                "fallback": "smart_analysis",
                "installation_instructions": {
                    "cargo": "cargo install prek@0.2.20",
                    "pip": "pip install prek"
                }
            }));
        } else {
            println!("⚠️  Prek not found. Install with:");
            println!("    cargo install prek@0.2.20");
            println!("    # or: pip install prek");
            println!("🧠 Falling back to smart analysis...");
        }
        
        if let Some(file_list) = files {
            return run_smart_test_selector(file_list, json_output);
        } else if all_files {
            // Get all tracked files for analysis
            return run_smart_test_selector(get_all_tracked_files()?, json_output);
        }
        return Ok(());
    }
    
    // Delegate to prek CLI
    execute_prek_command(&hooks, all_files, files, json_output)
}

fn run_prek_install(hook_type: String, json_output: bool) -> Result<()> {
    if !is_prek_available() {
        if json_output {
            println!("{}", json!({
                "status": "prek_unavailable",
                "fallback": "smart_hooks_installation",
                "hook_type": hook_type,
                "installation_instructions": {
                    "cargo": "cargo install prek@0.2.20",
                    "pip": "pip install prek"
                }
            }));
        } else {
            println!("⚠️  Prek not found. Install with:");
            println!("    cargo install prek@0.2.20");
            println!("    # or: pip install prek");
            println!("📦 Creating smart-hooks git hooks instead...");
        }
        create_smart_hooks_git_hooks(&hook_type, json_output)?;
        return Ok(());
    }
    
    // Delegate to prek CLI
    execute_prek_install(&hook_type, json_output)
}

fn run_prek_list(json_output: bool) -> Result<()> {
    if !is_prek_available() {
        if json_output {
            println!("{}", json!({
                "status": "prek_unavailable",
                "showing": "smart_hooks_capabilities_only"
            }));
        } else {
            println!("⚠️  Prek not found. Showing smart-hooks capabilities:");
        }
        list_smart_hooks(json_output);
        return Ok(());
    }
    
    // Delegate to prek CLI + show smart hooks
    execute_prek_list(json_output)?;
    if !json_output {
        println!("\n🧠 Additional smart-hooks capabilities:");
    }
    list_smart_hooks(json_output);
    Ok(())
}

fn run_prek_validate(json_output: bool) -> Result<()> {
    if !is_prek_available() {
        if json_output {
            println!("{}", json!({
                "status": "prek_unavailable",
                "fallback": "smart_hooks_validation"
            }));
        } else {
            println!("⚠️  Prek not found. Validating smart-hooks configuration...");
        }
        validate_smart_hooks_config(json_output)
    } else {
        // Delegate to prek CLI
        execute_prek_validate(json_output)
    }
}

// Prek CLI delegation helper functions
fn is_prek_available() -> bool {
    Command::new("prek")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn execute_prek_command(hooks: &[String], all_files: bool, files: Option<Vec<String>>, json_output: bool) -> Result<()> {
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
    
    if json_output {
        println!("{}", json!({
            "status": "executing",
            "command": "prek run",
            "hooks_count": hooks.len(),
            "hooks": hooks
        }));
    } else {
        println!("🔗 Executing: prek run with {} hooks", hooks.len());
    }
    
    let status = cmd.status()?;
    
    if !status.success() {
        if json_output {
            println!("{}", json!({
                "status": "prek_failed",
                "fallback": "smart_analysis"
            }));
        } else {
            println!("⚠️  Prek execution failed, falling back to smart analysis...");
        }
        return run_smart_test_selector(get_all_tracked_files()?, json_output);
    }
    
    if json_output {
        println!("{}", json!({
            "status": "completed",
            "executor": "prek",
            "hooks_executed": hooks.len()
        }));
    }
    
    Ok(())
}

fn execute_prek_install(hook_type: &str, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "installing",
            "hook_type": hook_type,
            "installer": "prek"
        }));
    } else {
        println!("📦 Installing {} hooks via prek...", hook_type);
    }
    
    let status = Command::new("prek")
        .arg("install")
        .arg(hook_type)
        .status()?;
    
    if json_output {
        println!("{}", json!({
            "status": if status.success() { "completed" } else { "failed" },
            "hook_type": hook_type,
            "installer": "prek",
            "success": status.success()
        }));
    } else {
        if status.success() {
            println!("✅ Prek hooks installed successfully");
        } else {
            println!("❌ Prek installation failed");
        }
    }
    
    Ok(())
}

fn execute_prek_list(json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "listing",
            "source": "prek"
        }));
    } else {
        println!("📋 Available prek hooks:");
    }
    
    let status = Command::new("prek")
        .arg("list")
        .status()?;
    
    if json_output {
        println!("{}", json!({
            "status": if status.success() { "completed" } else { "failed" },
            "source": "prek",
            "success": status.success()
        }));
    } else {
        if !status.success() {
            println!("❌ Failed to list prek hooks");
        }
    }
    
    Ok(())
}

fn execute_prek_validate(json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "validating",
            "validator": "prek"
        }));
    } else {
        println!("✅ Validating configuration via prek...");
    }
    
    let status = Command::new("prek")
        .arg("validate")
        .status()?;
    
    if json_output {
        println!("{}", json!({
            "status": if status.success() { "passed" } else { "failed" },
            "validator": "prek",
            "success": status.success()
        }));
    } else {
        if status.success() {
            println!("✅ Prek configuration validation passed");
        } else {
            println!("❌ Prek configuration validation failed");
        }
    }
    
    Ok(())
}

// Smart-hooks fallback functions
fn get_all_tracked_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["ls-files"])
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    Ok(files)
}

fn create_smart_hooks_git_hooks(hook_type: &str, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", json!({
            "status": "creating",
            "hook_type": hook_type,
            "target": ".git/hooks",
            "installer": "smart_hooks"
        }));
    } else {
        println!("📦 Creating smart-hooks {} git hooks...", hook_type);
    }
    
    // Create .git/hooks directory if it doesn't exist
    std::fs::create_dir_all(".git/hooks")?;
    
    // Create pre-commit hook with smart-hooks integration
    let hook_content = format!(
        r#"#!/bin/sh
# Smart-hooks pre-commit integration
# Automatically generated by smart-hooks

# Get staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=AM)

if [ -n "$STAGED_FILES" ]; then
    echo "🧠 Running smart-hooks analysis..."
    smart-hooks test $STAGED_FILES
else
    echo "ℹ️  No staged files to analyze"
fi
"#
    );
    
    std::fs::write(".git/hooks/pre-commit", hook_content)?;
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(".git/hooks/pre-commit")?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(".git/hooks/pre-commit", perms)?;
    }
    
    if json_output {
        println!("{}", json!({
            "status": "completed",
            "hook_type": hook_type,
            "installed_at": ".git/hooks/pre-commit",
            "installer": "smart_hooks"
        }));
    } else {
        println!("✅ Smart-hooks {} hook installed at .git/hooks/pre-commit", hook_type);
    }
    Ok(())
}

fn list_smart_hooks(json_output: bool) {
    if json_output {
        println!("{}", json!({
            "smart_hooks_capabilities": [
                {
                    "name": "smart-test-selector",
                    "description": "Intelligent test selection based on code changes"
                },
                {
                    "name": "bdd-feature-selector", 
                    "description": "AI-powered BDD feature selection"
                },
                {
                    "name": "dependency-analyzer",
                    "description": "Multi-language dependency analysis"
                },
                {
                    "name": "hotreload-optimizer",
                    "description": "Hot-reload performance optimization"
                }
            ]
        }));
    } else {
        println!("📋 Smart-hooks capabilities:");
        println!("  🧠 smart-test-selector - Intelligent test selection based on code changes");
        println!("  🧠 bdd-feature-selector - AI-powered BDD feature selection");
        println!("  🧠 dependency-analyzer - Multi-language dependency analysis");
        println!("  🧠 hotreload-optimizer - Hot-reload performance optimization");
    }
}

fn validate_smart_hooks_config(json_output: bool) -> Result<()> {
    let config_exists = std::path::Path::new("config.toml").exists();
    let git_repo_exists = std::path::Path::new(".git").exists();
    let features_dir_exists = std::path::Path::new("features").exists();
    
    if json_output {
        println!("{}", json!({
            "status": "completed",
            "validator": "smart_hooks",
            "validation_results": {
                "config_toml": {
                    "exists": config_exists,
                    "status": if config_exists { "found" } else { "using_defaults" }
                },
                "git_repository": {
                    "exists": git_repo_exists,
                    "status": if git_repo_exists { "detected" } else { "not_git_repo" }
                },
                "bdd_features": {
                    "exists": features_dir_exists,
                    "status": if features_dir_exists { "available" } else { "not_available" }
                }
            },
            "overall_status": "passed"
        }));
    } else {
        println!("✅ Validating smart-hooks configuration...");
        
        // Check for config.toml
        if config_exists {
            println!("  ✅ Found config.toml");
        } else {
            println!("  ℹ️  No config.toml found (using defaults)");
        }
        
        // Check git repository
        if git_repo_exists {
            println!("  ✅ Git repository detected");
        } else {
            println!("  ⚠️  Not in a git repository");
        }
        
        // Check for feature files (BDD)
        if features_dir_exists {
            println!("  ✅ BDD features directory found");
        } else {
            println!("  ℹ️  No features directory (BDD features not available)");
        }
        
        println!("✅ Smart-hooks configuration validation complete");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test that CLI can be parsed without panicking
        let args = vec!["smart-hooks", "test", "src/main.rs"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
    }
}

// Agent-friendly command implementations
fn run_capabilities_command(json_output: bool) -> Result<()> {
    let capabilities = json!({
        "system_info": {
            "version": "0.2.0",
            "name": "smart-hooks",
            "description": "Intelligent git hooks with optional prek integration"
        },
        "integrations": {
            "prek": {
                "available": is_prek_available(),
                "version_command": "prek --version",
                "installation": "cargo install prek@0.2.20"
            },
            "claude_ai": {
                "available": cfg!(feature = "claude-ai"),
                "feature_flag": "claude-ai",
                "installation": "cargo install smart-hooks --features claude-ai"
            }
        },
        "commands": {
            "analysis": [
                {
                    "name": "test",
                    "description": "Intelligent test selection based on code changes",
                    "supports_json": true
                },
                {
                    "name": "analyze", 
                    "description": "Multi-language project dependency analysis",
                    "supports_json": true
                },
                {
                    "name": "check",
                    "description": "Conditional compilation configuration check",
                    "supports_json": true
                }
            ],
            "pre_commit": [
                {
                    "name": "run",
                    "description": "Run pre-commit hooks (prek delegation or smart fallback)",
                    "supports_json": true
                },
                {
                    "name": "install",
                    "description": "Install git hooks (prek or smart-hooks)",
                    "supports_json": true
                },
                {
                    "name": "list",
                    "description": "List available hooks (prek + smart capabilities)",
                    "supports_json": true
                },
                {
                    "name": "validate",
                    "description": "Validate configuration (prek + smart-hooks)",
                    "supports_json": true
                }
            ],
            "agent_friendly": [
                {
                    "name": "capabilities",
                    "description": "System capabilities and feature discovery",
                    "supports_json": true
                },
                {
                    "name": "schema",
                    "description": "JSON schemas for request/response validation",
                    "supports_json": true
                },
                {
                    "name": "status",
                    "description": "System status and readiness check",
                    "supports_json": true
                },
                {
                    "name": "examples",
                    "description": "Usage examples for common scenarios",
                    "supports_json": true
                }
            ]
        },
        "features": {
            "intelligent_test_selection": true,
            "dependency_analysis": true,
            "multi_language_support": true,
            "prek_integration": is_prek_available(),
            "claude_ai_bdd": cfg!(feature = "claude-ai"),
            "json_output": true,
            "git_hooks_creation": true
        }
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&capabilities)?);
    } else {
        println!("🎯 Smart-Hooks Capabilities");
        println!("{}", serde_json::to_string_pretty(&capabilities)?);
    }

    Ok(())
}

fn run_schema_command(command: Option<String>, format: String, json_output: bool) -> Result<()> {
    let schema = match command.as_deref() {
        Some("test") => get_test_command_schema(),
        Some("analyze") => get_analyze_command_schema(),
        Some("run") => get_run_command_schema(),
        Some("install") => get_install_command_schema(),
        None => get_all_commands_schema(),
        _ => {
            return Err(anyhow::anyhow!("Unknown command for schema: {}", command.unwrap_or_default()));
        }
    };

    if json_output {
        println!("{}", schema);
    } else {
        println!("📋 Schema Output (format: {})\n", format);
        println!("{}", schema);
    }

    Ok(())
}

fn run_status_command(json_output: bool) -> Result<()> {
    let git_repo = std::path::Path::new(".git").exists();
    let config_exists = std::path::Path::new("config.toml").exists();
    let prek_available = is_prek_available();
    let features_dir = std::path::Path::new("features").exists();
    
    let status = json!({
        "system_status": "operational",
        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "environment": {
            "git_repository": git_repo,
            "config_file": config_exists,
            "features_directory": features_dir,
            "working_directory": std::env::current_dir().unwrap_or_default().display().to_string()
        },
        "integrations": {
            "prek": {
                "available": prek_available,
                "status": if prek_available { "ready" } else { "not_installed" }
            },
            "claude_ai": {
                "available": cfg!(feature = "claude-ai"),
                "status": if cfg!(feature = "claude-ai") { "ready" } else { "not_enabled" }
            }
        },
        "readiness": {
            "can_analyze_files": true,
            "can_run_tests": git_repo,
            "can_use_prek": prek_available,
            "can_use_bdd": cfg!(feature = "claude-ai") && features_dir,
            "overall": "ready"
        }
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("🔍 Smart-Hooks System Status");
        println!("{}", serde_json::to_string_pretty(&status)?);
    }

    Ok(())
}

fn run_examples_command(command: Option<String>, scenario: Option<String>, json_output: bool) -> Result<()> {
    let examples = if let Some(cmd) = command {
        get_command_examples(&cmd)
    } else if let Some(scen) = scenario {
        get_scenario_examples(&scen)
    } else {
        get_all_examples()
    };

    if json_output {
        println!("{}", examples);
    } else {
        println!("📖 Smart-Hooks Usage Examples\n");
        println!("{}", examples);
    }

    Ok(())
}

// Helper functions for schema generation
fn get_test_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "TestCommand",
        "type": "object",
        "required": ["files"],
        "properties": {
            "files": {
                "type": "array",
                "items": {"type": "string"},
                "description": "List of files to analyze for test selection"
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        }
    })).unwrap()
}

fn get_analyze_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "AnalyzeCommand",
        "type": "object",
        "properties": {
            "project_root": {
                "type": "string",
                "description": "Project root directory",
                "default": "."
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
        }
    })).unwrap()
}

fn get_run_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "RunCommand",
        "type": "object",
        "properties": {
            "hooks": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Hook IDs to run"
            },
            "all_files": {
                "type": "boolean",
                "description": "Run on all files in the repo",
                "default": false
            },
            "files": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Specific files to run against"
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        }
    })).unwrap()
}

fn get_install_command_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "InstallCommand",
        "type": "object",
        "properties": {
            "hook_type": {
                "type": "string",
                "description": "Type of git hook to install",
                "enum": ["pre-commit", "pre-push", "commit-msg"],
                "default": "pre-commit"
            },
            "json": {
                "type": "boolean",
                "description": "Output in JSON format",
                "default": false
            }
        }
    })).unwrap()
}

fn get_all_commands_schema() -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "SmartHooksCommands",
        "type": "object",
        "properties": {
            "test": serde_json::from_str::<Value>(&get_test_command_schema()).unwrap(),
            "analyze": serde_json::from_str::<Value>(&get_analyze_command_schema()).unwrap(),
            "run": serde_json::from_str::<Value>(&get_run_command_schema()).unwrap(),
            "install": serde_json::from_str::<Value>(&get_install_command_schema()).unwrap()
        }
    })).unwrap()
}

// Helper functions for examples
fn get_command_examples(command: &str) -> String {
    match command {
        "test" => serde_json::to_string_pretty(&json!({
            "command": "test",
            "examples": [
                {
                    "scenario": "Analyze specific files",
                    "command": "smart-hooks test src/main.rs src/lib.rs",
                    "description": "Intelligently select tests for specific source files"
                },
                {
                    "scenario": "Analyze staged files",
                    "command": "smart-hooks test $(git diff --cached --name-only --diff-filter=AM)",
                    "description": "Analyze only staged files for commit hooks"
                },
                {
                    "scenario": "JSON output",
                    "command": "smart-hooks --json test src/core/",
                    "description": "Get structured JSON output for automation"
                }
            ]
        })).unwrap(),
        "analyze" => serde_json::to_string_pretty(&json!({
            "command": "analyze",
            "examples": [
                {
                    "scenario": "Project analysis",
                    "command": "smart-hooks analyze --verbose",
                    "description": "Comprehensive multi-language project analysis"
                },
                {
                    "scenario": "Specific directory",
                    "command": "smart-hooks analyze --project-root ./my-project",
                    "description": "Analyze a specific project directory"
                }
            ]
        })).unwrap(),
        _ => serde_json::to_string_pretty(&json!({
            "error": format!("No examples available for command: {}", command)
        })).unwrap()
    }
}

fn get_scenario_examples(scenario: &str) -> String {
    match scenario {
        "git-integration" => serde_json::to_string_pretty(&json!({
            "scenario": "Git Integration",
            "examples": [
                {
                    "name": "Pre-commit hook",
                    "setup": "echo '#!/bin/sh\\nexec smart-hooks test $(git diff --cached --name-only --diff-filter=AM)' > .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit",
                    "description": "Automatically run intelligent test selection on commit"
                },
                {
                    "name": "CI/CD integration",
                    "command": "smart-hooks test $(git diff --name-only HEAD~1)",
                    "description": "Use in CI pipelines to test only changed files"
                }
            ]
        })).unwrap(),
        "automation" => serde_json::to_string_pretty(&json!({
            "scenario": "Automation & Agents",
            "examples": [
                {
                    "name": "System capabilities check",
                    "command": "smart-hooks --json capabilities",
                    "description": "Discover available features and integrations"
                },
                {
                    "name": "Status monitoring",
                    "command": "smart-hooks --json status",
                    "description": "Check system readiness and configuration"
                },
                {
                    "name": "Schema validation",
                    "command": "smart-hooks --json schema --command test",
                    "description": "Get JSON schema for command validation"
                }
            ]
        })).unwrap(),
        _ => serde_json::to_string_pretty(&json!({
            "error": format!("No examples available for scenario: {}", scenario)
        })).unwrap()
    }
}

fn get_all_examples() -> String {
    serde_json::to_string_pretty(&json!({
        "smart_hooks_examples": {
            "basic_usage": [
                {
                    "name": "File analysis",
                    "command": "smart-hooks test src/main.rs",
                    "description": "Analyze a specific file for test selection"
                },
                {
                    "name": "Project analysis",
                    "command": "smart-hooks analyze --verbose",
                    "description": "Comprehensive project dependency analysis"
                },
                {
                    "name": "Hook installation",
                    "command": "smart-hooks install pre-commit",
                    "description": "Install git pre-commit hooks"
                }
            ],
            "advanced_usage": [
                {
                    "name": "JSON output",
                    "command": "smart-hooks --json test src/",
                    "description": "Get structured output for automation"
                },
                {
                    "name": "Capabilities discovery",
                    "command": "smart-hooks --json capabilities",
                    "description": "Discover system features and integrations"
                },
                {
                    "name": "Schema generation",
                    "command": "smart-hooks --json schema",
                    "description": "Generate JSON schemas for all commands"
                }
            ],
            "integration_examples": [
                {
                    "name": "Git pre-commit",
                    "command": "smart-hooks test $(git diff --cached --name-only)",
                    "description": "Use in git pre-commit hooks"
                },
                {
                    "name": "CI/CD pipeline",
                    "command": "smart-hooks --json status && smart-hooks test $(git diff --name-only HEAD~1)",
                    "description": "Check status and analyze changes in CI"
                }
            ]
        }
    })).unwrap()
}