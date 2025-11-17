//! Python dependency analyzer for multi-language projects
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::super::LanguageDependencyAnalyzer;
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::multi_lang_types::{Language, LanguageConfig};

/// Python dependency analyzer
pub struct PythonDependencyAnalyzer {
    _language_config: LanguageConfig,
}

impl PythonDependencyAnalyzer {
    pub fn new(language_config: LanguageConfig) -> Self {
        Self {
            _language_config: language_config,
        }
    }
}

impl LanguageDependencyAnalyzer for PythonDependencyAnalyzer {
    fn analyze_file_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut dependencies = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("import ") || line.starts_with("from ") {
                dependencies.push(Dependency {
                    name: Self::extract_import_name(line),
                    dependency_type: crate::dependency::types::DependencyType::ModuleUse,
                    path: file_path.to_path_buf(),
                    weight: 0.85,
                });
            }
        }

        Ok(dependencies)
    }

    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>> {
        let mut test_targets = Vec::new();

        for file in changed_files {
            if file.extension().and_then(|s| s.to_str()) == Some("py") {
                let test_name = format!("test_{}", file.file_stem().unwrap().to_str().unwrap());

                test_targets.push(TestTarget {
                    name: test_name.clone(),
                    test_type: crate::dependency::types::TestType::Unit { module: test_name },
                    command: vec!["python".to_string(), "-m".to_string(), "pytest".to_string()],
                    dependencies: vec![file.clone()],
                    confidence: 0.8,
                });
            }
        }

        Ok(test_targets)
    }

    fn language(&self) -> Language {
        Language::Python
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "py" | "pyx" | "pyi")
        } else {
            false
        }
    }
}

impl PythonDependencyAnalyzer {
    fn extract_import_name(line: &str) -> String {
        if line.starts_with("import ") {
            line.strip_prefix("import ")
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else if line.starts_with("from ") {
            line.strip_prefix("from ")
                .unwrap_or("")
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
            language: Language::Python,
            source_dirs: vec![PathBuf::from("src")],
            file_patterns: vec!["*.py".to_string()],
            package_manager: PackageManager::Pip,
            test_framework: TestFramework::Pytest,
            commands: LanguageCommands {
                test_command: vec!["python".to_string(), "-m".to_string(), "pytest".to_string()],
                build_command: None,
                lint_command: Some(vec!["flake8".to_string()]),
                format_command: Some(vec!["black".to_string()]),
                install_command: Some(vec!["pip".to_string(), "install".to_string()]),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_python_file_handling() {
        let config = create_test_config();
        let analyzer = PythonDependencyAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("src/main.py")));
        assert!(analyzer.can_handle_file(&PathBuf::from("module.pyx")));
        assert!(analyzer.can_handle_file(&PathBuf::from("types.pyi")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.rs")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("script.js")));
    }

    #[test]
    fn test_import_name_extraction() {
        assert_eq!(
            PythonDependencyAnalyzer::extract_import_name("import os"),
            "os"
        );
        assert_eq!(
            PythonDependencyAnalyzer::extract_import_name("from pathlib import Path"),
            "pathlib"
        );
        assert_eq!(
            PythonDependencyAnalyzer::extract_import_name("import numpy as np"),
            "numpy"
        );
    }

    #[test]
    fn test_dependency_analysis() {
        let config = create_test_config();
        let analyzer = PythonDependencyAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".py").unwrap();
        writeln!(temp_file, "import os").unwrap();
        writeln!(temp_file, "from pathlib import Path").unwrap();
        writeln!(temp_file, "import numpy as np").unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name == "os"));
        assert!(deps.iter().any(|d| d.name == "pathlib"));
        assert!(deps.iter().any(|d| d.name == "numpy"));
    }

    #[test]
    fn test_affected_tests_discovery() {
        let config = create_test_config();
        let analyzer = PythonDependencyAnalyzer::new(config);

        let changed_files = vec![PathBuf::from("src/main.py"), PathBuf::from("src/utils.py")];

        let test_targets = analyzer.find_affected_tests(&changed_files).unwrap();
        assert_eq!(test_targets.len(), 2);
        assert!(test_targets.iter().any(|t| t.name.contains("test_main")));
        assert!(test_targets.iter().any(|t| t.name.contains("test_utils")));
    }
}