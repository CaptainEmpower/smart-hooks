pub mod crate_analyzer;
pub mod discovery;
pub mod file_manager;
pub mod manifest_parser;
pub mod test_config;
/// Project configuration module
/// Refactored into SRP-compliant submodules
pub mod types;

// Re-export main types for external use
pub use types::{
    AITestConfig, CrateInfo, CrateTestConfig, CrateType, ProjectMetadata, RustProjectConfig,
    TestSelectionMode, TestStrategy,
};

// Main project configuration implementation
impl RustProjectConfig {
    /// Discover a Rust project configuration by analyzing the workspace
    pub fn discover<P: AsRef<std::path::Path>>(start_path: P) -> anyhow::Result<Self> {
        discovery::ProjectDiscovery::discover(start_path)
    }

    /// Get all source files in the project
    pub fn get_all_source_files(&self) -> anyhow::Result<Vec<std::path::PathBuf>> {
        file_manager::ProjectFileManager::get_all_source_files(self)
    }

    /// Find which crate a file belongs to
    pub fn crate_for_file(&self, file_path: &std::path::Path) -> Option<&CrateInfo> {
        file_manager::ProjectFileManager::crate_for_file(self, file_path)
    }

    /// Save configuration to a file for caching
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        file_manager::ProjectFileManager::save_to_file(self, path)
    }

    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        file_manager::ProjectFileManager::load_from_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_single_crate_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Create a minimal Cargo.toml
        fs::write(
            project_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
description = "A test project"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        // Create src structure
        let src_dir = project_dir.join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "// lib.rs").unwrap();

        // Discover the project
        let config = RustProjectConfig::discover(project_dir).unwrap();

        assert_eq!(config.metadata.name, "test-project");
        assert_eq!(config.metadata.version, "0.1.0");
        assert_eq!(config.metadata.edition, "2021");
        assert_eq!(config.crates.len(), 1);
        assert_eq!(config.crates[0].name, "test-project");
        assert!(matches!(config.crates[0].crate_type, CrateType::Library));
        assert!(config.crates[0].dependencies.contains(&"serde".to_string()));
    }

    #[test]
    fn test_discover_workspace_project() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path();

        // Create workspace Cargo.toml
        fs::write(
            workspace_dir.join("Cargo.toml"),
            r#"
[workspace]
members = ["crate1", "crate2"]

[workspace.package]
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        // Create first crate
        let crate1_dir = workspace_dir.join("crate1");
        fs::create_dir_all(crate1_dir.join("src")).unwrap();
        fs::write(
            crate1_dir.join("Cargo.toml"),
            r#"
[package]
name = "crate1"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        fs::write(crate1_dir.join("src/lib.rs"), "// crate1 lib").unwrap();

        // Create second crate
        let crate2_dir = workspace_dir.join("crate2");
        fs::create_dir_all(crate2_dir.join("src")).unwrap();
        fs::write(
            crate2_dir.join("Cargo.toml"),
            r#"
[package]
name = "crate2"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        fs::write(crate2_dir.join("src/main.rs"), "fn main() {}").unwrap();

        // Discover the workspace
        let config = RustProjectConfig::discover(workspace_dir).unwrap();

        assert_eq!(config.crates.len(), 2);

        let crate1 = config.crates.iter().find(|c| c.name == "crate1").unwrap();
        assert!(matches!(crate1.crate_type, CrateType::Library));

        let crate2 = config.crates.iter().find(|c| c.name == "crate2").unwrap();
        assert!(matches!(crate2.crate_type, CrateType::Binary));
    }

    #[test]
    fn test_crate_for_file() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path();

        // Set up a simple workspace
        fs::write(
            workspace_dir.join("Cargo.toml"),
            r#"
[workspace]
members = ["lib1"]

[workspace.package]
name = "test-workspace"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let lib1_dir = workspace_dir.join("lib1");
        fs::create_dir_all(lib1_dir.join("src")).unwrap();
        fs::write(
            lib1_dir.join("Cargo.toml"),
            r#"
[package]
name = "lib1"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        fs::write(lib1_dir.join("src/lib.rs"), "// lib1").unwrap();

        let config = RustProjectConfig::discover(workspace_dir).unwrap();

        let lib1_file = lib1_dir.join("src/lib.rs");
        let crate_info = config.crate_for_file(&lib1_file).unwrap();
        assert_eq!(crate_info.name, "lib1");

        let external_file = workspace_dir.join("external.rs");
        assert!(config.crate_for_file(&external_file).is_none());
    }
}
