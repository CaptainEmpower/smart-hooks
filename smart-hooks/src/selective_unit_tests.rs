use anyhow::Result;
use clap::Parser;
use smart_hooks::utils::{extract_module_name, run_cargo_command};
use std::collections::HashSet;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "selective-unit-tests",
    about = "Run unit tests selectively based on changed files"
)]
struct Args {
    /// List of changed Rust files
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.files.is_empty() {
        println!("No Rust files changed, skipping unit tests");
        return Ok(());
    }

    println!(
        "🧪 Running selective unit tests for {} changed file(s)...",
        args.files.len()
    );

    // Extract module names from changed files
    let mut modules_to_test = HashSet::new();

    for file in &args.files {
        if let Some(module_name) = extract_module_name(file) {
            modules_to_test.insert(module_name);
        }
    }

    if modules_to_test.is_empty() {
        println!("No testable modules found in changed files");
        return Ok(());
    }

    let modules: Vec<_> = modules_to_test.into_iter().collect();
    println!("Testing modules: {}", modules.join(", "));

    // Change to the main crate directory
    let crate_dir = Path::new("crates/git-mvh");
    let mut test_failed = false;

    // Run tests for each module
    for module in &modules {
        println!("Testing module: {}", module);

        // Try to run tests for this specific module
        let test_args = vec!["test", "--lib", "--quiet", module];

        match run_cargo_command(&test_args, Some(crate_dir)) {
            Ok(true) => {
                println!("✅ Tests passed for module: {}", module);
            }
            Ok(false) => {
                println!("⚠️  Direct module test failed for: {}", module);

                // Fallback: run all tests that contain this module name
                let fallback_args = vec!["test", "--lib", "--quiet", "--", module];
                match run_cargo_command(&fallback_args, Some(crate_dir)) {
                    Ok(true) => {
                        println!("✅ Fallback tests passed for module: {}", module);
                    }
                    Ok(false) => {
                        println!("❌ Failed to run tests for module: {}", module);
                        test_failed = true;
                    }
                    Err(e) => {
                        eprintln!(
                            "❌ Error running fallback tests for module {}: {}",
                            module, e
                        );
                        test_failed = true;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error running tests for module {}: {}", module, e);
                test_failed = true;
            }
        }
    }

    if test_failed {
        eprintln!("❌ Some unit tests failed");
        std::process::exit(1);
    } else {
        println!("✅ All selective unit tests passed!");
        Ok(())
    }
}