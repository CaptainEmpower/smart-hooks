//! File impact analysis functionality
//! 
//! This module handles analysis of file changes to determine test requirements,
//! providing detailed understanding of how code changes affect the project.
//! Follows SRP by handling only file impact analysis concerns.

use anyhow::Result;

/// Analyze file impact to determine test requirements
pub fn analyze_file_impact(files: &[String]) -> Result<FileImpactAnalysis> {
    let mut analysis = FileImpactAnalysis::new();

    for file in files {
        if is_rust_source_file(file) {
            analysis.add_source_file(file.clone());
        } else if is_test_file(file) {
            analysis.add_test_file(file.clone());
        } else if is_config_file(file) {
            analysis.add_config_file(file.clone());
        } else {
            analysis.add_other_file(file.clone());
        }
    }

    Ok(analysis)
}

/// File impact analysis results
#[derive(Debug)]
pub struct FileImpactAnalysis {
    pub source_files: Vec<String>,
    pub test_files: Vec<String>,
    pub config_files: Vec<String>,
    pub other_files: Vec<String>,
}

impl FileImpactAnalysis {
    pub fn new() -> Self {
        Self {
            source_files: Vec::new(),
            test_files: Vec::new(),
            config_files: Vec::new(),
            other_files: Vec::new(),
        }
    }

    pub fn add_source_file(&mut self, file: String) {
        self.source_files.push(file);
    }

    pub fn add_test_file(&mut self, file: String) {
        self.test_files.push(file);
    }

    pub fn add_config_file(&mut self, file: String) {
        self.config_files.push(file);
    }

    pub fn add_other_file(&mut self, file: String) {
        self.other_files.push(file);
    }

    pub fn has_source_changes(&self) -> bool {
        !self.source_files.is_empty()
    }

    pub fn has_test_changes(&self) -> bool {
        !self.test_files.is_empty()
    }

    pub fn has_config_changes(&self) -> bool {
        !self.config_files.is_empty()
    }

    pub fn total_files(&self) -> usize {
        self.source_files.len() + self.test_files.len() + self.config_files.len() + self.other_files.len()
    }

    /// Get testing recommendations based on file changes
    pub fn get_testing_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.has_source_changes() {
            recommendations.push("Run unit tests for changed source files".to_string());
        }

        if self.has_test_changes() {
            recommendations.push("Validate modified test files".to_string());
        }

        if self.has_config_changes() {
            recommendations.push("Run integration tests to verify configuration changes".to_string());
        }

        if self.source_files.len() > 5 {
            recommendations.push("Consider running full test suite due to extensive changes".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("No significant changes detected".to_string());
        }

        recommendations
    }

    /// Get risk assessment based on file changes
    pub fn get_risk_assessment(&self) -> RiskLevel {
        let total_files = self.total_files();
        let has_critical_config = self.config_files.iter()
            .any(|f| f.contains("Cargo.toml") || f.contains("lib.rs") || f.contains("main.rs"));

        match (total_files, has_critical_config, self.has_source_changes()) {
            (0, _, _) => RiskLevel::None,
            (1..=2, false, false) => RiskLevel::Low,
            (1..=5, _, true) => RiskLevel::Medium,
            (3..=5, false, false) => RiskLevel::Low,
            (_, true, _) => RiskLevel::High,
            (6.., _, _) => RiskLevel::High,
        }
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "source_files": self.source_files,
            "test_files": self.test_files,
            "config_files": self.config_files,
            "other_files": self.other_files,
            "summary": {
                "total_files": self.total_files(),
                "has_source_changes": self.has_source_changes(),
                "has_test_changes": self.has_test_changes(),
                "has_config_changes": self.has_config_changes()
            },
            "recommendations": self.get_testing_recommendations(),
            "risk_level": format!("{:?}", self.get_risk_assessment())
        })
    }
}

/// Risk assessment levels for file changes
#[derive(Debug, PartialEq)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

/// Check if a file is a Rust source file
pub fn is_rust_source_file(file: &str) -> bool {
    file.ends_with(".rs") && !file.contains("/tests/") && !file.starts_with("tests/") && !file.contains("test_")
}

/// Check if a file is a test file
pub fn is_test_file(file: &str) -> bool {
    file.contains("/tests/") || file.starts_with("tests/") || file.contains("test_") || file.contains("_test.") || file.ends_with("_test.rs")
}

/// Check if a file is a configuration file
pub fn is_config_file(file: &str) -> bool {
    let config_files = [
        "Cargo.toml", "Cargo.lock", "config.toml", ".gitignore", 
        "package.json", "tsconfig.json", "requirements.txt", "Dockerfile"
    ];
    
    config_files.iter().any(|&config| file.ends_with(config))
}

/// Check if a file is a documentation file
pub fn is_documentation_file(file: &str) -> bool {
    file.ends_with(".md") || file.ends_with(".txt") || file.ends_with(".rst") || 
    file.contains("README") || file.contains("CHANGELOG") || file.contains("LICENSE")
}

/// Get file category as string
pub fn get_file_category(file: &str) -> String {
    if is_rust_source_file(file) {
        "source".to_string()
    } else if is_test_file(file) {
        "test".to_string()
    } else if is_config_file(file) {
        "config".to_string()
    } else if is_documentation_file(file) {
        "documentation".to_string()
    } else {
        "other".to_string()
    }
}

