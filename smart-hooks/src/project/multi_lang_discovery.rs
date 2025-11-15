/// Multi-language project discovery
/// Detects and configures support for multiple programming languages in a project
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::multi_lang_types::*;

/// Multi-language project discovery service
pub struct MultiLangProjectDiscovery;

impl MultiLangProjectDiscovery {
    /// Discover project configuration for all supported languages
    pub fn discover<P: AsRef<Path>>(project_root: P) -> Result<MultiLangProjectConfig> {
        let root = project_root.as_ref().to_path_buf();

        // Detect all languages in the project
        let languages = Self::detect_languages(&root)?;

        if languages.is_empty() {
            return Err(anyhow::anyhow!(
                "No supported programming languages detected in project"
            ));
        }

        // Determine primary language (most files or explicit configuration)
        let primary_language = Self::determine_primary_language(&languages, &root)?;

        // Create language-specific configurations
        let mut language_configs = Vec::new();
        for lang in &languages {
            if let Ok(config) = Self::create_language_config(lang.clone(), &root) {
                language_configs.push(config);
            }
        }

        // Extract project metadata
        let metadata = Self::extract_project_metadata(&root, &primary_language)?;

        // Determine build configuration
        let build_config = Self::detect_build_config(&root, &primary_language)?;

        Ok(MultiLangProjectConfig {
            project_root: root,
            primary_language,
            languages: language_configs,
            metadata,
            test_strategy: TestStrategy::default(),
            build_config,
        })
    }

    /// Detect all programming languages present in the project
    fn detect_languages(project_root: &Path) -> Result<Vec<Language>> {
        let mut detected_languages = Vec::new();

        // Check for language-specific configuration files and patterns
        let language_indicators = [
            (
                Language::Rust,
                vec!["Cargo.toml", "src/lib.rs", "src/main.rs"],
            ),
            (Language::TypeScript, vec!["tsconfig.json", "package.json"]),
            (
                Language::JavaScript,
                vec!["package.json", "package-lock.json"],
            ),
            (
                Language::Python,
                vec!["setup.py", "pyproject.toml", "requirements.txt", "Pipfile"],
            ),
            (Language::PHP, vec!["composer.json", "composer.lock"]),
            (Language::Go, vec!["go.mod", "go.sum"]),
            (
                Language::Java,
                vec!["pom.xml", "build.gradle", "gradle.properties"],
            ),
            (Language::CSharp, vec!["*.csproj", "*.sln", "project.json"]),
        ];

        for (language, indicators) in &language_indicators {
            let mut found = false;

            for indicator in indicators {
                if indicator.contains('*') {
                    // Glob pattern matching
                    if Self::glob_exists(project_root, indicator)? {
                        found = true;
                        break;
                    }
                } else {
                    // Direct file check
                    if project_root.join(indicator).exists() {
                        found = true;
                        break;
                    }
                }
            }

            if found {
                detected_languages.push(language.clone());
            }
        }

        // Also detect by file extensions in source directories
        Self::detect_by_file_extensions(project_root, &mut detected_languages)?;

        detected_languages.sort_by_key(|lang| format!("{:?}", lang));
        detected_languages.dedup();

        Ok(detected_languages)
    }

    /// Detect languages by scanning file extensions in source directories
    fn detect_by_file_extensions(project_root: &Path, languages: &mut Vec<Language>) -> Result<()> {
        let common_source_dirs = ["src", "lib", "app", "public", "pkg", "cmd", "internal"];

        for dir_name in &common_source_dirs {
            let dir_path = project_root.join(dir_name);
            if dir_path.exists() && dir_path.is_dir() {
                Self::scan_directory_for_languages(&dir_path, languages)?;
            }
        }

        Ok(())
    }

