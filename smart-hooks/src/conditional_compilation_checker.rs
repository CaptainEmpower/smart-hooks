/// Conditional Compilation Checker
/// Detects mismatched conditional compilation patterns that can cause unused import errors
use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "conditional-compilation-checker")]
#[command(about = "Check for conditional compilation mismatches in staged files")]
struct Args {
    /// Check all Rust files instead of just staged files
    #[arg(long)]
    all_files: bool,

    /// Project root directory (defaults to current directory)
    #[arg(long, short = 'd')]
    project_dir: Option<PathBuf>,

    /// Skip the actual cargo check (just detect patterns)
    #[arg(long)]
    detect_only: bool,

    /// Verbose output
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Debug)]
struct ConditionalPattern {
    file: PathBuf,
    line_number: usize,
    pattern: String,
    context: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let project_root = args
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    std::env::set_current_dir(&project_root)?;

    // Get files to check
    let files_to_check = if args.all_files {
        get_all_rust_files(&project_root)?
    } else {
        get_staged_rust_files()?
    };

    if files_to_check.is_empty() {
        if args.verbose {
            println!("⏭️  No Rust files to check");
        }
        return Ok(());
    }

    // Detect conditional compilation patterns
    let conditional_patterns = detect_conditional_patterns(&files_to_check)?;

    if conditional_patterns.is_empty() {
        if args.verbose {
            println!("⏭️  No conditional compilation patterns found");
        }
        return Ok(());
    }

    println!("🔍 Found {} conditional compilation pattern(s)", conditional_patterns.len());

    if args.verbose {
        for pattern in &conditional_patterns {
            println!("  📄 {}:{} - {}", pattern.file.display(), pattern.line_number, pattern.pattern);
            if !pattern.context.trim().is_empty() {
                println!("     Context: {}", pattern.context.trim());
            }
        }
    }

    if args.detect_only {
        println!("✅ Detection complete (--detect-only flag used)");
        return Ok(());
    }

    // Run cargo checks with different feature combinations
    println!("🔍 Running feature combination checks...");

    // Check with no default features
    println!("  📦 Checking with no default features...");
    if let Err(e) = run_cargo_check(&["--no-default-features", "--workspace"]) {
        println!("❌ Failed with no default features:");
        println!("{}", e);
        return Err(anyhow::anyhow!("Cargo check failed with no default features"));
    }

    // Check with all features
    println!("  🎯 Checking with all features...");
    if let Err(e) = run_cargo_check(&["--all-features", "--workspace"]) {
        println!("❌ Failed with all features:");
        println!("{}", e);
        return Err(anyhow::anyhow!("Cargo check failed with all features"));
    }

    // Check individual crates if we're in a workspace
    if has_workspace_members()? {
        println!("  🏗️  Checking individual workspace members...");
        let members = get_workspace_members()?;
        for member in members {
            if args.verbose {
                println!("    Checking crate: {}", member);
            }
            
            // Check member with no features
            if let Err(e) = run_cargo_check(&["--no-default-features", "-p", &member]) {
                println!("❌ Crate {} failed with no default features:", member);
                println!("{}", e);
                return Err(anyhow::anyhow!("Cargo check failed for crate {} with no default features", member));
            }

            // Check member with all features  
            if let Err(e) = run_cargo_check(&["--all-features", "-p", &member]) {
                println!("❌ Crate {} failed with all features:", member);
                println!("{}", e);
                return Err(anyhow::anyhow!("Cargo check failed for crate {} with all features", member));
            }
        }
    }

    println!("✅ All feature combination checks passed!");
    Ok(())
}

fn get_staged_rust_files() -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=AM"])
        .output()
        .context("Failed to get staged files")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("git diff command failed"));
    }

    let files: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.ends_with(".rs"))
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect();

    Ok(files)
}

