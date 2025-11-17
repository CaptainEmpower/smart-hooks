//! Language detection for multi-language projects
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::project::multi_lang_types::Language;

/// Language detection utilities
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect all supported languages in a project
    pub fn detect_languages(project_root: &Path) -> Result<Vec<Language>> {
        let mut languages = Vec::new();

        // Check by configuration files
        Self::detect_by_config_files(project_root, &mut languages)?;

        // Check by file extensions if no config files found
        if languages.is_empty() {
            Self::detect_by_file_extensions(project_root, &mut languages)?;
        }

        // Deduplicate
        languages.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        languages.dedup_by(|a, b| format!("{:?}", a) == format!("{:?}", b));

        Ok(languages)
    }

    /// Detect languages by looking for configuration files
    fn detect_by_config_files(project_root: &Path, languages: &mut Vec<Language>) -> Result<()> {
        // Rust
        if Self::glob_exists(project_root, "Cargo.toml")? {
            languages.push(Language::Rust);
        }

        // Node.js/TypeScript
        if Self::glob_exists(project_root, "package.json")? {
            languages.push(Language::TypeScript);
        }

        // Python
        if Self::glob_exists(project_root, "pyproject.toml")?
            || Self::glob_exists(project_root, "setup.py")?
            || Self::glob_exists(project_root, "requirements.txt")?
        {
            languages.push(Language::Python);
        }

        // PHP
        if Self::glob_exists(project_root, "composer.json")? {
            languages.push(Language::PHP);
        }

        // Go
        if Self::glob_exists(project_root, "go.mod")? {
            languages.push(Language::Go);
        }

        // Java
        if Self::glob_exists(project_root, "pom.xml")?
            || Self::glob_exists(project_root, "build.gradle")?
        {
            languages.push(Language::Java);
        }

        Ok(())
    }

    /// Detect languages by scanning file extensions
    fn detect_by_file_extensions(project_root: &Path, languages: &mut Vec<Language>) -> Result<()> {
        Self::scan_directory_for_languages(project_root, languages)?;
        Ok(())
    }

    /// Recursively scan directory for language-specific files
    fn scan_directory_for_languages(dir: &Path, languages: &mut Vec<Language>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() {
                // Skip common non-source directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if matches!(
                        dir_name,
                        "node_modules" | "target" | ".git" | "vendor" | "__pycache__"
                    ) {
                        continue;
                    }
                }
                Self::scan_directory_for_languages(&path, languages)?;
            } else if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if let Some(language) = Self::language_from_extension(extension) {
                    if !languages.contains(&language) {
                        languages.push(language);
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

    /// Check if a glob pattern exists in the project
    fn glob_exists(project_root: &Path, pattern: &str) -> Result<bool> {
        let full_pattern = project_root.join(pattern);
        Ok(full_pattern.exists())
    }

    /// Determine the primary language of a project
    pub fn determine_primary_language(
        languages: &[Language],
        project_root: &Path,
    ) -> Result<Language> {
        if languages.is_empty() {
            return Err(anyhow::anyhow!("No languages detected"));
        }

        if languages.len() == 1 {
            return Ok(languages[0].clone());
        }

        // Check for explicit configuration
        if let Ok(primary) = Self::detect_primary_from_config(project_root) {
            if languages.contains(&primary) {
                return Ok(primary);
            }
        }

        // Count files for each language and pick the most common
        let mut file_counts = Vec::new();
        for language in languages {
            let count = Self::count_language_files(project_root, language)?;
            file_counts.push((language.clone(), count));
        }

        file_counts.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(file_counts[0].0.clone())
    }

    /// Try to detect primary language from configuration files
    fn detect_primary_from_config(project_root: &Path) -> Result<Language> {
        // Rust projects are typically strongly typed
        if project_root.join("Cargo.toml").exists() {
            return Ok(Language::Rust);
        }

        // Go projects
        if project_root.join("go.mod").exists() {
            return Ok(Language::Go);
        }

        // Java projects
        if project_root.join("pom.xml").exists() || project_root.join("build.gradle").exists() {
            return Ok(Language::Java);
        }

        Err(anyhow::anyhow!("No primary language detected from config"))
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
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() {
                // Skip common non-source directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if matches!(
                        dir_name,
                        "node_modules" | "target" | ".git" | "vendor" | "__pycache__"
                    ) {
                        continue;
                    }
                }
                Self::count_files_recursive(&path, extensions, count)?;
            } else if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&extension) {
                    *count += 1;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(
            LanguageDetector::language_from_extension("rs"),
            Some(Language::Rust)
        );
        assert_eq!(
            LanguageDetector::language_from_extension("ts"),
            Some(Language::TypeScript)
        );
        assert_eq!(
            LanguageDetector::language_from_extension("py"),
            Some(Language::Python)
        );
        assert_eq!(
            LanguageDetector::language_from_extension("php"),
            Some(Language::PHP)
        );
        assert_eq!(LanguageDetector::language_from_extension("unknown"), None);
    }

    #[test]
    fn test_detect_languages_rust_project() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Create Rust project structure
        fs::write(
            project_root.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )?;
        fs::create_dir_all(project_root.join("src"))?;
        fs::write(project_root.join("src/main.rs"), "fn main() {}")?;

        let languages = LanguageDetector::detect_languages(project_root)?;
        assert!(languages.contains(&Language::Rust));
        Ok(())
    }

    #[test]
    fn test_detect_languages_by_extensions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        // Create various source files without config files
        fs::create_dir_all(project_root.join("src"))?;
        fs::write(project_root.join("src/main.rs"), "fn main() {}")?;
        fs::write(project_root.join("src/app.ts"), "console.log('hello');")?;
        fs::write(project_root.join("src/script.py"), "print('hello')")?;

        let languages = LanguageDetector::detect_languages(project_root)?;
        assert!(languages.contains(&Language::Rust));
        assert!(languages.contains(&Language::TypeScript));
        assert!(languages.contains(&Language::Python));
        Ok(())
    }

    #[test]
    fn test_determine_primary_language_single() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        let languages = vec![Language::Rust];
        let primary = LanguageDetector::determine_primary_language(&languages, project_root)?;
        assert_eq!(primary, Language::Rust);
        Ok(())
    }

    #[test]
    fn test_determine_primary_language_by_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        fs::write(
            project_root.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )?;

        let languages = vec![Language::Rust, Language::TypeScript];
        let primary = LanguageDetector::determine_primary_language(&languages, project_root)?;
        assert_eq!(primary, Language::Rust);
        Ok(())
    }

    #[test]
    fn test_count_language_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        fs::create_dir_all(project_root.join("src"))?;
        fs::write(project_root.join("src/main.rs"), "fn main() {}")?;
        fs::write(project_root.join("src/lib.rs"), "// lib")?;
        fs::write(project_root.join("src/app.ts"), "console.log('hello');")?;

        let rust_count = LanguageDetector::count_language_files(project_root, &Language::Rust)?;
        assert_eq!(rust_count, 2);

        let ts_count = LanguageDetector::count_language_files(project_root, &Language::TypeScript)?;
        assert_eq!(ts_count, 1);
        Ok(())
    }

    #[test]
    fn test_glob_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path();

        fs::write(
            project_root.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )?;

        assert!(LanguageDetector::glob_exists(project_root, "Cargo.toml")?);
        assert!(!LanguageDetector::glob_exists(
            project_root,
            "package.json"
        )?);
        Ok(())
    }
}