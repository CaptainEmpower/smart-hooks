//! Cache validation logic for hot reload system

use super::super::{cache::CacheEntry, tracking::ChangeSet, HotReloadConfig, HotReloadResult};
use super::cache_key::CacheKeyComputer;
use std::time::Duration;

/// Validates cache entries for correctness and freshness
pub struct CacheValidator {
    computer: CacheKeyComputer,
    config: HotReloadConfig,
}

impl CacheValidator {
    /// Create new cache validator
    pub fn new(computer: CacheKeyComputer, config: HotReloadConfig) -> Self {
        Self { computer, config }
    }

    /// Validate cache entry is still valid
    pub async fn validate_cache_entry(
        &self,
        entry: &CacheEntry,
        changes: &ChangeSet,
    ) -> HotReloadResult<bool> {
        // Check if entry has expired
        if let Some(ttl) = self.get_cache_ttl_for_entry(entry) {
            if entry.is_expired(ttl) {
                return Ok(false);
            }
        }

        // Check confidence threshold
        if entry.confidence_score < self.config.min_confidence_threshold {
            return Ok(false);
        }

        // Verify content hashes still match
        let current_content_hash = self
            .computer
            .compute_content_hash(&changes.all_files())
            .await?;
        if current_content_hash != entry.content_hash {
            return Ok(false);
        }

        // Verify dependency tree hasn't changed significantly
        let current_dep_hash = self.computer.compute_dependency_hash(changes).await?;
        if current_dep_hash != entry.dependency_hash {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get cache TTL for entry based on hook types
    pub fn get_cache_ttl_for_entry(&self, entry: &CacheEntry) -> Option<Duration> {
        // Find the shortest TTL among all hook types in the entry
        let mut min_ttl = None;

        for result in &entry.hook_results.results {
            if let Some(ttl) = self.config.cache_ttl.get(&result.hook_type) {
                match min_ttl {
                    None => min_ttl = Some(*ttl),
                    Some(current_min) => {
                        if ttl < &current_min {
                            min_ttl = Some(*ttl);
                        }
                    }
                }
            }
        }

        min_ttl
    }

    /// Check if cache entry is expired
    pub fn is_entry_expired(&self, entry: &CacheEntry, ttl: Duration) -> bool {
        entry.is_expired(ttl)
    }

    /// Validate confidence score meets threshold
    pub fn is_confidence_valid(&self, entry: &CacheEntry) -> bool {
        entry.confidence_score >= self.config.min_confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{cache::CacheEntry, HookResult, HookResults, HookType};
    use super::*;
    use crate::dependency::multi_lang_analyzer::RustMultiLangAnalyzer;
    use crate::project::multi_lang_types::{
        Language, LanguageCommands, LanguageConfig, PackageManager, TestFramework,
    };
    use std::collections::HashMap;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use tempfile::NamedTempFile;

    fn create_test_validator() -> CacheValidator {
        let language_config = LanguageConfig {
            language: Language::Rust,
            source_dirs: vec![PathBuf::from("src")],
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
        let analyzer = RustMultiLangAnalyzer::new(language_config);

        let mut cache_ttl = HashMap::new();
        cache_ttl.insert(HookType::Test, Duration::from_secs(300));
        cache_ttl.insert(HookType::Lint, Duration::from_secs(600));

        let config = HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 100 * 1024 * 1024,
            max_cache_entries: 1000,
            cache_ttl,
            min_confidence_threshold: 0.8,
            background_warming_enabled: false,
            fs_watching_enabled: false,
            cache_dir: Some(PathBuf::from("/tmp/test_cache")),
        };

        let config_hash = CacheKeyComputer::compute_config_hash(
            config.min_confidence_threshold,
            config.max_cache_entries,
            &config.cache_ttl,
        );
        let computer = CacheKeyComputer::new(analyzer, config_hash);

        CacheValidator::new(computer, config)
    }

    fn create_test_cache_entry(confidence: f64) -> CacheEntry {
        let mut hook_results = HookResults::new();
        hook_results.add_result(HookResult {
            hook_type: HookType::Test,
            exit_code: 0,
            stdout: "test passed".to_string(),
            stderr: "".to_string(),
            execution_time: Duration::from_millis(100),
            timestamp: SystemTime::now(),
        });

        CacheEntry::new(
            "test_content_hash".to_string(),
            "test_dep_hash".to_string(),
            confidence,
            hook_results,
            vec![PathBuf::from("test.rs")],
        )
    }

    #[tokio::test]
    async fn test_confidence_validation() {
        let validator = create_test_validator();

        // High confidence - should pass
        let high_confidence_entry = create_test_cache_entry(0.9);
        assert!(validator.is_confidence_valid(&high_confidence_entry));

        // Low confidence - should fail
        let low_confidence_entry = create_test_cache_entry(0.5);
        assert!(!validator.is_confidence_valid(&low_confidence_entry));
    }

    #[test]
    fn test_ttl_calculation() {
        let validator = create_test_validator();
        let entry = create_test_cache_entry(0.9);

        let ttl = validator.get_cache_ttl_for_entry(&entry);
        assert!(ttl.is_some());
        // Should get Test TTL which is 300 seconds
        assert_eq!(ttl.unwrap(), Duration::from_secs(300));
    }

    #[test]
    fn test_expiration_check() {
        let validator = create_test_validator();
        let entry = create_test_cache_entry(0.9);

        // Fresh entry should not be expired
        assert!(!validator.is_entry_expired(&entry, Duration::from_secs(300)));

        // Create an old entry
        let mut old_entry = create_test_cache_entry(0.9);
        old_entry.created_at = SystemTime::now() - Duration::from_secs(1000);

        // Old entry should be expired
        assert!(validator.is_entry_expired(&old_entry, Duration::from_secs(300)));
    }

    #[tokio::test]
    async fn test_full_validation() {
        let validator = create_test_validator();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn test() {{}}").unwrap();

        let mut changes = ChangeSet::new();
        changes.add_changed_file(temp_file.path().to_path_buf());

        // Create entry with current hashes
        let content_hash = validator
            .computer
            .compute_content_hash(&changes.all_files())
            .await
            .unwrap();
        let dep_hash = validator
            .computer
            .compute_dependency_hash(&changes)
            .await
            .unwrap();

        let mut hook_results = HookResults::new();
        hook_results.add_result(HookResult {
            hook_type: HookType::Test,
            exit_code: 0,
            stdout: "test passed".to_string(),
            stderr: "".to_string(),
            execution_time: Duration::from_millis(100),
            timestamp: SystemTime::now(),
        });

        let valid_entry = CacheEntry::new(
            content_hash,
            dep_hash.clone(),
            0.9,
            hook_results,
            changes.all_files(),
        );

        // Should be valid
        let is_valid = validator
            .validate_cache_entry(&valid_entry, &changes)
            .await
            .unwrap();
        assert!(is_valid);

        // Test with wrong content hash
        let invalid_entry = CacheEntry::new(
            "wrong_hash".to_string(),
            dep_hash.clone(),
            0.9,
            valid_entry.hook_results.clone(),
            changes.all_files(),
        );

        let is_invalid = validator
            .validate_cache_entry(&invalid_entry, &changes)
            .await
            .unwrap();
        assert!(!is_invalid);
    }
}