fn get_all_rust_files(project_root: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("find")
        .args([
            project_root.to_str().unwrap(),
            "-name", "*.rs",
            "-not", "-path", "*/target/*",
            "-not", "-path", "*/.git/*"
        ])
        .output()
        .context("Failed to find Rust files")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("find command failed"));
    }

    let files: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(PathBuf::from)
        .collect();

    Ok(files)
}

fn detect_conditional_patterns(files: &[PathBuf]) -> Result<Vec<ConditionalPattern>> {
    let mut patterns = Vec::new();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("#[cfg(") {
                // Extract the cfg pattern
                if let Some(start) = line.find("#[cfg(") {
                    if let Some(end) = line[start..].find(")]") {
                        let pattern = &line[start..start + end + 2];
                        
                        // Get some context (next few lines for imports)
                        let context_lines: Vec<&str> = content.lines()
                            .skip(line_num + 1)
                            .take(3)
                            .collect();
                        let context = context_lines.join(" ");

                        patterns.push(ConditionalPattern {
                            file: file.clone(),
                            line_number: line_num + 1,
                            pattern: pattern.to_string(),
                            context,
                        });
                    }
                }
            }
        }
    }

    Ok(patterns)
}

fn run_cargo_check(args: &[&str]) -> Result<()> {
    let output = Command::new("cargo")
        .arg("check")
        .args(args)
        .output()
        .context("Failed to run cargo check")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for warnings and errors
    let combined_output = format!("{}\n{}", stdout, stderr);
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Cargo check failed:\n{}", combined_output));
    }

    // Check for unused import warnings specifically
    if combined_output.contains("unused import") || combined_output.contains("unused imports") {
        return Err(anyhow::anyhow!("Found unused imports:\n{}", combined_output));
    }

    // Check for other warnings that might indicate conditional compilation issues
    if combined_output.contains("warning:") {
        // Filter out acceptable warnings (could be made configurable)
        let warning_lines: Vec<&str> = combined_output
            .lines()
            .filter(|line| line.contains("warning:"))
            .collect();
        
        for warning in warning_lines {
            // These are the types of warnings we care about for conditional compilation
            if warning.contains("unused") || warning.contains("dead_code") || warning.contains("unreachable_code") {
                return Err(anyhow::anyhow!("Found compilation warnings that may indicate conditional compilation issues:\n{}", combined_output));
            }
        }
    }

    Ok(())
}

fn has_workspace_members() -> Result<bool> {
    Ok(Path::new("Cargo.toml").exists() && 
       std::fs::read_to_string("Cargo.toml")?.contains("[workspace]"))
}

fn get_workspace_members() -> Result<Vec<String>> {
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    let parsed: toml::Value = cargo_toml.parse()?;
    
    let mut members = Vec::new();
    if let Some(workspace) = parsed.get("workspace") {
        if let Some(member_array) = workspace.get("members").and_then(|v| v.as_array()) {
            for member in member_array {
                if let Some(member_str) = member.as_str() {
                    // Extract package name from member path
                    let member_path = Path::new(member_str);
                    let cargo_toml_path = member_path.join("Cargo.toml");
                    
                    if cargo_toml_path.exists() {
                        let member_cargo = std::fs::read_to_string(cargo_toml_path)?;
                        let member_parsed: toml::Value = member_cargo.parse()?;
                        
                        if let Some(package) = member_parsed.get("package") {
                            if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
                                members.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_conditional_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        fs::write(&test_file, r#"
#[cfg(feature = "test")]
use some_crate::feature_specific;

#[cfg(not(test))]
use another_crate::not_test;

fn main() {}
"#).unwrap();

        let patterns = detect_conditional_patterns(&[test_file]).unwrap();
        assert_eq!(patterns.len(), 2);
        assert!(patterns[0].pattern.contains("cfg(feature = \"test\")"));
        assert!(patterns[1].pattern.contains("cfg(not(test))"));
    }

    #[test]
    fn test_no_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        fs::write(&test_file, r#"
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}
"#).unwrap();

        let patterns = detect_conditional_patterns(&[test_file]).unwrap();
        assert_eq!(patterns.len(), 0);
    }
}