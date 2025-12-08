//! Build configuration detection utilities
use anyhow::Result;
use std::path::Path;

use crate::project::multi_lang_types::{BuildConfig, BuildTool, Language};

/// Build configuration detection utilities
pub struct BuildDetector;

impl BuildDetector {
    /// Detect build configuration for a project
    pub fn detect_build_config(
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

    /// Check for specific build tool configuration files
    pub fn has_build_tool_config(project_root: &Path, language: &Language) -> bool {
        match language {
            Language::Rust => project_root.join("Cargo.toml").exists(),
            Language::TypeScript | Language::JavaScript => {
                project_root.join("package.json").exists()
            }
            Language::Python => {
                project_root.join("setup.py").exists()
                    || project_root.join("pyproject.toml").exists()
            }
            Language::PHP => project_root.join("composer.json").exists(),
            Language::Go => project_root.join("go.mod").exists(),
            Language::Java => {
                project_root.join("pom.xml").exists() || project_root.join("build.gradle").exists()
            }
            Language::CSharp => {
                project_root.join("project.json").exists() || Self::has_csproj_files(project_root)
            }
            Language::Unknown(_) => false,
        }
    }

    /// Check for .csproj files in the project
    fn has_csproj_files(project_root: &Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(project_root) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "csproj" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Detect build tool priority for multi-language projects
    pub fn detect_primary_build_tool(project_root: &Path, languages: &[Language]) -> BuildTool {
        // Priority order for build tools
        let build_priorities = [
            (Language::Rust, BuildTool::Cargo),
            (Language::TypeScript, BuildTool::Vite),
            (Language::JavaScript, BuildTool::WebpackJS),
            (Language::Go, BuildTool::GoBuild),
            (Language::Java, BuildTool::Maven),
            (Language::Python, BuildTool::SetupPy),
            (Language::PHP, BuildTool::ComposerPhp),
            (Language::CSharp, BuildTool::DotNet),
        ];

        for (lang, tool) in &build_priorities {
            if languages.contains(lang) && Self::has_build_tool_config(project_root, lang) {
                return tool.clone();
            }
        }

        BuildTool::Unknown("none".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust_build_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )?;

        let config = BuildDetector::detect_build_config(temp_dir.path(), &Language::Rust)?;
        assert!(matches!(config.build_tool, BuildTool::Cargo));
        assert_eq!(config.targets, vec!["default".to_string()]);
        Ok(())
    }

    #[test]
    fn test_detect_typescript_vite_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("vite.config.ts"), "export default {}")?;

        let config = BuildDetector::detect_build_config(temp_dir.path(), &Language::TypeScript)?;
        assert!(matches!(config.build_tool, BuildTool::Vite));
        Ok(())
    }

    #[test]
    fn test_detect_javascript_webpack_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("webpack.config.js"),
            "module.exports = {}",
        )?;

        let config = BuildDetector::detect_build_config(temp_dir.path(), &Language::JavaScript)?;
        assert!(matches!(config.build_tool, BuildTool::WebpackJS));
        Ok(())
    }

    #[test]
    fn test_detect_python_setup_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(
            temp_dir.path().join("setup.py"),
            "from setuptools import setup",
        )?;

        let config = BuildDetector::detect_build_config(temp_dir.path(), &Language::Python)?;
        assert!(matches!(config.build_tool, BuildTool::SetupPy));
        Ok(())
    }

    #[test]
    fn test_detect_java_gradle_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("build.gradle"), "plugins {}")?;

        let config = BuildDetector::detect_build_config(temp_dir.path(), &Language::Java)?;
        assert!(matches!(config.build_tool, BuildTool::Gradle));
        Ok(())
    }

    #[test]
    fn test_detect_java_maven_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("pom.xml"), "<project></project>")?;

        let config = BuildDetector::detect_build_config(temp_dir.path(), &Language::Java)?;
        assert!(matches!(config.build_tool, BuildTool::Maven));
        Ok(())
    }

    #[test]
    fn test_has_build_tool_config_rust() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!BuildDetector::has_build_tool_config(
            temp_dir.path(),
            &Language::Rust
        ));

        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        assert!(BuildDetector::has_build_tool_config(
            temp_dir.path(),
            &Language::Rust
        ));
    }

    #[test]
    fn test_has_build_tool_config_node() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!BuildDetector::has_build_tool_config(
            temp_dir.path(),
            &Language::TypeScript
        ));

        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        assert!(BuildDetector::has_build_tool_config(
            temp_dir.path(),
            &Language::TypeScript
        ));
        assert!(BuildDetector::has_build_tool_config(
            temp_dir.path(),
            &Language::JavaScript
        ));
    }

    #[test]
    fn test_has_csproj_files() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!BuildDetector::has_csproj_files(temp_dir.path()));

        fs::write(temp_dir.path().join("test.csproj"), "<Project></Project>").unwrap();
        assert!(BuildDetector::has_csproj_files(temp_dir.path()));
    }

    #[test]
    fn test_detect_primary_build_tool() {
        let temp_dir = TempDir::new().unwrap();
        let languages = vec![Language::Rust, Language::TypeScript];

        // No config files
        let tool = BuildDetector::detect_primary_build_tool(temp_dir.path(), &languages);
        assert!(matches!(tool, BuildTool::Unknown(_)));

        // Add Rust config
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let tool = BuildDetector::detect_primary_build_tool(temp_dir.path(), &languages);
        assert!(matches!(tool, BuildTool::Cargo));
    }

    #[test]
    fn test_unknown_language_build_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let unknown_lang = Language::Unknown("unknown".to_string());

        let config = BuildDetector::detect_build_config(temp_dir.path(), &unknown_lang)?;
        assert!(matches!(config.build_tool, BuildTool::Unknown(_)));
        Ok(())
    }
}