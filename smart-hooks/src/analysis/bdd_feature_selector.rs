use anyhow::Result;
use serde::{Deserialize, Serialize};
/// Claude AI-powered BDD Feature Selection
/// Automatically selects existing BDD features/scenarios that would best validate staged file changes
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct BddFeatureSelection {
    pub should_run_bdd: bool,
    pub confidence: f32,
    pub reasoning: String,
    pub selected_features: Vec<SelectedFeature>,
    pub suggested_test_focus: Vec<String>,
    pub risk_areas: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectedFeature {
    pub feature_file: String,
    pub scenarios: Vec<String>,
    pub priority: String, // "high", "medium", "low"
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct FileChange {
    pub file_path: String,
    pub change_type: String, // "modified", "added", "deleted"
    pub diff_summary: Option<String>,
}

/// Ask Claude to select which BDD features should run based on staged changes
#[cfg(feature = "claude-ai")]
pub async fn select_bdd_features_with_claude(
    staged_files: &[FileChange],
    available_features: &[String],
    project_context: &str,
) -> Result<BddFeatureSelection> {
    let staged_files_json = serde_json::to_string_pretty(staged_files)?;
    let features_list = available_features.join("\n  - ");

    let prompt = format!(
        r#"You are a testing expert analyzing code changes to select the most relevant BDD features to run.

## Project Context
{}

## Staged File Changes
```json
{}
```

## Available BDD Features
  - {}

## Task
Analyze the staged changes and select which BDD features/scenarios would best validate these changes. Consider:

1. **Direct Impact**: Which features test functionality directly modified?
2. **Integration Risk**: Which features test integration points that might be affected?
3. **Regression Risk**: Which features test areas that could break due to these changes?
4. **Business Logic**: Which features validate business rules that might be impacted?

Return JSON in this exact format:
{{
    "should_run_bdd": boolean,
    "confidence": float (0.0-1.0),
    "reasoning": "explanation of analysis and selection rationale",
    "selected_features": [
        {{
            "feature_file": "feature_filename.feature",
            "scenarios": ["specific scenario names if known", "or 'all scenarios'"],
            "priority": "high|medium|low",
            "reason": "why this feature is relevant to the changes"
        }}
    ],
    "suggested_test_focus": ["area 1", "area 2"],
    "risk_areas": ["potential risk area 1", "potential risk area 2"]
}}"#,
        project_context, staged_files_json, features_list
    );

    let output = Command::new("claude")
        .arg("-p")
        .arg("--output-format")
        .arg("json")
        .arg("--system-prompt")
        .arg("You are a testing expert. Always respond with valid JSON matching the exact format requested.")
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

    // Parse Claude CLI response
    let claude_response: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| anyhow::anyhow!("Failed to parse Claude wrapper response: {}", e))?;

    let result_text = claude_response["result"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No 'result' field in Claude response"))?;

    // Extract JSON from result
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

    let selection: BddFeatureSelection = serde_json::from_str(json_content).map_err(|e| {
        eprintln!("Failed to parse BDD selection JSON: {}", e);
        eprintln!("Extracted JSON content: {}", json_content);
        anyhow::anyhow!("Failed to parse Claude feature selection: {}", e)
    })?;

    Ok(selection)
}

/// Fallback feature selection using static analysis with configurable patterns
/// Domain-agnostic approach based on structural patterns rather than business logic
pub fn select_bdd_features_static_with_config(
    staged_files: &[FileChange],
    available_features: &[String],
    config: &crate::analysis::config::TestSelectorConfig,
) -> Result<BddFeatureSelection> {
    let mut selected_features = Vec::new();
    let mut risk_areas = Vec::new();

    // Use configurable pattern matching
    for file_change in staged_files {
        let file_path = &file_change.file_path;

        // Match features based on structural patterns from config
        for feature in available_features {
            if should_run_feature_for_file_with_config(file_path, feature, config) {
                let priority = if file_change.change_type == "modified"
                    && config.matches_structural_pattern(file_path)
                {
                    "high"
                } else if config.matches_structural_pattern(file_path) {
                    "medium"
                } else {
                    "low"
                };

                selected_features.push(SelectedFeature {
                    feature_file: feature.clone(),
                    scenarios: vec!["all scenarios".to_string()],
                    priority: priority.to_string(),
                    reason: format!(
                        "File {} matches structural pattern for feature {}",
                        file_path, feature
                    ),
                });
            }
        }

        // Identify risk areas based on structural patterns from config
        if config.matches_structural_pattern(file_path) {
            if let Some(patterns) = &config.bdd_structural_patterns {
                if patterns
                    .core_business_logic
                    .iter()
                    .any(|p| file_path.contains(p))
                {
                    risk_areas.push("Core business logic".to_string());
                }
                if patterns
                    .error_handling
                    .iter()
                    .any(|p| file_path.contains(p))
                {
                    risk_areas.push("Error handling and data types".to_string());
                }
                if patterns
                    .api_interfaces
                    .iter()
                    .any(|p| file_path.contains(p))
                {
                    risk_areas.push("API interfaces".to_string());
                }
            }
        }
    }

    risk_areas.dedup();

    Ok(BddFeatureSelection {
        should_run_bdd: !selected_features.is_empty(),
        confidence: if selected_features.is_empty() {
            0.0
        } else {
            0.7
        },
        reasoning: format!(
            "Static analysis with configurable patterns identified {} relevant features",
            selected_features.len()
        ),
        selected_features,
        suggested_test_focus: vec![
            "Regression testing".to_string(),
            "Integration validation".to_string(),
        ],
        risk_areas,
    })
}

/// Fallback feature selection using static analysis
/// Domain-agnostic approach based on structural patterns rather than business logic
pub fn select_bdd_features_static(
    staged_files: &[FileChange],
    available_features: &[String],
) -> Result<BddFeatureSelection> {
    // Use default configuration for backward compatibility
    let config = crate::analysis::config::TestSelectorConfig::default();
    select_bdd_features_static_with_config(staged_files, available_features, &config)
}

fn should_run_feature_for_file_with_config(
    file_path: &str,
    _feature: &str,
    config: &crate::analysis::config::TestSelectorConfig,
) -> bool {
    // Use configurable structural patterns instead of hard-coded logic
    config.matches_structural_pattern(file_path)
}

/// Hybrid approach: try Claude AI, fall back to static analysis
pub async fn select_bdd_features_hybrid(
    staged_files: &[FileChange],
    available_features: &[String],
    _project_context: &str,
) -> Result<BddFeatureSelection> {
    #[cfg(feature = "claude-ai")]
    {
        match select_bdd_features_with_claude(staged_files, available_features, _project_context)
            .await
        {
            Ok(selection) => return Ok(selection),
            Err(e) => {
                eprintln!(
                    "Claude AI feature selection failed, falling back to static analysis: {}",
                    e
                );
            }
        }
    }

    select_bdd_features_static(staged_files, available_features)
}

/// Discover available BDD features in the project
pub fn discover_bdd_features(project_root: &Path) -> Result<Vec<String>> {
    let mut features = Vec::new();

    // Look for common BDD feature directories
    let feature_dirs = [
        "features",
        "tests/features",
        "tests/bdd",
        "spec/features",
        "integration/features",
    ];

    for dir in &feature_dirs {
        let feature_path = project_root.join(dir);
        if feature_path.exists() {
            if let Ok(entries) = std::fs::read_dir(feature_path) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".feature") {
                            features.push(filename.to_string());
                        }
                    }
                }
            }
        }
    }

    // If no .feature files found, look for test files that might be BDD-style
    if features.is_empty() {
        let test_dirs = ["tests", "test", "spec"];
        for dir in &test_dirs {
            let test_path = project_root.join(dir);
            if let Ok(entries) = std::fs::read_dir(test_path) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.contains("bdd")
                            || filename.contains("feature")
                            || filename.contains("scenario")
                        {
                            features.push(filename.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(features)
}

/// Get staged file changes from git
pub fn get_staged_changes() -> Result<Vec<FileChange>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-status"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to get staged changes: {}", e))?;

    if !output.status.success() {
        return Ok(Vec::new()); // No staged changes
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut changes = Vec::new();

    for line in output_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let change_type = match parts[0] {
                "A" => "added",
                "M" => "modified",
                "D" => "deleted",
                "R" => "renamed",
                _ => "modified",
            };

            changes.push(FileChange {
                file_path: parts[1].to_string(),
                change_type: change_type.to_string(),
                diff_summary: None, // Could be enhanced to include actual diff
            });
        }
    }

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_bdd_features() {
        let temp_dir = tempfile::tempdir().unwrap();
        let features_dir = temp_dir.path().join("features");
        std::fs::create_dir(&features_dir).unwrap();

        // Create some test feature files
        std::fs::write(features_dir.join("user_registration.feature"), "").unwrap();
        std::fs::write(features_dir.join("payment_processing.feature"), "").unwrap();

        let features = discover_bdd_features(temp_dir.path()).unwrap();

        assert_eq!(features.len(), 2);
        assert!(features.contains(&"user_registration.feature".to_string()));
        assert!(features.contains(&"payment_processing.feature".to_string()));
    }

    #[test]
    fn test_should_run_feature_for_file() {
        // Test structural patterns - agnostic to feature names
        let config = crate::analysis::config::TestSelectorConfig::default();
        assert!(should_run_feature_for_file_with_config(
            "src/core/validator.rs",
            "any.feature",
            &config
        ));
        assert!(should_run_feature_for_file_with_config(
            "src/apply/handler.rs",
            "any.feature",
            &config
        ));
        assert!(should_run_feature_for_file_with_config(
            "src/service/processor.rs",
            "any.feature",
            &config
        ));
        assert!(should_run_feature_for_file_with_config(
            "src/api/controller.rs",
            "any.feature",
            &config
        ));
        assert!(!should_run_feature_for_file_with_config(
            "README.md",
            "any.feature",
            &config
        ));
        assert!(!should_run_feature_for_file_with_config(
            "src/utils/helpers.rs",
            "any.feature",
            &config
        ));
    }

    #[test]
    fn test_static_feature_selection() {
        let staged_files = vec![
            FileChange {
                file_path: "src/core/move_validator.rs".to_string(),
                change_type: "modified".to_string(),
                diff_summary: None,
            },
            FileChange {
                file_path: "src/apply/strategy.rs".to_string(),
                change_type: "added".to_string(),
                diff_summary: None,
            },
        ];

        let available_features = vec![
            "file_move_validation.feature".to_string(),
            "conflict_resolution.feature".to_string(),
            "user_authentication.feature".to_string(),
        ];

        let selection = select_bdd_features_static(&staged_files, &available_features).unwrap();

        assert!(selection.should_run_bdd);
        assert!(selection.confidence > 0.0);
        assert!(!selection.selected_features.is_empty());
        assert!(selection
            .risk_areas
            .contains(&"Core business logic".to_string()));
    }

    #[test]
    fn static_selection_picks_features_for_a_core_business_logic_change() {
        let staged_files = vec![FileChange {
            file_path: "src/core/business_processor.rs".to_string(),
            change_type: "modified".to_string(),
            diff_summary: Some("Added core business logic".to_string()),
        }];

        let available_features = vec!["business_logic.feature".to_string()];

        let selection = select_bdd_features_static(&staged_files, &available_features).unwrap();

        assert!(selection.should_run_bdd);
        assert_eq!(
            selection
                .selected_features
                .iter()
                .map(|f| f.feature_file.as_str())
                .collect::<Vec<_>>(),
            vec!["business_logic.feature"]
        );
    }
}
