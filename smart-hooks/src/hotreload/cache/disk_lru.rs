//! Disk-based LRU cache implementation

use super::{storage::CacheStorage, CacheEntry, CacheKey, InvalidationPattern};
use crate::hotreload::{CacheStatistics, HotReloadError, HotReloadResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::RwLock;

/// LRU tracking information for cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LruEntry {
    key: CacheKey,
    last_accessed: SystemTime,
    size_bytes: usize,
}

/// Disk-based cache with LRU eviction policy
pub struct DiskLruCache {
    cache_dir: PathBuf,
    max_size_bytes: u64,
    max_entries: usize,
    lru_tracker: RwLock<LruTracker>,
    stats: RwLock<CacheStats>,
}

#[derive(Debug)]
struct LruTracker {
    entries: HashMap<CacheKey, LruEntry>,
    access_order: Vec<CacheKey>, // LRU order (oldest first)
    current_size_bytes: u64,
}

#[derive(Debug, Clone)]
struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
    total_requests: u64,
}

impl DiskLruCache {
    /// Create new disk LRU cache with specified limits
    pub async fn new(
        cache_dir: impl AsRef<Path>,
        max_size_bytes: u64,
        max_entries: usize,
    ) -> HotReloadResult<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).await?;

        let cache = Self {
            cache_dir,
            max_size_bytes,
            max_entries,
            lru_tracker: RwLock::new(LruTracker {
                entries: HashMap::new(),
                access_order: Vec::new(),
                current_size_bytes: 0,
            }),
            stats: RwLock::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                total_requests: 0,
            }),
        };

        // Load existing cache entries
        cache.load_existing_entries().await?;

        Ok(cache)
    }

    /// Load existing cache entries from disk
    async fn load_existing_entries(&self) -> HotReloadResult<()> {
        let mut entries = tokio::fs::read_dir(&self.cache_dir).await?;
        let mut lru_tracker = self.lru_tracker.write().await;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("cache") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let key = CacheKey::new(file_stem.to_string());

                    // Get file metadata for size and access time
                    if let Ok(metadata) = entry.metadata().await {
                        let size_bytes = metadata.len() as usize;
                        let last_accessed = metadata
                            .accessed()
                            .or_else(|_| metadata.modified())
                            .unwrap_or_else(|_| SystemTime::now());

                        let lru_entry = LruEntry {
                            key: key.clone(),
                            last_accessed,
                            size_bytes,
                        };

                        lru_tracker.entries.insert(key.clone(), lru_entry);
                        lru_tracker.access_order.push(key);
                        lru_tracker.current_size_bytes += size_bytes as u64;
                    }
                }
            }
        }

        // Sort access order by last accessed time
        {
            let entries_ref = &lru_tracker.entries;
            let mut order = lru_tracker.access_order.clone();
            order.sort_by(|a, b| {
                let time_a = entries_ref.get(a).map(|e| e.last_accessed);
                let time_b = entries_ref.get(b).map(|e| e.last_accessed);
                time_a.cmp(&time_b)
            });
            lru_tracker.access_order = order;
        }

        Ok(())
    }

    /// Get file path for cache key
    fn cache_file_path(&self, key: &CacheKey) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", key.as_str()))
    }

    /// Evict least recently used entries to make space
    async fn evict_if_needed(&self, new_entry_size: usize) -> HotReloadResult<()> {
        let mut lru_tracker = self.lru_tracker.write().await;
        let mut stats = self.stats.write().await;

        // Check if we need to evict based on size
        let target_size = self.max_size_bytes.saturating_sub(new_entry_size as u64);

        while lru_tracker.current_size_bytes > target_size
            || lru_tracker.entries.len() >= self.max_entries
        {
            if lru_tracker.access_order.is_empty() {
                break;
            }

            // Remove oldest entry
            let oldest_key = lru_tracker.access_order.remove(0);
            if let Some(entry) = lru_tracker.entries.remove(&oldest_key) {
                // Delete file from disk
                let file_path = self.cache_file_path(&oldest_key);
                if let Err(e) = fs::remove_file(&file_path).await {
                    tracing::warn!("Failed to remove cache file {:?}: {}", file_path, e);
                }

                lru_tracker.current_size_bytes = lru_tracker
                    .current_size_bytes
                    .saturating_sub(entry.size_bytes as u64);

                stats.evictions += 1;

                tracing::debug!("Evicted cache entry: {}", oldest_key);
            }
        }

        Ok(())
    }

    /// Update LRU tracking for accessed entry
    async fn update_lru(&self, key: &CacheKey) {
        let mut lru_tracker = self.lru_tracker.write().await;

        // Update access time
        if let Some(entry) = lru_tracker.entries.get_mut(key) {
            entry.last_accessed = SystemTime::now();
        }

        // Move to end of access order (most recently used)
        if let Some(pos) = lru_tracker.access_order.iter().position(|k| k == key) {
            let key = lru_tracker.access_order.remove(pos);
            lru_tracker.access_order.push(key);
        }
    }
}

