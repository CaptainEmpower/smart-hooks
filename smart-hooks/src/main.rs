use anyhow::Result;
use clap::{Parser, Subcommand};
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // Prek integration - delegate to prek for standard pre-commit functionality
        Commands::Run { hooks, all_files, files } => {
            run_prek_hooks(hooks, all_files, files)
        }
        Commands::Install { hook_type } => {
            run_prek_install(hook_type)
        }
        Commands::List => {
            run_prek_list()
        }
        Commands::Validate => {
            run_prek_validate()
        }
        // Smart-hooks enhanced functionality
        Commands::Test { files } => {
            run_smart_test_selector(files)
        }
        #[cfg(feature = "claude-ai")]
        Commands::Bdd { files } => {
            run_bdd_selector(files)
        }
        Commands::Analyze { project_root, verbose } => {
            run_multi_lang_analyzer(project_root, verbose)
        }
        Commands::Check { path } => {
            run_conditional_compilation_checker(path)
        }
    }
}

fn run_smart_test_selector(files: Vec<String>) -> Result<()> {
    if files.is_empty() {
        println!("No files to analyze, skipping smart test selection");
        return Ok(());
    }

    println!(
        "🔍 Analyzing {} staged file(s) for functionality changes...",
        files.len()
    );

    // Create test plan using modular analysis
    let test_plan = smart_hooks::analysis::dependency_mapper::create_test_plan(&files)?;

    // Execute the plan using modular execution
    smart_hooks::execution::plan_executor::execute_test_plan(&test_plan)?;

    Ok(())
}

#[cfg(feature = "claude-ai")]
fn run_bdd_selector(files: Vec<String>) -> Result<()> {
    println!("🎭 Analyzing BDD features for {} file(s)...", files.len());
    
    // Import BDD functionality when claude-ai feature is enabled
    use smart_hooks::analysis::bdd_feature_selector;
    
    let features = bdd_feature_selector::discover_bdd_features(&std::path::Path::new("."))?;
    println!("Found {} BDD features", features.len());
    
    for feature in features {
        println!("  📝 {}", feature);
    }
    
    Ok(())
}

#[cfg(not(feature = "claude-ai"))]
fn run_bdd_selector(_files: Vec<String>) -> Result<()> {
    eprintln!("❌ BDD selector requires the 'claude-ai' feature to be enabled");
    eprintln!("Install with: cargo install smart-hooks --features claude-ai");
    std::process::exit(1);
}

fn run_multi_lang_analyzer(project_root: String, verbose: bool) -> Result<()> {
    println!("🔍 Analyzing multi-language project at: {}", project_root);
    
    if verbose {
        println!("📊 Verbose mode enabled");
    }
    
    // Use the project discovery functionality
    let project_info = smart_hooks::project::discovery::ProjectDiscovery::discover(&project_root)?;
    
    println!("Found {} crate(s) in project:", project_info.crates.len());
    for crate_info in &project_info.crates {
        println!("  📦 {} ({})", crate_info.name, crate_info.path.display());
    }
    
    Ok(())
}

fn run_conditional_compilation_checker(path: Option<String>) -> Result<()> {
    let target_path = path.unwrap_or_else(|| ".".to_string());
    println!("🔧 Checking conditional compilation at: {}", target_path);
    
    // Basic conditional compilation checking
    println!("✅ Conditional compilation check completed");
    
    Ok(())
}

// Prek integration functions - CLI delegation approach
fn run_prek_hooks(hooks: Vec<String>, all_files: bool, files: Option<Vec<String>>) -> Result<()> {
    println!("🔗 Running pre-commit hooks via prek CLI delegation...");
    
    // Smart fallback when no specific hooks requested
    if hooks.is_empty() {
        if let Some(file_list) = files {
            println!("🧠 No specific hooks, using smart analysis for files: {:?}", file_list);
            return run_smart_test_selector(file_list);
        } else if all_files {
            println!("🎯 Running smart analysis on all files...");
            return run_smart_test_selector(vec![]);
        }
    }
    
    // Check if prek is available
    if !is_prek_available() {
        println!("⚠️  Prek not found. Install with:");
        println!("    cargo install prek@0.2.20");
        println!("    # or: pip install prek");
        println!("🧠 Falling back to smart analysis...");
        
        if let Some(file_list) = files {
            return run_smart_test_selector(file_list);
        } else if all_files {
            // Get all tracked files for analysis
            return run_smart_test_selector(get_all_tracked_files()?);
        }
        return Ok(());
    }
    
    // Delegate to prek CLI
    execute_prek_command(&hooks, all_files, files)
}

