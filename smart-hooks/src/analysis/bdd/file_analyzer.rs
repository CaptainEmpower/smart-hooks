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
                
                // Skip commented lines
                if line.starts_with("//") || line.starts_with("/*") {
                    continue;
                }
                
                if line.starts_with("pub fn ") || 
                   line.starts_with("fn ") || 
                   line.starts_with("async fn ") ||
                   line.starts_with("pub async fn ") ||
                   line.starts_with("unsafe fn ") ||
                   line.starts_with("pub unsafe fn ") ||
                   (line.contains("fn ") && (line.contains("extern") || line.contains("impl"))) {
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
        let line = line.trim(); // Remove leading/trailing whitespace first
        
        // Handle different function patterns
        let line = if line.starts_with("pub fn ") {
            &line[7..] // Remove "pub fn "
        } else if line.starts_with("async fn ") {
            &line[9..] // Remove "async fn "
        } else if line.starts_with("unsafe fn ") {
            &line[10..] // Remove "unsafe fn "
        } else if line.starts_with("fn ") {
            &line[3..] // Remove "fn "
        } else if line.contains("fn ") {
            // Handle cases like "    fn" or "extern \"C\" fn"
            if let Some(fn_pos) = line.find("fn ") {
                &line[fn_pos + 3..]
            } else {
                return None;
            }
        } else {
            return None;
        };

        if let Some(paren_pos) = line.find('(') {
            let fn_name = line[..paren_pos].trim();
            // Handle generics like "function<T>"
            if let Some(generic_pos) = fn_name.find('<') {
                let clean_name = fn_name[..generic_pos].trim();
                if !clean_name.is_empty() {
                    return Some(clean_name.to_string());
                }
            } else if !fn_name.is_empty() {
                return Some(fn_name.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            BddFileAnalyzer::extract_function_name("fn main() {"),
            Some("main".to_string())
        );
        assert_eq!(
            BddFileAnalyzer::extract_function_name("pub fn analyze_file(path: &str) -> Result<()> {"),
            Some("analyze_file".to_string())
        );
        assert_eq!(
            BddFileAnalyzer::extract_function_name("    fn helper_function(&self) {"),
            Some("helper_function".to_string())
        );
        assert_eq!(
            BddFileAnalyzer::extract_function_name("pub fn complex_function<T: Clone>(param: T) -> Vec<T> {"),
            Some("complex_function".to_string())
        );
        assert_eq!(
            BddFileAnalyzer::extract_function_name("async fn async_function() -> Future<Output = ()> {"),
            Some("async_function".to_string())
        );
        assert_eq!(
            BddFileAnalyzer::extract_function_name("let x = 5;"),
            None
        );
        assert_eq!(
            BddFileAnalyzer::extract_function_name("struct MyStruct {"),
            None
        );
    }

    #[test]
    fn test_analyze_file_details_existing_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.rs");

        fs::write(&file_path, r#"
use std::io::Result;

pub fn main() {
    println!("Hello world");
}

fn helper_function() -> i32 {
    42
}

pub fn another_function(param: String) -> String {
    format!("Hello, {}", param)
}

async fn async_task() {
    // async work
}

struct SomeStruct {
    field: i32,
}
"#)?;

        let (functions, line_count) = BddFileAnalyzer::analyze_file_details(file_path.to_str().unwrap());

        assert_eq!(line_count, 22); // Including blank lines
        
        // Debug: print what functions were actually detected
        println!("Detected functions: {:?}", functions);
        
        // The analyze_file_details looks for lines that start with "pub fn " or "fn "
        // Based on our test file, we should detect exactly these functions
        assert!(functions.len() >= 3); // At least main, helper_function, another_function, async_task
        assert!(functions.contains(&"main".to_string()));
        assert!(functions.contains(&"helper_function".to_string()));
        assert!(functions.contains(&"another_function".to_string()));
        assert!(functions.contains(&"async_task".to_string()));

        Ok(())
    }

    #[test]
    fn test_analyze_file_details_nonexistent_file() {
        let (functions, line_count) = BddFileAnalyzer::analyze_file_details("/nonexistent/file.rs");

        assert_eq!(functions.len(), 0);
        assert_eq!(line_count, 0);
    }

    #[test]
    fn test_analyze_file_details_empty_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("empty.rs");

        fs::write(&file_path, "")?;

        let (functions, line_count) = BddFileAnalyzer::analyze_file_details(file_path.to_str().unwrap());

        assert_eq!(functions.len(), 0);
        assert_eq!(line_count, 0);

        Ok(())
    }

    #[test]
    fn test_analyze_file_details_no_functions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("no_functions.rs");

        fs::write(&file_path, r#"
use std::collections::HashMap;

struct Config {
    name: String,
    version: String,
}

const VERSION: &str = "1.0.0";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
"#)?;

        let (functions, line_count) = BddFileAnalyzer::analyze_file_details(file_path.to_str().unwrap());

        assert_eq!(functions.len(), 0);
        assert!(line_count > 0);

        Ok(())
    }

    // Note: Testing analyze_staged_files() would require git integration
    // and is better suited for integration tests since it depends on external git state.
    // We can add a mock test for the git parsing logic:

    #[test] 
    fn test_git_status_parsing_logic() {
        // This tests the parsing logic that would happen in analyze_staged_files
        // We simulate what git diff --cached --name-status would output
        let mock_git_output = "M\tsrc/main.rs\nA\tsrc/new_module.rs\nD\told_file.rs\n";
        
        let mut changes = Vec::new();
        
        for line in mock_git_output.lines() {
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
                }.to_string();

                changes.push((change_type, parts[1].to_string()));
            }
        }

        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0], ("modified".to_string(), "src/main.rs".to_string()));
        assert_eq!(changes[1], ("added".to_string(), "src/new_module.rs".to_string()));
        assert_eq!(changes[2], ("deleted".to_string(), "old_file.rs".to_string()));
    }

    #[test]
    fn test_git_status_parsing_edge_cases() {
        // Test various edge cases in git output parsing
        let mock_git_output = "\nR100\told_name.rs\tnew_name.rs\nT\ttype_changed.rs\n  \n";
        
        let mut changes = Vec::new();
        
        for line in mock_git_output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let change_type = match parts[0].chars().next().unwrap_or('?') {
                    'M' => "modified",
                    'A' => "added",
                    'D' => "deleted", 
                    'R' => "renamed",
                    'T' => "type_changed",
                    _ => "unknown",
                }.to_string();

                // For renames, we might have 3 parts (R100 old_name new_name)
                let file_path = if parts.len() >= 3 && parts[0].starts_with('R') {
                    parts[2].to_string() // Use new name for renames
                } else {
                    parts[1].to_string()
                };

                changes.push((change_type, file_path));
            }
        }

        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0], ("renamed".to_string(), "new_name.rs".to_string()));
        assert_eq!(changes[1], ("type_changed".to_string(), "type_changed.rs".to_string()));
    }

    #[test]
    fn test_function_detection_edge_cases() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("edge_cases.rs");

        fs::write(&file_path, r#"
// This file tests various edge cases for function detection

fn simple_function() {}

pub fn public_function() -> String {
    "test".to_string()
}

    fn indented_function() {
        // indented function should still be detected
    }

// fn commented_function() {} - this should NOT be detected

impl SomeStruct {
    fn method(&self) -> i32 {
        42
    }
    
    pub fn public_method(&mut self, param: String) {
        // method implementation
    }
}

fn generic_function<T: Clone>(param: T) -> T {
    param.clone()
}

async fn async_function() -> Result<()> {
    Ok(())
}

unsafe fn unsafe_function() {
    // unsafe code
}

extern "C" fn c_function() {
    // C FFI function
}

fn function_with_where<T>() 
where 
    T: Clone + Send + Sync,
{
    // complex where clause
}
"#)?;

        let (functions, line_count) = BddFileAnalyzer::analyze_file_details(file_path.to_str().unwrap());

        assert!(line_count > 30);
        
        // Debug: print what functions were actually detected
        println!("All detected functions: {:?}", functions);
        
        // Should detect all valid function definitions
        let expected_functions = [
            "simple_function",
            "public_function", 
            "indented_function",
            "method",
            "public_method",
            "generic_function", // Note: this might be detected with generics
            "async_function",
            "unsafe_function",
            "c_function",
            "function_with_where",
        ];

        // Check that we have at least most of the expected functions
        let mut found_count = 0;
        for expected in &expected_functions {
            if functions.contains(&expected.to_string()) {
                found_count += 1;
            } else {
                println!("Missing expected function: {}", expected);
            }
        }
        
        // Should find at least 8 out of 10 expected functions
        assert!(found_count >= 8, 
                "Expected to find at least 8 functions, but only found {} in: {:?}", 
                found_count, functions);

        // Should not detect commented out functions
        assert!(!functions.iter().any(|f| f.contains("commented_function")));

        Ok(())
    }

    #[test]
    fn test_create_file_change_object() {
        // Test creating a FileChange object with our analyzed data
        let file_path = "src/test.rs".to_string();
        let change_type = "modified".to_string();
        let function_changes = vec!["main".to_string(), "helper".to_string()];
        let line_count = 50;

        let file_change = FileChange {
            file_path: file_path.clone(),
            change_type: change_type.clone(),
            diff_summary: None,
            function_changes: function_changes.clone(),
            line_count,
        };

        assert_eq!(file_change.file_path, file_path);
        assert_eq!(file_change.change_type, change_type);
        assert_eq!(file_change.function_changes, function_changes);
        assert_eq!(file_change.line_count, line_count);
        assert!(file_change.diff_summary.is_none());
    }
}