#[async_trait]
impl CacheStorage for DiskLruCache {
    async fn get(&self, key: &CacheKey) -> HotReloadResult<Option<CacheEntry>> {
        let file_path = self.cache_file_path(key);

        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        match fs::read(&file_path).await {
            Ok(data) => {
                match bincode::deserialize::<CacheEntry>(&data) {
                    Ok(mut entry) => {
                        // Update access tracking
                        self.update_lru(key).await;
                        entry.mark_accessed();

                        // Update cache entry on disk with new access time
                        let serialized = bincode::serialize(&entry)
                            .map_err(|e| HotReloadError::Serialization(e))?;

                        if let Err(e) = fs::write(&file_path, &serialized).await {
                            tracing::warn!("Failed to update cache entry access time: {}", e);
                        }

                        let mut stats = self.stats.write().await;
                        stats.hits += 1;

                        Ok(Some(entry))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cache entry {}: {}", key, e);
                        // Remove corrupted cache file
                        let _ = fs::remove_file(&file_path).await;

                        let mut stats = self.stats.write().await;
                        stats.misses += 1;

                        Ok(None)
                    }
                }
            }
            Err(_) => {
                let mut stats = self.stats.write().await;
                stats.misses += 1;
                Ok(None)
            }
        }
    }

    async fn store(&self, key: CacheKey, entry: &CacheEntry) -> HotReloadResult<()> {
        let serialized = bincode::serialize(entry).map_err(|e| HotReloadError::Serialization(e))?;

        // Evict entries if needed before storing
        self.evict_if_needed(serialized.len()).await?;

        let file_path = self.cache_file_path(&key);
        fs::write(&file_path, &serialized)
            .await
            .map_err(|e| HotReloadError::Io(e))?;

        // Update LRU tracking
        {
            let mut lru_tracker = self.lru_tracker.write().await;

            let lru_entry = LruEntry {
                key: key.clone(),
                last_accessed: SystemTime::now(),
                size_bytes: serialized.len(),
            };

            // Remove if already exists (update case)
            if let Some(old_entry) = lru_tracker.entries.get(&key) {
                lru_tracker.current_size_bytes = lru_tracker
                    .current_size_bytes
                    .saturating_sub(old_entry.size_bytes as u64);

                if let Some(pos) = lru_tracker.access_order.iter().position(|k| k == &key) {
                    lru_tracker.access_order.remove(pos);
                }
            }

            lru_tracker.entries.insert(key.clone(), lru_entry);
            lru_tracker.access_order.push(key);
            lru_tracker.current_size_bytes += serialized.len() as u64;
        }

        Ok(())
    }

    async fn remove(&self, key: &CacheKey) -> HotReloadResult<bool> {
        let file_path = self.cache_file_path(key);

        let removed = fs::remove_file(&file_path).await.is_ok();

        if removed {
            let mut lru_tracker = self.lru_tracker.write().await;

            if let Some(entry) = lru_tracker.entries.remove(key) {
                lru_tracker.current_size_bytes = lru_tracker
                    .current_size_bytes
                    .saturating_sub(entry.size_bytes as u64);

                if let Some(pos) = lru_tracker.access_order.iter().position(|k| k == key) {
                    lru_tracker.access_order.remove(pos);
                }
            }
        }

        Ok(removed)
    }

