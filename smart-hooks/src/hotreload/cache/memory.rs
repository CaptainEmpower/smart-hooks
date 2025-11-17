//! In-memory cache implementation for development and testing

use super::{storage::CacheStorage, CacheEntry, CacheKey, InvalidationPattern};
use crate::hotreload::{CacheStatistics, HotReloadResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Simple in-memory cache for development and testing
pub struct MemoryCache {
    storage: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone, Default)]
struct CacheStats {
    hits: u64,
    misses: u64,
    total_requests: u64,
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryCache {
    /// Create new in-memory cache
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
}

#[async_trait]
impl CacheStorage for MemoryCache {
    async fn get(&self, key: &CacheKey) -> HotReloadResult<Option<CacheEntry>> {
        let storage = self.storage.read().await;
        let mut stats = self.stats.write().await;

        stats.total_requests += 1;

        if let Some(mut entry) = storage.get(key).cloned() {
            entry.mark_accessed();
            stats.hits += 1;
            Ok(Some(entry))
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    async fn store(&self, key: CacheKey, entry: &CacheEntry) -> HotReloadResult<()> {
        let mut storage = self.storage.write().await;
        storage.insert(key, entry.clone());
        Ok(())
    }

    async fn remove(&self, key: &CacheKey) -> HotReloadResult<bool> {
        let mut storage = self.storage.write().await;
        Ok(storage.remove(key).is_some())
    }

    async fn invalidate_pattern(&self, pattern: &InvalidationPattern) -> HotReloadResult<usize> {
        let mut storage = self.storage.write().await;
        let keys_to_remove: Vec<CacheKey> = storage
            .iter()
            .filter_map(|(key, entry)| {
                let should_remove = match pattern {
                    InvalidationPattern::All => true,
                    InvalidationPattern::OlderThan(duration) => {
                        entry.created_at.elapsed().unwrap_or_default() > *duration
                    }
                    InvalidationPattern::LowConfidence(threshold) => {
                        entry.confidence_score < *threshold
                    }
                    InvalidationPattern::Files(files) => {
                        entry.cached_files.iter().any(|f| files.contains(f))
                    }
                    InvalidationPattern::Glob(glob_pattern) => key.as_str().contains(glob_pattern),
                };

                if should_remove {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        let removed_count = keys_to_remove.len();
        for key in keys_to_remove {
            storage.remove(&key);
        }

        Ok(removed_count)
    }

    async fn keys(&self) -> HotReloadResult<Vec<CacheKey>> {
        let storage = self.storage.read().await;
        Ok(storage.keys().cloned().collect())
    }

    async fn stats(&self) -> HotReloadResult<CacheStatistics> {
        let stats = self.stats.read().await;
        let storage = self.storage.read().await;

        let hit_rate = if stats.total_requests > 0 {
            stats.hits as f64 / stats.total_requests as f64
        } else {
            0.0
        };

        // Calculate total cache size
        let cache_size_bytes = storage.values().map(|entry| entry.size_bytes as u64).sum();

        Ok(CacheStatistics {
            hit_rate,
            total_requests: stats.total_requests,
            cache_hits: stats.hits,
            cache_misses: stats.misses,
            average_hit_time: Duration::from_micros(100), // Very fast for memory
            average_miss_time: Duration::from_millis(10), // Still fast
            cache_size_bytes,
            cached_entries: storage.len(),
            warming_efficiency: 0.9, // Memory cache has high efficiency
            invalidation_rate: 0.05, // Low invalidation rate
        })
    }

    async fn cleanup_expired(&self, ttl: Duration) -> HotReloadResult<usize> {
        self.invalidate_pattern(&InvalidationPattern::OlderThan(ttl))
            .await
    }

    async fn clear(&self) -> HotReloadResult<()> {
        let mut storage = self.storage.write().await;
        storage.clear();
        Ok(())
    }

    async fn size_bytes(&self) -> HotReloadResult<u64> {
        let storage = self.storage.read().await;
        let total_size = storage.values().map(|entry| entry.size_bytes as u64).sum();
        Ok(total_size)
    }

    async fn entry_count(&self) -> HotReloadResult<usize> {
        let storage = self.storage.read().await;
        Ok(storage.len())
    }
}