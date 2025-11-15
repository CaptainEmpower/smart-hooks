//! Cache key computation for hot reload caching system

use super::super::{cache::CacheKey, tracking::ChangeSet, HotReloadError, HotReloadResult};
use crate::dependency::multi_lang_analyzer::{LanguageDependencyAnalyzer, RustMultiLangAnalyzer};
use blake3::Hasher;
use std::path::PathBuf;
use std::sync::Arc;

/// Computes cache keys for file sets and dependency graphs
pub struct CacheKeyComputer {
    analyzer: Arc<RustMultiLangAnalyzer>,
    config_hash: String,
}

impl Clone for CacheKeyComputer {
    fn clone(&self) -> Self {
        Self {
            analyzer: Arc::clone(&self.analyzer),
            config_hash: self.config_hash.clone(),
        }
    }
}

impl CacheKeyComputer {
    /// Create new cache key computer
    pub fn new(analyzer: RustMultiLangAnalyzer, config_hash: String) -> Self {
        Self {
            analyzer: Arc::new(analyzer),
            config_hash,
        }
    }

    /// Compute deterministic cache key from files and dependencies
    pub async fn compute_cache_key(&self, changes: &ChangeSet) -> HotReloadResult<CacheKey> {
        let mut hasher = Hasher::new();

        // Sort files for deterministic hashing
        let mut all_files = changes.all_files();
        all_files.sort();

        for file in &all_files {
            // Hash file content using file tracker
            let content = tokio::fs::read(file).await.map_err(|e| {
                HotReloadError::Cache(format!("Failed to read file {}: {}", file.display(), e))
            })?;
            hasher.update(&content);

            // Hash file path for uniqueness
            hasher.update(file.to_string_lossy().as_bytes());
        }

        // Hash dependency information
        let dependency_hash = self.compute_dependency_hash(changes).await?;
        hasher.update(dependency_hash.as_bytes());

        // Hash configuration to invalidate on config changes
        hasher.update(self.config_hash.as_bytes());

        Ok(CacheKey::new(hasher.finalize().to_hex().to_string()))
    }

    /// Compute content hash for multiple files
    pub async fn compute_content_hash(&self, files: &[PathBuf]) -> HotReloadResult<String> {
        let mut hasher = Hasher::new();

        for file_path in files {
            let content = tokio::fs::read(file_path).await.map_err(|e| {
                HotReloadError::Cache(format!(
                    "Failed to read file {}: {}",
                    file_path.display(),
                    e
                ))
            })?;
            hasher.update(&content);
            hasher.update(file_path.to_string_lossy().as_bytes());
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Compute dependency tree hash
    pub async fn compute_dependency_hash(&self, changes: &ChangeSet) -> HotReloadResult<String> {
        let mut hasher = Hasher::new();

        // Analyze dependencies for changed files
        for file in &changes.all_files() {
            if let Ok(deps) = self.analyzer.analyze_file_dependencies(file) {
                for dep in deps {
                    hasher.update(dep.name.as_bytes());
                    hasher.update(dep.path.to_string_lossy().as_bytes());
                    hasher.update(&dep.weight.to_le_bytes());
                }
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Compute configuration hash from config values
    pub fn compute_config_hash(
        min_confidence_threshold: f64,
        max_cache_entries: usize,
        cache_ttl: &std::collections::HashMap<super::super::HookType, std::time::Duration>,
    ) -> String {
        let mut hasher = Hasher::new();

        // Hash relevant config values that affect execution
        hasher.update(&min_confidence_threshold.to_le_bytes());
        hasher.update(&max_cache_entries.to_le_bytes());

        // Hash cache TTL settings
        for (hook_type, ttl) in cache_ttl {
            hasher.update(format!("{:?}", hook_type).as_bytes());
            hasher.update(&ttl.as_secs().to_le_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::multi_lang_types::{
        Language, LanguageCommands, LanguageConfig, PackageManager, TestFramework,
    };
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_analyzer() -> RustMultiLangAnalyzer {
        let language_config = LanguageConfig {
            language: Language::Rust,
            source_dirs: vec![std::path::PathBuf::from("src")],
            file_patterns: vec!["*.rs".to_string()],
            package_manager: PackageManager::Cargo,
            test_framework: TestFramework::RustTest,
            commands: LanguageCommands {
                build_command: Some(vec!["cargo".to_string(), "build".to_string()]),
                test_command: vec!["cargo".to_string(), "test".to_string()],
                format_command: Some(vec!["cargo".to_string(), "fmt".to_string()]),
                lint_command: Some(vec!["cargo".to_string(), "clippy".to_string()]),
                install_command: Some(vec!["cargo".to_string(), "build".to_string()]),
            },
            metadata: HashMap::new(),
        };
        RustMultiLangAnalyzer::new(language_config)
    }

    #[tokio::test]
    async fn test_cache_key_deterministic() {
        let analyzer = create_test_analyzer();
        let computer = CacheKeyComputer::new(analyzer, "test_config".to_string());

        // Create test files
        let mut temp_file1 = NamedTempFile::new().unwrap();
        writeln!(temp_file1, "fn test1() {{}}").unwrap();

        let mut temp_file2 = NamedTempFile::new().unwrap();
        writeln!(temp_file2, "fn test2() {{}}").unwrap();

        let mut changes1 = ChangeSet::new();
        changes1.add_changed_file(temp_file1.path().to_path_buf());
        changes1.add_changed_file(temp_file2.path().to_path_buf());

        let mut changes2 = ChangeSet::new();
        changes2.add_changed_file(temp_file2.path().to_path_buf());
        changes2.add_changed_file(temp_file1.path().to_path_buf());

        let key1 = computer.compute_cache_key(&changes1).await.unwrap();
        let key2 = computer.compute_cache_key(&changes2).await.unwrap();

        // Same files in different order should produce same key
        assert_eq!(key1.as_str(), key2.as_str());
    }

    #[tokio::test]
    async fn test_content_hash() {
        let analyzer = create_test_analyzer();
        let computer = CacheKeyComputer::new(analyzer, "test_config".to_string());

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn test() {{}}").unwrap();

        let hash1 = computer
            .compute_content_hash(&[temp_file.path().to_path_buf()])
            .await
            .unwrap();

        let hash2 = computer
            .compute_content_hash(&[temp_file.path().to_path_buf()])
            .await
            .unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // BLAKE3 produces 32-byte = 64 hex char hash
    }

    #[test]
    fn test_config_hash() {
        use super::super::super::HookType;
        use std::time::Duration;

        let mut cache_ttl = HashMap::new();
        cache_ttl.insert(HookType::Test, Duration::from_secs(300));
        cache_ttl.insert(HookType::Lint, Duration::from_secs(600));

        let hash1 = CacheKeyComputer::compute_config_hash(0.8, 1000, &cache_ttl);
        let hash2 = CacheKeyComputer::compute_config_hash(0.8, 1000, &cache_ttl);
        let hash3 = CacheKeyComputer::compute_config_hash(0.9, 1000, &cache_ttl); // Different confidence

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // BLAKE3 hex hash
    }

    #[tokio::test]
    async fn test_dependency_hash() {
        let analyzer = create_test_analyzer();
        let computer = CacheKeyComputer::new(analyzer, "test_config".to_string());

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "use std::collections::HashMap;").unwrap();

        let mut changes = ChangeSet::new();
        changes.add_changed_file(temp_file.path().to_path_buf());

        let hash = computer.compute_dependency_hash(&changes).await.unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 hex hash
    }
}