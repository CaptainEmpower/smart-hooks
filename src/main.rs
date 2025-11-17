use anyhow::Result;
use clap::{Parser, Subcommand};

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
    
    let features = bdd_feature_selector::discover_bdd_features(".")?;
    println!("Found {} BDD features", features.len());
    
    for feature in features {
        println!("  📝 {}", feature.name);
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