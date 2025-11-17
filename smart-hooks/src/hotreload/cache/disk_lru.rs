//! Disk-based LRU cache implementation

use super::{storage::CacheStorage, CacheEntry, CacheKey, InvalidationPattern};
use crate::hotreload::{CacheStatistics, HotReloadError, HotReloadResult};
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;
use tokio::sync::RwLock;

mod disk_ops;
mod lru_types;

use disk_ops::DiskOpsManager;
use lru_types::{CacheStats, LruTracker};

/// Disk-based cache with LRU eviction policy
pub struct DiskLruCache {
    disk_ops: DiskOpsManager,
    max_size_bytes: u64,
    max_entries: usize,
    lru_tracker: RwLock<LruTracker>,
    stats: RwLock<CacheStats>,
}

impl DiskLruCache {
    /// Create new disk LRU cache with specified limits
    pub async fn new(
        cache_dir: impl AsRef<Path>,
        max_size_bytes: u64,
        max_entries: usize,
    ) -> HotReloadResult<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        let disk_ops = DiskOpsManager::new(cache_dir);

        // Ensure cache directory exists
        disk_ops.ensure_cache_dir().await?;

        let cache = Self {
            disk_ops,
            max_size_bytes,
            max_entries,
            lru_tracker: RwLock::new(LruTracker::new()),
            stats: RwLock::new(CacheStats::default()),
        };

        // Load existing cache entries
        cache.load_existing_entries().await?;

        Ok(cache)
    }

    /// Load existing cache entries from disk
    async fn load_existing_entries(&self) -> HotReloadResult<()> {
        let cache_keys = self.disk_ops.list_cache_files().await?;
        let mut lru_tracker = self.lru_tracker.write().await;

        for key in cache_keys {
            if self.disk_ops.cache_file_exists(&key).await {
                if let Ok(size_bytes) = self.disk_ops.cache_file_size(&key).await {
                    lru_tracker.add_entry(key, size_bytes as usize);
                }
            }
        }

        Ok(())
    }

    /// Evict least recently used entries to make space
    async fn evict_if_needed(&self, new_entry_size: usize) -> HotReloadResult<()> {
        let mut lru_tracker = self.lru_tracker.write().await;
        let mut stats = self.stats.write().await;

        // Check if we need to evict based on size or entry count
        let target_size = self.max_size_bytes.saturating_sub(new_entry_size as u64);

        while lru_tracker.is_over_limits(target_size, self.max_entries) {
            let lru_keys = lru_tracker.get_lru_keys(1);
            if lru_keys.is_empty() {
                break; // No more entries to evict
            }

            let lru_key = &lru_keys[0];

            // Remove from tracker
            if let Some(_entry) = lru_tracker.remove_entry(lru_key) {
                stats.record_eviction();

                // Remove file from disk
                if let Err(e) = self.disk_ops.remove_cache_file(lru_key).await {
                    tracing::warn!("Failed to remove evicted cache file: {}", e);
                }

                tracing::debug!("Evicted cache entry: {}", lru_key);
            }
        }

        Ok(())
    }

    /// Update LRU tracking for key access
    async fn update_lru(&self, key: &CacheKey) {
        let mut lru_tracker = self.lru_tracker.write().await;
        lru_tracker.update_access(key);
    }
}

#[async_trait]
impl CacheStorage for DiskLruCache {
    async fn get(&self, key: &CacheKey) -> HotReloadResult<Option<CacheEntry>> {
        match self.disk_ops.read_cache_file(key).await {
            Ok(data) => {
                match bincode::deserialize::<CacheEntry>(&data) {
                    Ok(mut entry) => {
                        // Update access tracking
                        self.update_lru(key).await;
                        entry.mark_accessed();

                        // Update cache entry on disk with new access time
                        let serialized =
                            bincode::serialize(&entry).map_err(HotReloadError::Serialization)?;

                        if let Err(e) = self.disk_ops.write_cache_file(key, &serialized).await {
                            tracing::warn!("Failed to update cache entry access time: {}", e);
                        }

                        // Record hit
                        {
                            let mut stats = self.stats.write().await;
                            stats.record_hit();
                        }

                        Ok(Some(entry))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cache entry {}: {}", key, e);
                        // Remove corrupted cache file
                        let _ = self.disk_ops.remove_cache_file(key).await;

                        // Record miss
                        {
                            let mut stats = self.stats.write().await;
                            stats.record_miss();
                        }

                        Ok(None)
                    }
                }
            }
            Err(_) => {
                // File doesn't exist or can't be read - this is a miss
                {
                    let mut stats = self.stats.write().await;
                    stats.record_miss();
                }
                Ok(None)
            }
        }
    }

    async fn store(&self, key: CacheKey, entry: &CacheEntry) -> HotReloadResult<()> {
        let serialized = bincode::serialize(entry).map_err(HotReloadError::Serialization)?;

        // Check if we need to evict before adding new entry
        self.evict_if_needed(serialized.len()).await?;

        // Write to disk
        self.disk_ops.write_cache_file(&key, &serialized).await?;

        // Update LRU tracking
        {
            let mut lru_tracker = self.lru_tracker.write().await;
            lru_tracker.add_entry(key.clone(), serialized.len());
        }

        tracing::debug!("Stored cache entry: {} ({} bytes)", key, serialized.len());
        Ok(())
    }

    async fn remove(&self, key: &CacheKey) -> HotReloadResult<bool> {
        let existed = self.disk_ops.cache_file_exists(key).await;

        // Remove from disk
        self.disk_ops.remove_cache_file(key).await?;

        // Remove from LRU tracker
        {
            let mut lru_tracker = self.lru_tracker.write().await;
            lru_tracker.remove_entry(key);
        }

        Ok(existed)
    }

