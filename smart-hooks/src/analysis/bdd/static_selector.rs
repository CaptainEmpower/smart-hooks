/// Static BDD feature selection without AI
use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::analysis::bdd::types::{BddFeatureSelection, FileChange, SelectedFeature};

/// Static feature selection based on file patterns and heuristics
pub struct StaticBddSelector;

impl StaticBddSelector {
    /// Select features based on static analysis of changed files
    pub fn select_features_for_files(
        feature_files: &[String],
        changed_files: &[FileChange],
    ) -> Result<BddFeatureSelection> {
        let mut selected_features = Vec::new();
        let mut confidence_scores = Vec::new();

        for feature_file in feature_files {
            let feature_confidence =
                Self::calculate_feature_relevance(feature_file, changed_files)?;

            if feature_confidence > 0.3 {
                let scenarios = Self::extract_relevant_scenarios(feature_file, changed_files)?;
                let priority = if feature_confidence > 0.7 {
                    "high"
                } else if feature_confidence > 0.5 {
                    "medium"
                } else {
                    "low"
                };

                selected_features.push(SelectedFeature::new(
                    feature_file.clone(),
                    scenarios,
                    priority.to_string(),
                    format!(
                        "File changes suggest relevance (confidence: {:.1}%)",
                        feature_confidence * 100.0
                    ),
                ));

                confidence_scores.push(feature_confidence);
            }
        }

        let overall_confidence = if confidence_scores.is_empty() {
            0.0
        } else {
            confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32
        };

        Ok(BddFeatureSelection {
            should_run_bdd: !selected_features.is_empty(),
            confidence: overall_confidence,
            reasoning: format!(
                "Static analysis found {} relevant features based on file changes",
                selected_features.len()
            ),
            selected_features,
            suggested_test_focus: Self::suggest_test_focus(changed_files),
            risk_areas: Self::identify_risk_areas(changed_files),
        })
    }

    fn calculate_feature_relevance(
        feature_file: &str,
        changed_files: &[FileChange],
    ) -> Result<f32> {
        let feature_content = fs::read_to_string(feature_file)?;
        let feature_lower = feature_content.to_lowercase();

        let mut relevance_score: f32 = 0.0;

        for changed_file in changed_files {
            let file_name = Path::new(&changed_file.file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Check if feature mentions the changed file/module
            // Try both underscore and space versions
            let file_name_spaced = file_name.replace("_", " ");
            if feature_lower.contains(&file_name) || feature_lower.contains(&file_name_spaced) {
                relevance_score += 0.4;
            }

            // Check for function-level matches
            for function_change in &changed_file.function_changes {
                let func_lower = function_change.to_lowercase();
                let func_spaced = func_lower.replace("_", " ");
                let func_parts: Vec<&str> = func_lower.split("_").collect();

                // Try exact match, spaced version, and individual words
                if feature_lower.contains(&func_lower)
                    || feature_lower.contains(&func_spaced)
                    || func_parts.iter().all(|part| feature_lower.contains(part))
                {
                    relevance_score += 0.3;
                }
            }

            // File type relevance
            if changed_file.file_path.contains("lib.rs")
                || changed_file.file_path.contains("main.rs")
            {
                relevance_score += 0.2;
            }
        }

        Ok(relevance_score.min(1.0))
    }

    fn extract_relevant_scenarios(
        feature_file: &str,
        _changed_files: &[FileChange],
    ) -> Result<Vec<String>> {
        let content = fs::read_to_string(feature_file)?;
        let mut scenarios = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Scenario:") || line.starts_with("Scenario Outline:") {
                let scenario_name = line
                    .strip_prefix("Scenario:")
                    .or_else(|| line.strip_prefix("Scenario Outline:"))
                    .unwrap_or(line)
                    .trim();
                scenarios.push(scenario_name.to_string());
            }
        }

        // For now, return all scenarios - could be made more intelligent
        Ok(scenarios)
    }

    fn suggest_test_focus(changed_files: &[FileChange]) -> Vec<String> {
        let mut focus_areas = Vec::new();

        for file in changed_files {
            if file.file_path.contains("error") || file.file_path.contains("validation") {
                focus_areas.push("Error handling and validation".to_string());
            }

            if file.file_path.contains("api") || file.file_path.contains("handler") {
                focus_areas.push("API endpoints and request handling".to_string());
            }

            if file.file_path.contains("auth") || file.file_path.contains("security") {
                focus_areas.push("Authentication and authorization".to_string());
            }

            if file.file_path.contains("db") || file.file_path.contains("repository") {
                focus_areas.push("Database operations and data consistency".to_string());
            }
        }

        focus_areas.sort();
        focus_areas.dedup();
        focus_areas
    }

    fn identify_risk_areas(changed_files: &[FileChange]) -> Vec<String> {
        let mut risk_areas = Vec::new();

        for file in changed_files {
            if file.line_count > 100 {
                risk_areas.push(format!("Large file change in {}", file.file_path));
            }

            if file.function_changes.len() > 5 {
                risk_areas.push(format!("Multiple function changes in {}", file.file_path));
            }

            if file.file_path.contains("lib.rs") || file.file_path.contains("main.rs") {
                risk_areas.push("Changes to core library files".to_string());
            }
        }

        risk_areas
    }
}