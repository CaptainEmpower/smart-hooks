//! CLI definition.
//!
//! smart-hooks does one thing: given a list of changed Rust files, work out
//! which tests can observe the change and run them. See
//! `docs/adr/0002-narrow-scope-to-rust-test-impact-selection.md`.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "smart-hooks",
    about = "Select and run the Rust tests affected by a set of changed files",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Emit machine-readable JSON instead of human-readable output
    #[arg(long, global = true)]
    pub json: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Select and run the tests affected by the given files
    Test {
        /// Changed files to analyse; pre-commit passes these as arguments
        files: Vec<String>,

        /// Print the selected test plan without running anything
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

/// Reject flag combinations that contradict each other.
pub fn validate_cli_args(cli: &Cli) -> Result<()> {
    if cli.quiet && cli.verbose {
        return Err(anyhow::anyhow!("Cannot specify both --quiet and --verbose"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("expected these arguments to parse")
    }

    #[test]
    fn test_subcommand_collects_files() {
        let cli = parse(&["smart-hooks", "test", "src/main.rs", "src/lib.rs"]);

        let Commands::Test { files, dry_run } = cli.command;
        assert_eq!(files, vec!["src/main.rs", "src/lib.rs"]);
        assert!(!dry_run);
    }

    #[test]
    fn global_flags_are_accepted_before_the_subcommand() {
        let cli = parse(&["smart-hooks", "--json", "--verbose", "test"]);

        assert!(cli.json);
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn dry_run_is_opt_in() {
        let cli = parse(&["smart-hooks", "test", "--dry-run", "src/lib.rs"]);

        let Commands::Test { dry_run, .. } = cli.command;
        assert!(dry_run);
    }

    #[test]
    fn quiet_and_verbose_together_are_rejected() {
        let cli = parse(&["smart-hooks", "--quiet", "--verbose", "test"]);

        let err = validate_cli_args(&cli).expect_err("quiet + verbose must be rejected");
        assert_eq!(err.to_string(), "Cannot specify both --quiet and --verbose");
    }

    #[test]
    fn quiet_alone_is_accepted() {
        let cli = parse(&["smart-hooks", "--quiet", "test"]);

        validate_cli_args(&cli).expect("quiet alone is a valid combination");
    }

    #[test]
    fn removed_subcommands_are_rejected_rather_than_parsed_as_files() {
        for removed in [
            "lint", "format", "summary", "analyze", "check", "bdd", "run",
        ] {
            let err = Cli::try_parse_from(["smart-hooks", removed])
                .expect_err("removed subcommand must not parse");
            assert_eq!(
                err.kind(),
                clap::error::ErrorKind::InvalidSubcommand,
                "`{removed}` should be rejected as an unknown subcommand"
            );
        }
    }
}
