use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Common utilities for git-mvh pre-commit hooks
pub mod utils {
    use super::{Command, ImpactLevel, Path, Result};

    /// Extract module name from a Rust source file path
    pub fn extract_module_name(file_path: &str) -> Option<String> {
        // Remove the crates/git-mvh/src/ prefix and .rs suffix
        let prefix = "crates/git-mvh/src/";
        if !file_path.starts_with(prefix) {
            return None;
        }

        let relative_path = &file_path[prefix.len()..];
        if !relative_path.ends_with(".rs") {
            return None;
        }

        let module_path = &relative_path[..relative_path.len() - 3];

        // Skip main.rs and lib.rs as they don't have specific tests
        if module_path == "main" || module_path == "lib" {
            return None;
        }

        // Convert path to module name (/ to ::, remove /mod)
        let module_name = module_path.replace('/', "::").replace("::mod", "");

        if module_name.is_empty() {
            None
        } else {
            Some(module_name)
        }
    }

    /// Run cargo command and return success status
    pub fn run_cargo_command(args: &[&str], cwd: Option<&Path>) -> Result<bool> {
        let mut cmd = Command::new("cargo");
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            eprintln!("Command failed: cargo {}", args.join(" "));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(false);
        }

        Ok(true)
    }

    /// Determine impact level based on changed files using robust pattern matching
    pub fn determine_impact_level(changed_files: &[String]) -> ImpactLevel {
        use std::path::Path;

        // Critical files that always trigger high impact
        let critical_files = &["lib.rs", "main.rs", "error.rs", "types.rs"];

        // High impact: Core business logic modules
        let high_impact_modules = &[
            "core/move_validator.rs",
            "core/history_processor.rs",
            "core/file_mover_service.rs",
            "apply/strategy.rs",
            "fast_export/types.rs",
            "fast_export/parser.rs", // Parser changes are critical
        ];

        // Medium impact: Major functional areas
        let medium_impact_prefixes = &["core/", "apply/", "fast_export/", "strategy/"];

        // Low impact: Supporting modules
        let low_impact_prefixes = &["path/", "repository/", "history/"];

        // Dependency changes always require integration tests
        let dependency_files = &["Cargo.toml", "crates/git-mvh/Cargo.toml"];

        let mut max_impact = ImpactLevel::None;

        for file in changed_files {
            let file_path = Path::new(file);

            // Check for dependency changes (always high impact)
            if dependency_files.iter().any(|&dep| file.ends_with(dep)) {
                return ImpactLevel::High;
            }

            // Extract the relative path from git-mvh src directory
            let relative_file = if let Ok(stripped) = file_path.strip_prefix("crates/git-mvh/src") {
                stripped.to_string_lossy().replace('\\', "/")
            } else if file.contains("crates/git-mvh/src/") {
                file.split("crates/git-mvh/src/")
                    .nth(1)
                    .unwrap_or("")
                    .replace('\\', "/")
            } else {
                continue; // Skip non-source files
            };

            // Check critical files first
            if let Some(filename) = file_path.file_name() {
                if critical_files.iter().any(|&f| filename == f) {
                    return ImpactLevel::High;
                }
            }

            // Check high impact modules (exact matches)
            if high_impact_modules
                .iter()
                .any(|&pattern| relative_file == pattern)
            {
                return ImpactLevel::High;
            }

            // Check medium impact prefixes
            if medium_impact_prefixes
                .iter()
                .any(|&prefix| relative_file.starts_with(prefix))
            {
                max_impact = max_impact.max(ImpactLevel::Medium);
                continue;
            }

            // Check low impact prefixes
            if low_impact_prefixes
                .iter()
                .any(|&prefix| relative_file.starts_with(prefix))
            {
                max_impact = max_impact.max(ImpactLevel::Low);
                continue;
            }

            // Any other .rs file in src is at least low impact
            if relative_file.ends_with(".rs") {
                max_impact = max_impact.max(ImpactLevel::Low);
            }
        }

        max_impact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
}

impl ImpactLevel {
    /// Return the higher of two impact levels
    pub fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }
}

impl std::fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactLevel::None => write!(f, "none"),
            ImpactLevel::Low => write!(f, "low"),
            ImpactLevel::Medium => write!(f, "medium"),
            ImpactLevel::High => write!(f, "high"),
        }
    }
}