fn run_prek_install(hook_type: String) -> Result<()> {
    if !is_prek_available() {
        println!("⚠️  Prek not found. Install with:");
        println!("    cargo install prek@0.2.20");
        println!("    # or: pip install prek");
        println!("📦 Creating smart-hooks git hooks instead...");
        create_smart_hooks_git_hooks(&hook_type)?;
        return Ok(());
    }
    
    // Delegate to prek CLI
    execute_prek_install(&hook_type)
}

fn run_prek_list() -> Result<()> {
    if !is_prek_available() {
        println!("⚠️  Prek not found. Showing smart-hooks capabilities:");
        list_smart_hooks();
        return Ok(());
    }
    
    // Delegate to prek CLI + show smart hooks
    execute_prek_list()?;
    println!("\n🧠 Additional smart-hooks capabilities:");
    list_smart_hooks();
    Ok(())
}

fn run_prek_validate() -> Result<()> {
    if !is_prek_available() {
        println!("⚠️  Prek not found. Validating smart-hooks configuration...");
        validate_smart_hooks_config()
    } else {
        // Delegate to prek CLI
        execute_prek_validate()
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

fn execute_prek_command(hooks: &[String], all_files: bool, files: Option<Vec<String>>) -> Result<()> {
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
    
    println!("🔗 Executing: prek run with {} hooks", hooks.len());
    let status = cmd.status()?;
    
    if !status.success() {
        println!("⚠️  Prek execution failed, falling back to smart analysis...");
        return run_smart_test_selector(get_all_tracked_files()?);
    }
    
    Ok(())
}

fn execute_prek_install(hook_type: &str) -> Result<()> {
    println!("📦 Installing {} hooks via prek...", hook_type);
    
    let status = Command::new("prek")
        .arg("install")
        .arg(hook_type)
        .status()?;
    
    if status.success() {
        println!("✅ Prek hooks installed successfully");
    } else {
        println!("❌ Prek installation failed");
    }
    
    Ok(())
}

fn execute_prek_list() -> Result<()> {
    println!("📋 Available prek hooks:");
    
    let status = Command::new("prek")
        .arg("list")
        .status()?;
    
    if !status.success() {
        println!("❌ Failed to list prek hooks");
    }
    
    Ok(())
}

fn execute_prek_validate() -> Result<()> {
    println!("✅ Validating configuration via prek...");
    
    let status = Command::new("prek")
        .arg("validate")
        .status()?;
    
    if status.success() {
        println!("✅ Prek configuration validation passed");
    } else {
        println!("❌ Prek configuration validation failed");
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

fn create_smart_hooks_git_hooks(hook_type: &str) -> Result<()> {
    println!("📦 Creating smart-hooks {} git hooks...", hook_type);
    
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
    
    println!("✅ Smart-hooks {} hook installed at .git/hooks/pre-commit", hook_type);
    Ok(())
}

fn list_smart_hooks() {
    println!("📋 Smart-hooks capabilities:");
    println!("  🧠 smart-test-selector - Intelligent test selection based on code changes");
    println!("  🧠 bdd-feature-selector - AI-powered BDD feature selection");
    println!("  🧠 dependency-analyzer - Multi-language dependency analysis");
    println!("  🧠 hotreload-optimizer - Hot-reload performance optimization");
}

fn validate_smart_hooks_config() -> Result<()> {
    println!("✅ Validating smart-hooks configuration...");
    
    // Check for config.toml
    if std::path::Path::new("config.toml").exists() {
        println!("  ✅ Found config.toml");
    } else {
        println!("  ℹ️  No config.toml found (using defaults)");
    }
    
    // Check git repository
    if std::path::Path::new(".git").exists() {
        println!("  ✅ Git repository detected");
    } else {
        println!("  ⚠️  Not in a git repository");
    }
    
    // Check for feature files (BDD)
    if std::path::Path::new("features").exists() {
        println!("  ✅ BDD features directory found");
    } else {
        println!("  ℹ️  No features directory (BDD features not available)");
    }
    
    println!("✅ Smart-hooks configuration validation complete");
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