    async fn invalidate_pattern(&self, pattern: &InvalidationPattern) -> HotReloadResult<usize> {
        let cache_keys = self.disk_ops.list_cache_files().await?;
        let mut invalidated_count = 0;

        for key in cache_keys {
            if pattern.matches(&key) {
                // Remove from disk
                if let Err(e) = self.disk_ops.remove_cache_file(&key).await {
                    tracing::warn!("Failed to remove invalidated cache file {}: {}", key, e);
                    continue;
                }

                // Remove from LRU tracker
                {
                    let mut lru_tracker = self.lru_tracker.write().await;
                    lru_tracker.remove_entry(&key);
                }

                invalidated_count += 1;
                tracing::debug!("Invalidated cache entry: {}", key);
            }
        }

        Ok(invalidated_count)
    }

    async fn keys(&self) -> HotReloadResult<Vec<CacheKey>> {
        self.disk_ops.list_cache_files().await
    }

    async fn stats(&self) -> HotReloadResult<CacheStatistics> {
        let stats = self.stats.read().await;
        let lru_tracker = self.lru_tracker.read().await;

        Ok(CacheStatistics {
            hit_rate: stats.hit_rate(),
            total_requests: stats.total_requests,
            cache_hits: stats.hits,
            cache_misses: stats.misses,
            average_hit_time: Duration::from_millis(50), // Approximate for disk I/O
            average_miss_time: Duration::from_millis(200), // Approximate
            cache_size_bytes: lru_tracker.current_size_bytes,
            cached_entries: lru_tracker.entries.len(),
            warming_efficiency: 0.8, // TODO: Implement proper tracking
            invalidation_rate: 0.1,  // TODO: Implement proper tracking
        })
    }

    async fn cleanup_expired(&self, _ttl: Duration) -> HotReloadResult<usize> {
        // This implementation doesn't track individual entry ages,
        // so we'll implement a simple cleanup of orphaned files
        let valid_keys = {
            let lru_tracker = self.lru_tracker.read().await;
            lru_tracker.entries.keys().cloned().collect::<Vec<_>>()
        };

        self.disk_ops.cleanup_orphaned_files(&valid_keys).await
    }

    async fn clear(&self) -> HotReloadResult<()> {
        // Clear LRU tracker
        {
            let mut lru_tracker = self.lru_tracker.write().await;
            *lru_tracker = LruTracker::new();
        }

        // Remove all cache files
        let cache_keys = self.disk_ops.list_cache_files().await?;
        for key in cache_keys {
            if let Err(e) = self.disk_ops.remove_cache_file(&key).await {
                tracing::warn!("Failed to remove cache file during clear: {}", e);
            }
        }

        // Reset stats
        {
            let mut stats = self.stats.write().await;
            *stats = CacheStats::default();
        }

        tracing::info!("Cleared all cache entries");
        Ok(())
    }

    async fn size_bytes(&self) -> HotReloadResult<u64> {
        let lru_tracker = self.lru_tracker.read().await;
        Ok(lru_tracker.current_size_bytes)
    }

    async fn entry_count(&self) -> HotReloadResult<usize> {
        let lru_tracker = self.lru_tracker.read().await;
        Ok(lru_tracker.entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotreload::{HookResult, HookResults, HookType};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    async fn create_test_cache() -> (DiskLruCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskLruCache::new(temp_dir.path(), 1024 * 1024, 100)
            .await
            .unwrap();
        (cache, temp_dir)
    }

    fn create_test_entry() -> CacheEntry {
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
            "content_hash_123".to_string(),
            "dep_hash_456".to_string(),
            0.9,
            hook_results,
            vec![PathBuf::from("test.rs")],
        )
    }

    #[tokio::test]
    async fn test_cache_store_and_retrieve() {
        let (cache, _temp_dir) = create_test_cache().await;

        let key = CacheKey::new("test_key".to_string());
        let entry = create_test_entry();

        // Store entry
        cache.store(key.clone(), &entry).await.unwrap();

        // Retrieve entry
        let retrieved = cache.get(&key).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.content_hash(), entry.content_hash());
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let (cache, _temp_dir) = create_test_cache().await;

        let key = CacheKey::new("test_key".to_string());
        let entry = create_test_entry();

        // Store entry
        cache.store(key.clone(), &entry).await.unwrap();
        assert!(cache.get(&key).await.unwrap().is_some());

        // Remove entry
        let existed = cache.remove(&key).await.unwrap();
        assert!(existed);
        assert!(cache.get(&key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let (cache, _temp_dir) = create_test_cache().await;

        // Store some entries
        for i in 0..3 {
            let key = CacheKey::new(format!("key_{}", i));
            let entry = create_test_entry();
            cache.store(key, &entry).await.unwrap();
        }

        // Verify entries exist
        assert_eq!(cache.entry_count().await.unwrap(), 3);

        // Clear cache
        cache.clear().await.unwrap();

        // Verify cache is empty
        assert_eq!(cache.entry_count().await.unwrap(), 0);
        assert_eq!(cache.size_bytes().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let (cache, _temp_dir) = create_test_cache().await;

        let key = CacheKey::new("stats_test".to_string());
        let entry = create_test_entry();

        // Initial stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.hit_rate, 0.0);

        // Miss - key doesn't exist
        let _result = cache.get(&key).await.unwrap();
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.hit_rate, 0.0);

        // Store and hit
        cache.store(key.clone(), &entry).await.unwrap();
        let _result = cache.get(&key).await.unwrap();

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.hit_rate, 0.5); // 1 hit, 1 miss
    }
}