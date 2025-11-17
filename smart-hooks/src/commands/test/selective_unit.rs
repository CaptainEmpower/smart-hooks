/// Selective unit testing implementation using generic project discovery
use anyhow::{Context, Result};
use smart_hooks::dependency::{DependencyAnalyzer, RustDependencyAnalyzer};
use smart_hooks::project::RustProjectConfig;
use std::path::PathBuf;
use std::process::Command;

pub async fn run(files: Vec<String>, verbose: bool) -> Result<()> {
    // Discover the project structure
    let project_config = RustProjectConfig::discover(".").context(
        "Failed to discover project structure. Make sure you're in a Rust project directory.",
    )?;

    if verbose {
        println!(
            "📊 Discovered project: {} ({})",
            project_config.metadata.name, project_config.metadata.version
        );
        println!("📦 Found {} crate(s)", project_config.crates.len());
    }

    // Convert file strings to paths and filter for existing files
    let file_paths: Vec<PathBuf> = files
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    if file_paths.is_empty() {
        println!("⏭️  No valid files to analyze");
        return Ok(());
    }

    if verbose {
        println!(
            "🔍 Analyzing {} file(s) for test selection",
            file_paths.len()
        );
    }

    // Use dependency analyzer to find affected tests
    let analyzer = RustDependencyAnalyzer::new(project_config);
    let test_targets = analyzer
        .find_affected_tests(&file_paths)
        .context("Failed to analyze test dependencies")?;

    if test_targets.is_empty() {
        println!("⏭️  No tests found for the specified files");
        return Ok(());
    }

    if verbose {
        println!("🎯 Found {} test target(s)", test_targets.len());
        for target in &test_targets {
            println!(
                "  - {} (confidence: {:.1}%)",
                target.name,
                target.confidence * 100.0
            );
        }
    }

    // Group tests by crate and run them
    let mut success_count = 0;
    let mut total_count = 0;

    for test_target in test_targets {
        total_count += 1;

        if verbose {
            println!("🧪 Running test: {}", test_target.name);
        }

        let mut cmd = Command::new(&test_target.command[0]);
        cmd.args(&test_target.command[1..]);

        if verbose {
            cmd.arg("--verbose");
        }

        match cmd.status() {
            Ok(status) if status.success() => {
                success_count += 1;
                if verbose {
                    println!("✅ Test passed: {}", test_target.name);
                }
            }
            Ok(_) => {
                if verbose {
                    println!("❌ Test failed: {}", test_target.name);
                }
            }
            Err(e) => {
                if verbose {
                    println!("⚠️  Could not run test {}: {}", test_target.name, e);
                }
            }
        }
    }

    println!(
        "📋 Test Summary: {}/{} tests passed",
        success_count, total_count
    );

    if success_count < total_count {
        return Err(anyhow::anyhow!("Some tests failed"));
    }

    Ok(())
}