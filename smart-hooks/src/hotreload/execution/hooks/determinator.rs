//! Logic for determining which hooks should run based on file changes

use super::super::super::{tracking::ChangeSet, HookType, HotReloadResult};

/// Determines which hooks should run based on file changes
pub struct HookDeterminator;

impl HookDeterminator {
    /// Determine which hooks to run based on file changes
    pub async fn determine_hooks_to_run(changes: &ChangeSet) -> HotReloadResult<Vec<HookType>> {
        let mut hooks = Vec::new();

        // Always run tests for code changes
        if !changes.changed_files.is_empty() {
            hooks.push(HookType::Test);
        }

        // Run linting for source code files
        if Self::has_source_code_changes(changes) {
            hooks.push(HookType::Lint);
        }

        // Run formatting check
        hooks.push(HookType::Format);

        // Run analysis for dependency files
        if Self::has_dependency_file_changes(changes) {
            hooks.push(HookType::Analysis);
        }

        Ok(hooks)
    }

    /// Check if changes include source code files
    pub fn has_source_code_changes(changes: &ChangeSet) -> bool {
        let source_extensions = [
            ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".php", ".go", ".java", ".cs",
        ];

        changes.all_files().iter().any(|file| {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                source_extensions.contains(&format!(".{}", ext).as_str())
            } else {
                false
            }
        })
    }

    /// Check if changes include dependency files
    pub fn has_dependency_file_changes(changes: &ChangeSet) -> bool {
        let dep_files = [
            "Cargo.toml",
            "package.json",
            "requirements.txt",
            "composer.json",
            "go.mod",
        ];

        changes.all_files().iter().any(|file| {
            if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
                dep_files.contains(&file_name)
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_hook_determination() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let rust_file = temp_dir.path().join("src/lib.rs");
        fs::create_dir_all(rust_file.parent().unwrap()).unwrap();
        fs::write(&rust_file, "fn test() {}").unwrap();

        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        // Test source code changes
        let mut changes_with_rust = ChangeSet::new();
        changes_with_rust.add_changed_file(rust_file);

        let hooks = HookDeterminator::determine_hooks_to_run(&changes_with_rust)
            .await
            .unwrap();

        assert!(hooks.contains(&HookType::Test));
        assert!(hooks.contains(&HookType::Lint));
        assert!(hooks.contains(&HookType::Format));

        // Test dependency file changes
        let mut changes_with_deps = ChangeSet::new();
        changes_with_deps.add_changed_file(cargo_toml);

        let hooks = HookDeterminator::determine_hooks_to_run(&changes_with_deps)
            .await
            .unwrap();

        assert!(hooks.contains(&HookType::Test));
        assert!(hooks.contains(&HookType::Format));
        assert!(hooks.contains(&HookType::Analysis));
    }

    #[test]
    fn test_source_code_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Test Rust file
        let rust_file = temp_dir.path().join("src/lib.rs");
        let mut changes_with_rust = ChangeSet::new();
        changes_with_rust.add_changed_file(rust_file);

        assert!(HookDeterminator::has_source_code_changes(
            &changes_with_rust
        ));

        // Test non-source file
        let readme = temp_dir.path().join("README.md");
        let mut changes_no_source = ChangeSet::new();
        changes_no_source.add_changed_file(readme);

        assert!(!HookDeterminator::has_source_code_changes(
            &changes_no_source
        ));
    }

    #[test]
    fn test_dependency_file_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Test Cargo.toml
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        let mut changes_with_deps = ChangeSet::new();
        changes_with_deps.add_changed_file(cargo_toml);

        assert!(HookDeterminator::has_dependency_file_changes(
            &changes_with_deps
        ));

        // Test non-dependency file
        let rust_file = temp_dir.path().join("src/lib.rs");
        let mut changes_no_deps = ChangeSet::new();
        changes_no_deps.add_changed_file(rust_file);

        assert!(!HookDeterminator::has_dependency_file_changes(
            &changes_no_deps
        ));
    }
}