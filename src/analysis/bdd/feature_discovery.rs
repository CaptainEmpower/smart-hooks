/// BDD feature file discovery and parsing
use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::analysis::bdd::types::BddTestContext;

/// Discovers and analyzes BDD feature files in a project
pub struct BddFeatureDiscovery;

impl BddFeatureDiscovery {
    /// Discover all .feature files and extract scenarios
    pub fn discover_bdd_features<P: AsRef<Path>>(project_root: P) -> Result<Vec<String>> {
        let mut feature_files = Vec::new();
        let project_path = project_root.as_ref();

        // Common locations for BDD features
        let feature_dirs = [
            project_path.join("features"),
            project_path.join("tests/features"),
            project_path.join("test/features"),
            project_path.join("specs"),
            project_path.join("acceptance"),
        ];

        for dir in &feature_dirs {
            if dir.exists() {
                Self::find_feature_files(dir, &mut feature_files)?;
            }
        }

        Ok(feature_files)
    }

    fn find_feature_files(dir: &Path, feature_files: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::find_feature_files(&path, feature_files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("feature") {
                feature_files.push(path.to_string_lossy().to_string());
            }
        }
        Ok(())
    }

    /// Extract scenarios from feature files
    pub fn extract_scenarios(feature_files: &[String]) -> Result<Vec<String>> {
        let mut scenarios = Vec::new();

        for feature_file in feature_files {
            let content = fs::read_to_string(feature_file)?;
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
        }

        Ok(scenarios)
    }

    /// Create BDD test context from project analysis
    pub fn create_bdd_context<P: AsRef<Path>>(
        project_root: P,
        changed_files: Vec<crate::analysis::bdd::types::FileChange>,
    ) -> Result<BddTestContext> {
        let feature_files = Self::discover_bdd_features(&project_root)?;
        let existing_scenarios = Self::extract_scenarios(&feature_files)?;

        let project_context = format!(
            "Rust project with {} BDD features and {} scenarios",
            feature_files.len(),
            existing_scenarios.len()
        );

        Ok(BddTestContext {
            feature_files,
            existing_scenarios,
            changed_files,
            project_context,
        })
    }
}