    /// Recursively scan directory for language files
    fn scan_directory_for_languages(dir: &Path, languages: &mut Vec<Language>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories (with depth limit)
                Self::scan_directory_for_languages(&path, languages)?;
            } else if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                // Check file extension against known languages
                if let Some(lang) = Self::language_from_extension(extension) {
                    if !languages.contains(&lang) {
                        languages.push(lang);
                    }
                }
            }
        }

        Ok(())
    }

    /// Map file extension to programming language
    fn language_from_extension(extension: &str) -> Option<Language> {
        match extension {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" => Some(Language::JavaScript),
            "py" | "pyx" | "pyi" => Some(Language::Python),
            "php" | "phtml" | "php3" | "php4" | "php5" => Some(Language::PHP),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "cs" => Some(Language::CSharp),
            _ => None,
        }
    }

    /// Simple glob pattern matching
    fn glob_exists(project_root: &Path, pattern: &str) -> Result<bool> {
        if !pattern.contains('*') {
            return Ok(project_root.join(pattern).exists());
        }

        // Simple implementation for *.ext patterns
        if let Some(extension) = pattern.strip_prefix("*.") {
            for entry in fs::read_dir(project_root)? {
                let entry = entry?;
                if let Some(file_ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if file_ext == extension {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Determine the primary programming language
    fn determine_primary_language(languages: &[Language], project_root: &Path) -> Result<Language> {
        if languages.is_empty() {
            return Err(anyhow::anyhow!("No languages detected"));
        }

        // If only one language, it's primary
        if languages.len() == 1 {
            return Ok(languages[0].clone());
        }

        // Priority order for determining primary language
        let priority = [
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::JavaScript,
            Language::Go,
            Language::Java,
            Language::PHP,
            Language::CSharp,
        ];

        // Check priority order
        for preferred in &priority {
            if languages.contains(preferred) {
                return Ok(preferred.clone());
            }
        }

        // Fallback: count files for each language
        let mut file_counts: HashMap<Language, usize> = HashMap::new();
        for language in languages {
            let count = Self::count_language_files(project_root, language)?;
            file_counts.insert(language.clone(), count);
        }

        // Return language with most files
        file_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .ok_or_else(|| anyhow::anyhow!("Could not determine primary language"))
    }

    /// Count files for a specific language
    fn count_language_files(project_root: &Path, language: &Language) -> Result<usize> {
        let extensions = language.file_extensions();
        let mut count = 0;

        Self::count_files_recursive(project_root, &extensions, &mut count)?;
        Ok(count)
    }

    /// Recursively count files with specific extensions
    fn count_files_recursive(dir: &Path, extensions: &[&str], count: &mut usize) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::count_files_recursive(&path, extensions, count)?;
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.contains(&ext) {
                    *count += 1;
                }
            }
        }

        Ok(())
    }

    /// Create language-specific configuration
    fn create_language_config(language: Language, project_root: &Path) -> Result<LanguageConfig> {
        let source_dirs = Self::find_source_directories(project_root, &language)?;
        let file_patterns = language
            .file_extensions()
            .iter()
            .map(|ext| format!("**/*.{}", ext))
            .collect();

        let package_manager = Self::detect_package_manager(project_root, &language)?;
        let test_framework = Self::detect_test_framework(project_root, &language)?;
        let commands = Self::create_language_commands(&language, &package_manager, &test_framework);

        Ok(LanguageConfig {
            language,
            source_dirs,
            file_patterns,
            package_manager,
            test_framework,
            commands,
            metadata: HashMap::new(),
        })
    }

    /// Find source directories for a language
    fn find_source_directories(project_root: &Path, language: &Language) -> Result<Vec<PathBuf>> {
        let mut source_dirs = Vec::new();

        for dir_name in language.common_source_dirs() {
            let dir_path = project_root.join(dir_name);
            if dir_path.exists() && dir_path.is_dir() {
                source_dirs.push(dir_path);
            }
        }

        // If no standard directories found, check for language files in project root
        if source_dirs.is_empty() && Self::has_language_files(project_root, language)? {
            source_dirs.push(project_root.to_path_buf());
        }

        Ok(source_dirs)
    }

    /// Check if directory contains files for the given language
    fn has_language_files(dir: &Path, language: &Language) -> Result<bool> {
        let extensions = language.file_extensions();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if extensions.contains(&ext) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Detect package manager for a language
    fn detect_package_manager(project_root: &Path, language: &Language) -> Result<PackageManager> {
        match language {
            Language::Rust => Ok(PackageManager::Cargo),
            Language::TypeScript | Language::JavaScript => {
                if project_root.join("pnpm-lock.yaml").exists() {
                    Ok(PackageManager::Pnpm)
                } else if project_root.join("yarn.lock").exists() {
                    Ok(PackageManager::Yarn)
                } else {
                    Ok(PackageManager::Npm)
                }
            }
            Language::Python => {
                if project_root.join("pyproject.toml").exists() {
                    Ok(PackageManager::Poetry)
                } else if project_root.join("Pipfile").exists() {
                    Ok(PackageManager::Pipenv)
                } else {
                    Ok(PackageManager::Pip)
                }
            }
            Language::PHP => Ok(PackageManager::Composer),
            Language::Go => Ok(PackageManager::GoMod),
            Language::Java => {
                if project_root.join("build.gradle").exists()
                    || project_root.join("gradle.properties").exists()
                {
                    Ok(PackageManager::Gradle)
                } else {
                    Ok(PackageManager::Maven)
                }
            }
            Language::CSharp => Ok(PackageManager::NuGet),
            Language::Unknown(_) => Ok(PackageManager::Unknown("unknown".to_string())),
        }
    }

    /// Detect test framework for a language
    fn detect_test_framework(_project_root: &Path, language: &Language) -> Result<TestFramework> {
        // For now, use defaults. In a real implementation, this would
        // check package.json, cargo.toml, etc. for specific test dependencies
        let framework = match language {
            Language::Rust => TestFramework::RustTest,
            Language::TypeScript | Language::JavaScript => TestFramework::Jest, // Default, could detect others
            Language::Python => TestFramework::Pytest, // Default, could detect unittest
            Language::PHP => TestFramework::PHPUnit,
            Language::Go => TestFramework::GoTest,
            Language::Java => TestFramework::JUnit,
            Language::CSharp => TestFramework::MSTest,
            Language::Unknown(_) => TestFramework::Unknown("unknown".to_string()),
        };

        Ok(framework)
    }

    /// Create language-specific commands
    fn create_language_commands(
        language: &Language,
        package_manager: &PackageManager,
        test_framework: &TestFramework,
    ) -> LanguageCommands {
        match language {
            Language::Rust => LanguageCommands {
                test_command: vec!["cargo".to_string(), "test".to_string()],
                build_command: Some(vec!["cargo".to_string(), "build".to_string()]),
                lint_command: Some(vec!["cargo".to_string(), "clippy".to_string()]),
                format_command: Some(vec!["cargo".to_string(), "fmt".to_string()]),
                install_command: Some(vec!["cargo".to_string(), "build".to_string()]),
            },
            Language::TypeScript | Language::JavaScript => {
                let pm_cmd = match package_manager {
                    PackageManager::Pnpm => "pnpm",
                    PackageManager::Yarn => "yarn",
                    _ => "npm",
                };

                LanguageCommands {
                    test_command: match test_framework {
                        TestFramework::Jest => vec![pm_cmd.to_string(), "test".to_string()],
                        TestFramework::Vitest => {
                            vec![pm_cmd.to_string(), "run".to_string(), "test".to_string()]
                        }
                        _ => vec![pm_cmd.to_string(), "test".to_string()],
                    },
                    build_command: Some(vec![
                        pm_cmd.to_string(),
                        "run".to_string(),
                        "build".to_string(),
                    ]),
                    lint_command: Some(vec![
                        pm_cmd.to_string(),
                        "run".to_string(),
                        "lint".to_string(),
                    ]),
                    format_command: Some(vec![
                        pm_cmd.to_string(),
                        "run".to_string(),
                        "format".to_string(),
                    ]),
                    install_command: Some(vec![pm_cmd.to_string(), "install".to_string()]),
                }
            }
            Language::Python => LanguageCommands {
                test_command: vec!["python".to_string(), "-m".to_string(), "pytest".to_string()],
                build_command: Some(vec![
                    "python".to_string(),
                    "setup.py".to_string(),
                    "build".to_string(),
                ]),
                lint_command: Some(vec![
                    "python".to_string(),
                    "-m".to_string(),
                    "flake8".to_string(),
                ]),
                format_command: Some(vec![
                    "python".to_string(),
                    "-m".to_string(),
                    "black".to_string(),
                    ".".to_string(),
                ]),
                install_command: Some(vec![
                    "pip".to_string(),
                    "install".to_string(),
                    "-r".to_string(),
                    "requirements.txt".to_string(),
                ]),
            },
            Language::PHP => LanguageCommands {
                test_command: vec!["vendor/bin/phpunit".to_string()],
                build_command: None,
                lint_command: Some(vec!["vendor/bin/phpcs".to_string()]),
                format_command: Some(vec![
                    "vendor/bin/php-cs-fixer".to_string(),
                    "fix".to_string(),
                ]),
                install_command: Some(vec!["composer".to_string(), "install".to_string()]),
            },
            _ => LanguageCommands {
                test_command: vec!["echo".to_string(), "No test command configured".to_string()],
                build_command: None,
                lint_command: None,
                format_command: None,
                install_command: None,
            },
        }
    }

    /// Extract project metadata from various sources
    fn extract_project_metadata(
        project_root: &Path,
        primary_language: &Language,
    ) -> Result<ProjectMetadata> {
        // Try to extract from language-specific files
        match primary_language {
            Language::Rust => Self::extract_rust_metadata(project_root),
            Language::TypeScript | Language::JavaScript => {
                Self::extract_node_metadata(project_root)
            }
            Language::Python => Self::extract_python_metadata(project_root),
            Language::PHP => Self::extract_php_metadata(project_root),
            _ => {
                // Generic fallback
                Ok(ProjectMetadata {
                    name: project_root
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    version: "0.0.0".to_string(),
                    description: None,
                    repository: None,
                    license: None,
                    authors: Vec::new(),
                })
            }
        }
    }

    /// Extract metadata from Cargo.toml
    fn extract_rust_metadata(project_root: &Path) -> Result<ProjectMetadata> {
        let cargo_toml = project_root.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(anyhow::anyhow!("Cargo.toml not found"));
        }

        let content = fs::read_to_string(&cargo_toml)?;
        let toml: toml::Value = content.parse()?;

        let package = toml
            .get("package")
            .or_else(|| toml.get("workspace"))
            .context("No package or workspace section in Cargo.toml")?;

        Ok(ProjectMetadata {
            name: package
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: package
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            description: package
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: package
                .get("repository")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            license: package
                .get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            authors: package
                .get("authors")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    /// Extract metadata from package.json
    fn extract_node_metadata(project_root: &Path) -> Result<ProjectMetadata> {
        let package_json = project_root.join("package.json");
        if !package_json.exists() {
            return Err(anyhow::anyhow!("package.json not found"));
        }

        let content = fs::read_to_string(&package_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        Ok(ProjectMetadata {
            name: json
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: json
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            description: json
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: json
                .get("repository")
                .and_then(|v| v.as_str().or_else(|| v.get("url").and_then(|u| u.as_str())))
                .map(|s| s.to_string()),
            license: json
                .get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            authors: json
                .get("author")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
                .unwrap_or_default(),
        })
    }

    /// Extract metadata from Python project files
    fn extract_python_metadata(_project_root: &Path) -> Result<ProjectMetadata> {
        // Simplified for now - would parse setup.py, pyproject.toml, etc.
        Ok(ProjectMetadata {
            name: "python-project".to_string(),
            version: "0.0.0".to_string(),
            description: None,
            repository: None,
            license: None,
            authors: Vec::new(),
        })
    }

    /// Extract metadata from composer.json
    fn extract_php_metadata(_project_root: &Path) -> Result<ProjectMetadata> {
        // Simplified for now - would parse composer.json
        Ok(ProjectMetadata {
            name: "php-project".to_string(),
            version: "0.0.0".to_string(),
            description: None,
            repository: None,
            license: None,
            authors: Vec::new(),
        })
    }

    /// Detect build configuration
    fn detect_build_config(
        project_root: &Path,
        primary_language: &Language,
    ) -> Result<BuildConfig> {
        let build_tool = match primary_language {
            Language::Rust => BuildTool::Cargo,
            Language::TypeScript | Language::JavaScript => {
                if project_root.join("vite.config.js").exists()
                    || project_root.join("vite.config.ts").exists()
                {
                    BuildTool::Vite
                } else if project_root.join("webpack.config.js").exists() {
                    BuildTool::WebpackJS
                } else {
                    BuildTool::Unknown("npm".to_string())
                }
            }
            Language::Python => {
                if project_root.join("setup.py").exists() {
                    BuildTool::SetupPy
                } else {
                    BuildTool::Unknown("python".to_string())
                }
            }
            Language::PHP => BuildTool::ComposerPhp,
            Language::Go => BuildTool::GoBuild,
            Language::Java => {
                if project_root.join("build.gradle").exists() {
                    BuildTool::Gradle
                } else {
                    BuildTool::Maven
                }
            }
            Language::CSharp => BuildTool::DotNet,
            Language::Unknown(_) => BuildTool::Unknown("unknown".to_string()),
        };

        Ok(BuildConfig {
            build_tool,
            targets: vec!["default".to_string()],
            environment: std::collections::HashMap::new(),
        })
    }
}