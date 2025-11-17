//! Multi-language dependency analysis coordinator
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::analyzers::{
    PhpDependencyAnalyzer, PythonDependencyAnalyzer, RustMultiLangAnalyzer,
    TypeScriptDependencyAnalyzer,
};
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::multi_lang_types::{Language, MultiLangProjectConfig};

/// Trait for language-specific dependency analyzers
pub trait LanguageDependencyAnalyzer {
    /// Analyze dependencies for a specific file in this language
    fn analyze_file_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>>;

    /// Find tests affected by changes to given files
    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>>;

    /// Get the language this analyzer handles
    fn language(&self) -> Language;

    /// Check if this analyzer can handle the given file
    fn can_handle_file(&self, file_path: &Path) -> bool;
}

/// Multi-language dependency analyzer coordinator
pub struct MultiLangDependencyAnalyzer {
    project_config: MultiLangProjectConfig,
    language_analyzers: HashMap<Language, Box<dyn LanguageDependencyAnalyzer>>,
}

impl MultiLangDependencyAnalyzer {
    /// Create a new multi-language dependency analyzer
    pub fn new(project_config: MultiLangProjectConfig) -> Self {
        let mut analyzer = Self {
            project_config,
            language_analyzers: HashMap::new(),
        };

        analyzer.register_default_analyzers();
        analyzer
    }

    /// Register default language analyzers
    fn register_default_analyzers(&mut self) {
        for language_config in &self.project_config.languages {
            match language_config.language {
                Language::Rust => {
                    self.language_analyzers
                        .entry(Language::Rust)
                        .or_insert_with(|| {
                            let rust_analyzer = RustMultiLangAnalyzer::new(language_config.clone());
                            Box::new(rust_analyzer)
                        });
                }
                Language::TypeScript => {
                    let ts_analyzer = TypeScriptDependencyAnalyzer::new(language_config.clone());
                    self.language_analyzers
                        .insert(Language::TypeScript, Box::new(ts_analyzer));
                }
                Language::Python => {
                    let py_analyzer = PythonDependencyAnalyzer::new(language_config.clone());
                    self.language_analyzers
                        .insert(Language::Python, Box::new(py_analyzer));
                }
                Language::PHP => {
                    let php_analyzer = PhpDependencyAnalyzer::new(language_config.clone());
                    self.language_analyzers
                        .insert(Language::PHP, Box::new(php_analyzer));
                }
                _ => {
                    // Add other language analyzers as needed
                }
            }
        }
    }

    /// Analyze dependencies across all languages
    pub fn analyze_project_dependencies(&self) -> Result<HashMap<PathBuf, Vec<Dependency>>> {
        let mut all_dependencies = HashMap::new();

        for language_config in &self.project_config.languages {
            if let Some(analyzer) = self.language_analyzers.get(&language_config.language) {
                for source_dir in &language_config.source_dirs {
                    let files = self.find_source_files(source_dir, &language_config.language)?;

                    for file in files {
                        if analyzer.can_handle_file(&file) {
                            let dependencies = analyzer.analyze_file_dependencies(&file)?;
                            all_dependencies.insert(file, dependencies);
                        }
                    }
                }
            }
        }

        Ok(all_dependencies)
    }

    /// Find affected tests across all languages for given changed files
    pub fn find_cross_language_affected_tests(
        &self,
        changed_files: &[PathBuf],
    ) -> Result<Vec<TestTarget>> {
        let mut all_test_targets = Vec::new();
        let files_by_language = self.group_files_by_language(changed_files);

        for (language, files) in files_by_language {
            if let Some(analyzer) = self.language_analyzers.get(&language) {
                let test_targets = analyzer.find_affected_tests(&files)?;
                all_test_targets.extend(test_targets);
            }
        }

        Ok(all_test_targets)
    }

