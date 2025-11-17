use anyhow::{Context, Result};
use clap::Parser;
use smart_hooks::utils::run_cargo_command;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "smart-test-selector",
    about = "Intelligently select tests based on functionality changes in staged files"
)]
struct Args {
    /// List of staged files to analyze
    files: Vec<String>,
}

#[derive(Debug, Default)]
struct TestPlan {
    unit_tests: HashSet<String>,
    integration_tests: bool,
    bdd_tests: bool,
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

    let test_plan = analyze_changes(&args.files)?;
    execute_test_plan(&test_plan)?;

    Ok(())
}

fn analyze_changes(files: &[String]) -> Result<TestPlan> {
    let mut plan = TestPlan::default();

    for file in files {
        let path = Path::new(file);

        // Skip non-Rust files
        if !file.ends_with(".rs") {
            continue;
        }

        println!("🔍 Analyzing {}", file);

        // Check if it's a core functionality file
        if is_core_functionality_file(path) {
            analyze_file_content(path, &mut plan)?;
        }

        // Map file paths to test requirements
        match file.as_str() {
            f if f.contains("core/move_validator.rs") => {
                plan.unit_tests.insert("core::move_validator".to_string());
                plan.integration_tests = true;
            }
            f if f.contains("core/history_processor.rs") => {
                plan.unit_tests
                    .insert("core::history_processor".to_string());
            }
            f if f.contains("core/file_mover_service.rs") => {
                plan.unit_tests
                    .insert("core::file_mover_service".to_string());
                plan.bdd_tests = true; // Service changes affect user scenarios
            }
            f if f.contains("apply/") => {
                plan.bdd_tests = true; // Apply logic changes affect behavior
            }
            f if f.contains("strategy/") => {
                plan.bdd_tests = true; // Strategy changes affect behavior
            }
            f if f.contains("types.rs") || f.contains("error.rs") => {
                plan.integration_tests = true; // Core type changes need integration tests
            }
            _ => {}
        }
    }

    Ok(plan)
}

fn is_core_functionality_file(path: &Path) -> bool {
    path.to_string_lossy().contains("crates/git-mvh/src/")
        && !path.to_string_lossy().contains("/tests/")
}

fn analyze_file_content(path: &Path, plan: &mut TestPlan) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Look for key functionality patterns
    if content.contains("pub fn execute_move")
        || content.contains("pub fn move_file")
        || content.contains("MoveResult")
    {
        println!("  📁 Detected move operation changes");
        plan.bdd_tests = true;
    }

    if content.contains("ConflictStrategy") || content.contains("resolve_conflict") {
        println!("  ⚔️  Detected conflict resolution changes");
        plan.bdd_tests = true;
    }

    if content.contains("pub struct")
        && (content.contains("MoveOptions")
            || content.contains("GitRepository")
            || content.contains("MoveResult"))
    {
        println!("  🏗️  Detected core type changes");
        plan.integration_tests = true;
    }

    if content.contains("impl") && content.contains("GitMvhError") {
        println!("  🚨 Detected error handling changes");
        plan.integration_tests = true;
    }

    Ok(())
}

fn execute_test_plan(plan: &TestPlan) -> Result<()> {
    let mut tests_run = 0;

    // Run unit tests
    if !plan.unit_tests.is_empty() {
        println!("🧪 Running targeted unit tests...");
        for test in &plan.unit_tests {
            println!("  Running: {}", test);
            run_cargo_command(
                &["test", test, "--lib", "--quiet"],
                Some(Path::new("crates/git-mvh")),
            )
            .with_context(|| format!("Unit test failed: {}", test))?;
            tests_run += 1;
        }
    }

    // Run integration tests
    if plan.integration_tests {
        println!("🔗 Running integration tests for core changes...");
        run_cargo_command(
            &["test", "--test", "integration_test", "--quiet"],
            Some(Path::new("crates/git-mvh")),
        )
        .context("Integration tests failed")?;
        tests_run += 1;
    }

    // Run BDD tests
    if plan.bdd_tests {
        println!("🎭 Running BDD tests for behavioral changes...");
        run_cargo_command(
            &["test", "--test", "cucumber_tests"],
            Some(Path::new("crates/git-mvh")),
        )
        .context("BDD tests failed")?;
        tests_run += 1;
    }

    if tests_run == 0 {
        println!("✅ No functionality changes detected, skipping behavioral tests");
    } else {
        println!(
            "✅ Completed {} test suite(s) based on change analysis",
            tests_run
        );
    }

    Ok(())
}