/// Analyze files by programming language
pub fn analyze_by_language(files: &[String]) -> std::collections::HashMap<String, Vec<String>> {
    let mut language_map = std::collections::HashMap::new();
    
    for file in files {
        let language = if file.ends_with(".rs") {
            "rust"
        } else if file.ends_with(".js") || file.ends_with(".ts") {
            "javascript/typescript"
        } else if file.ends_with(".py") {
            "python"
        } else if file.ends_with(".java") {
            "java"
        } else if file.ends_with(".go") {
            "go"
        } else if file.ends_with(".c") || file.ends_with(".cpp") {
            "c/c++"
        } else {
            "other"
        };
        
        language_map.entry(language.to_string())
            .or_insert_with(Vec::new)
            .push(file.clone());
    }
    
    language_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_impact_analysis() {
        let mut analysis = FileImpactAnalysis::new();
        assert_eq!(analysis.total_files(), 0);

        analysis.add_source_file("src/main.rs".to_string());
        analysis.add_test_file("tests/integration.rs".to_string());
        analysis.add_config_file("Cargo.toml".to_string());
        analysis.add_other_file("README.md".to_string());

        assert_eq!(analysis.total_files(), 4);
        assert!(analysis.has_source_changes());
        assert!(analysis.has_test_changes());
        assert!(analysis.has_config_changes());
    }

    #[test]
    fn test_analyze_file_impact() {
        let files = vec![
            "src/main.rs".to_string(),
            "tests/integration.rs".to_string(),
            "Cargo.toml".to_string(),
            "README.md".to_string(),
        ];

        let analysis = analyze_file_impact(&files).unwrap();
        assert_eq!(analysis.source_files.len(), 1);
        assert_eq!(analysis.test_files.len(), 1);
        assert_eq!(analysis.config_files.len(), 1);
        assert_eq!(analysis.other_files.len(), 1);
    }

    #[test]
    fn test_is_rust_source_file() {
        assert!(is_rust_source_file("src/main.rs"));
        assert!(is_rust_source_file("src/lib.rs"));
        assert!(!is_rust_source_file("tests/integration.rs"));
        assert!(!is_rust_source_file("src/test_helper.rs"));
        assert!(!is_rust_source_file("README.md"));
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file("tests/integration.rs"));
        assert!(is_test_file("src/test_helper.rs"));
        assert!(is_test_file("src/main_test.rs"));
        assert!(!is_test_file("src/main.rs"));
        assert!(!is_test_file("src/lib.rs"));
    }

    #[test]
    fn test_is_config_file() {
        assert!(is_config_file("Cargo.toml"));
        assert!(is_config_file("config.toml"));
        assert!(is_config_file("package.json"));
        assert!(is_config_file("tsconfig.json"));
        assert!(!is_config_file("src/main.rs"));
        assert!(!is_config_file("README.md"));
    }

    #[test]
    fn test_file_categorization() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "tests/integration.rs".to_string(),
            "src/test_utils.rs".to_string(),
            "Cargo.toml".to_string(),
            "README.md".to_string(),
            "Dockerfile".to_string(),
        ];

        let analysis = analyze_file_impact(&files).unwrap();
        
        // Should have 2 source files: main.rs and lib.rs
        assert_eq!(analysis.source_files.len(), 2);
        // Should have 2 test files: integration.rs and test_utils.rs
        assert_eq!(analysis.test_files.len(), 2);
        // Should have 2 config files: Cargo.toml and Dockerfile
        assert_eq!(analysis.config_files.len(), 2);
        // Should have 1 other file: README.md
        assert_eq!(analysis.other_files.len(), 1);
    }

    #[test]
    fn test_get_testing_recommendations() {
        let mut analysis = FileImpactAnalysis::new();
        analysis.add_source_file("src/main.rs".to_string());
        analysis.add_config_file("Cargo.toml".to_string());
        
        let recommendations = analysis.get_testing_recommendations();
        assert!(recommendations.len() > 0);
        assert!(recommendations.iter().any(|r| r.contains("unit tests")));
        assert!(recommendations.iter().any(|r| r.contains("integration tests")));
    }

    #[test]
    fn test_risk_assessment() {
        let mut low_risk = FileImpactAnalysis::new();
        low_risk.add_other_file("README.md".to_string());
        assert_eq!(low_risk.get_risk_assessment(), RiskLevel::Low);

        let mut high_risk = FileImpactAnalysis::new();
        high_risk.add_config_file("Cargo.toml".to_string());
        assert_eq!(high_risk.get_risk_assessment(), RiskLevel::High);
    }

    #[test]
    fn test_to_json() {
        let mut analysis = FileImpactAnalysis::new();
        analysis.add_source_file("src/main.rs".to_string());
        analysis.add_test_file("tests/test.rs".to_string());
        
        let json = analysis.to_json();
        assert!(json["source_files"].is_array());
        assert!(json["summary"]["total_files"].is_number());
        assert!(json["recommendations"].is_array());
        assert!(json["risk_level"].is_string());
    }

    #[test]
    fn test_get_file_category() {
        assert_eq!(get_file_category("src/main.rs"), "source");
        assert_eq!(get_file_category("tests/test.rs"), "test");
        assert_eq!(get_file_category("Cargo.toml"), "config");
        assert_eq!(get_file_category("README.md"), "documentation");
        assert_eq!(get_file_category("image.png"), "other");
    }

    #[test]
    fn test_analyze_by_language() {
        let files = vec![
            "src/main.rs".to_string(),
            "script.py".to_string(),
            "app.js".to_string(),
            "Config.java".to_string(),
        ];
        
        let language_map = analyze_by_language(&files);
        assert!(language_map.contains_key("rust"));
        assert!(language_map.contains_key("python"));
        assert!(language_map.contains_key("javascript/typescript"));
        assert!(language_map.contains_key("java"));
    }
}