    /// Group files by their programming language
    fn group_files_by_language(&self, files: &[PathBuf]) -> HashMap<Language, Vec<PathBuf>> {
        let mut files_by_language: HashMap<Language, Vec<PathBuf>> = HashMap::new();

        for file in files {
            if let Some(language) = self.detect_file_language(file) {
                files_by_language
                    .entry(language)
                    .or_default()
                    .push(file.clone());
            }
        }

        files_by_language
    }

    /// Detect the programming language of a file
    fn detect_file_language(&self, file_path: &Path) -> Option<Language> {
        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            match extension {
                "rs" => Some(Language::Rust),
                "ts" | "tsx" => Some(Language::TypeScript),
                "js" | "jsx" | "mjs" => Some(Language::JavaScript),
                "py" | "pyx" | "pyi" => Some(Language::Python),
                "php" | "phtml" => Some(Language::PHP),
                "go" => Some(Language::Go),
                "java" => Some(Language::Java),
                "cs" => Some(Language::CSharp),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Find source files in a directory for a specific language
    fn find_source_files(&self, dir: &Path, language: &Language) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let extensions = language.file_extensions();

        Self::find_files_recursive(dir, &extensions, &mut files)?;
        Ok(files)
    }

    /// Recursively find files with specific extensions
    fn find_files_recursive(
        dir: &Path,
        extensions: &[&str],
        files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::find_files_recursive(&path, extensions, files)?;
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.contains(&ext) {
                    files.push(path);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::multi_lang_types::{
        BuildConfig, LanguageCommands, LanguageConfig, PackageManager, ProjectMetadata,
        TestFramework, TestStrategy,
    };
    use std::collections::HashMap;

    fn create_test_project_config() -> MultiLangProjectConfig {
        MultiLangProjectConfig {
            project_root: PathBuf::from("/tmp/test"),
            primary_language: Language::Rust,
            languages: vec![LanguageConfig {
                language: Language::Rust,
                source_dirs: vec![PathBuf::from("src")],
                file_patterns: vec!["*.rs".to_string()],
                package_manager: PackageManager::Cargo,
                test_framework: TestFramework::RustTest,
                commands: LanguageCommands {
                    test_command: vec!["cargo".to_string(), "test".to_string()],
                    build_command: Some(vec!["cargo".to_string(), "build".to_string()]),
                    lint_command: None,
                    format_command: None,
                    install_command: None,
                },
                metadata: HashMap::new(),
            }],
            metadata: ProjectMetadata {
                name: "test-project".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                repository: None,
                license: None,
                authors: vec![],
            },
            test_strategy: TestStrategy::default(),
            build_config: BuildConfig::default(),
        }
    }

    #[test]
    fn test_multi_lang_analyzer_creation() {
        let config = create_test_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);
        assert_eq!(analyzer.language_analyzers.len(), 1);
    }

    #[test]
    fn test_detect_file_language() {
        let config = create_test_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        assert_eq!(
            analyzer.detect_file_language(&PathBuf::from("main.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            analyzer.detect_file_language(&PathBuf::from("index.ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            analyzer.detect_file_language(&PathBuf::from("script.py")),
            Some(Language::Python)
        );
        assert_eq!(
            analyzer.detect_file_language(&PathBuf::from("controller.php")),
            Some(Language::PHP)
        );
        assert_eq!(
            analyzer.detect_file_language(&PathBuf::from("unknown.txt")),
            None
        );
    }

    #[test]
    fn test_group_files_by_language() {
        let config = create_test_project_config();
        let analyzer = MultiLangDependencyAnalyzer::new(config);

        let files = vec![
            PathBuf::from("main.rs"),
            PathBuf::from("script.py"),
            PathBuf::from("index.ts"),
            PathBuf::from("controller.php"),
        ];

        let grouped = analyzer.group_files_by_language(&files);
        assert_eq!(grouped.len(), 4);
        assert!(grouped.contains_key(&Language::Rust));
        assert!(grouped.contains_key(&Language::Python));
        assert!(grouped.contains_key(&Language::TypeScript));
        assert!(grouped.contains_key(&Language::PHP));
    }
}