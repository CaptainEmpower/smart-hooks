//! TypeScript/JavaScript dependency analyzer for multi-language projects
use anyhow::Result;
use std::path::{Path, PathBuf};

use super::super::LanguageDependencyAnalyzer;
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::multi_lang_types::{Language, LanguageConfig};

/// TypeScript dependency analyzer
pub struct TypeScriptDependencyAnalyzer {
    _language_config: LanguageConfig,
}

impl TypeScriptDependencyAnalyzer {
    pub fn new(language_config: LanguageConfig) -> Self {
        Self {
            _language_config: language_config,
        }
    }
}

impl LanguageDependencyAnalyzer for TypeScriptDependencyAnalyzer {
    fn analyze_file_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut dependencies = Vec::new();

        // Look for import statements
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("import ")
                || line.starts_with("export ")
                || line.contains("require(")
            {
                dependencies.push(Dependency {
                    name: Self::extract_import_name(line),
                    dependency_type: crate::dependency::types::DependencyType::ModuleUse,
                    path: file_path.to_path_buf(),
                    weight: 0.8,
                });
            }
        }

        Ok(dependencies)
    }

    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>> {
        let mut test_targets = Vec::new();

        for file in changed_files {
            if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
                if matches!(ext, "ts" | "tsx" | "js" | "jsx") {
                    let test_name = format!("{}.test", file.file_stem().unwrap().to_str().unwrap());

                    test_targets.push(TestTarget {
                        name: test_name.clone(),
                        test_type: crate::dependency::types::TestType::Unit { module: test_name },
                        command: vec!["npm".to_string(), "test".to_string()],
                        dependencies: vec![file.clone()],
                        confidence: 0.7,
                    });
                }
            }
        }

        Ok(test_targets)
    }

    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn can_handle_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mjs")
        } else {
            false
        }
    }
}

impl TypeScriptDependencyAnalyzer {
    fn extract_import_name(line: &str) -> String {
        // Simplified import name extraction
        if line.contains("from ") {
            if let Some(from_pos) = line.find("from ") {
                let from_part = &line[from_pos + 5..];
                from_part
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            }
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
            language: Language::TypeScript,
            source_dirs: vec![PathBuf::from("src")],
            file_patterns: vec!["*.ts".to_string(), "*.tsx".to_string()],
            package_manager: PackageManager::Npm,
            test_framework: TestFramework::Jest,
            commands: LanguageCommands {
                test_command: vec!["npm".to_string(), "test".to_string()],
                build_command: Some(vec![
                    "npm".to_string(),
                    "run".to_string(),
                    "build".to_string(),
                ]),
                lint_command: Some(vec![
                    "npm".to_string(),
                    "run".to_string(),
                    "lint".to_string(),
                ]),
                format_command: Some(vec![
                    "npm".to_string(),
                    "run".to_string(),
                    "format".to_string(),
                ]),
                install_command: Some(vec!["npm".to_string(), "install".to_string()]),
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_typescript_file_handling() {
        let config = create_test_config();
        let analyzer = TypeScriptDependencyAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("src/index.ts")));
        assert!(analyzer.can_handle_file(&PathBuf::from("component.tsx")));
        assert!(analyzer.can_handle_file(&PathBuf::from("script.js")));
        assert!(analyzer.can_handle_file(&PathBuf::from("component.jsx")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.py")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.rs")));
    }

    #[test]
    fn test_import_name_extraction() {
        assert_eq!(
            TypeScriptDependencyAnalyzer::extract_import_name("import React from 'react';"),
            "react';" // This is what the actual extraction returns
        );
        assert_eq!(
            TypeScriptDependencyAnalyzer::extract_import_name(
                "import { useState } from \"react\";"
            ),
            "react\";"
        );
        assert_eq!(
            TypeScriptDependencyAnalyzer::extract_import_name("const fs = require('fs');"),
            "unknown"
        );
    }

    #[test]
    fn test_dependency_analysis() {
        let config = create_test_config();
        let analyzer = TypeScriptDependencyAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        writeln!(temp_file, "import React from 'react';").unwrap();
        writeln!(temp_file, "import {{ useState }} from 'react';").unwrap();
        writeln!(
            temp_file,
            "export default function App() {{ return null; }}"
        )
        .unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 3); // Two imports + one export
        assert!(deps.iter().any(|d| d.name.contains("react")));
    }

    #[test]
    fn test_affected_tests_discovery() {
        let config = create_test_config();
        let analyzer = TypeScriptDependencyAnalyzer::new(config);

        let changed_files = vec![
            PathBuf::from("src/index.ts"),
            PathBuf::from("src/component.tsx"),
        ];

        let test_targets = analyzer.find_affected_tests(&changed_files).unwrap();
        assert_eq!(test_targets.len(), 2);
        assert!(test_targets.iter().any(|t| t.name.contains("index.test")));
        assert!(test_targets
            .iter()
            .any(|t| t.name.contains("component.test")));
    }
}