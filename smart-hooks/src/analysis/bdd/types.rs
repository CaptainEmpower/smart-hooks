/// Types for BDD feature selection
use serde::{Deserialize, Serialize};

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
    pub function_changes: Vec<String>,
    pub line_count: usize,
}

#[derive(Debug)]
pub struct BddTestContext {
    pub feature_files: Vec<String>,
    pub existing_scenarios: Vec<String>,
    pub changed_files: Vec<FileChange>,
    pub project_context: String,
}

impl Default for BddFeatureSelection {
    fn default() -> Self {
        Self {
            should_run_bdd: false,
            confidence: 0.0,
            reasoning: "No BDD features found or analysis failed".to_string(),
            selected_features: vec![],
            suggested_test_focus: vec![],
            risk_areas: vec![],
        }
    }
}

impl SelectedFeature {
    pub fn new(
        feature_file: String,
        scenarios: Vec<String>,
        priority: String,
        reason: String,
    ) -> Self {
        Self {
            feature_file,
            scenarios,
            priority,
            reason,
        }
    }
}