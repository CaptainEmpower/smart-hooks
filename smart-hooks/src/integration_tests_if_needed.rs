use anyhow::Result;
use clap::Parser;
use smart_hooks::{
    utils::{determine_impact_level, run_cargo_command},
    ImpactLevel,
};
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "integration-tests-if-needed",
    about = "Run integration tests when core modules change"
)]
struct Args {
    /// List of changed Rust files in core areas
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.files.is_empty() {
        println!("No core files changed, skipping integration tests");
        return Ok(());
    }

    println!(
        "🔍 Analyzing {} changed core file(s) for integration test impact...",
        args.files.len()
    );

    let impact_level = determine_impact_level(&args.files);
    println!("Impact level determined: {}", impact_level);

    // Change to project root
    let project_root = Path::new(".");

    match impact_level {
        ImpactLevel::High => {
            println!("🚨 High impact changes detected - running comprehensive test suite");
            println!("Changed files: {}", args.files.join(", "));

            // Run comprehensive test suite for high impact changes
            let integration_tests = [
                "integration_test",
                "comprehensive_test",
                "critical_path_integration_test",
                "error_handling_integration_test",
                "fast_export_pipeline_integration_test",
                "strategy_integration_test",
                "streaming_integration_test",
                "complex_patch_application_integration_test",
                "multi_file_batch_operations_integration_test",
                "real_repo_test",
            ];

            let mut any_failed = false;

            // Run all integration tests individually
            println!("Running {} integration tests...", integration_tests.len());
            for test in &integration_tests {
                let test_args = vec!["test", "--test", test, "--quiet"];
                match run_cargo_command(&test_args, Some(project_root)) {
                    Ok(true) => println!("✅ {}", test),
                    Ok(false) => {
                        eprintln!("❌ {} failed", test);
                        any_failed = true;
                    }
                    Err(e) => {
                        println!("⚠️  {} not found or error: {}", test, e);
                    }
                }
            }

            // Run property tests
            let test_args = vec!["test", "--test", "property_tests", "--quiet"];
            match run_cargo_command(&test_args, Some(project_root)) {
                Ok(true) => println!("✅ property tests"),
                Ok(false) => {
                    eprintln!("❌ property tests failed");
                    any_failed = true;
                }
                Err(e) => {
                    println!("⚠️  property tests not found or error: {}", e);
                }
            }

            // Run BDD tests (without --quiet)
            let test_args = vec!["test", "--test", "cucumber_tests"];
            match run_cargo_command(&test_args, Some(project_root)) {
                Ok(true) => println!("✅ BDD tests"),
                Ok(false) => {
                    eprintln!("❌ BDD tests failed");
                    any_failed = true;
                }
                Err(e) => {
                    println!("⚠️  BDD tests not found or error: {}", e);
                }
            }

            if any_failed {
                eprintln!("❌ Some comprehensive tests failed");
                std::process::exit(1);
            }

            println!("✅ All comprehensive tests passed!");
        }

        ImpactLevel::Medium => {
            println!("⚠️  Medium impact changes detected - running core integration tests");
            println!("Changed files: {}", args.files.join(", "));

            // Run critical integration tests
            let critical_tests = [
                "integration_test",
                "critical_path_integration_test",
                "comprehensive_test",
                "strategy_integration_test",
            ];

            let mut any_failed = false;
            for test in &critical_tests {
                let test_args = vec!["test", "--test", test, "--quiet"];
                match run_cargo_command(&test_args, Some(project_root)) {
                    Ok(true) => println!("✅ Passed: {}", test),
                    Ok(false) => {
                        println!("⚠️  Test failed: {} (continuing...)", test);
                        any_failed = true;
                    }
                    Err(e) => {
                        println!(
                            "⚠️  Test not found or error: {} - {} (continuing...)",
                            test, e
                        );
                    }
                }
            }

            if any_failed {
                eprintln!("❌ Some critical integration tests failed");
                std::process::exit(1);
            }
        }

        ImpactLevel::Low => {
            println!("ℹ️  Low impact changes detected - running minimal integration tests");
            println!("Changed files: {}", args.files.join(", "));

            // Run just the basic integration test
            let test_args = vec!["test", "--test", "integration_test", "--quiet"];
            match run_cargo_command(&test_args, Some(project_root)) {
                Ok(true) => println!("✅ Basic integration tests passed"),
                Ok(false) => {
                    eprintln!("❌ Basic integration test failed");
                    std::process::exit(1);
                }
                Err(e) => {
                    println!("⚠️  Basic integration test not found: {}", e);
                }
            }
        }

        ImpactLevel::None => {
            println!("ℹ️  No significant impact detected - skipping integration tests");
            println!("Changed files: {}", args.files.join(", "));
        }
    }

    println!("✅ Integration test check completed!");
    Ok(())
}