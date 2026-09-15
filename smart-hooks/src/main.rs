//! smart-hooks — select and run the Rust tests affected by a set of changed files.
//!
//! Intended to be run *under* a hook runner such as prek, not as a front-end to
//! one. See `docs/adr/0001-consume-prek-rather-than-wrap-it.md`.

use anyhow::Result;

mod cli;
mod smart_analysis;

use cli::{parse_cli, validate_cli_args, Commands};
use smart_analysis::run_smart_test_selector;

fn main() -> Result<()> {
    let cli = parse_cli();
    validate_cli_args(&cli)?;

    match cli.command {
        Commands::Test { files, dry_run } => run_smart_test_selector(files, cli.json, dry_run),
    }
}
