/// File change analysis for BDD feature selection
use anyhow::Result;
use std::process::Command;

use crate::analysis::bdd::types::FileChange;

/// Analyzes file changes to provide context for BDD feature selection
pub struct BddFileAnalyzer;

impl BddFileAnalyzer {
    /// Analyze staged files to create FileChange objects
    pub fn analyze_staged_files() -> Result<Vec<FileChange>> {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-status"])
            .output()?;

        let mut changes = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let change_type = match parts[0] {
                    "M" => "modified",
                    "A" => "added",
                    "D" => "deleted",
                    _ => "unknown",
                }
                .to_string();

                let file_path = parts[1].to_string();

                // Analyze file for more details if it exists
                let (function_changes, line_count) = Self::analyze_file_details(&file_path);

                changes.push(FileChange {
                    file_path,
                    change_type,
                    diff_summary: None,
                    function_changes,
                    line_count,
                });
            }
        }

        Ok(changes)
    }

    fn analyze_file_details(file_path: &str) -> (Vec<String>, usize) {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let mut functions = Vec::new();
            let line_count = content.lines().count();

            // Simple function detection for Rust files
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("pub fn ") || line.starts_with("fn ") {
                    if let Some(fn_name) = Self::extract_function_name(line) {
                        functions.push(fn_name);
                    }
                }
            }

            (functions, line_count)
        } else {
            (Vec::new(), 0)
        }
    }

    fn extract_function_name(line: &str) -> Option<String> {
        let line = line.strip_prefix("pub fn ").unwrap_or(line);
        let line = line.strip_prefix("fn ").unwrap_or(line);

        if let Some(paren_pos) = line.find('(') {
            let fn_name = line[..paren_pos].trim();
            if !fn_name.is_empty() {
                return Some(fn_name.to_string());
            }
        }

        None
    }
}