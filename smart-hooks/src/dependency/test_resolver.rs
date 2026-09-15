/// Test target resolution for Rust projects
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::dependency::types::{TestTarget, TestType};
use crate::project::{CrateInfo, RustProjectConfig};

/// Resolves test targets for changed files in Rust projects
pub struct RustTestResolver {
    project_config: RustProjectConfig,
}

impl RustTestResolver {
    pub fn new(project_config: RustProjectConfig) -> Self {
        Self { project_config }
    }

    /// Find all test files that might be affected by changes
    pub fn find_affected_test_targets(&self, changed_files: &[PathBuf]) -> Result<Vec<TestTarget>> {
        let mut test_targets = Vec::new();

        for file in changed_files {
            if let Some(crate_info) = self.project_config.crate_for_file(file) {
                // Add unit tests for this module
                let module_name = self.extract_module_name(file, crate_info)?;
                test_targets.push(TestTarget {
                    name: format!("{}::{}", crate_info.name, module_name),
                    test_type: TestType::Unit {
                        module: module_name.clone(),
                    },
                    command: vec![
                        "cargo".to_string(),
                        "test".to_string(),
                        "--lib".to_string(),
                        module_name,
                        "-p".to_string(),
                        crate_info.name.clone(),
                    ],
                    dependencies: vec![file.to_path_buf()],
                    confidence: 0.9,
                });

                // Add integration tests if this is a public module
                if self.is_public_interface(file)? {
                    for test_dir in &crate_info.test_config.integration_test_dirs {
                        if test_dir.exists() {
                            test_targets.extend(self.find_integration_tests(test_dir, file)?);
                        }
                    }
                }
            }
        }

        Ok(test_targets)
    }

    fn extract_module_name(&self, file: &Path, crate_info: &CrateInfo) -> Result<String> {
        // Canonicalize both paths to handle symlinks and path differences
        let file_canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        let crate_canonical = crate_info
            .path
            .canonicalize()
            .unwrap_or_else(|_| crate_info.path.clone());
        let src_dir = crate_canonical.join("src");

        let relative_path = file_canonical
            .strip_prefix(&src_dir)
            .context("File is not in src directory")?;

        let module_name = relative_path
            .with_extension("")
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("::");

        Ok(module_name)
    }

    fn is_public_interface(&self, file: &Path) -> Result<bool> {
        let content = fs::read_to_string(file)?;

        // Check for pub items
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("pub fn ")
                || line.starts_with("pub struct ")
                || line.starts_with("pub enum ")
                || line.starts_with("pub trait ")
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn find_integration_tests(
        &self,
        test_dir: &Path,
        changed_file: &Path,
    ) -> Result<Vec<TestTarget>> {
        let mut targets = Vec::new();

        for entry in fs::read_dir(test_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Check if this integration test might depend on the changed file
                let confidence = self.calculate_integration_test_confidence(&path, changed_file)?;

                if confidence > 0.3 {
                    targets.push(TestTarget {
                        name: path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        test_type: TestType::Integration {
                            test_file: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                        },
                        command: vec![
                            "cargo".to_string(),
                            "test".to_string(),
                            "--test".to_string(),
                            path.file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                        ],
                        dependencies: vec![path],
                        confidence,
                    });
                }
            }
        }

        Ok(targets)
    }

    fn calculate_integration_test_confidence(
        &self,
        test_file: &Path,
        changed_file: &Path,
    ) -> Result<f32> {
        let test_content = fs::read_to_string(test_file)?;

        // Simple heuristics for now - could be made more sophisticated
        let changed_module = changed_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        let contains_module_ref = test_content.contains(changed_module.as_ref());
        let contains_use_statement =
            test_content.contains("use ") && test_content.contains(changed_module.as_ref());

        match (contains_module_ref, contains_use_statement) {
            (true, true) => Ok(0.8),
            (true, false) => Ok(0.5),
            (false, true) => Ok(0.6),
            (false, false) => Ok(0.1),
        }
    }
}
