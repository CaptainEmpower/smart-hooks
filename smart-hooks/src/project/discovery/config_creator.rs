//! Language configuration creation utilities
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::project::multi_lang_types::*;

/// Language configuration creation utilities
pub struct ConfigCreator;

impl ConfigCreator {
    /// Create language-specific configuration
    pub fn create_language_config(
        language: Language,
        project_root: &Path,
    ) -> Result<LanguageConfig> {
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
    pub fn find_source_directories(
        project_root: &Path,
        language: &Language,
    ) -> Result<Vec<PathBuf>> {
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
    pub fn has_language_files(dir: &Path, language: &Language) -> Result<bool> {
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
    pub fn detect_package_manager(
        project_root: &Path,
        language: &Language,
    ) -> Result<PackageManager> {
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
    pub fn detect_test_framework(
        _project_root: &Path,
        language: &Language,
    ) -> Result<TestFramework> {
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
    pub fn create_language_commands(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_source_directories_rust() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Create src directory
        fs::create_dir_all(project_root.join("src"))?;
        fs::write(project_root.join("src/main.rs"), "fn main() {}")?;

        let source_dirs = ConfigCreator::find_source_directories(project_root, &Language::Rust)?;
        assert_eq!(source_dirs.len(), 1);
        assert_eq!(source_dirs[0], project_root.join("src"));
        Ok(())
    }

    #[test]
    fn test_find_source_directories_fallback() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Create Rust file in project root
        fs::write(project_root.join("main.rs"), "fn main() {}")?;

        let source_dirs = ConfigCreator::find_source_directories(project_root, &Language::Rust)?;
        assert_eq!(source_dirs.len(), 1);
        assert_eq!(source_dirs[0], project_root);
        Ok(())
    }

    #[test]
    fn test_has_language_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // No Rust files initially
        assert!(!ConfigCreator::has_language_files(
            project_root,
            &Language::Rust
        )?);

        // Add Rust file
        fs::write(project_root.join("main.rs"), "fn main() {}")?;
        assert!(ConfigCreator::has_language_files(
            project_root,
            &Language::Rust
        )?);
        Ok(())
    }

    #[test]
    fn test_detect_package_manager_node() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Default npm
        let pm = ConfigCreator::detect_package_manager(project_root, &Language::TypeScript)?;
        assert!(matches!(pm, PackageManager::Npm));

        // Yarn
        fs::write(project_root.join("yarn.lock"), "")?;
        let pm = ConfigCreator::detect_package_manager(project_root, &Language::TypeScript)?;
        assert!(matches!(pm, PackageManager::Yarn));

        // pnpm (higher priority)
        fs::write(project_root.join("pnpm-lock.yaml"), "")?;
        let pm = ConfigCreator::detect_package_manager(project_root, &Language::TypeScript)?;
        assert!(matches!(pm, PackageManager::Pnpm));
        Ok(())
    }

    #[test]
    fn test_detect_package_manager_python() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Default pip
        let pm = ConfigCreator::detect_package_manager(project_root, &Language::Python)?;
        assert!(matches!(pm, PackageManager::Pip));

        // Pipenv
        fs::write(project_root.join("Pipfile"), "")?;
        let pm = ConfigCreator::detect_package_manager(project_root, &Language::Python)?;
        assert!(matches!(pm, PackageManager::Pipenv));

        // Poetry (higher priority)
        fs::write(project_root.join("pyproject.toml"), "")?;
        let pm = ConfigCreator::detect_package_manager(project_root, &Language::Python)?;
        assert!(matches!(pm, PackageManager::Poetry));
        Ok(())
    }

    #[test]
    fn test_detect_test_framework() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        let tf = ConfigCreator::detect_test_framework(project_root, &Language::Rust)?;
        assert!(matches!(tf, TestFramework::RustTest));

        let tf = ConfigCreator::detect_test_framework(project_root, &Language::TypeScript)?;
        assert!(matches!(tf, TestFramework::Jest));

        let tf = ConfigCreator::detect_test_framework(project_root, &Language::Python)?;
        assert!(matches!(tf, TestFramework::Pytest));
        Ok(())
    }

    #[test]
    fn test_create_rust_commands() {
        let commands = ConfigCreator::create_language_commands(
            &Language::Rust,
            &PackageManager::Cargo,
            &TestFramework::RustTest,
        );

        assert_eq!(commands.test_command, vec!["cargo", "test"]);
        assert_eq!(
            commands.build_command,
            Some(vec!["cargo".to_string(), "build".to_string()])
        );
        assert_eq!(
            commands.lint_command,
            Some(vec!["cargo".to_string(), "clippy".to_string()])
        );
        assert_eq!(
            commands.format_command,
            Some(vec!["cargo".to_string(), "fmt".to_string()])
        );
    }

    #[test]
    fn test_create_typescript_commands() {
        let commands = ConfigCreator::create_language_commands(
            &Language::TypeScript,
            &PackageManager::Pnpm,
            &TestFramework::Jest,
        );

        assert_eq!(commands.test_command, vec!["pnpm", "test"]);
        assert_eq!(
            commands.install_command,
            Some(vec!["pnpm".to_string(), "install".to_string()])
        );
    }

    #[test]
    fn test_create_language_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Create basic Rust project structure
        fs::create_dir_all(project_root.join("src"))?;
        fs::write(project_root.join("src/main.rs"), "fn main() {}")?;

        let config = ConfigCreator::create_language_config(Language::Rust, project_root)?;

        assert!(matches!(config.language, Language::Rust));
        assert!(matches!(config.package_manager, PackageManager::Cargo));
        assert!(matches!(config.test_framework, TestFramework::RustTest));
        assert_eq!(config.source_dirs.len(), 1);
        assert!(config.file_patterns.contains(&"**/*.rs".to_string()));
        Ok(())
    }

    #[test]
    fn test_unknown_language_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let unknown_lang = Language::Unknown("unknown".to_string());

        let pm = ConfigCreator::detect_package_manager(temp_dir.path(), &unknown_lang)?;
        assert!(matches!(pm, PackageManager::Unknown(_)));

        let tf = ConfigCreator::detect_test_framework(temp_dir.path(), &unknown_lang)?;
        assert!(matches!(tf, TestFramework::Unknown(_)));

        let commands = ConfigCreator::create_language_commands(&unknown_lang, &pm, &tf);
        assert!(commands
            .test_command
            .contains(&"No test command configured".to_string()));
        Ok(())
    }
}