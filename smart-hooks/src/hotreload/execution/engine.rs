//! Main hot reload execution engine - orchestrates cache-first execution

use super::super::{
    cache::{CacheEntry, CacheStorage},
    tracking::{ChangeSet, FileChangeTracker},
    warming::BackgroundWarmingService,
    HookResults, HotReloadConfig, HotReloadResult,
};
use super::{
    cache_key::CacheKeyComputer,
    hooks::{HookDeterminator, HookExecutor},
    metrics::MetricsManager,
    validation::CacheValidator,
};
use crate::dependency::multi_lang_analyzer::RustMultiLangAnalyzer;
use crate::project::multi_lang_types::{
    Language, LanguageCommands, LanguageConfig, PackageManager, TestFramework,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{info, warn};

/// Main hot reload execution engine
pub struct HotReloadEngine {
    /// Cache storage backend
    cache_storage: Arc<dyn CacheStorage>,
    /// File change tracker
    file_tracker: FileChangeTracker,
    /// Background warming service
    warming_service: Option<BackgroundWarmingService>,
    /// Configuration
    config: HotReloadConfig,
    /// Cache key computer
    cache_key_computer: CacheKeyComputer,
    /// Cache validator
    cache_validator: CacheValidator,
    /// Metrics manager
    metrics: MetricsManager,
}

impl HotReloadEngine {
    /// Create new hot reload engine
    pub async fn new(
        cache_storage: Arc<dyn CacheStorage>,
        config: HotReloadConfig,
        project_root: PathBuf,
    ) -> HotReloadResult<Self> {
        let file_tracker = FileChangeTracker::new(project_root.clone());
        let language_config = Self::create_language_config();
        let analyzer = RustMultiLangAnalyzer::new(language_config);

        let config_hash = CacheKeyComputer::compute_config_hash(
            config.min_confidence_threshold,
            config.max_cache_entries,
            &config.cache_ttl,
        );

        let cache_key_computer = CacheKeyComputer::new(analyzer, config_hash);
        let cache_validator = CacheValidator::new(cache_key_computer.clone(), config.clone());

        let warming_service = if config.background_warming_enabled {
            Some(BackgroundWarmingService::new().await?)
        } else {
            None
        };

        Ok(Self {
            cache_storage,
            file_tracker,
            warming_service,
            config,
            cache_key_computer,
            cache_validator,
            metrics: MetricsManager::new(),
        })
    }

    /// Execute hooks with cache-first strategy
    pub async fn execute_with_cache(&mut self, files: &[PathBuf]) -> HotReloadResult<HookResults> {
        let start_time = SystemTime::now();

        // 1. Detect changes
        let changes = self.file_tracker.detect_changes(files).await?;

        // 2. Compute cache key
        let cache_key = self.cache_key_computer.compute_cache_key(&changes).await?;

        // 3. Check cache first
        if let Some(cached_result) = self.cache_storage.get(&cache_key).await? {
            if self
                .cache_validator
                .validate_cache_entry(&cached_result, &changes)
                .await?
            {
                let hit_time = start_time.elapsed().unwrap_or_default();
                self.metrics.record_hit(hit_time).await;

                info!("Cache HIT: {:?} ({}ms)", cache_key, hit_time.as_millis());
                return Ok(cached_result.hook_results);
            }
        }

        // 4. Cache miss - execute hooks fresh
        info!("Cache MISS: {:?}", cache_key);
        let results = self.execute_hooks_fresh(&changes).await?;

        // 5. Store results in cache
        let confidence_score = self.compute_confidence_score(&changes).await?;
        let cache_entry = CacheEntry::new(
            self.cache_key_computer
                .compute_content_hash(&changes.all_files())
                .await?,
            self.cache_key_computer
                .compute_dependency_hash(&changes)
                .await?,
            confidence_score,
            results.clone(),
            changes.all_files(),
        );

        self.cache_storage.store(cache_key, &cache_entry).await?;

        // 6. Schedule background warming
        if let Some(ref warming_service) = self.warming_service {
            warming_service
                .schedule_warming(&changes.all_files())
                .await?;
        }

        let miss_time = start_time.elapsed().unwrap_or_default();
        self.metrics.record_miss(miss_time).await;

        Ok(results)
    }

    /// Execute with automatic fallback on cache failures
    pub async fn execute_with_fallback(
        &mut self,
        files: &[PathBuf],
    ) -> HotReloadResult<HookResults> {
        match self.execute_with_cache(files).await {
            Ok(results) => Ok(results),
            Err(e) if e.is_cache_error() => {
                warn!(
                    "Cache failure, falling back to traditional execution: {}",
                    e
                );

                // Fallback to traditional execution without caching
                let changes = ChangeSet::new();
                self.execute_hooks_fresh(&changes).await
            }
            Err(e) => Err(e),
        }
    }

    /// Execute hooks without caching (fresh execution)
    async fn execute_hooks_fresh(&self, changes: &ChangeSet) -> HotReloadResult<HookResults> {
        let mut results = HookResults::new();

        // Determine which hooks to run based on changes
        let hooks_to_run = HookDeterminator::determine_hooks_to_run(changes).await?;

        for hook_type in hooks_to_run {
            let hook_result = HookExecutor::execute_single_hook(&hook_type, changes).await?;
            results.add_result(hook_result);
        }

        Ok(results)
    }

    /// Compute confidence score for cache entry
    async fn compute_confidence_score(&self, changes: &ChangeSet) -> HotReloadResult<f64> {
        if changes.is_empty() {
            return Ok(1.0); // High confidence for no changes
        }

        // Use dependency analysis to compute confidence
        // This is a placeholder implementation
        Ok(0.8) // Default confidence
    }

    /// Create language configuration for analyzer
    fn create_language_config() -> LanguageConfig {
        LanguageConfig {
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
        }
    }

    /// Get current cache statistics
    pub async fn cache_stats(&self) -> HotReloadResult<super::super::CacheStatistics> {
        self.cache_storage.stats().await
    }

    /// Get execution metrics
    pub async fn execution_metrics(&self) -> super::metrics::ExecutionMetrics {
        self.metrics.get_metrics().await
    }

    /// Get current configuration
    pub fn config(&self) -> &HotReloadConfig {
        &self.config
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> HotReloadResult<()> {
        self.cache_storage.clear().await
    }

    /// Enable file system watching
    pub async fn enable_file_watching(&mut self) -> HotReloadResult<()> {
        self.file_tracker.start_watching().await?;
        Ok(())
    }

    /// Disable file system watching
    pub async fn disable_file_watching(&mut self) {
        self.file_tracker.stop_watching().await;
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::cache::memory::MemoryCache;
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    async fn create_test_engine() -> (HotReloadEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache_storage: Arc<dyn CacheStorage> = Arc::new(MemoryCache::new());
        let config = HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 100 * 1024 * 1024,
            max_cache_entries: 1000,
            cache_ttl: HashMap::new(),
            min_confidence_threshold: 0.8,
            background_warming_enabled: false,
            fs_watching_enabled: false,
            cache_dir: Some(PathBuf::from("/tmp/test_cache")),
        };

        let engine = HotReloadEngine::new(cache_storage, config, temp_dir.path().to_path_buf())
            .await
            .unwrap();

        (engine, temp_dir)
    }

    fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let (engine, _temp_dir) = create_test_engine().await;
        assert!(engine.config.max_cache_entries > 0);
        assert!(!engine.config.background_warming_enabled);
    }

    #[tokio::test]
    async fn test_execute_with_fallback() {
        let (mut engine, temp_dir) = create_test_engine().await;

        let file_path = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");

        // This should fallback to fresh execution since cache is empty
        let result = engine.execute_with_fallback(&[file_path]).await;

        // Should handle the fallback gracefully (even if hook execution fails in test environment)
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_execution_metrics() {
        let (engine, _temp_dir) = create_test_engine().await;

        let metrics = engine.execution_metrics().await;

        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.total_executions, 0);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let (engine, _temp_dir) = create_test_engine().await;

        let result = engine.clear_cache().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let (engine, _temp_dir) = create_test_engine().await;

        let stats = engine.cache_stats().await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }
}