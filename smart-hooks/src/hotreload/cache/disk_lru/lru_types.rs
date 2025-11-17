//! LRU cache tracking types and statistics
use crate::hotreload::cache::CacheKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// LRU tracking information for cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LruEntry {
    pub key: CacheKey,
    pub last_accessed: SystemTime,
    pub size_bytes: usize,
}

/// Internal LRU tracking state
#[derive(Debug)]
pub struct LruTracker {
    pub entries: HashMap<CacheKey, LruEntry>,
    pub access_order: Vec<CacheKey>, // LRU order (oldest first)
    pub current_size_bytes: u64,
}

impl LruTracker {
    /// Create new LRU tracker
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            current_size_bytes: 0,
        }
    }

    /// Add entry to tracking
    pub fn add_entry(&mut self, key: CacheKey, size_bytes: usize) {
        let entry = LruEntry {
            key: key.clone(),
            last_accessed: SystemTime::now(),
            size_bytes,
        };

        // Remove existing if present
        if let Some(old_entry) = self.entries.remove(&key) {
            self.current_size_bytes -= old_entry.size_bytes as u64;
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
        }

        // Add new entry
        self.entries.insert(key.clone(), entry);
        self.access_order.push(key);
        self.current_size_bytes += size_bytes as u64;
    }

    /// Update access order for existing entry
    pub fn update_access(&mut self, key: &CacheKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_accessed = SystemTime::now();

            // Move to end of access order (most recently used)
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                let key_owned = self.access_order.remove(pos);
                self.access_order.push(key_owned);
            }
        }
    }

    /// Remove entry from tracking
    pub fn remove_entry(&mut self, key: &CacheKey) -> Option<LruEntry> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size_bytes -= entry.size_bytes as u64;
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Get least recently used keys up to specified count
    pub fn get_lru_keys(&self, count: usize) -> Vec<CacheKey> {
        self.access_order.iter().take(count).cloned().collect()
    }

    /// Check if cache is over size limits
    pub fn is_over_limits(&self, max_size_bytes: u64, max_entries: usize) -> bool {
        self.current_size_bytes > max_size_bytes || self.entries.len() > max_entries
    }
}

/// Cache performance statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
}

impl CacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.hits as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }

    /// Calculate cache miss rate
    #[allow(dead_code)]
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    /// Record cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.total_requests += 1;
    }

    /// Record cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.total_requests += 1;
    }

    /// Record cache eviction
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_tracker_basic_operations() {
        let mut tracker = LruTracker::new();
        let key1 = CacheKey::new("key1".to_string());
        let key2 = CacheKey::new("key2".to_string());

        // Add entries
        tracker.add_entry(key1.clone(), 100);
        tracker.add_entry(key2.clone(), 200);

        assert_eq!(tracker.entries.len(), 2);
        assert_eq!(tracker.current_size_bytes, 300);
        assert_eq!(tracker.access_order, vec![key1.clone(), key2.clone()]);
    }

    #[test]
    fn test_lru_tracker_update_access() {
        let mut tracker = LruTracker::new();
        let key1 = CacheKey::new("key1".to_string());
        let key2 = CacheKey::new("key2".to_string());

        tracker.add_entry(key1.clone(), 100);
        tracker.add_entry(key2.clone(), 200);

        // Update access for key1 (should move to end)
        tracker.update_access(&key1);
        assert_eq!(tracker.access_order, vec![key2.clone(), key1.clone()]);
    }

    #[test]
    fn test_lru_tracker_remove_entry() {
        let mut tracker = LruTracker::new();
        let key1 = CacheKey::new("key1".to_string());
        let key2 = CacheKey::new("key2".to_string());

        tracker.add_entry(key1.clone(), 100);
        tracker.add_entry(key2.clone(), 200);

        let removed = tracker.remove_entry(&key1);
        assert!(removed.is_some());
        assert_eq!(tracker.entries.len(), 1);
        assert_eq!(tracker.current_size_bytes, 200);
        assert_eq!(tracker.access_order, vec![key2.clone()]);
    }

    #[test]
    fn test_lru_tracker_get_lru_keys() {
        let mut tracker = LruTracker::new();
        let keys = vec![
            CacheKey::new("key1".to_string()),
            CacheKey::new("key2".to_string()),
            CacheKey::new("key3".to_string()),
        ];

        for (i, key) in keys.iter().enumerate() {
            tracker.add_entry(key.clone(), (i + 1) * 100);
        }

        let lru_keys = tracker.get_lru_keys(2);
        assert_eq!(lru_keys.len(), 2);
        assert_eq!(lru_keys, vec![keys[0].clone(), keys[1].clone()]);
    }

    #[test]
    fn test_lru_tracker_over_limits() {
        let mut tracker = LruTracker::new();

        // Test size limit
        tracker.add_entry(CacheKey::new("key1".to_string()), 600);
        assert!(tracker.is_over_limits(500, 10));
        assert!(!tracker.is_over_limits(1000, 10));

        // Test entry count limit
        for i in 0..5 {
            tracker.add_entry(CacheKey::new(format!("key{}", i)), 10);
        }
        assert!(tracker.is_over_limits(1000, 3));
        assert!(!tracker.is_over_limits(1000, 10));
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();

        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.miss_rate(), 1.0);

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
        assert!((stats.miss_rate() - 1.0 / 3.0).abs() < f64::EPSILON);

        stats.record_eviction();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_lru_entry_creation() {
        let key = CacheKey::new("test_key".to_string());
        let entry = LruEntry {
            key: key.clone(),
            last_accessed: SystemTime::now(),
            size_bytes: 100,
        };

        assert_eq!(entry.key, key);
        assert_eq!(entry.size_bytes, 100);
    }
}