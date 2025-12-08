//! CLI configuration and command definitions
//! 
//! This module defines the command-line interface structure, argument parsing,
//! and command definitions for smart-hooks. Follows SRP by handling only
//! CLI configuration concerns.

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
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Output results in JSON format for programmatic usage
    #[arg(long, global = true, help = "Output results in JSON format for agents and automation")]
    pub json: bool,
    
    /// Verbose output
    #[arg(short, long, global = true, help = "Enable verbose logging")]
    pub verbose: bool,
    
    /// Quiet mode (suppress non-essential output)
    #[arg(short, long, global = true, help = "Suppress non-essential output")]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
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

/// Parse command line arguments and return CLI configuration
pub fn parse_cli() -> Cli {
    Cli::parse()
}

/// Check if the command supports JSON output
pub fn supports_json_output(_command: &Commands) -> bool {
    // All commands support JSON output in our implementation
    true
}

/// Get command name as string for logging/debugging
pub fn command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Run { .. } => "run",
        Commands::Install { .. } => "install", 
        Commands::List => "list",
        Commands::Validate => "validate",
        Commands::Test { .. } => "test",
        #[cfg(feature = "claude-ai")]
        Commands::Bdd { .. } => "bdd",
        Commands::Analyze { .. } => "analyze",
        Commands::Check { .. } => "check",
        Commands::Capabilities => "capabilities",
        Commands::Schema { .. } => "schema",
        Commands::Status => "status",
        Commands::Examples { .. } => "examples",
    }
}

/// Validate CLI arguments for logical consistency
pub fn validate_cli_args(cli: &Cli) -> Result<()> {
    // Ensure quiet and verbose aren't both set
    if cli.quiet && cli.verbose {
        return Err(anyhow::anyhow!("Cannot specify both --quiet and --verbose"));
    }
    
    // Validate command-specific arguments
    match &cli.command {
        Commands::Schema { command, .. } => {
            if let Some(cmd) = command {
                let valid_commands = ["test", "analyze", "run", "install", "all"];
                if !valid_commands.contains(&cmd.as_str()) {
                    return Err(anyhow::anyhow!("Invalid schema command: {}. Valid options: {}", cmd, valid_commands.join(", ")));
                }
            }
        },
        Commands::Examples { command, scenario } => {
            if command.is_some() && scenario.is_some() {
                return Err(anyhow::anyhow!("Cannot specify both --command and --scenario for examples"));
            }
        },
        _ => {}
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing() {
        let args = vec!["smart-hooks", "test", "src/main.rs"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());
        
        if let Ok(cli) = cli {
            assert!(!cli.json);
            assert!(!cli.verbose);
            assert!(!cli.quiet);
            assert!(matches!(cli.command, Commands::Test { .. }));
        }
    }

    #[test]
    fn test_global_flags() {
        let args = vec!["smart-hooks", "--json", "--verbose", "capabilities"];
        let cli = Cli::try_parse_from(args).unwrap();
        
        assert!(cli.json);
        assert!(cli.verbose);
        assert!(!cli.quiet);
        assert!(matches!(cli.command, Commands::Capabilities));
    }

    #[test]
    fn test_command_name() {
        let test_cmd = Commands::Test { files: vec![] };
        assert_eq!(command_name(&test_cmd), "test");
        
        let capabilities_cmd = Commands::Capabilities;
        assert_eq!(command_name(&capabilities_cmd), "capabilities");
    }

    #[test]
    fn test_supports_json_output() {
        let test_cmd = Commands::Test { files: vec![] };
        assert!(supports_json_output(&test_cmd));
        
        let capabilities_cmd = Commands::Capabilities;
        assert!(supports_json_output(&capabilities_cmd));
    }

    #[test]
    fn test_validate_cli_args_conflicting_flags() {
        let cli = Cli {
            command: Commands::Test { files: vec![] },
            json: false,
            verbose: true,
            quiet: true,
        };
        
        assert!(validate_cli_args(&cli).is_err());
    }

    #[test]
    fn test_validate_cli_args_valid() {
        let cli = Cli {
            command: Commands::Test { files: vec![] },
            json: true,
            verbose: true,
            quiet: false,
        };
        
        assert!(validate_cli_args(&cli).is_ok());
    }

    #[test]
    fn test_validate_schema_command() {
        let cli = Cli {
            command: Commands::Schema { 
                command: Some("invalid".to_string()), 
                format: "json".to_string() 
            },
            json: false,
            verbose: false,
            quiet: false,
        };
        
        assert!(validate_cli_args(&cli).is_err());
    }

    #[test]
    fn test_validate_examples_command_conflict() {
        let cli = Cli {
            command: Commands::Examples { 
                command: Some("test".to_string()), 
                scenario: Some("automation".to_string()) 
            },
            json: false,
            verbose: false,
            quiet: false,
        };
        
        assert!(validate_cli_args(&cli).is_err());
    }
}