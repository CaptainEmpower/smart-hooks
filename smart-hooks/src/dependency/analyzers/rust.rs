//! Rust dependency analyzer for multi-language projects
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::super::LanguageDependencyAnalyzer;
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::multi_lang_types::{Language, LanguageConfig};

/// Rust dependency analyzer adapted for multi-language context
pub struct RustMultiLangAnalyzer {
    _language_config: LanguageConfig,
}

impl RustMultiLangAnalyzer {
    pub fn new(language_config: LanguageConfig) -> Self {
        Self {
            _language_config: language_config,
        }
    }
}

impl LanguageDependencyAnalyzer for RustMultiLangAnalyzer {
    fn analyze_file_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut dependencies = Vec::new();

        // Look for use statements and mod declarations
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("use ") || line.starts_with("mod ") {
                dependencies.push(Dependency {
                    name: Self::extract_dependency_name(line),
                    dependency_type: crate::dependency::types::DependencyType::ModuleUse,
                    path: file_path.to_path_buf(),
                    weight: 0.9,
                });
            }
        }

        Ok(dependencies)
    }

    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>> {
        let mut test_targets = Vec::new();

        for file in changed_files {
            if file.extension().and_then(|s| s.to_str()) == Some("rs") {
                let test_name = format!("test_{}", file.file_stem().unwrap().to_str().unwrap());

                test_targets.push(TestTarget {
                    name: test_name.clone(),
                    test_type: crate::dependency::types::TestType::Unit { module: test_name },
                    command: vec!["cargo".to_string(), "test".to_string()],
                    dependencies: vec![file.clone()],
                    confidence: 0.8,
                });
            }
        }

        Ok(test_targets)
    }

    fn language(&self) -> Language {
        Language::Rust
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        file_path.extension().and_then(|s| s.to_str()) == Some("rs")
    }
}

impl RustMultiLangAnalyzer {
    fn extract_dependency_name(line: &str) -> String {
        if let Some(use_part) = line.strip_prefix("use ") {
            use_part
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else if let Some(mod_part) = line.strip_prefix("mod ") {
            mod_part
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::multi_lang_types::{LanguageCommands, PackageManager, TestFramework};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config() -> LanguageConfig {
        LanguageConfig {
            language: Language::Rust,
            source_dirs: vec![PathBuf::from("src")],
            file_patterns: vec!["*.rs".to_string()],
            package_manager: PackageManager::Cargo,
            test_framework: TestFramework::RustTest,
            commands: LanguageCommands {
                test_command: vec!["cargo".to_string(), "test".to_string()],
                build_command: Some(vec!["cargo".to_string(), "build".to_string()]),
                lint_command: Some(vec!["cargo".to_string(), "clippy".to_string()]),
                format_command: Some(vec!["cargo".to_string(), "fmt".to_string()]),
                install_command: Some(vec!["cargo".to_string(), "install".to_string()]),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_rust_file_handling() {
        let config = create_test_config();
        let analyzer = RustMultiLangAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("src/main.rs")));
        assert!(analyzer.can_handle_file(&PathBuf::from("lib.rs")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.py")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("test.js")));
    }

    #[test]
    fn test_dependency_name_extraction() {
        assert_eq!(
            RustMultiLangAnalyzer::extract_dependency_name("use std::collections::HashMap;"),
            "std::collections::HashMap;"
        );
        assert_eq!(
            RustMultiLangAnalyzer::extract_dependency_name("mod tests;"),
            "tests;"
        );
        assert_eq!(
            RustMultiLangAnalyzer::extract_dependency_name("invalid line"),
            "unknown"
        );
    }

    #[test]
    fn test_dependency_analysis() {
        let config = create_test_config();
        let analyzer = RustMultiLangAnalyzer::new(config);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "use std::collections::HashMap;").unwrap();
        writeln!(temp_file, "mod tests;").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name.contains("HashMap")));
        assert!(deps.iter().any(|d| d.name.contains("tests")));
    }

    #[test]
    fn test_affected_tests_discovery() {
        let config = create_test_config();
        let analyzer = RustMultiLangAnalyzer::new(config);

        let changed_files = vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")];

        let test_targets = analyzer.find_affected_tests(&changed_files).unwrap();
        assert_eq!(test_targets.len(), 2);
        assert!(test_targets.iter().any(|t| t.name.contains("test_main")));
        assert!(test_targets.iter().any(|t| t.name.contains("test_lib")));
    }
}