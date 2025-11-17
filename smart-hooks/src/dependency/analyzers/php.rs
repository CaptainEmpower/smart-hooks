//! PHP dependency analyzer for multi-language projects
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::super::LanguageDependencyAnalyzer;
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::multi_lang_types::{Language, LanguageConfig};

/// PHP dependency analyzer
pub struct PhpDependencyAnalyzer {
    _language_config: LanguageConfig,
}

impl PhpDependencyAnalyzer {
    pub fn new(language_config: LanguageConfig) -> Self {
        Self {
            _language_config: language_config,
        }
    }
}

impl LanguageDependencyAnalyzer for PhpDependencyAnalyzer {
    fn analyze_file_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("use ")
                || line.starts_with("require ")
                || line.starts_with("include ")
            {
                dependencies.push(Dependency {
                    name: Self::extract_dependency_name(line),
                    dependency_type: crate::dependency::types::DependencyType::ModuleUse,
                    path: file_path.to_path_buf(),
                    weight: 0.75,
                });
            }
        }

        Ok(dependencies)
    }

    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>> {
        let mut test_targets = Vec::new();

        for file in changed_files {
            if file.extension().and_then(|s| s.to_str()) == Some("php") {
                let test_name = format!("{}Test", file.file_stem().unwrap().to_str().unwrap());

                test_targets.push(TestTarget {
                    name: test_name.clone(),
                    test_type: crate::dependency::types::TestType::Unit { module: test_name },
                    command: vec!["vendor/bin/phpunit".to_string(), "--filter".to_string()],
                    dependencies: vec![file.clone()],
                    confidence: 0.7,
                });
            }
        }

        Ok(test_targets)
    }

    fn language(&self) -> Language {
        Language::PHP
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "php" | "phtml")
        } else {
            false
        }
    }
}

impl PhpDependencyAnalyzer {
    fn extract_dependency_name(line: &str) -> String {
        if line.starts_with("use ") {
            line.strip_prefix("use ")
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else if line.contains("require") || line.contains("include") {
            // Extract file path from require/include statements
            if let Some(start) = line.find("'") {
                if let Some(end) = line[start + 1..].find("'") {
                    return line[start + 1..start + 1 + end].to_string();
                }
            }
            if let Some(start) = line.find("\"") {
                if let Some(end) = line[start + 1..].find("\"") {
                    return line[start + 1..start + 1 + end].to_string();
                }
            }
            "unknown".to_string()
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
            language: Language::PHP,
            source_dirs: vec![PathBuf::from("src")],
            file_patterns: vec!["*.php".to_string()],
            package_manager: PackageManager::Composer,
            test_framework: TestFramework::PHPUnit,
            commands: LanguageCommands {
                test_command: vec!["vendor/bin/phpunit".to_string()],
                build_command: None,
                lint_command: Some(vec!["vendor/bin/phpcs".to_string()]),
                format_command: Some(vec!["vendor/bin/phpcbf".to_string()]),
                install_command: Some(vec!["composer".to_string(), "install".to_string()]),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_php_file_handling() {
        let config = create_test_config();
        let analyzer = PhpDependencyAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("src/Controller.php")));
        assert!(analyzer.can_handle_file(&PathBuf::from("template.phtml")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.rs")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("script.js")));
    }

    #[test]
    fn test_dependency_name_extraction() {
        assert_eq!(
            PhpDependencyAnalyzer::extract_dependency_name("use App\\Controller\\HomeController;"),
            "App\\Controller\\HomeController;"
        );
        assert_eq!(
            PhpDependencyAnalyzer::extract_dependency_name("require 'vendor/autoload.php';"),
            "vendor/autoload.php"
        );
        assert_eq!(
            PhpDependencyAnalyzer::extract_dependency_name("include \"config.php\";"),
            "config.php"
        );
    }

    #[test]
    fn test_dependency_analysis() {
        let config = create_test_config();
        let analyzer = PhpDependencyAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".php").unwrap();
        writeln!(temp_file, "<?php").unwrap();
        writeln!(temp_file, "use App\\Controller\\BaseController;").unwrap();
        writeln!(temp_file, "require 'vendor/autoload.php';").unwrap();
        writeln!(temp_file, "include 'config.php';").unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name.contains("BaseController")));
        assert!(deps.iter().any(|d| d.name.contains("autoload")));
        assert!(deps.iter().any(|d| d.name.contains("config")));
    }

    #[test]
    fn test_affected_tests_discovery() {
        let config = create_test_config();
        let analyzer = PhpDependencyAnalyzer::new(config);

        let changed_files = vec![
            PathBuf::from("src/Controller.php"),
            PathBuf::from("src/Model.php"),
        ];

        let test_targets = analyzer.find_affected_tests(&changed_files).unwrap();
        assert_eq!(test_targets.len(), 2);
        assert!(test_targets
            .iter()
            .any(|t| t.name.contains("ControllerTest")));
        assert!(test_targets.iter().any(|t| t.name.contains("ModelTest")));
    }
}