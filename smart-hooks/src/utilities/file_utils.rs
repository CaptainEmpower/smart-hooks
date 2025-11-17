use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// File operation utilities
/// Focused on safe file reading and content analysis
/// Safely read file content with proper error handling
pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Check if file path represents a Rust source file
pub fn is_rust_file(file_path: &str) -> bool {
    file_path.ends_with(".rs")
}

/// Check if file path is in the main crate (not tests)
pub fn is_core_functionality_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/src/") && !path_str.contains("/tests/") && path_str.ends_with(".rs")
}

/// Extract relative path from full file path
pub fn get_relative_path(file_path: &str, prefix: &str) -> Option<String> {
    if !file_path.starts_with(prefix) {
        return None;
    }

    let relative = &file_path[prefix.len()..];
    Some(relative.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rust_file() {
        assert!(is_rust_file("src/main.rs"));
        assert!(is_rust_file("core/mover.rs"));
        assert!(!is_rust_file("Cargo.toml"));
        assert!(!is_rust_file("README.md"));
    }

    #[test]
    fn test_is_core_functionality_file() {
        let core_path = Path::new("crates/git-mvh/src/core/mover.rs");
        let test_path = Path::new("crates/git-mvh/tests/integration.rs");

        assert!(is_core_functionality_file(core_path));
        assert!(!is_core_functionality_file(test_path));
    }

    #[test]
    fn test_get_relative_path() {
        assert_eq!(
            get_relative_path("crates/git-mvh/src/core/mover.rs", "crates/git-mvh/src/"),
            Some("core/mover.rs".to_string())
        );
        assert_eq!(
            get_relative_path("other/file.rs", "crates/git-mvh/src/"),
            None
        );
    }
}