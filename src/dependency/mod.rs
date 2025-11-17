pub mod analyzer;
pub mod graph;
pub mod multi_lang_analyzer;
pub mod rust_analyzer;
pub mod rust_parser;
pub mod test_resolver;
/// Dependency analysis module for smart-hooks
/// Refactored into SRP-compliant submodules
pub mod types;

// Re-export main types for external use
pub use analyzer::DependencyAnalyzer;
pub use graph::DependencyGraph;
pub use multi_lang_analyzer::{
    LanguageDependencyAnalyzer, MultiLangDependencyAnalyzer, PhpDependencyAnalyzer,
    PythonDependencyAnalyzer, TypeScriptDependencyAnalyzer,
};
pub use rust_analyzer::RustDependencyAnalyzer;
pub use types::{Dependency, DependencyType, TestTarget, TestType};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_rust_dependency_analyzer() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Create a test project
        fs::write(
            project_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let src_dir = project_dir.join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub mod utils;
pub mod core;

use crate::utils::helper_function;
use std::collections::HashMap;

pub fn main_function() -> Result<(), ()> {
    let result = helper_function();
    Ok(())
}
"#,
        )
        .unwrap();

        fs::write(
            src_dir.join("utils.rs"),
            r#"
pub fn helper_function() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_function() {
        assert_eq!(helper_function(), 42);
    }
}
"#,
        )
        .unwrap();

        // Analyze dependencies
        let analyzer = RustDependencyAnalyzer::discover_from_path(project_dir).unwrap();
        let lib_file = src_dir.join("lib.rs");
        let dependencies = analyzer.analyze_file_dependencies(&lib_file).unwrap();

        // Should find dependency on utils module
        let utils_dep = dependencies.iter().find(|d| d.name.contains("utils"));

        assert!(utils_dep.is_some());

        // Test finding affected tests
        let changed_files = vec![src_dir.join("utils.rs")];
        let test_targets = analyzer.find_affected_tests(&changed_files).unwrap();
        assert!(!test_targets.is_empty());
    }
}