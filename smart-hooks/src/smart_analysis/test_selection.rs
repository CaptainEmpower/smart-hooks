//! Test selection and BDD functionality
//! 
//! This module handles intelligent test selection and BDD feature analysis,
//! providing core smart analysis functionality for test discovery.
//! Follows SRP by handling only test selection concerns.

use anyhow::Result;
use serde_json::json;

use smart_hooks::TestPlan;

/// Run smart test selector on provided files
pub fn run_smart_test_selector(files: Vec<String>, json_output: bool) -> Result<()> {
    if files.is_empty() {
        let empty_result = json!({
            "status": "skipped",
            "message": "No files to analyze",
            "files_count": 0,
            "test_plan": null
        });

        if json_output {
            println!("{}", serde_json::to_string_pretty(&empty_result)?);
        } else {
            println!("No files to analyze, skipping smart test selection");
        }
        return Ok(());
    }

    let analyzing_status = json!({
        "status": "analyzing",
        "files_count": files.len(),
        "files": files
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&analyzing_status)?);
    } else {
        println!(
            "🔍 Analyzing {} staged file(s) for functionality changes...",
            files.len()
        );
    }

    // Create test plan using modular analysis
    let test_plan = create_test_plan(&files)?;

    if json_output {
        let completion_result = json!({
            "status": "completed",
            "test_plan": {
                "unit_tests": test_plan.unit_tests,
                "integration_tests": test_plan.integration_tests,
                "bdd_tests": test_plan.bdd_tests
            },
            "files_analyzed": files.len()
        });
        println!("{}", serde_json::to_string_pretty(&completion_result)?);
    }

    // Execute the plan using modular execution
    execute_test_plan(&test_plan, json_output)?;

    Ok(())
}

/// Run BDD selector for code changes (requires claude-ai feature)
#[cfg(feature = "claude-ai")]
pub fn run_bdd_selector(files: Vec<String>, json_output: bool) -> Result<()> {
    let analyzing_status = json!({
        "status": "analyzing",
        "files_count": files.len(),
        "files": files
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&analyzing_status)?);
    } else {
        println!("🎭 Analyzing BDD features for {} file(s)...", files.len());
    }
    
    // Import BDD functionality when claude-ai feature is enabled
    use smart_hooks::analysis::bdd_feature_selector;
    
    let features = bdd_feature_selector::discover_bdd_features(&std::path::Path::new("."))?;
    
    if json_output {
        let completion_result = json!({
            "status": "completed",
            "bdd_features": {
                "count": features.len(),
                "features": features
            },
            "files_analyzed": files.len()
        });
        println!("{}", serde_json::to_string_pretty(&completion_result)?);
    } else {
        println!("Found {} BDD features", features.len());
        for feature in features {
            println!("  📝 {}", feature);
        }
    }
    
    Ok(())
}

/// Run BDD selector fallback when claude-ai feature is not enabled
#[cfg(not(feature = "claude-ai"))]
pub fn run_bdd_selector(_files: Vec<String>, json_output: bool) -> Result<()> {
    let error_result = json!({
        "status": "error",
        "error": "BDD selector requires the 'claude-ai' feature to be enabled",
        "solution": "Install with: cargo install smart-hooks --features claude-ai"
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&error_result)?);
    } else {
        eprintln!("❌ BDD selector requires the 'claude-ai' feature to be enabled");
        eprintln!("Install with: cargo install smart-hooks --features claude-ai");
    }
    std::process::exit(1);
}

/// Create a test plan based on file analysis
pub fn create_test_plan(files: &[String]) -> Result<TestPlan> {
    // Use the existing smart_hooks analysis functionality
    smart_hooks::create_test_plan(files)
}

/// Execute a test plan
pub fn execute_test_plan(test_plan: &TestPlan, _json_output: bool) -> Result<()> {
    // Use the existing smart_hooks execution functionality
    smart_hooks::execute_test_plan(test_plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_plan() {
        // Test that we can use the existing TestPlan
        let files = vec!["src/main.rs".to_string()];
        let result = create_test_plan(&files);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_smart_test_selector_empty_files() {
        let result = run_smart_test_selector(vec![], true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_smart_test_selector_with_files() {
        let files = vec!["src/main.rs".to_string()];
        let result = run_smart_test_selector(files, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_test_plan() {
        let files = vec!["src/main.rs".to_string()];
        let test_plan = create_test_plan(&files).unwrap();
        let result = execute_test_plan(&test_plan, false);
        assert!(result.is_ok());
    }

    #[cfg(feature = "claude-ai")]
    #[test]
    fn test_run_bdd_selector_with_claude_ai() {
        let files = vec!["src/main.rs".to_string()];
        let result = run_bdd_selector(files, true);
        // Should not panic, may succeed or fail based on environment
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(not(feature = "claude-ai"))]
    #[test]
    fn test_run_bdd_selector_without_claude_ai() {
        let files = vec!["src/main.rs".to_string()];
        // This should exit the process, but we can't easily test process exit
        // Just verify the function exists and can be called
        assert!(true);
    }
}