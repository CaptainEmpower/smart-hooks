/// Dependency graph for managing project dependencies and test targets
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::dependency::types::{Dependency, TestTarget};

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// All dependencies in the project
    _dependencies: HashMap<PathBuf, Vec<Dependency>>,
    /// Reverse lookup: which files depend on this file
    dependents: HashMap<PathBuf, Vec<PathBuf>>,
    /// Test targets mapped by their dependencies
    test_targets: HashMap<PathBuf, Vec<TestTarget>>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new(
        dependencies: HashMap<PathBuf, Vec<Dependency>>,
        dependents: HashMap<PathBuf, Vec<PathBuf>>,
        test_targets: HashMap<PathBuf, Vec<TestTarget>>,
    ) -> Self {
        Self {
            _dependencies: dependencies,
            dependents,
            test_targets,
        }
    }

    /// Get all files that depend on the given files (transitively)
    pub fn get_transitive_dependents(&self, files: &[PathBuf]) -> Vec<PathBuf> {
        let mut result = HashSet::new();
        let mut to_process: Vec<PathBuf> = files.to_vec();

        while let Some(file) = to_process.pop() {
            if let Some(deps) = self.dependents.get(&file) {
                for dep in deps {
                    if result.insert(dep.clone()) {
                        to_process.push(dep.clone());
                    }
                }
            }
        }

        result.into_iter().collect()
    }

    /// Get all test targets that should be run for the given files
    pub fn get_test_targets_for_files(&self, files: &[PathBuf]) -> Vec<TestTarget> {
        let mut targets = Vec::new();
        let affected_files = self.get_transitive_dependents(files);

        for file in files.iter().chain(affected_files.iter()) {
            if let Some(file_targets) = self.test_targets.get(file) {
                targets.extend(file_targets.clone());
            }
        }

        targets
    }
}