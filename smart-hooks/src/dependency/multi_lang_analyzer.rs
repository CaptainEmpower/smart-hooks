//! Multi-language dependency analyzer - main module
//!
//! This module provides a unified interface for dependency analysis across multiple
//! programming languages. It coordinates language-specific analyzers to provide
//! comprehensive dependency tracking and test selection.

// Re-export the main coordinator and trait
pub use super::multi_lang_coordinator::{LanguageDependencyAnalyzer, MultiLangDependencyAnalyzer};

// Re-export language-specific analyzers for direct access if needed
pub use super::analyzers::{
    PhpDependencyAnalyzer, PythonDependencyAnalyzer, RustMultiLangAnalyzer,
    TypeScriptDependencyAnalyzer,
};

// Preserve the original extensive test suite that was in the monolithic module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::types::{DependencyType, TestType};
    use crate::project::multi_lang_types::{
        BuildConfig, Language, LanguageCommands, LanguageConfig, MultiLangProjectConfig,
        PackageManager, ProjectMetadata, TestFramework, TestStrategy,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_language_config(language: Language) -> LanguageConfig {
        LanguageConfig {
            language,
            source_dirs: vec![PathBuf::from("src")],
            file_patterns: vec!["*.rs".to_string(), "*.ts".to_string(), "*.py".to_string()],
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

    fn create_multi_lang_project_config() -> MultiLangProjectConfig {
        MultiLangProjectConfig {
            project_root: PathBuf::from("/tmp/test"),
            primary_language: Language::Rust,
            languages: vec![
                create_test_language_config(Language::Rust),
                create_test_language_config(Language::TypeScript),
                create_test_language_config(Language::Python),
                create_test_language_config(Language::PHP),
            ],
            metadata: ProjectMetadata {
                name: "test-project".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test project".to_string()),
                repository: None,
                license: None,
                authors: vec!["test".to_string()],
            },
            test_strategy: TestStrategy::default(),
            build_config: BuildConfig::default(),
        }
    }

    #[test]
    fn test_multi_lang_analyzer_creation() {
        let config = create_multi_lang_project_config();
        let _analyzer = MultiLangDependencyAnalyzer::new(config);

        // Should have created the analyzer successfully
        // Note: dependencies will be empty since no actual source files exist
    }

    #[test]
    fn test_detect_file_language() {
        let config = create_multi_lang_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        // Test file language detection
        let rust_files = vec![PathBuf::from("main.rs"), PathBuf::from("lib.rs")];
        let ts_files = vec![PathBuf::from("index.ts"), PathBuf::from("app.tsx")];
        let py_files = vec![PathBuf::from("main.py"), PathBuf::from("utils.py")];
        let php_files = vec![PathBuf::from("index.php"), PathBuf::from("config.php")];

        let all_files = [rust_files, ts_files, py_files, php_files].concat();
        let test_targets = analyzer
            .find_cross_language_affected_tests(&all_files)
            .unwrap();

        // Should generate test targets for each language
        assert!(test_targets.len() > 0);
    }

    #[test]
    fn test_find_files_recursive_nonexistent_directory() {
        let config = create_multi_lang_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        // Test with non-existent directory
        let nonexistent_files = vec![PathBuf::from("nonexistent/file.rs")];
        let result = analyzer.find_cross_language_affected_tests(&nonexistent_files);

        // Should not crash but may still create test targets for detected files
        assert!(result.is_ok());
        // Note: The analyzer will still generate test targets based on file extensions
    }

    #[test]
    fn test_find_cross_language_affected_tests() {
        let config = create_multi_lang_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        let changed_files = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/app.ts"),
            PathBuf::from("src/utils.py"),
            PathBuf::from("src/controller.php"),
        ];

        let test_targets = analyzer
            .find_cross_language_affected_tests(&changed_files)
            .unwrap();

        // Should find test targets for each language
        assert_eq!(test_targets.len(), 4);

        // Check that we have targets for each language (adjusting for actual command patterns)
        assert!(test_targets
            .iter()
            .any(|t| t.command.contains(&"cargo".to_string())));
        assert!(test_targets
            .iter()
            .any(|t| t.command.contains(&"npm".to_string())));
        assert!(test_targets
            .iter()
            .any(|t| t.command.contains(&"pytest".to_string())));
        assert!(test_targets
            .iter()
            .any(|t| t.command.iter().any(|cmd| cmd.contains("phpunit"))));
    }

    #[test]
    fn test_group_files_by_language() {
        let config = create_multi_lang_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        let mixed_files = vec![
            PathBuf::from("main.rs"),
            PathBuf::from("app.ts"),
            PathBuf::from("script.py"),
            PathBuf::from("index.php"),
            PathBuf::from("unknown.txt"), // Should be ignored
        ];

        let test_targets = analyzer
            .find_cross_language_affected_tests(&mixed_files)
            .unwrap();

        // Should have 4 targets (excluding unknown.txt)
        assert_eq!(test_targets.len(), 4);
    }

    #[test]
    fn test_confidence_scores() {
        let config = create_multi_lang_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        let test_files = vec![
            PathBuf::from("main.rs"),
            PathBuf::from("app.ts"),
            PathBuf::from("script.py"),
            PathBuf::from("controller.php"),
        ];

        let test_targets = analyzer
            .find_cross_language_affected_tests(&test_files)
            .unwrap();

        // All test targets should have confidence scores
        for target in &test_targets {
            assert!(target.confidence > 0.0);
            assert!(target.confidence <= 1.0);
        }
    }

    #[test]
    fn test_dependency_weights() {
        let config = create_test_language_config(Language::Rust);
        let analyzer = RustMultiLangAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "use std::collections::HashMap;").unwrap();
        writeln!(temp_file, "mod tests;").unwrap();

        let dependencies = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();

        // Check that dependencies have appropriate weights
        for dep in &dependencies {
            assert!(dep.weight > 0.0);
            assert!(dep.weight <= 1.0);
            assert_eq!(dep.dependency_type, DependencyType::ModuleUse);
        }
    }

    #[test]
    fn test_extract_dependency_names() {
        let config = create_test_language_config(Language::Rust);
        let analyzer = RustMultiLangAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "use std::collections::HashMap;").unwrap();
        writeln!(temp_file, "use serde::{{Serialize, Deserialize}};").unwrap();
        writeln!(temp_file, "mod utils;").unwrap();

        let dependencies = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();

        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.iter().any(|d| d.name.contains("HashMap")));
        assert!(dependencies.iter().any(|d| d.name.contains("serde")));
        assert!(dependencies.iter().any(|d| d.name.contains("utils")));
    }

    #[test]
    fn test_language_analyzers_completeness() {
        let config = create_multi_lang_project_config();
        let _analyzer = MultiLangDependencyAnalyzer::new(config.clone());

        // Verify all configured languages have analyzers
        for language_config in &config.languages {
            match language_config.language {
                Language::Rust => {
                    assert!(RustMultiLangAnalyzer::new(language_config.clone())
                        .can_handle_file(&PathBuf::from("test.rs")));
                }
                Language::TypeScript => {
                    assert!(TypeScriptDependencyAnalyzer::new(language_config.clone())
                        .can_handle_file(&PathBuf::from("test.ts")));
                }
                Language::Python => {
                    assert!(PythonDependencyAnalyzer::new(language_config.clone())
                        .can_handle_file(&PathBuf::from("test.py")));
                }
                Language::PHP => {
                    assert!(PhpDependencyAnalyzer::new(language_config.clone())
                        .can_handle_file(&PathBuf::from("test.php")));
                }
                _ => {
                    // Other languages may not be implemented yet
                }
            }
        }
    }

    #[test]
    fn test_analyze_project_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create test source files
        let rust_file = src_dir.join("main.rs");
        fs::write(&rust_file, "use std::collections::HashMap;\nfn main() {}").unwrap();

        let mut config = create_multi_lang_project_config();
        config.project_root = temp_dir.path().to_path_buf();
        config.languages[0].source_dirs = vec![src_dir];

        let analyzer = MultiLangDependencyAnalyzer::new(config);
        let dependencies = analyzer.analyze_project_dependencies().unwrap();

        // Should find dependencies in the created Rust file
        assert!(!dependencies.is_empty());
        assert!(dependencies.contains_key(&rust_file));
    }

    // Language-specific analyzer tests
    #[test]
    fn test_rust_analyzer_file_handling() {
        let config = create_test_language_config(Language::Rust);
        let analyzer = RustMultiLangAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("main.rs")));
        assert!(analyzer.can_handle_file(&PathBuf::from("lib.rs")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.py")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("test.js")));
        assert_eq!(analyzer.language(), Language::Rust);
    }

    #[test]
    fn test_rust_analyzer_dependency_analysis() {
        let config = create_test_language_config(Language::Rust);
        let analyzer = RustMultiLangAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(temp_file, "use std::collections::HashMap;").unwrap();
        writeln!(temp_file, "mod tests;").unwrap();
        writeln!(temp_file, "fn main() {{}}").unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_rust_analyzer_test_discovery() {
        let config = create_test_language_config(Language::Rust);
        let analyzer = RustMultiLangAnalyzer::new(config);

        let changed_files = vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")];

        let test_targets = analyzer.find_affected_tests(&changed_files).unwrap();
        assert_eq!(test_targets.len(), 2);

        // Check test target structure
        for target in &test_targets {
            assert!(target.name.starts_with("test_"));
            assert_eq!(target.command, vec!["cargo", "test"]);
            assert!(target.confidence > 0.0);

            match &target.test_type {
                TestType::Unit { module } => {
                    assert!(module.starts_with("test_"));
                }
                _ => panic!("Expected Unit test type"),
            }
        }
    }

    #[test]
    fn test_typescript_analyzer_file_handling() {
        let config = create_test_language_config(Language::TypeScript);
        let analyzer = TypeScriptDependencyAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("index.ts")));
        assert!(analyzer.can_handle_file(&PathBuf::from("app.tsx")));
        assert!(analyzer.can_handle_file(&PathBuf::from("script.js")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.rs")));
        assert_eq!(analyzer.language(), Language::TypeScript);
    }

    #[test]
    fn test_typescript_analyzer_dependency_analysis() {
        let config = create_test_language_config(Language::TypeScript);
        let analyzer = TypeScriptDependencyAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".ts").unwrap();
        writeln!(temp_file, "import React from 'react';").unwrap();
        writeln!(
            temp_file,
            "export default function App() {{ return null; }}"
        )
        .unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 2); // One import + one export
    }

    #[test]
    fn test_python_analyzer_file_handling() {
        let config = create_test_language_config(Language::Python);
        let analyzer = PythonDependencyAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("main.py")));
        assert!(analyzer.can_handle_file(&PathBuf::from("utils.pyx")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.rs")));
        assert_eq!(analyzer.language(), Language::Python);
    }

    #[test]
    fn test_python_analyzer_dependency_analysis() {
        let config = create_test_language_config(Language::Python);
        let analyzer = PythonDependencyAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".py").unwrap();
        writeln!(temp_file, "import os").unwrap();
        writeln!(temp_file, "from pathlib import Path").unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_php_analyzer_file_handling() {
        let config = create_test_language_config(Language::PHP);
        let analyzer = PhpDependencyAnalyzer::new(config);

        assert!(analyzer.can_handle_file(&PathBuf::from("index.php")));
        assert!(analyzer.can_handle_file(&PathBuf::from("template.phtml")));
        assert!(!analyzer.can_handle_file(&PathBuf::from("main.rs")));
        assert_eq!(analyzer.language(), Language::PHP);
    }

    #[test]
    fn test_php_analyzer_dependency_analysis() {
        let config = create_test_language_config(Language::PHP);
        let analyzer = PhpDependencyAnalyzer::new(config);

        let mut temp_file = NamedTempFile::with_suffix(".php").unwrap();
        writeln!(temp_file, "<?php").unwrap();
        writeln!(temp_file, "use App\\Controller\\BaseController;").unwrap();
        writeln!(temp_file, "require 'vendor/autoload.php';").unwrap();

        let deps = analyzer
            .analyze_file_dependencies(temp_file.path())
            .unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_find_source_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create various test files
        fs::write(src_dir.join("main.rs"), "// Rust file").unwrap();
        fs::write(src_dir.join("app.ts"), "// TypeScript file").unwrap();
        fs::write(src_dir.join("utils.py"), "# Python file").unwrap();
        fs::write(src_dir.join("readme.txt"), "Not a source file").unwrap();

        let mut config = create_multi_lang_project_config();
        config.project_root = temp_dir.path().to_path_buf();

        let analyzer = MultiLangDependencyAnalyzer::new(config);
        let dependencies = analyzer.analyze_project_dependencies();

        assert!(dependencies.is_ok());
        // The actual source discovery happens when source directories are configured correctly
    }

    #[test]
    fn test_find_files_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        // Create files at different depths
        fs::write(temp_dir.path().join("top.rs"), "// Top level").unwrap();
        fs::write(nested_dir.join("deep.rs"), "// Deep nested").unwrap();

        let config = create_multi_lang_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        // Test the recursive file finding would work with proper source directory setup
        let test_files = vec![temp_dir.path().join("top.rs"), nested_dir.join("deep.rs")];

        let test_targets = analyzer
            .find_cross_language_affected_tests(&test_files)
            .unwrap();
        assert_eq!(test_targets.len(), 2);
    }
}