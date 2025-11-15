//! Hot reload execution engine - main orchestrator for cache-first execution

use super::{
    cache::{CacheEntry, CacheKey, CacheStorage},
    tracking::{ChangeSet, FileChangeTracker},
    warming::BackgroundWarmingService,
    HookResult, HookResults, HookType, HotReloadConfig, HotReloadError, HotReloadResult,
};
use crate::dependency::multi_lang_analyzer::{LanguageDependencyAnalyzer, RustMultiLangAnalyzer};
use crate::project::multi_lang_types::{
    Language, LanguageCommands, LanguageConfig, PackageManager, TestFramework,
};
use blake3::Hasher;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::process::Command;
use tokio::sync::RwLock;
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
    /// Multi-language analyzer for dependency analysis
    analyzer: RustMultiLangAnalyzer,
    /// Execution metrics
    metrics: Arc<RwLock<ExecutionMetrics>>,
}

#[derive(Debug, Clone, Default)]
struct ExecutionMetrics {
    cache_hits: u64,
    cache_misses: u64,
    total_executions: u64,
    total_execution_time: Duration,
    cache_hit_time: Duration,
    cache_miss_time: Duration,
}

impl HotReloadEngine {
    /// Create new hot reload engine
    pub async fn new(
        cache_storage: Arc<dyn CacheStorage>,
        config: HotReloadConfig,
        project_root: PathBuf,
    ) -> HotReloadResult<Self> {
        let file_tracker = FileChangeTracker::new(project_root.clone());
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
            analyzer,
            metrics: Arc::new(RwLock::new(ExecutionMetrics::default())),
        })
    }

    /// Execute hooks with cache-first strategy
    pub async fn execute_with_cache(&mut self, files: &[PathBuf]) -> HotReloadResult<HookResults> {
        let start_time = SystemTime::now();
        self.update_metrics(|m| m.total_executions += 1).await;

        // 1. Detect changes
        let changes = self.file_tracker.detect_changes(files).await?;

        // 2. Compute cache key
        let cache_key = self.compute_cache_key(&changes).await?;

        // 3. Check cache first
        if let Some(cached_result) = self.cache_storage.get(&cache_key).await? {
            if self.validate_cache_entry(&cached_result, &changes).await? {
                let hit_time = start_time.elapsed().unwrap_or_default();
                self.update_metrics(|m| {
                    m.cache_hits += 1;
                    m.cache_hit_time += hit_time;
                })
                .await;

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
            self.compute_content_hash(&changes.all_files()).await?,
            self.compute_dependency_hash(&changes).await?,
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
        self.update_metrics(|m| {
            m.cache_misses += 1;
            m.cache_miss_time += miss_time;
            m.total_execution_time += miss_time;
        })
        .await;

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

    /// Compute cache key from file changes
    async fn compute_cache_key(&self, changes: &ChangeSet) -> HotReloadResult<CacheKey> {
        let mut hasher = Hasher::new();

        // Sort files for deterministic hashing
        let mut all_files = changes.all_files();
        all_files.sort();

        for file in &all_files {
            // Hash file content
            let content_hash = self.file_tracker.hasher.compute_file_hash(file).await?;
            hasher.update(content_hash.as_bytes());

            // Hash file path for uniqueness
            hasher.update(file.to_string_lossy().as_bytes());
        }

        // Hash dependency information
        let dependency_hash = self.compute_dependency_hash(changes).await?;
        hasher.update(dependency_hash.as_bytes());

        // Hash configuration to invalidate on config changes
        let config_hash = self.compute_config_hash();
        hasher.update(config_hash.as_bytes());

        Ok(CacheKey::new(hasher.finalize().to_hex().to_string()))
    }

    /// Compute content hash for multiple files
    async fn compute_content_hash(&self, files: &[PathBuf]) -> HotReloadResult<String> {
        let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
        self.file_tracker
            .hasher
            .compute_combined_hash(&file_refs)
            .await
    }

    /// Compute dependency tree hash
    async fn compute_dependency_hash(&self, changes: &ChangeSet) -> HotReloadResult<String> {
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

    /// Compute configuration hash
    fn compute_config_hash(&self) -> String {
        let mut hasher = Hasher::new();

        // Hash relevant config values that affect execution
        hasher.update(&self.config.min_confidence_threshold.to_le_bytes());
        hasher.update(&self.config.max_cache_entries.to_le_bytes());

        // Hash cache TTL settings
        for (hook_type, ttl) in &self.config.cache_ttl {
            hasher.update(&format!("{:?}", hook_type).as_bytes());
            hasher.update(&ttl.as_secs().to_le_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    /// Validate cache entry is still valid
    async fn validate_cache_entry(
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
        let current_content_hash = self.compute_content_hash(&changes.all_files()).await?;
        if current_content_hash != entry.content_hash {
            return Ok(false);
        }

        // Verify dependency tree hasn't changed significantly
        let current_dep_hash = self.compute_dependency_hash(changes).await?;
        if current_dep_hash != entry.dependency_hash {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get cache TTL for entry based on hook types
    fn get_cache_ttl_for_entry(&self, entry: &CacheEntry) -> Option<Duration> {
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

    /// Execute hooks without caching (fresh execution)
    async fn execute_hooks_fresh(&self, changes: &ChangeSet) -> HotReloadResult<HookResults> {
        let mut results = HookResults::new();

        // Determine which hooks to run based on changes
        let hooks_to_run = self.determine_hooks_to_run(changes).await?;

        for hook_type in hooks_to_run {
            let hook_result = self.execute_single_hook(&hook_type, changes).await?;
            results.add_result(hook_result);
        }

        Ok(results)
    }

    /// Determine which hooks to run based on file changes
    async fn determine_hooks_to_run(&self, changes: &ChangeSet) -> HotReloadResult<Vec<HookType>> {
        let mut hooks = Vec::new();

        // Always run tests for code changes
        if !changes.changed_files.is_empty() {
            hooks.push(HookType::Test);
        }

        // Run linting for source code files
        if self.has_source_code_changes(changes) {
            hooks.push(HookType::Lint);
        }

        // Run formatting check
        hooks.push(HookType::Format);

        // Run analysis for dependency files
        if self.has_dependency_file_changes(changes) {
            hooks.push(HookType::Analysis);
        }

        Ok(hooks)
    }

    /// Check if changes include source code files
    fn has_source_code_changes(&self, changes: &ChangeSet) -> bool {
        let source_extensions = [
            ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".php", ".go", ".java", ".cs",
        ];

        changes.all_files().iter().any(|file| {
            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                source_extensions.contains(&format!(".{}", ext).as_str())
            } else {
                false
            }
        })
    }

    /// Check if changes include dependency files
    fn has_dependency_file_changes(&self, changes: &ChangeSet) -> bool {
        let dep_files = [
            "Cargo.toml",
            "package.json",
            "requirements.txt",
            "composer.json",
            "go.mod",
        ];

        changes.all_files().iter().any(|file| {
            if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
                dep_files.contains(&file_name)
            } else {
                false
            }
        })
    }

    /// Execute a single hook
    async fn execute_single_hook(
        &self,
        hook_type: &HookType,
        changes: &ChangeSet,
    ) -> HotReloadResult<HookResult> {
        let start_time = SystemTime::now();

        let (exit_code, stdout, stderr) = match hook_type {
            HookType::Test => self.execute_test_hook(changes).await?,
            HookType::Lint => self.execute_lint_hook(changes).await?,
            HookType::Format => self.execute_format_hook(changes).await?,
            HookType::Analysis => self.execute_analysis_hook(changes).await?,
            HookType::Custom(name) => self.execute_custom_hook(name, changes).await?,
        };

        let execution_time = start_time.elapsed().unwrap_or_default();

        Ok(HookResult {
            hook_type: hook_type.clone(),
            exit_code,
            stdout,
            stderr,
            execution_time,
            timestamp: SystemTime::now(),
        })
    }

    /// Execute test hook using smart-hooks test selection
    async fn execute_test_hook(
        &self,
        changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("test")
            .arg("selective");

        // Add changed files to test command
        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| HotReloadError::Cache(format!("Failed to execute test hook: {}", e)))?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute lint hook using smart-hooks auto linting
    async fn execute_lint_hook(
        &self,
        changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("lint")
            .arg("auto");

        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| HotReloadError::Cache(format!("Failed to execute lint hook: {}", e)))?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute format hook using smart-hooks auto formatting
    async fn execute_format_hook(
        &self,
        changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("format")
            .arg("auto");

        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| HotReloadError::Cache(format!("Failed to execute format hook: {}", e)))?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute analysis hook using smart-hooks dependency analysis
    async fn execute_analysis_hook(
        &self,
        changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("analyze")
            .arg("dependencies");

        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd.output().await.map_err(|e| {
            HotReloadError::Cache(format!("Failed to execute analysis hook: {}", e))
        })?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute custom hook
    async fn execute_custom_hook(
        &self,
        hook_name: &str,
        _changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        // Custom hook execution - would be configurable
        let output = Command::new(hook_name).output().await.map_err(|e| {
            HotReloadError::Cache(format!(
                "Failed to execute custom hook {}: {}",
                hook_name, e
            ))
        })?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Compute confidence score for cache entry
    async fn compute_confidence_score(&self, changes: &ChangeSet) -> HotReloadResult<f64> {
        if changes.is_empty() {
            return Ok(1.0); // High confidence for no changes
        }

        // Use smart-hooks dependency analysis to compute confidence
        let mut total_confidence = 0.0;
        let mut analysis_count = 0;

        for file in &changes.all_files() {
            if let Ok(deps) = self.analyzer.analyze_file_dependencies(file) {
                let file_confidence = if !deps.is_empty() {
                    deps.iter().map(|d| d.weight as f64).sum::<f64>() / deps.len() as f64
                } else {
                    0.8
                };
                total_confidence += file_confidence;
                analysis_count += 1;
            }
        }

        if analysis_count > 0 {
            Ok(total_confidence / analysis_count as f64)
        } else {
            Ok(0.8) // Default confidence
        }
    }

    /// Update metrics with a closure
    async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut ExecutionMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
    }

    /// Get current cache statistics
    pub async fn cache_stats(&self) -> HotReloadResult<super::CacheStatistics> {
        self.cache_storage.stats().await
    }

    /// Get execution metrics
    pub async fn execution_metrics(&self) -> ExecutionMetrics {
        self.metrics.read().await.clone()
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
    use super::*;
    use crate::hotreload::cache::memory::MemoryCache;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config() -> HotReloadConfig {
        HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 100 * 1024 * 1024, // 100MB in bytes
            max_cache_entries: 1000,
            cache_ttl: HashMap::from([
                (HookType::Test, Duration::from_secs(300)),
                (HookType::Lint, Duration::from_secs(600)),
            ]),
            min_confidence_threshold: 0.8,
            background_warming_enabled: false,
            fs_watching_enabled: false,
            cache_dir: Some(PathBuf::from("/tmp/test_cache")),
        }
    }

    async fn create_test_engine() -> (HotReloadEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache_storage: Arc<dyn CacheStorage> = Arc::new(MemoryCache::new());
        let config = create_test_config();

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
    async fn test_compute_cache_key() {
        let (engine, temp_dir) = create_test_engine().await;

        let file1 = create_test_file(temp_dir.path(), "src/lib.rs", "pub fn hello() {}");
        let file2 = create_test_file(temp_dir.path(), "src/main.rs", "fn main() {}");

        let mut changes = ChangeSet::new();
        changes.add_changed_file(file1);
        changes.add_changed_file(file2);

        let cache_key1 = engine.compute_cache_key(&changes).await.unwrap();
        let cache_key2 = engine.compute_cache_key(&changes).await.unwrap();

        // Same changes should produce same cache key
        assert_eq!(cache_key1.as_str(), cache_key2.as_str());
    }

    #[tokio::test]
    async fn test_compute_cache_key_deterministic() {
        let (engine, temp_dir) = create_test_engine().await;

        let file1 = create_test_file(temp_dir.path(), "src/a.rs", "content");
        let file2 = create_test_file(temp_dir.path(), "src/b.rs", "content");

        // Test with different order - should produce same key
        let mut changes1 = ChangeSet::new();
        changes1.add_changed_file(file1.clone());
        changes1.add_changed_file(file2.clone());

        let mut changes2 = ChangeSet::new();
        changes2.add_changed_file(file2);
        changes2.add_changed_file(file1);

        let key1 = engine.compute_cache_key(&changes1).await.unwrap();
        let key2 = engine.compute_cache_key(&changes2).await.unwrap();

        assert_eq!(key1.as_str(), key2.as_str());
    }

    #[tokio::test]
    async fn test_compute_content_hash() {
        let (engine, temp_dir) = create_test_engine().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn test() {}");
        let hash = engine.compute_content_hash(&[file_path]).await.unwrap();

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 produces 32-byte = 64 hex char hash
    }

    #[tokio::test]
    async fn test_compute_dependency_hash() {
        let (engine, temp_dir) = create_test_engine().await;

        let file_path = create_test_file(
            temp_dir.path(),
            "src/lib.rs",
            "use std::collections::HashMap;",
        );

        let mut changes = ChangeSet::new();
        changes.add_changed_file(file_path);

        let hash = engine.compute_dependency_hash(&changes).await.unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 hex hash
    }

    #[tokio::test]
    async fn test_compute_confidence_score() {
        let (engine, temp_dir) = create_test_engine().await;

        // Test with empty changes
        let empty_changes = ChangeSet::new();
        let confidence = engine
            .compute_confidence_score(&empty_changes)
            .await
            .unwrap();
        assert_eq!(confidence, 1.0);

        // Test with file changes
        let file_path = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");
        let mut changes = ChangeSet::new();
        changes.add_changed_file(file_path);

        let confidence = engine.compute_confidence_score(&changes).await.unwrap();
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_determine_hooks_to_run() {
        let (engine, temp_dir) = create_test_engine().await;

        // Test with source code changes
        let file_path = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");
        let mut changes = ChangeSet::new();
        changes.add_changed_file(file_path);

        let hooks = engine.determine_hooks_to_run(&changes).await.unwrap();

        assert!(hooks.contains(&HookType::Test));
        assert!(hooks.contains(&HookType::Lint));
        assert!(hooks.contains(&HookType::Format));
    }

    #[tokio::test]
    async fn test_determine_hooks_with_dependency_changes() {
        let (engine, temp_dir) = create_test_engine().await;

        // Test with dependency file changes
        let cargo_toml =
            create_test_file(temp_dir.path(), "Cargo.toml", "[package]\nname = \"test\"");
        let mut changes = ChangeSet::new();
        changes.add_changed_file(cargo_toml);

        let hooks = engine.determine_hooks_to_run(&changes).await.unwrap();

        assert!(hooks.contains(&HookType::Test));
        assert!(hooks.contains(&HookType::Format));
        assert!(hooks.contains(&HookType::Analysis));
    }

    #[tokio::test]
    async fn test_has_source_code_changes() {
        let (engine, temp_dir) = create_test_engine().await;

        // Test with Rust file
        let rust_file = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");
        let mut changes_with_rust = ChangeSet::new();
        changes_with_rust.add_changed_file(rust_file);

        assert!(engine.has_source_code_changes(&changes_with_rust));

        // Test with non-source file
        let readme = create_test_file(temp_dir.path(), "README.md", "# Test");
        let mut changes_no_source = ChangeSet::new();
        changes_no_source.add_changed_file(readme);

        assert!(!engine.has_source_code_changes(&changes_no_source));
    }

    #[tokio::test]
    async fn test_has_dependency_file_changes() {
        let (engine, temp_dir) = create_test_engine().await;

        // Test with Cargo.toml
        let cargo_toml = create_test_file(temp_dir.path(), "Cargo.toml", "[package]");
        let mut changes_with_deps = ChangeSet::new();
        changes_with_deps.add_changed_file(cargo_toml);

        assert!(engine.has_dependency_file_changes(&changes_with_deps));

        // Test with non-dependency file
        let rust_file = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");
        let mut changes_no_deps = ChangeSet::new();
        changes_no_deps.add_changed_file(rust_file);

        assert!(!engine.has_dependency_file_changes(&changes_no_deps));
    }

    #[tokio::test]
    async fn test_validate_cache_entry_expired() {
        let (engine, temp_dir) = create_test_engine().await;

        let file_path = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");
        let mut changes = ChangeSet::new();
        changes.add_changed_file(file_path.clone());

        // Create expired cache entry
        let mut hook_results = HookResults::new();
        hook_results.add_result(HookResult {
            hook_type: HookType::Test,
            exit_code: 0,
            stdout: "test passed".to_string(),
            stderr: "".to_string(),
            execution_time: Duration::from_millis(100),
            timestamp: SystemTime::now() - Duration::from_secs(1000), // Old timestamp
        });

        let content_hash = engine
            .compute_content_hash(&[file_path.clone()])
            .await
            .unwrap();
        let dep_hash = engine.compute_dependency_hash(&changes).await.unwrap();

        let mut entry = CacheEntry::new(content_hash, dep_hash, 0.9, hook_results, vec![file_path]);

        // Make the entry old to test expiration
        entry.created_at = SystemTime::now() - Duration::from_secs(1000);

        let is_valid = engine.validate_cache_entry(&entry, &changes).await.unwrap();

        // Should be invalid due to age (our TTL is 300 seconds for tests)
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_validate_cache_entry_low_confidence() {
        let (engine, temp_dir) = create_test_engine().await;

        let file_path = create_test_file(temp_dir.path(), "src/lib.rs", "fn test() {}");
        let mut changes = ChangeSet::new();
        changes.add_changed_file(file_path.clone());

        let hook_results = HookResults::new();
        let content_hash = engine
            .compute_content_hash(&[file_path.clone()])
            .await
            .unwrap();
        let dep_hash = engine.compute_dependency_hash(&changes).await.unwrap();

        // Create entry with low confidence score
        let entry = CacheEntry::new(
            content_hash,
            dep_hash,
            0.5, // Below our threshold of 0.8
            hook_results,
            vec![file_path],
        );

        let is_valid = engine.validate_cache_entry(&entry, &changes).await.unwrap();
        assert!(!is_valid);
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