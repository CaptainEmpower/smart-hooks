use anyhow::Result;
use serde::{Deserialize, Serialize};
/// Claude AI-powered BDD Test Detection
/// Alternative approach using Claude CLI directly for intelligent analysis
///
/// This module demonstrates how to use Claude AI to identify behavioral testing needs
/// by analyzing code semantics rather than pattern matching.
use std::path::Path;

#[cfg(feature = "claude-ai")]
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeBddAnalysis {
    pub should_have_bdd_tests: bool,
    pub confidence: f32,
    pub reasoning: String,
    pub suggested_scenarios: Vec<String>,
    pub user_facing_features: Vec<String>,
    pub business_rules: Vec<String>,
    pub integration_points: Vec<String>,
}

/// Use Claude CLI to analyze code for BDD testing requirements
#[cfg(feature = "claude-ai")]
pub async fn analyze_with_claude(
    file_path: &Path,
    file_content: &str,
) -> Result<ClaudeBddAnalysis> {
    let prompt = format!(
        r#"Analyze this Rust code file ({}) for Behavior-Driven Development (BDD) testing needs.

Code:
```rust
{}
```

Please analyze and return a JSON response with the following structure:
{{
    "should_have_bdd_tests": boolean,
    "confidence": float (0.0-1.0),
    "reasoning": "explanation of why BDD tests are or aren't needed",
    "suggested_scenarios": ["scenario 1", "scenario 2"],
    "user_facing_features": ["feature descriptions"],
    "business_rules": ["business rule descriptions"],
    "integration_points": ["external dependencies or I/O"]
}}

Consider these factors:
1. User-facing APIs and public interfaces
2. Business logic and domain rules
3. State mutations and side effects
4. Error handling and edge cases
5. Integration with external systems
6. Complex workflows or processes

Focus on identifying code that represents BEHAVIOR rather than implementation details."#,
        file_path.display(),
        file_content
    );

    // Use Claude CLI directly with correct flags
    let output = Command::new("claude")
        .arg("-p")  // Print mode
        .arg("--output-format")
        .arg("json")
        .arg("--system-prompt")
        .arg("You are a Rust testing expert. Always respond with valid JSON matching the requested format.")
        .arg(&prompt)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute claude command: {}", e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Claude command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in Claude response: {}", e))?;

    // Claude CLI returns a wrapped JSON response with metadata
    // Extract the actual content from the "result" field
    let claude_response: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| anyhow::anyhow!("Failed to parse Claude wrapper response: {}", e))?;

    let result_text = claude_response["result"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'result' field in Claude response"))?;

    // Extract JSON from the result text (Claude sometimes wraps it in markdown)
    let json_start = result_text
        .find("```json\n")
        .map(|i| i + 8)
        .or_else(|| result_text.find("{"))
        .ok_or_else(|| anyhow::anyhow!("No JSON found in Claude result"))?;

    let json_end = if result_text.contains("```") {
        result_text.rfind("\n```").unwrap_or(result_text.len())
    } else {
        result_text
            .rfind("}")
            .map(|i| i + 1)
            .unwrap_or(result_text.len())
    };

    let json_content = &result_text[json_start..json_end];

    // Parse the actual BDD analysis JSON
    let analysis: ClaudeBddAnalysis = serde_json::from_str(json_content).map_err(|e| {
        eprintln!("Failed to parse BDD analysis JSON: {}", e);
        eprintln!("Extracted JSON content: {}", json_content);
        anyhow::anyhow!("Failed to parse Claude analysis: {}", e)
    })?;

    Ok(analysis)
}

/// Fallback static analysis when Claude AI is not available
pub fn analyze_with_fallback(file_path: &Path, _file_content: &str) -> Result<ClaudeBddAnalysis> {
    // Use our static analysis as fallback
    let patterns = crate::analysis::bdd_detector::detect_behavior_patterns(file_path)?;

    Ok(ClaudeBddAnalysis {
        should_have_bdd_tests: patterns.should_run_bdd_tests(0.5),
        confidence: patterns.confidence_score,
        reasoning: format!(
            "Static analysis detected: APIs={}, State={}, Business={}, Errors={}, Integration={}",
            patterns.has_user_facing_apis,
            patterns.has_state_mutations,
            patterns.has_business_logic,
            patterns.has_error_scenarios,
            patterns.has_integration_points
        ),
        suggested_scenarios: patterns
            .suggested_scenarios()
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        user_facing_features: if patterns.has_user_facing_apis {
            vec!["Public API detected".to_string()]
        } else {
            vec![]
        },
        business_rules: if patterns.has_business_logic {
            vec!["Business logic patterns detected".to_string()]
        } else {
            vec![]
        },
        integration_points: if patterns.has_integration_points {
            vec!["External integration detected".to_string()]
        } else {
            vec![]
        },
    })
}

/// Hybrid approach: Use Claude AI with static analysis fallback
pub async fn analyze_hybrid(file_path: &Path, file_content: &str) -> Result<ClaudeBddAnalysis> {
    #[cfg(feature = "claude-ai")]
    {
        // Try Claude AI first
        match analyze_with_claude(file_path, file_content).await {
            Ok(analysis) => return Ok(analysis),
            Err(e) => {
                eprintln!(
                    "Claude AI analysis failed, falling back to static analysis: {}",
                    e
                );
            }
        }
    }

    // Fallback to static analysis
    analyze_with_fallback(file_path, file_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_analysis() {
        let content = r#"
        pub struct MoveExecutor {
            repository: GitRepository,
        }

        impl MoveExecutor {
            pub fn execute_move(&mut self, options: &MoveOptions) -> Result<MoveResult, MoveError> {
                self.validate_move_constraints(&options)?;
                self.apply_move_strategy(&options)
            }

            fn validate_move_constraints(&self, options: &MoveOptions) -> Result<()> {
                // Business rule: ensure source file exists
                if !options.source_path.exists() {
                    return Err(MoveError::SourceNotFound);
                }
                Ok(())
            }
        }
        "#;

        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let analysis = analyze_with_fallback(file.path(), content).unwrap();

        assert!(analysis.should_have_bdd_tests);
        assert!(analysis.confidence > 0.5);
        assert!(analysis.reasoning.contains("APIs=true"));
        assert!(analysis.suggested_scenarios.len() > 0);
    }

    #[tokio::test]
    async fn test_hybrid_analysis() {
        let content = r#"
        pub fn simple_add(a: i32, b: i32) -> i32 {
            a + b
        }
        "#;

        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        // This will use fallback since Claude AI feature is not enabled
        let analysis = analyze_hybrid(file.path(), content).await.unwrap();

        assert!(!analysis.should_have_bdd_tests); // Simple function doesn't need BDD
        assert!(analysis.confidence < 0.5);
    }
}