//! Safe file reading and path classification.

use crate::utilities::path_segments::path_segments;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Read a file, naming the path in the error on failure.
pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Whether the path names a Rust source file.
///
/// This is a question about the path, not the filesystem: it does not check
/// that the file exists.
pub fn is_rust_file(file_path: &str) -> bool {
    file_path.ends_with(".rs")
}

/// Whether the path is a Rust source file inside a crate's `src` directory and
/// outside its `tests` directory.
///
/// Segment-aware so that repository-relative paths (`src/lib.rs`), which is what
/// a hook runner passes, are recognised as readily as nested ones
/// (`crates/foo/src/lib.rs`). See issue #7.
pub fn is_core_functionality_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if !path_str.ends_with(".rs") {
        return false;
    }

    let segments = path_segments(&path_str);

    // Any `tests` segment disqualifies the path, wherever it sits: `src/tests/`
    // is test code inside a crate, and `tests/src/` is a fixture crate under an
    // integration-test directory. Checking only after the source root let the
    // latter through.
    if segments.contains(&"tests") {
        return false;
    }

    segments.contains(&"src")
}

/// Strip `prefix` from `file_path`, or `None` if it does not start with it.
pub fn get_relative_path(file_path: &str, prefix: &str) -> Option<String> {
    file_path.strip_prefix(prefix).map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_files_are_recognised_by_extension_alone() {
        assert!(is_rust_file("src/main.rs"));
        assert!(is_rust_file("core/mover.rs"));
        // Existence is deliberately not consulted.
        assert!(is_rust_file("/non/existent/file.rs"));
        assert!(!is_rust_file("Cargo.toml"));
        assert!(!is_rust_file("README.md"));
    }

    #[test]
    fn core_files_are_recognised_from_repository_relative_paths() {
        assert!(is_core_functionality_file(Path::new("src/lib.rs")));
        assert!(is_core_functionality_file(Path::new("src/core/mover.rs")));
        assert!(is_core_functionality_file(Path::new("./src/main.rs")));
    }

    #[test]
    fn core_files_are_recognised_from_nested_crate_paths() {
        assert!(is_core_functionality_file(Path::new(
            "crates/git-mvh/src/core/mover.rs"
        )));
    }

    #[cfg(windows)]
    #[test]
    fn native_windows_paths_are_classified_like_git_paths() {
        // Regression for the review on #10 — see module_utils for the cause.
        assert!(is_core_functionality_file(Path::new(
            r"C:\Users\runner\Temp\.tmpAbC\src\calculator.rs"
        )));
        assert!(!is_core_functionality_file(Path::new(
            r"C:\Users\runner\Temp\.tmpAbC\tests\src\fixture.rs"
        )));
    }

    #[cfg(unix)]
    #[test]
    fn a_backslash_is_part_of_a_unix_filename_not_a_separator() {
        // Regression for the review on #11: one file at the repository root,
        // not a file under `src/`, so it is not crate functionality.
        assert!(!is_core_functionality_file(Path::new(r"src\core\mover.rs")));
    }

    #[test]
    fn fixtures_under_a_tests_directory_are_not_core_files() {
        // `tests/src/...` is a fixture crate, not this crate's source.
        assert!(!is_core_functionality_file(Path::new(
            "tests/src/example.rs"
        )));
        assert!(!is_core_functionality_file(Path::new(
            "tests/fixtures/demo/src/lib.rs"
        )));
        assert!(!is_core_functionality_file(Path::new(
            "crates/foo/tests/src/helper.rs"
        )));
    }

    #[test]
    fn test_code_and_non_rust_paths_are_not_core_files() {
        assert!(!is_core_functionality_file(Path::new(
            "crates/git-mvh/tests/integration.rs"
        )));
        assert!(!is_core_functionality_file(Path::new(
            "src/tests/helpers.rs"
        )));
        assert!(!is_core_functionality_file(Path::new("tests/thing.rs")));
        assert!(!is_core_functionality_file(Path::new("build.rs")));
        assert!(!is_core_functionality_file(Path::new("src/notes.md")));
    }

    #[test]
    fn relative_paths_are_stripped_by_prefix() {
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