    async fn invalidate_pattern(&self, pattern: &InvalidationPattern) -> HotReloadResult<usize> {
        let keys = self.keys().await?;
        let mut removed_count = 0;

        for key in keys {
            let should_remove = match pattern {
                InvalidationPattern::All => true,
                InvalidationPattern::OlderThan(duration) => {
                    // Check if entry is older than duration
                    if let Ok(Some(entry)) = self.get(&key).await {
                        entry.created_at.elapsed().unwrap_or_default() > *duration
                    } else {
                        false
                    }
                }
                InvalidationPattern::LowConfidence(threshold) => {
                    // Check if confidence is below threshold
                    if let Ok(Some(entry)) = self.get(&key).await {
                        entry.confidence_score < *threshold
                    } else {
                        false
                    }
                }
                InvalidationPattern::Files(files) => {
                    // Check if entry involves any of the specified files
                    if let Ok(Some(entry)) = self.get(&key).await {
                        entry.cached_files.iter().any(|f| files.contains(f))
                    } else {
                        false
                    }
                }
                InvalidationPattern::Glob(glob_pattern) => {
                    // Simple glob matching (can be enhanced with regex)
                    key.as_str().contains(glob_pattern)
                }
            };

            if should_remove && self.remove(&key).await? {
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    async fn keys(&self) -> HotReloadResult<Vec<CacheKey>> {
        let lru_tracker = self.lru_tracker.read().await;
        Ok(lru_tracker.entries.keys().cloned().collect())
    }

    async fn stats(&self) -> HotReloadResult<CacheStatistics> {
        let stats = self.stats.read().await;
        let lru_tracker = self.lru_tracker.read().await;

        let hit_rate = if stats.total_requests > 0 {
            stats.hits as f64 / stats.total_requests as f64
        } else {
            0.0
        };

        Ok(CacheStatistics {
            hit_rate,
            total_requests: stats.total_requests,
            cache_hits: stats.hits,
            cache_misses: stats.misses,
            average_hit_time: Duration::from_millis(50), // Approximate
            average_miss_time: Duration::from_millis(500), // Approximate
            cache_size_bytes: lru_tracker.current_size_bytes,
            cached_entries: lru_tracker.entries.len(),
            warming_efficiency: 0.8, // TODO: Implement proper tracking
            invalidation_rate: 0.1,  // TODO: Implement proper tracking
        })
    }

    async fn cleanup_expired(&self, ttl: Duration) -> HotReloadResult<usize> {
        self.invalidate_pattern(&InvalidationPattern::OlderThan(ttl))
            .await
    }

    async fn clear(&self) -> HotReloadResult<()> {
        self.invalidate_pattern(&InvalidationPattern::All).await?;
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
        let retrieved = cache.get(&key).await.unwrap().unwrap();
        assert_eq!(retrieved.content_hash, entry.content_hash);
        assert_eq!(retrieved.dependency_hash, entry.dependency_hash);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let (cache, _temp_dir) = create_test_cache().await;

        let key = CacheKey::new("nonexistent_key".to_string());
        let result = cache.get(&key).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_removal() {
        let (cache, _temp_dir) = create_test_cache().await;

        let key = CacheKey::new("test_key".to_string());
        let entry = create_test_entry();

        // Store and then remove
        cache.store(key.clone(), &entry).await.unwrap();
        let removed = cache.remove(&key).await.unwrap();

        assert!(removed);

        // Verify it's gone
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        // Create small cache that will force eviction
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskLruCache::new(temp_dir.path(), 1000, 2).await.unwrap(); // Very small limits

        // Add three entries (should evict first one)
        for i in 0..3 {
            let key = CacheKey::new(format!("key_{}", i));
            let entry = create_test_entry();
            cache.store(key, &entry).await.unwrap();
        }

        // First entry should be evicted
        let first_key = CacheKey::new("key_0".to_string());
        let result = cache.get(&first_key).await.unwrap();
        assert!(result.is_none());

        // Last entry should still exist
        let last_key = CacheKey::new("key_2".to_string());
        let result = cache.get(&last_key).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_cache_invalidation_by_pattern() {
        let (cache, _temp_dir) = create_test_cache().await;

        // Store multiple entries
        for i in 0..5 {
            let key = CacheKey::new(format!("test_key_{}", i));
            let entry = create_test_entry();
            cache.store(key, &entry).await.unwrap();
        }

        // Invalidate entries matching pattern
        let removed = cache
            .invalidate_pattern(&InvalidationPattern::Glob("test_key_1".to_string()))
            .await
            .unwrap();
        assert_eq!(removed, 1);

        // Verify specific entry is gone
        let key = CacheKey::new("test_key_1".to_string());
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());

        // Verify others still exist
        let key = CacheKey::new("test_key_0".to_string());
        let result = cache.get(&key).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let (cache, _temp_dir) = create_test_cache().await;

        // Store multiple entries
        for i in 0..3 {
            let key = CacheKey::new(format!("key_{}", i));
            let entry = create_test_entry();
            cache.store(key, &entry).await.unwrap();
        }

        // Clear all entries
        cache.clear().await.unwrap();

        // Verify all entries are gone
        for i in 0..3 {
            let key = CacheKey::new(format!("key_{}", i));
            let result = cache.get(&key).await.unwrap();
            assert!(result.is_none());
        }
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let (cache, _temp_dir) = create_test_cache().await;

        let key = CacheKey::new("test_key".to_string());
        let entry = create_test_entry();

        // Store entry and access it
        cache.store(key.clone(), &entry).await.unwrap();
        cache.get(&key).await.unwrap();

        // Try to get non-existent entry
        let missing_key = CacheKey::new("missing".to_string());
        cache.get(&missing_key).await.unwrap();

        // Check statistics
        let stats = cache.stats().await.unwrap();
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
        assert!(stats.total_requests > 0);
        assert!(stats.hit_rate > 0.0 && stats.hit_rate < 1.0);
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        // Create cache and store entry
        {
            let cache = DiskLruCache::new(&cache_dir, 1024 * 1024, 100)
                .await
                .unwrap();
            let key = CacheKey::new("persistent_key".to_string());
            let entry = create_test_entry();
            cache.store(key, &entry).await.unwrap();
        } // Cache dropped here

        // Create new cache instance and verify entry persists
        {
            let cache = DiskLruCache::new(&cache_dir, 1024 * 1024, 100)
                .await
                .unwrap();
            let key = CacheKey::new("persistent_key".to_string());
            let result = cache.get(&key).await.unwrap();
            assert!(result.is_some());
        }
    }

    #[tokio::test]
    async fn test_cache_size_tracking() {
        let (cache, _temp_dir) = create_test_cache().await;

        let initial_size = cache.size_bytes().await.unwrap();
        assert_eq!(initial_size, 0);

        // Add entry and check size increased
        let key = CacheKey::new("size_test".to_string());
        let entry = create_test_entry();
        cache.store(key.clone(), &entry).await.unwrap();

        let after_store_size = cache.size_bytes().await.unwrap();
        assert!(after_store_size > initial_size);

        // Remove entry and check size decreased
        cache.remove(&key).await.unwrap();
        let after_remove_size = cache.size_bytes().await.unwrap();
        assert_eq!(after_remove_size, initial_size);
    }

    #[tokio::test]
    async fn test_entry_count() {
        let (cache, _temp_dir) = create_test_cache().await;

        assert_eq!(cache.entry_count().await.unwrap(), 0);

        // Add entries
        for i in 0..5 {
            let key = CacheKey::new(format!("count_key_{}", i));
            let entry = create_test_entry();
            cache.store(key, &entry).await.unwrap();
        }

        assert_eq!(cache.entry_count().await.unwrap(), 5);
    }
}