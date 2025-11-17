/// Module name extraction utilities
/// Focused on converting file paths to Rust module names
/// Extract module name from a Rust source file path
pub fn extract_module_name(file_path: &str) -> Option<String> {
    // Find the src/ directory and extract everything after it
    let src_index = file_path.find("/src/")?;
    let relative_path = &file_path[src_index + 5..]; // Skip "/src/"

    if !relative_path.ends_with(".rs") {
        return None;
    }

    let module_path = &relative_path[..relative_path.len() - 3];

    // Skip main.rs and lib.rs as they don't have specific tests
    if module_path == "main" || module_path == "lib" {
        return None;
    }

    // Convert path to module name (/ to ::, remove /mod)
    let module_name = module_path.replace('/', "::").replace("::mod", "");

    Some(module_name)
}

/// Check if a file path represents a core module
pub fn is_core_module(file_path: &str) -> bool {
    file_path.contains("/core/")
        || file_path.contains("/types.rs")
        || file_path.contains("/error.rs")
        || file_path.contains("/lib.rs")
        || file_path.contains("/main.rs")
}

/// Check if a file path represents behavioral logic
pub fn is_behavioral_module(file_path: &str) -> bool {
    file_path.contains("/apply/")
        || file_path.contains("/strategy/")
        || file_path.contains("/fast_export/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_module_name_valid() {
        assert_eq!(
            extract_module_name("any-crate/src/core/mover.rs"),
            Some("core::mover".to_string())
        );
        assert_eq!(
            extract_module_name("project/src/apply/strategy.rs"),
            Some("apply::strategy".to_string())
        );
        assert_eq!(
            extract_module_name("crates/git-mvh/src/types.rs"),
            Some("types".to_string())
        );
    }

    #[test]
    fn test_extract_module_name_invalid() {
        assert_eq!(extract_module_name("any-crate/src/main.rs"), None);
        assert_eq!(extract_module_name("project/src/lib.rs"), None);
        assert_eq!(extract_module_name("other/file.rs"), None);
        assert_eq!(extract_module_name("no-src-dir/file.rs"), None);
    }

    #[test]
    fn test_is_core_module() {
        assert!(is_core_module("any-path/src/core/mover.rs"));
        assert!(is_core_module("project/src/types.rs"));
        assert!(is_core_module("crate/src/error.rs"));
        assert!(is_core_module("some/src/lib.rs"));
        assert!(!is_core_module("project/src/apply/strategy.rs"));
    }

    #[test]
    fn test_is_behavioral_module() {
        assert!(is_behavioral_module("any-crate/src/apply/strategy.rs"));
        assert!(is_behavioral_module("project/src/strategy/adaptive.rs"));
        assert!(is_behavioral_module("crate/src/fast_export/parser.rs"));
        assert!(!is_behavioral_module("project/src/core/mover.rs"));
    }
}