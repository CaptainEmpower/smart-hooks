/// Cargo.toml manifest parsing functionality
use anyhow::Result;
use toml;

/// Parses Cargo.toml files to extract dependencies and features
pub struct ManifestParser;

impl ManifestParser {
    /// Extract dependencies from Cargo.toml
    pub fn extract_dependencies(manifest: &toml::Value) -> Result<Vec<String>> {
        let mut dependencies = Vec::new();

        // Regular dependencies
        if let Some(deps) = manifest.get("dependencies") {
            if let Some(table) = deps.as_table() {
                dependencies.extend(table.keys().cloned());
            }
        }

        // Dev dependencies
        if let Some(dev_deps) = manifest.get("dev-dependencies") {
            if let Some(table) = dev_deps.as_table() {
                dependencies.extend(table.keys().cloned());
            }
        }

        Ok(dependencies)
    }

    /// Extract features from Cargo.toml
    pub fn extract_features(manifest: &toml::Value) -> Result<Vec<String>> {
        let mut features = Vec::new();

        if let Some(features_table) = manifest.get("features") {
            if let Some(table) = features_table.as_table() {
                features.extend(table.keys().cloned());
            }
        }

        Ok(features)
    }
}