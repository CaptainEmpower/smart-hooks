//! Compatibility shim for existing ProjectDiscovery functionality
use crate::project::types::{
    CrateInfo, CrateTestConfig, CrateType, ProjectMetadata, RustProjectConfig, TestStrategy,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Project discovery for Rust projects (compatibility interface)
pub struct ProjectDiscovery;

impl ProjectDiscovery {
    /// Discover a Rust project configuration by analyzing the workspace
    pub fn discover<P: AsRef<Path>>(start_path: P) -> Result<RustProjectConfig> {
        let start_path = start_path.as_ref();
        let (project_root, manifest_path) = Self::find_project_root(start_path)?;

        // Parse the root manifest
        let manifest_content =
            fs::read_to_string(&manifest_path).context("Failed to read Cargo.toml")?;
        let manifest: toml::Value = manifest_content
            .parse()
            .context("Failed to parse Cargo.toml")?;

        // Extract metadata
        let metadata = Self::extract_metadata(&manifest, &manifest_path)?;

        // Check if this is a workspace or a single crate
        let crates = if manifest.get("workspace").is_some() {
            Self::discover_workspace_crates(&project_root, &manifest)?
        } else {
            vec![Self::discover_single_crate(
                &project_root,
                &manifest,
                &metadata,
            )?]
        };

        Ok(RustProjectConfig {
            workspace_root: project_root,
            metadata,
            crates,
            test_strategy: TestStrategy::default(),
        })
    }

    /// Find the project root by looking for Cargo.toml
    fn find_project_root(start_path: &Path) -> Result<(PathBuf, PathBuf)> {
        let mut current = start_path.to_path_buf();

        loop {
            let manifest_path = current.join("Cargo.toml");
            if manifest_path.exists() {
                return Ok((current, manifest_path));
            }

            current = current
                .parent()
                .context("Could not find Cargo.toml in current directory or parent directories")?
                .to_path_buf();
        }
    }

    /// Extract project metadata from manifest
    fn extract_metadata(manifest: &toml::Value, manifest_path: &Path) -> Result<ProjectMetadata> {
        let package = manifest
            .get("package")
            .or_else(|| manifest.get("workspace").and_then(|w| w.get("package")))
            .context("No [package] section found in Cargo.toml")?;

        Ok(ProjectMetadata {
            name: package
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(
                    manifest_path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown"),
                )
                .to_string(),
            version: package
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.0")
                .to_string(),
            edition: package
                .get("edition")
                .and_then(|v| v.as_str())
                .unwrap_or("2021")
                .to_string(),
            description: package
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: manifest
                .get("package")
                .and_then(|p| p.get("repository"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Discover workspace crates
    fn discover_workspace_crates(
        project_root: &Path,
        manifest: &toml::Value,
    ) -> Result<Vec<CrateInfo>> {
        let workspace = manifest
            .get("workspace")
            .context("No workspace section found")?;

        let members = workspace
            .get("members")
            .and_then(|m| m.as_array())
            .context("No workspace members found")?;

        let mut crates = Vec::new();
        for member in members {
            let member_path = member.as_str().context("Invalid workspace member path")?;

            let crate_path = project_root.join(member_path);
            let crate_manifest_path = crate_path.join("Cargo.toml");

            if crate_manifest_path.exists() {
                let manifest_content = fs::read_to_string(&crate_manifest_path)?;
                let member_manifest: toml::Value = manifest_content.parse()?;
                let member_metadata =
                    Self::extract_metadata(&member_manifest, &crate_manifest_path)?;
                let crate_info =
                    Self::discover_single_crate(&crate_path, &member_manifest, &member_metadata)?;
                crates.push(crate_info);
            }
        }

        Ok(crates)
    }

    /// Discover a single crate
    fn discover_single_crate(
        crate_path: &Path,
        manifest: &toml::Value,
        metadata: &ProjectMetadata,
    ) -> Result<CrateInfo> {
        let src_path = crate_path.join("src");
        let main_rs = src_path.join("main.rs");
        let _lib_rs = src_path.join("lib.rs");

        let crate_type = if main_rs.exists() {
            CrateType::Binary
        } else {
            CrateType::Library // Default for lib.rs or missing files
        };

        // Extract dependencies
        let dependencies = manifest
            .get("dependencies")
            .and_then(|d| d.as_table())
            .map(|table| table.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        Ok(CrateInfo {
            name: metadata.name.clone(),
            path: crate_path.to_path_buf(),
            manifest_path: crate_path.join("Cargo.toml"),
            crate_type,
            dependencies,
            features: Vec::new(), // Would be extracted from manifest in full implementation
            test_config: CrateTestConfig::default(),
        })
    }
}