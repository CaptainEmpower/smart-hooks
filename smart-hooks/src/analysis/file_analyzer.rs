use crate::utilities::file_utils;
use anyhow::Result;
use std::path::Path;

/// File content analysis for detecting functionality changes
/// Focused on static analysis of Rust source code

#[derive(Debug, Default, PartialEq)]
pub struct FunctionalityFlags {
    pub has_move_operations: bool,
    pub has_conflict_resolution: bool,
    pub has_core_types: bool,
    pub has_error_handling: bool,
}

/// Analyze file content to detect functionality patterns
pub fn analyze_file_content(path: &Path) -> Result<FunctionalityFlags> {
    let content = file_utils::read_file_content(path)?;

    let mut flags = FunctionalityFlags::default();

    // Detect move operation changes
    if contains_move_operations(&content) {
        flags.has_move_operations = true;
    }

    // Detect conflict resolution changes
    if contains_conflict_resolution(&content) {
        flags.has_conflict_resolution = true;
    }

    // Detect core type changes
    if contains_core_types(&content) {
        flags.has_core_types = true;
    }

    // Detect error handling changes
    if contains_error_handling(&content) {
        flags.has_error_handling = true;
    }

    Ok(flags)
}

fn contains_move_operations(content: &str) -> bool {
    content.contains("pub fn execute_move")
        || content.contains("pub fn move_file")
        || content.contains("MoveResult")
        || content.contains("move_files")
}

fn contains_conflict_resolution(content: &str) -> bool {
    content.contains("ConflictStrategy")
        || content.contains("resolve_conflict")
        || content.contains("handle_conflict")
}

fn contains_core_types(content: &str) -> bool {
    content.contains("pub struct")
        && (content.contains("MoveOptions")
            || content.contains("GitRepository")
            || content.contains("MoveResult"))
}

fn contains_error_handling(content: &str) -> bool {
    content.contains("impl") && content.contains("GitMvhError")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_move_operations() {
        assert!(contains_move_operations("pub fn execute_move() {}"));
        assert!(contains_move_operations("let result: MoveResult = ..."));
        assert!(!contains_move_operations("pub fn other_function() {}"));
    }

    #[test]
    fn test_contains_conflict_resolution() {
        assert!(contains_conflict_resolution("use ConflictStrategy;"));
        assert!(contains_conflict_resolution("fn resolve_conflict() {}"));
        assert!(!contains_conflict_resolution("fn other_function() {}"));
    }

    #[test]
    fn test_contains_core_types() {
        assert!(contains_core_types("pub struct MoveOptions { ... }"));
        assert!(contains_core_types("pub struct GitRepository { ... }"));
        assert!(!contains_core_types("struct OtherStruct { ... }"));
        assert!(!contains_core_types("MoveOptions { ... }")); // Missing pub struct
    }

    #[test]
    fn test_contains_error_handling() {
        assert!(contains_error_handling("impl GitMvhError { ... }"));
        assert!(!contains_error_handling("impl OtherError { ... }"));
    }

    #[test]
    fn test_functionality_flags_default() {
        let flags = FunctionalityFlags::default();
        assert!(!flags.has_move_operations);
        assert!(!flags.has_conflict_resolution);
        assert!(!flags.has_core_types);
        assert!(!flags.has_error_handling);
    }
}
