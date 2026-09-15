/// Core trait for dependency analysis
use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::dependency::graph::DependencyGraph;
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::RustProjectConfig;

/// Trait for dependency analysis across different languages and project types
pub trait DependencyAnalyzer: Send + Sync {
    /// Analyze dependencies for a single file
    fn analyze_file_dependencies(&self, file: &Path) -> Result<Vec<Dependency>>;

    /// Find all test targets that should be run when files change
    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>>;

    /// Build a complete dependency graph for the project
    fn build_dependency_graph(&self, project: &RustProjectConfig) -> Result<DependencyGraph>;
}
