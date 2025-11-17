//! Cache storage implementations for hot reload system

use super::HookResults;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

pub mod disk_lru;
pub mod memory;
pub mod storage;

pub use disk_lru::DiskLruCache;
pub use memory::MemoryCache;
pub use storage::CacheStorage;

/// Unique identifier for cache entries based on content and dependency hashes
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheKey(pub String);

impl CacheKey {
    pub fn new(hash: String) -> Self {
        CacheKey(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Cache entry containing hook results and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// SHA-256 hash of combined file contents
    pub content_hash: String,
    /// Hash of dependency tree state
    pub dependency_hash: String,
    /// Smart-hooks confidence score for this cache entry
    pub confidence_score: f64,
    /// Cached hook execution results
    pub hook_results: HookResults,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last accessed timestamp for LRU
    pub accessed_at: SystemTime,
    /// Size in bytes for cache management
    pub size_bytes: usize,
    /// Files that were analyzed for this cache entry
    pub cached_files: Vec<PathBuf>,
    /// Cache metadata and statistics
    pub metadata: CacheMetadata,
}

impl CacheEntry {
    pub fn new(
        content_hash: String,
        dependency_hash: String,
        confidence_score: f64,
        hook_results: HookResults,
        cached_files: Vec<PathBuf>,
    ) -> Self {
        let now = SystemTime::now();
        let metadata = CacheMetadata::new();

        // Estimate size for cache management
        let size_bytes = Self::estimate_size(&hook_results, &cached_files);

        Self {
            content_hash,
            dependency_hash,
            confidence_score,
            hook_results,
            created_at: now,
            accessed_at: now,
            size_bytes,
            cached_files,
            metadata,
        }
    }

    /// Update access time for LRU tracking
    pub fn mark_accessed(&mut self) {
        self.accessed_at = SystemTime::now();
    }

    /// Check if cache entry has expired based on TTL
    pub fn is_expired(&self, ttl: std::time::Duration) -> bool {
        self.created_at.elapsed().unwrap_or_default() > ttl
    }

    /// Get content hash for this cache entry
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    /// Estimate memory/disk size of cache entry
    fn estimate_size(hook_results: &HookResults, cached_files: &[PathBuf]) -> usize {
        let mut size = 0;

        // Hook results size
        for result in &hook_results.results {
            size += result.stdout.len();
            size += result.stderr.len();
            size += 64; // Fixed overhead per result
        }

        // File paths size
        for file in cached_files {
            size += file.to_string_lossy().len();
        }

        size + 256 // Base metadata overhead
    }
}

impl From<CacheEntry> for HookResults {
    fn from(entry: CacheEntry) -> Self {
        entry.hook_results
    }
}

/// Metadata associated with cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Number of times this cache entry has been accessed
    pub access_count: u64,
    /// Hash of the smart-hooks configuration when this was cached
    pub config_hash: String,
    /// Version of smart-hooks that created this cache entry
    pub version: String,
    /// Additional tags for cache organization
    pub tags: HashMap<String, String>,
}

impl Default for CacheMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheMetadata {
    pub fn new() -> Self {
        Self {
            access_count: 0,
            config_hash: String::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            tags: HashMap::new(),
        }
    }
}

/// Pattern for cache invalidation operations
#[derive(Debug, Clone)]
pub enum InvalidationPattern {
    /// Invalidate all cache entries for specific files
    Files(Vec<PathBuf>),
    /// Invalidate entries matching a glob pattern
    Glob(String),
    /// Invalidate entries older than specified duration
    OlderThan(std::time::Duration),
    /// Invalidate entries with confidence below threshold
    LowConfidence(f64),
    /// Invalidate all entries (cache clear)
    All,
}

/// Scope of files affected by cache invalidation
#[derive(Debug, Clone)]
pub struct InvalidationScope {
    /// Files directly affected
    pub affected_files: Vec<(PathBuf, f64)>, // (file, confidence)
    /// Tests that need to be invalidated
    pub affected_tests: Vec<(PathBuf, f64)>, // (test_file, confidence)
}

impl Default for InvalidationScope {
    fn default() -> Self {
        Self::new()
    }
}

impl InvalidationScope {
    pub fn new() -> Self {
        Self {
            affected_files: Vec::new(),
            affected_tests: Vec::new(),
        }
    }

    pub fn add_affected_file(&mut self, file: PathBuf, confidence: f64) {
        self.affected_files.push((file, confidence));
    }

    pub fn add_affected_test(&mut self, test_file: PathBuf, confidence: f64) {
        self.affected_tests.push((test_file, confidence));
    }

    pub fn is_empty(&self) -> bool {
        self.affected_files.is_empty() && self.affected_tests.is_empty()
    }
}

impl InvalidationPattern {
    /// Check if this pattern matches the given cache key
    pub fn matches(&self, key: &CacheKey) -> bool {
        match self {
            InvalidationPattern::All => true,
            InvalidationPattern::Glob(pattern) => {
                // Simple glob matching - would use a proper glob library in real implementation
                key.as_str().contains(&pattern.replace('*', ""))
            }
            // Other patterns would need cache entry access to implement properly
            _ => false,
        }
    }
}