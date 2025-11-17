use anyhow::Result;
use clap::Parser;
use smart_hooks::{create_test_plan, execute_test_plan};

/// Smart test selector using modular SRP architecture
/// Focused only on CLI argument parsing and coordination

#[derive(Parser)]
#[command(
    name = "smart-test-selector",
    about = "Intelligently select tests based on functionality changes in staged files"
)]
struct Args {
    /// List of staged files to analyze
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.files.is_empty() {
        println!("No files to analyze, skipping smart test selection");
        return Ok(());
    }

    println!(
        "🔍 Analyzing {} staged file(s) for functionality changes...",
        args.files.len()
    );

    // Create test plan using modular analysis
    let test_plan = create_test_plan(&args.files)?;

    // Execute the plan using modular execution
    execute_test_plan(&test_plan)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_with_empty_args() {
        // Test that empty args doesn't panic
        let result = create_test_plan(&[]);
        assert!(result.is_ok());
    }
}