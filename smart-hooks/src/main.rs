/// Smart Hooks - Unified CLI for git-mvh pre-commit hooks
/// Consolidates all smart testing and validation functionality into a single binary
use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "smart-hooks")]
#[command(about = "Intelligent pre-commit hooks for git-mvh project")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test-related hooks
    #[command(subcommand)]
    Test(TestCommands),

    /// BDD (Behavior-Driven Development) hooks
    #[command(subcommand)]
    Bdd(BddCommands),

    /// Code analysis and validation hooks
    #[command(subcommand)]
    Check(CheckCommands),

    /// Code formatting commands
    #[command(subcommand)]
    Format(FormatCommands),

    /// Code linting commands
    #[command(subcommand)]
    Lint(LintCommands),

    /// Dependency analysis commands
    #[command(subcommand)]
    Analyze(AnalyzeCommands),

    /// Project summary and reporting
    Summary {
        /// Project directory to analyze
        #[arg(short, long, default_value = ".")]
        project_dir: std::path::PathBuf,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,

        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum TestCommands {
    /// Run selective unit tests for changed modules
    #[command(name = "selective")]
    Selective {
        /// Files to analyze (passed from pre-commit)
        files: Vec<String>,

        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Run integration tests if needed based on changes
    #[command(name = "integration")]
    Integration {
        /// Force running integration tests regardless of changes
        #[arg(long)]
        force: bool,

        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Smart test selector using advanced dependency analysis
    #[command(name = "smart-selector")]
    SmartSelector {
        /// Files to analyze (passed from pre-commit)
        files: Vec<String>,

        /// Enable BDD integration
        #[arg(long)]
        with_bdd: bool,

        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum BddCommands {
    /// Use Claude AI to intelligently select BDD features based on changes
    #[command(name = "claude-selector")]
    ClaudeSelector {
        /// Project root directory
        #[arg(long, short = 'd')]
        project_dir: Option<std::path::PathBuf>,

        /// Project context description for Claude analysis
        #[arg(long, short = 'c')]
        context: Option<String>,

        /// Only show what would be selected (dry-run)
        #[arg(long)]
        dry_run: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        output: String,

        /// Minimum confidence threshold
        #[arg(long, default_value = "0.5")]
        confidence_threshold: f32,
    },
}

#[derive(Subcommand)]
enum CheckCommands {
    /// Check for conditional compilation mismatches
    #[command(name = "conditional-compilation")]
    ConditionalCompilation {
        /// Check all Rust files instead of just staged files
        #[arg(long)]
        all_files: bool,

        /// Project root directory
        #[arg(long, short = 'd')]
        project_dir: Option<std::path::PathBuf>,

        /// Skip cargo check (just detect patterns)
        #[arg(long)]
        detect_only: bool,

        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum FormatCommands {
    /// Auto-format code using appropriate tools for detected languages
    Auto {
        /// Files to format (passed from pre-commit)
        files: Vec<String>,

        /// Check only mode (don't modify files)
        #[arg(long)]
        check: bool,

        /// Language to format (auto-detect if not specified)
        #[arg(long)]
        language: Option<String>,

        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum LintCommands {
    /// Run linting using appropriate tools for detected languages
    Auto {
        /// Files to lint (passed from pre-commit)
        files: Vec<String>,

        /// Language to lint (auto-detect if not specified)
        #[arg(long)]
        language: Option<String>,

        /// Fix issues automatically where possible
        #[arg(long)]
        fix: bool,

        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum AnalyzeCommands {
    /// Analyze project dependencies and provide impact analysis
    Dependencies {
        /// Files that have changed
        files: Vec<String>,

        /// Show detailed dependency graph
        #[arg(long)]
        graph: bool,

        /// Focus on specific language
        #[arg(long)]
        language: Option<String>,

        /// Verbose output
        #[arg(long, short)]
        verbose: bool,
    },
}

#[cfg(feature = "claude-ai")]
#[tokio::main]
async fn main() -> Result<()> {
    run_main().await
}

#[cfg(not(feature = "claude-ai"))]
fn main() -> Result<()> {
    futures::executor::block_on(run_main())
}

async fn run_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test(test_cmd) => match test_cmd {
            TestCommands::Selective { files, verbose } => {
                commands::test::selective_unit::run(files, verbose).await
            }
            TestCommands::Integration { force, verbose } => {
                commands::test::integration::run(force, verbose).await
            }
            TestCommands::SmartSelector {
                files,
                with_bdd,
                verbose,
            } => commands::test::smart_selector::run(files, with_bdd, verbose).await,
        },
        Commands::Bdd(bdd_cmd) => match bdd_cmd {
            BddCommands::ClaudeSelector {
                project_dir,
                context,
                dry_run,
                output,
                confidence_threshold,
            } => {
                commands::bdd::claude_selector::run(
                    project_dir,
                    context,
                    dry_run,
                    output,
                    confidence_threshold,
                )
                .await
            }
        },
        Commands::Check(check_cmd) => match check_cmd {
            CheckCommands::ConditionalCompilation {
                all_files,
                project_dir,
                detect_only,
                verbose,
            } => {
                commands::check::conditional_compilation::run(
                    all_files,
                    project_dir,
                    detect_only,
                    verbose,
                )
                .await
            }
        },
        Commands::Format(format_cmd) => match format_cmd {
            FormatCommands::Auto {
                files,
                check,
                language,
                verbose,
            } => commands::format::auto::run(files, check, language, verbose).await,
        },
        Commands::Lint(lint_cmd) => match lint_cmd {
            LintCommands::Auto {
                files,
                language,
                fix,
                verbose,
            } => commands::lint::auto::run(files, language, fix, verbose).await,
        },
        Commands::Analyze(analyze_cmd) => match analyze_cmd {
            AnalyzeCommands::Dependencies {
                files,
                graph,
                language,
                verbose,
            } => commands::analyze::dependencies::run(files, graph, language, verbose).await,
        },
        Commands::Summary {
            project_dir,
            format,
            verbose,
        } => commands::summary::run(project_dir, format, verbose).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[tokio::test]
    async fn test_cli_parsing() {
        // Test help parsing
        let result = Cli::try_parse_from(&["smart-hooks", "--help"]);
        assert!(result.is_err()); // Help returns an error (expected behavior for clap)

        // Test version parsing
        let result = Cli::try_parse_from(&["smart-hooks", "--version"]);
        assert!(result.is_err()); // Version returns an error (expected behavior for clap)
    }

    #[tokio::test]
    async fn test_test_subcommands() {
        // Test selective-unit command parsing
        let cli = Cli::try_parse_from(&[
            "smart-hooks",
            "test",
            "selective-unit",
            "--verbose",
            "file1.rs",
            "file2.rs",
        ])
        .unwrap();

        match cli.command {
            Commands::Test(TestCommands::Selective { files, verbose }) => {
                assert_eq!(files, vec!["file1.rs", "file2.rs"]);
                assert!(verbose);
            }
            _ => panic!("Expected selective-unit command"),
        }

        // Test integration command parsing
        let cli =
            Cli::try_parse_from(&["smart-hooks", "test", "integration", "--force", "--verbose"])
                .unwrap();

        match cli.command {
            Commands::Test(TestCommands::Integration { force, verbose }) => {
                assert!(force);
                assert!(verbose);
            }
            _ => panic!("Expected integration command"),
        }

        // Test smart-selector command parsing
        let cli = Cli::try_parse_from(&[
            "smart-hooks",
            "test",
            "smart-selector",
            "--with-bdd",
            "file.rs",
        ])
        .unwrap();

        match cli.command {
            Commands::Test(TestCommands::SmartSelector {
                files,
                with_bdd,
                verbose,
            }) => {
                assert_eq!(files, vec!["file.rs"]);
                assert!(with_bdd);
                assert!(!verbose);
            }
            _ => panic!("Expected smart-selector command"),
        }
    }

    #[tokio::test]
    async fn test_bdd_subcommands() {
        // Test claude-selector command parsing
        let cli = Cli::try_parse_from(&[
            "smart-hooks",
            "bdd",
            "claude-selector",
            "--project-dir",
            "/path/to/project",
            "--context",
            "test context",
            "--dry-run",
            "--output",
            "json",
            "--confidence-threshold",
            "0.8",
        ])
        .unwrap();

        match cli.command {
            Commands::Bdd(BddCommands::ClaudeSelector {
                project_dir,
                context,
                dry_run,
                output,
                confidence_threshold,
            }) => {
                assert_eq!(
                    project_dir,
                    Some(std::path::PathBuf::from("/path/to/project"))
                );
                assert_eq!(context, Some("test context".to_string()));
                assert!(dry_run);
                assert_eq!(output, "json");
                assert_eq!(confidence_threshold, 0.8);
            }
            _ => panic!("Expected claude-selector command"),
        }
    }

    #[tokio::test]
    async fn test_check_subcommands() {
        // Test conditional-compilation command parsing
        let cli = Cli::try_parse_from(&[
            "smart-hooks",
            "check",
            "conditional-compilation",
            "--all-files",
            "--project-dir",
            "/path/to/project",
            "--detect-only",
            "--verbose",
        ])
        .unwrap();

        match cli.command {
            Commands::Check(CheckCommands::ConditionalCompilation {
                all_files,
                project_dir,
                detect_only,
                verbose,
            }) => {
                assert!(all_files);
                assert_eq!(
                    project_dir,
                    Some(std::path::PathBuf::from("/path/to/project"))
                );
                assert!(detect_only);
                assert!(verbose);
            }
            _ => panic!("Expected conditional-compilation command"),
        }
    }

    #[tokio::test]
    async fn test_command_defaults() {
        // Test that defaults work correctly
        let cli = Cli::try_parse_from(&["smart-hooks", "bdd", "claude-selector"]).unwrap();

        match cli.command {
            Commands::Bdd(BddCommands::ClaudeSelector {
                project_dir,
                context,
                dry_run,
                output,
                confidence_threshold,
            }) => {
                assert_eq!(project_dir, None);
                assert_eq!(context, None);
                assert!(!dry_run);
                assert_eq!(output, "text"); // Default value
                assert_eq!(confidence_threshold, 0.5); // Default value
            }
            _ => panic!("Expected claude-selector command"),
        }
    }
}