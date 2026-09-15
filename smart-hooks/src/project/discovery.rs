/// Project discovery logic for finding and analyzing Rust projects
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::project::types::{CrateInfo, ProjectMetadata, RustProjectConfig, TestStrategy};

/// Handles discovery and analysis of Rust project structures
pub struct ProjectDiscovery;

impl ProjectDiscovery {
    /// Discover a Rust project configuration by analyzing the workspace
    pub fn discover<P: AsRef<Path>>(start_path: P) -> Result<RustProjectConfig> {
        let start_path = start_path.as_ref();

        // Find the workspace root by looking for Cargo.toml
        let workspace_root =
            Self::find_workspace_root(start_path).context("Failed to find workspace root")?;

        // Parse the root Cargo.toml to understand the workspace structure
        let root_manifest = workspace_root.join("Cargo.toml");
        let root_toml: toml::Value = toml::from_str(
            &fs::read_to_string(&root_manifest).context("Failed to read root Cargo.toml")?,
        )
        .context("Failed to parse root Cargo.toml")?;

        // Determine if this is a workspace or single crate
        let crates = if root_toml.get("workspace").is_some() {
            Self::discover_workspace_crates(&workspace_root, &root_toml)?
        } else {
            vec![Self::discover_single_crate(&workspace_root, &root_toml)?]
        };

        // Extract project metadata
        let metadata = Self::extract_project_metadata(&root_toml, &workspace_root)?;

        // Create default test strategy
        let test_strategy = TestStrategy::default_for_project(&metadata);

        Ok(RustProjectConfig {
            workspace_root,
            crates,
            test_strategy,
            metadata,
        })
    }

    /// Find the workspace root by walking up the directory tree
    fn find_workspace_root(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path
            .canonicalize()
            .context("Failed to canonicalize start path")?;

        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                return Ok(current);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => anyhow::bail!("No Cargo.toml found in directory hierarchy"),
            }
        }
    }

    /// Discover all crates in a workspace
    fn discover_workspace_crates(
        workspace_root: &Path,
        root_toml: &toml::Value,
    ) -> Result<Vec<CrateInfo>> {
        let mut crates = Vec::new();

        if let Some(workspace) = root_toml.get("workspace") {
            if let Some(members) = workspace.get("members") {
                if let Some(member_array) = members.as_array() {
                    for member in member_array {
                        if let Some(member_path) = member.as_str() {
                            let crate_path = workspace_root.join(member_path);
                            if crate_path.exists() {
                                let crate_info = Self::analyze_crate(&crate_path)?;
                                crates.push(crate_info);
                            }
                        }
                    }
                }
            }
        }

        Ok(crates)
    }

    /// Discover a single crate project
    fn discover_single_crate(workspace_root: &Path, _root_toml: &toml::Value) -> Result<CrateInfo> {
        Self::analyze_crate(workspace_root)
    }

    /// Analyze a single crate to extract its information
    fn analyze_crate(crate_path: &Path) -> Result<CrateInfo> {
        let manifest_path = crate_path.join("Cargo.toml");
        let manifest_content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
        let manifest: toml::Value = toml::from_str(&manifest_content)
            .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;

        // Extract basic crate information
        let package = manifest
            .get("package")
            .context("No [package] section found")?;
        let name = package
            .get("name")
            .and_then(|n| n.as_str())
            .context("No package name found")?
            .to_string();

        // Determine crate type
        let src_path = crate_path.join("src");
        let crate_type =
            crate::project::crate_analyzer::CrateAnalyzer::determine_crate_type(&src_path)?;

        // Extract dependencies
        let dependencies =
            crate::project::manifest_parser::ManifestParser::extract_dependencies(&manifest)?;

        // Extract features
        let features =
            crate::project::manifest_parser::ManifestParser::extract_features(&manifest)?;

        // Create test configuration
        let test_config = crate::project::test_config::TestConfigBuilder::create_test_config(
            crate_path,
            &crate_type,
        )?;

        Ok(CrateInfo {
            name,
            path: crate_path.to_path_buf(),
            manifest_path,
            crate_type,
            dependencies,
            features,
            test_config,
        })
    }

    /// Extract project metadata from the root Cargo.toml
    fn extract_project_metadata(
        root_toml: &toml::Value,
        workspace_root: &Path,
    ) -> Result<ProjectMetadata> {
        let package = if let Some(workspace) = root_toml.get("workspace") {
            // Workspace - try to get metadata from workspace.package or first member
            workspace.get("package").or_else(|| {
                // If no workspace.package, try to get from first member
                root_toml.get("package")
            })
        } else {
            root_toml.get("package")
        }
        .context("No package metadata found")?;

        let name = package
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or_else(|| {
                workspace_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
            })
            .to_string();

        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();

        let edition = package
            .get("edition")
            .and_then(|e| e.as_str())
            .unwrap_or("2021")
            .to_string();

        let description = package
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let repository = package
            .get("repository")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());

        Ok(ProjectMetadata {
            name,
            version,
            edition,
            description,
            repository,
        })
    }
}

impl TestStrategy {
    /// Create a default test strategy for a project
    pub fn default_for_project(metadata: &ProjectMetadata) -> Self {
        let selection_mode =
            if metadata.name.contains("large") || metadata.name.contains("enterprise") {
                crate::project::types::TestSelectionMode::Smart
            } else {
                crate::project::types::TestSelectionMode::Conservative
            };

        TestStrategy {
            selection_mode,
            auto_integration_tests: true,
            run_doc_tests: true,
            max_analysis_time: 30,
            ai_config: Some(crate::project::types::AITestConfig {
                confidence_threshold: 0.5,
                project_context: format!(
                    "Rust project: {} ({}). {}",
                    metadata.name,
                    metadata.version,
                    metadata.description.as_deref().unwrap_or("No description")
                ),
                enable_bdd_selection: false,
            }),
        }
    }
}
