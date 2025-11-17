//! Project metadata extraction utilities
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::project::multi_lang_types::ProjectMetadata;

/// Project metadata extraction utilities
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract metadata from Cargo.toml
    pub fn extract_rust_metadata(project_root: &Path) -> Result<ProjectMetadata> {
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
    pub fn extract_node_metadata(project_root: &Path) -> Result<ProjectMetadata> {
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
    pub fn extract_python_metadata(_project_root: &Path) -> Result<ProjectMetadata> {
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
    pub fn extract_php_metadata(_project_root: &Path) -> Result<ProjectMetadata> {
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

    /// Create generic fallback metadata
    pub fn extract_generic_metadata(project_root: &Path) -> ProjectMetadata {
        ProjectMetadata {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_generic_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = MetadataExtractor::extract_generic_metadata(temp_dir.path());

        assert_eq!(metadata.version, "0.0.0");
        assert!(metadata.description.is_none());
        assert!(metadata.authors.is_empty());
    }

    #[test]
    fn test_rust_metadata_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = MetadataExtractor::extract_rust_metadata(temp_dir.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cargo.toml not found"));
    }

    #[test]
    fn test_rust_metadata_extraction() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cargo_content = r#"
[package]
name = "test-project"
version = "1.0.0"
description = "A test project"
authors = ["Test Author <test@example.com>"]
license = "MIT"
repository = "https://github.com/test/test-project"
"#;

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;

        let metadata = MetadataExtractor::extract_rust_metadata(temp_dir.path())?;
        assert_eq!(metadata.name, "test-project");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, Some("A test project".to_string()));
        assert_eq!(metadata.license, Some("MIT".to_string()));
        assert_eq!(metadata.authors, vec!["Test Author <test@example.com>"]);
        Ok(())
    }

    #[test]
    fn test_node_metadata_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = MetadataExtractor::extract_node_metadata(temp_dir.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("package.json not found"));
    }

    #[test]
    fn test_node_metadata_extraction() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_content = r#"{
  "name": "test-node-project",
  "version": "2.0.0",
  "description": "A test Node.js project",
  "author": "Node Author",
  "license": "Apache-2.0",
  "repository": "https://github.com/test/node-project"
}"#;

        fs::write(temp_dir.path().join("package.json"), package_content)?;

        let metadata = MetadataExtractor::extract_node_metadata(temp_dir.path())?;
        assert_eq!(metadata.name, "test-node-project");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(
            metadata.description,
            Some("A test Node.js project".to_string())
        );
        assert_eq!(metadata.license, Some("Apache-2.0".to_string()));
        assert_eq!(metadata.authors, vec!["Node Author"]);
        Ok(())
    }

    #[test]
    fn test_python_metadata_placeholder() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let metadata = MetadataExtractor::extract_python_metadata(temp_dir.path())?;

        assert_eq!(metadata.name, "python-project");
        assert_eq!(metadata.version, "0.0.0");
        Ok(())
    }

    #[test]
    fn test_php_metadata_placeholder() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let metadata = MetadataExtractor::extract_php_metadata(temp_dir.path())?;

        assert_eq!(metadata.name, "php-project");
        assert_eq!(metadata.version, "0.0.0");
        Ok(())
    }

    #[test]
    fn test_workspace_cargo_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cargo_content = r#"
[workspace]
name = "workspace-project"
version = "3.0.0"
description = "A workspace project"
"#;

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_content)?;

        let metadata = MetadataExtractor::extract_rust_metadata(temp_dir.path())?;
        assert_eq!(metadata.name, "workspace-project");
        assert_eq!(metadata.version, "3.0.0");
        Ok(())
    }

    #[test]
    fn test_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "invalid toml content").unwrap();

        let result = MetadataExtractor::extract_rust_metadata(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "invalid json content").unwrap();

        let result = MetadataExtractor::extract_node_metadata(temp_dir.path());
        assert!(result.is_err());
    }
}