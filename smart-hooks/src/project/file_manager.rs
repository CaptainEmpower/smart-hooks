/// File management operations for project configuration
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::project::types::{CrateInfo, RustProjectConfig};

/// Handles file operations and source file discovery
pub struct ProjectFileManager;

impl ProjectFileManager {
    /// Get all source files in the project
    pub fn get_all_source_files(config: &RustProjectConfig) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for crate_info in &config.crates {
            let src_dir = crate_info.path.join("src");
            if src_dir.exists() {
                Self::collect_rust_files(&src_dir, &mut files)?;
            }
        }

        Ok(files)
    }

    /// Recursively collect all .rs files in a directory
    fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_rust_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Find which crate a file belongs to
    pub fn crate_for_file<'a>(
        config: &'a RustProjectConfig,
        file_path: &Path,
    ) -> Option<&'a CrateInfo> {
        // Try to canonicalize the file path for comparison
        let file_canonical = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());

        config.crates.iter().find(|crate_info| {
            // Try to canonicalize the crate path too
            let crate_canonical = crate_info
                .path
                .canonicalize()
                .unwrap_or_else(|_| crate_info.path.clone());
            file_canonical.starts_with(&crate_canonical)
        })
    }

    /// Save configuration to a file for caching
    pub fn save_to_file<P: AsRef<Path>>(config: &RustProjectConfig, path: P) -> Result<()> {
        let content =
            toml::to_string_pretty(config).context("Failed to serialize configuration")?;
        fs::write(path, content).context("Failed to write configuration file")?;
        Ok(())
    }

    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<RustProjectConfig> {
        let content = fs::read_to_string(path).context("Failed to read configuration file")?;
        let config: RustProjectConfig =
            toml::from_str(&content).context("Failed to parse configuration file")?;
        Ok(config)
    }
}