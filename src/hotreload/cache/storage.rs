//! Cache storage trait and common utilities

use super::{CacheEntry, CacheKey, InvalidationPattern};
use crate::hotreload::{CacheStatistics, HotReloadResult};
use async_trait::async_trait;

/// Trait for cache storage implementations
#[async_trait]
pub trait CacheStorage: Send + Sync {
    /// Retrieve cache entry by key
    async fn get(&self, key: &CacheKey) -> HotReloadResult<Option<CacheEntry>>;

    /// Store cache entry with key
    async fn store(&self, key: CacheKey, entry: &CacheEntry) -> HotReloadResult<()>;

    /// Remove cache entry by key
    async fn remove(&self, key: &CacheKey) -> HotReloadResult<bool>;

    /// Invalidate cache entries matching pattern
    async fn invalidate_pattern(&self, pattern: &InvalidationPattern) -> HotReloadResult<usize>;

    /// Get list of all cache keys
    async fn keys(&self) -> HotReloadResult<Vec<CacheKey>>;

    /// Get cache statistics
    async fn stats(&self) -> HotReloadResult<CacheStatistics>;

    /// Clean up expired entries
    async fn cleanup_expired(&self, ttl: std::time::Duration) -> HotReloadResult<usize>;

    /// Clear all cache entries
    async fn clear(&self) -> HotReloadResult<()>;

    /// Get current cache size in bytes
    async fn size_bytes(&self) -> HotReloadResult<u64>;

    /// Get number of cache entries
    async fn entry_count(&self) -> HotReloadResult<usize>;
}