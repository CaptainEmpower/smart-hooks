/// Main Rust dependency analyzer implementation
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::dependency::analyzer::DependencyAnalyzer;
use crate::dependency::graph::DependencyGraph;
use crate::dependency::rust_parser::RustDependencyParser;
use crate::dependency::test_resolver::RustTestResolver;
use crate::dependency::types::{Dependency, TestTarget};
use crate::project::RustProjectConfig;

/// Rust-specific implementation of dependency analysis
pub struct RustDependencyAnalyzer {
    #[allow(dead_code)]
    project_config: RustProjectConfig,
    parser: RustDependencyParser,
    test_resolver: RustTestResolver,
    cache: HashMap<PathBuf, Vec<Dependency>>,
}

impl RustDependencyAnalyzer {
    pub fn new(project_config: RustProjectConfig) -> Self {
        let parser = RustDependencyParser::new(project_config.clone());
        let test_resolver = RustTestResolver::new(project_config.clone());

        Self {
            project_config,
            parser,
            test_resolver,
            cache: HashMap::new(),
        }
    }

    /// Create analyzer by discovering the project from a path
    pub fn discover_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let project_config = RustProjectConfig::discover(path)?;
        Ok(Self::new(project_config))
    }
}

impl DependencyAnalyzer for RustDependencyAnalyzer {
    fn analyze_file_dependencies(&self, file: &Path) -> Result<Vec<Dependency>> {
        // Check cache first
        if let Some(cached) = self.cache.get(file) {
            return Ok(cached.clone());
        }

        let content = fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;

        let dependencies = self.parser.analyze_rust_imports(&content, file)?;

        // Note: In a real implementation, we'd want mutable cache access
        // For now, we'll skip caching to keep the interface simple

        Ok(dependencies)
    }

    fn find_affected_tests(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>> {
        self.test_resolver.find_affected_test_targets(changed_files)
    }

    fn build_dependency_graph(&self, project: &RustProjectConfig) -> Result<DependencyGraph> {
        let mut dependencies = HashMap::new();
        let mut dependents = HashMap::new();
        let mut test_targets = HashMap::new();

        // Analyze all source files
        let all_files = project.get_all_source_files()?;

        for file in &all_files {
            let file_deps = self.analyze_file_dependencies(file)?;

            for dep in &file_deps {
                // Add to dependents map (reverse lookup)
                dependents
                    .entry(dep.path.clone())
                    .or_insert_with(Vec::new)
                    .push(file.clone());
            }

            dependencies.insert(file.clone(), file_deps);
        }

        // Find test targets for each file
        for file in &all_files {
            let targets = self
                .test_resolver
                .find_affected_test_targets(std::slice::from_ref(file))?;
            if !targets.is_empty() {
                test_targets.insert(file.clone(), targets);
            }
        }

        Ok(DependencyGraph::new(dependencies, dependents, test_targets))
    }
}

// Re-import the context trait for error handling
use anyhow::Context;
