/// Multi-language dependency analyzer
/// Provides unified dependency analysis across multiple programming languages
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// Multi-language dependency analyzer that coordinates language-specific analyzers
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
        // Register analyzers for detected languages
        for language_config in &self.project_config.languages {
            match language_config.language {
                Language::Rust => {
                    // Create Rust analyzer if not already registered
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

        // Get all source files from all languages
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

        // Group files by language
        let files_by_language = self.group_files_by_language(changed_files);

        // Analyze each language separately
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

/// Rust dependency analyzer adapted for multi-language context
pub struct RustMultiLangAnalyzer {
    _language_config: crate::project::multi_lang_types::LanguageConfig,
}

impl RustMultiLangAnalyzer {
    pub fn new(language_config: crate::project::multi_lang_types::LanguageConfig) -> Self {
        Self {
            _language_config: language_config,
        }
    }
}

impl LanguageDependencyAnalyzer for RustMultiLangAnalyzer {
    fn analyze_file_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>> {
        // Simplified Rust dependency analysis
        // In a full implementation, this would parse use statements, mod declarations, etc.
        let content = std::fs::read_to_string(file_path)?;
        let mut dependencies = Vec::new();

        // Look for use statements and mod declarations
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("use ") || line.starts_with("mod ") {
                // Extract dependency information
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

        // Simple heuristic: for each changed Rust file, look for corresponding test files
        for file in changed_files {
            if file.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Look for test files in the same directory or tests directory
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
        // Simple extraction - in practice would be more sophisticated
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

/// TypeScript dependency analyzer
pub struct TypeScriptDependencyAnalyzer {
    _language_config: crate::project::multi_lang_types::LanguageConfig,
}

impl TypeScriptDependencyAnalyzer {
    pub fn new(language_config: crate::project::multi_lang_types::LanguageConfig) -> Self {
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
                if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
                    // Look for corresponding test files
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

/// Python dependency analyzer
pub struct PythonDependencyAnalyzer {
    _language_config: crate::project::multi_lang_types::LanguageConfig,
}

impl PythonDependencyAnalyzer {
    pub fn new(language_config: crate::project::multi_lang_types::LanguageConfig) -> Self {
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

/// PHP dependency analyzer
pub struct PhpDependencyAnalyzer {
    _language_config: crate::project::multi_lang_types::LanguageConfig,
}

impl PhpDependencyAnalyzer {
    pub fn new(language_config: crate::project::multi_lang_types::LanguageConfig) -> Self {
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
            // Extract from require/include statements
            "file_dependency".to_string()
        } else {
            "unknown".to_string()
        }
    }
}