/// Rust-specific parsing logic for dependency analysis
use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::dependency::types::{Dependency, DependencyType};
use crate::project::RustProjectConfig;

/// Parser for Rust source code dependencies
pub struct RustDependencyParser {
    project_config: RustProjectConfig,
}

impl RustDependencyParser {
    pub fn new(project_config: RustProjectConfig) -> Self {
        Self { project_config }
    }

    /// Analyze Rust imports and use statements
    pub fn analyze_rust_imports(&self, content: &str, file_path: &Path) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Parse use statements
            if line.starts_with("use ") {
                if let Some(dep) = self.parse_use_statement(line, file_path, line_num)? {
                    dependencies.push(dep);
                }
            }

            // Parse mod statements
            if line.starts_with("mod ") {
                if let Some(dep) = self.parse_mod_statement(line, file_path, line_num)? {
                    dependencies.push(dep);
                }
            }

            // Parse extern crate statements
            if line.starts_with("extern crate ") {
                if let Some(dep) = self.parse_extern_crate(line, file_path, line_num)? {
                    dependencies.push(dep);
                }
            }
        }

        // Look for function calls and other dependencies
        dependencies.extend(self.analyze_function_calls(content, file_path)?);

        Ok(dependencies)
    }

    fn parse_use_statement(
        &self,
        line: &str,
        file_path: &Path,
        _line_num: usize,
    ) -> Result<Option<Dependency>> {
        // Extract the module path from use statement
        let use_part = line.strip_prefix("use ").unwrap_or(line);
        let use_part = use_part.trim_end_matches(';').trim();

        // Skip std library imports for now
        if use_part.starts_with("std::") || use_part.starts_with("core::") {
            return Ok(None);
        }

        // Extract the module path (everything except the last component which might be a function/type)
        let module_path = if let Some(last_colon) = use_part.rfind("::") {
            &use_part[..last_colon]
        } else {
            use_part // Single component, treat as module
        };

        // Try to resolve to a file in the project
        if let Some(resolved_path) = self.resolve_module_path(module_path, file_path)? {
            return Ok(Some(Dependency {
                name: module_path.to_string(),
                dependency_type: DependencyType::ModuleUse,
                path: resolved_path,
                weight: 0.8, // High weight for explicit imports
            }));
        }

        Ok(None)
    }

    fn parse_mod_statement(
        &self,
        line: &str,
        file_path: &Path,
        _line_num: usize,
    ) -> Result<Option<Dependency>> {
        let mod_part = line.strip_prefix("mod ").unwrap_or(line);
        let mod_part = mod_part.trim_end_matches(';').trim();

        // Try to find the module file
        if let Some(mod_path) = self.find_module_file(mod_part, file_path)? {
            return Ok(Some(Dependency {
                name: mod_part.to_string(),
                dependency_type: DependencyType::ModuleUse,
                path: mod_path,
                weight: 0.9, // Very high weight for module declarations
            }));
        }

        Ok(None)
    }

    fn parse_extern_crate(
        &self,
        line: &str,
        _file_path: &Path,
        _line_num: usize,
    ) -> Result<Option<Dependency>> {
        let _crate_part = line.strip_prefix("extern crate ").unwrap_or(line);
        let _crate_part = _crate_part.trim_end_matches(';').trim();

        // For now, we don't track external crate dependencies as file dependencies
        // This could be extended to track Cargo.toml changes
        Ok(None)
    }

    fn analyze_function_calls(&self, content: &str, _file_path: &Path) -> Result<Vec<Dependency>> {
        // Basic function call analysis - could be enhanced with AST parsing
        let mut dependencies = Vec::new();

        // Look for common patterns like crate::module::function()
        for line in content.lines() {
            // This is a simplified pattern - would benefit from proper AST parsing
            if let Some(call_match) = self.extract_qualified_calls(line) {
                if let Some(resolved_path) = self.resolve_function_call_path(&call_match)? {
                    dependencies.push(Dependency {
                        name: call_match,
                        dependency_type: DependencyType::FunctionCall,
                        path: resolved_path,
                        weight: 0.5,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn extract_qualified_calls(&self, line: &str) -> Option<String> {
        // Look for patterns like crate::module::function or self::function
        // This is a simplified regex-like approach
        if let Some(start) = line.find("::") {
            let before_colon = &line[..start];
            if let Some(word_start) = before_colon.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            {
                let module_part = &before_colon[word_start + 1..];
                if !module_part.is_empty() {
                    return Some(format!("{}::", module_part));
                }
            }
        }
        None
    }

    fn resolve_module_path(
        &self,
        module_path: &str,
        current_file: &Path,
    ) -> Result<Option<PathBuf>> {
        // Try to resolve the module path to an actual file
        let parts: Vec<&str> = module_path.split("::").collect();

        if parts.is_empty() {
            return Ok(None);
        }

        // Handle crate-relative paths
        if parts[0] == "crate" || parts[0] == "super" || parts[0] == "self" {
            return self.resolve_relative_path(&parts, current_file);
        }

        // Handle absolute paths from crate root
        if let Some(crate_info) = self.project_config.crate_for_file(current_file) {
            let src_dir = crate_info.path.join("src");

            // For simple module names like "utils", try to resolve to utils.rs
            if parts.len() == 1 {
                let module_name = parts[0];

                // Try module_name.rs
                let rs_file = src_dir.join(format!("{}.rs", module_name));
                if rs_file.exists() {
                    return Ok(Some(rs_file));
                }

                // Try module_name/mod.rs
                let mod_file = src_dir.join(module_name).join("mod.rs");
                if mod_file.exists() {
                    return Ok(Some(mod_file));
                }
            } else {
                // Handle nested paths
                let mut path = src_dir;

                for part in &parts[0..parts.len().saturating_sub(1)] {
                    path = path.join(part);
                }

                // Try different file extensions
                for ext in &["rs", "mod.rs"] {
                    let candidate = if *ext == "mod.rs" {
                        path.join(ext)
                    } else {
                        path.with_extension(ext)
                    };

                    if candidate.exists() {
                        return Ok(Some(candidate));
                    }
                }
            }
        }

        Ok(None)
    }

    fn resolve_relative_path(
        &self,
        parts: &[&str],
        current_file: &Path,
    ) -> Result<Option<PathBuf>> {
        // Handle relative module resolution
        let mut target_dir = current_file.parent().unwrap_or(current_file).to_path_buf();

        // Handle the first part (crate, super, self)
        match parts[0] {
            "super" => {
                target_dir = target_dir.parent().unwrap_or(&target_dir).to_path_buf();
            }
            "self" => {
                // Stay in current directory
            }
            "crate" => {
                // Go to crate root
                if let Some(crate_info) = self.project_config.crate_for_file(current_file) {
                    target_dir = crate_info.path.join("src");
                }
            }
            _ => {}
        }

        // Navigate through the remaining parts
        let remaining_parts: Vec<&str> = parts.iter().skip(1).cloned().collect();
        if remaining_parts.is_empty() {
            return Ok(None);
        }

        // For the last part, try to find it as a file
        if remaining_parts.len() == 1 {
            let module_name = remaining_parts[0];

            // Try module_name.rs
            let rs_file = target_dir.join(format!("{}.rs", module_name));
            if rs_file.exists() {
                return Ok(Some(rs_file));
            }

            // Try module_name/mod.rs
            let mod_file = target_dir.join(module_name).join("mod.rs");
            if mod_file.exists() {
                return Ok(Some(mod_file));
            }
        } else {
            // Navigate through intermediate directories
            for part in &remaining_parts[0..remaining_parts.len() - 1] {
                target_dir = target_dir.join(part);
            }

            let final_part = remaining_parts.last().unwrap();

            // Try to find the final file
            for ext in &["rs", "mod.rs"] {
                let candidate = if *ext == "mod.rs" {
                    target_dir.join(final_part).join(ext)
                } else {
                    target_dir.join(format!("{}.{}", final_part, ext))
                };

                if candidate.exists() {
                    return Ok(Some(candidate));
                }
            }
        }

        Ok(None)
    }

    fn find_module_file(&self, module_name: &str, current_file: &Path) -> Result<Option<PathBuf>> {
        let current_dir = current_file.parent().unwrap_or(current_file);

        // Try module_name.rs
        let rs_file = current_dir.join(format!("{}.rs", module_name));
        if rs_file.exists() {
            return Ok(Some(rs_file));
        }

        // Try module_name/mod.rs
        let mod_file = current_dir.join(module_name).join("mod.rs");
        if mod_file.exists() {
            return Ok(Some(mod_file));
        }

        Ok(None)
    }

    fn resolve_function_call_path(&self, _call_pattern: &str) -> Result<Option<PathBuf>> {
        // This would need more sophisticated analysis
        // For now, return None to avoid false positives
        Ok(None)
    }
}
