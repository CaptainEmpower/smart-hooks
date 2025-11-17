//! Hot Reload Module - Intelligent caching and proactive hook execution
//!
//! This module implements content-addressable caching with intelligent invalidation
//! for dramatic performance improvements in pre-commit workflows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub mod cache;
pub mod execution;
pub mod tracking;
pub mod warming;

pub use cache::{CacheEntry, CacheKey, CacheStorage, DiskLruCache};
pub use execution::HotReloadEngine;
pub use tracking::{ChangeEvent, ChangeSet, FileChangeTracker};
pub use warming::{BackgroundWarmingService, WarmingTask};

/// Configuration for hot reload system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    /// Enable hot reload functionality
    pub enabled: bool,
    /// Maximum cache size in bytes
    pub max_cache_size_bytes: u64,
    /// Maximum number of cache entries
    pub max_cache_entries: usize,
    /// Cache time-to-live for different hook types
    pub cache_ttl: HashMap<HookType, Duration>,
    /// Minimum confidence score for cache hits
    pub min_confidence_threshold: f64,
    /// Enable background warming
    pub background_warming_enabled: bool,
    /// File system watching enabled
    pub fs_watching_enabled: bool,
    /// Cache directory path
    pub cache_dir: Option<PathBuf>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        let mut cache_ttl = HashMap::new();
        cache_ttl.insert(HookType::Test, Duration::from_secs(3600)); // 1 hour
        cache_ttl.insert(HookType::Lint, Duration::from_secs(1800)); // 30 minutes
        cache_ttl.insert(HookType::Format, Duration::from_secs(86400)); // 24 hours
        cache_ttl.insert(HookType::Analysis, Duration::from_secs(14400)); // 4 hours

        Self {
            enabled: true,
            max_cache_size_bytes: 500 * 1024 * 1024, // 500MB
            max_cache_entries: 10000,
            cache_ttl,
            min_confidence_threshold: 0.7,
            background_warming_enabled: true,
            fs_watching_enabled: true,
            cache_dir: None, // Will be set to project/.git/hooks-cache
        }
    }
}

/// Types of hooks for different caching strategies
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum HookType {
    Test,
    Lint,
    Format,
    Analysis,
    Custom(String),
}

impl HookType {
    /// Convert hook type to string representation
    pub fn as_str(&self) -> &str {
        match self {
            HookType::Test => "test",
            HookType::Lint => "lint",
            HookType::Format => "format",
            HookType::Analysis => "analysis",
            HookType::Custom(name) => name,
        }
    }
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub hook_type: HookType,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub timestamp: SystemTime,
}

/// Collection of hook results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResults {
    pub results: Vec<HookResult>,
    pub overall_success: bool,
    pub total_execution_time: Duration,
}

impl Default for HookResults {
    fn default() -> Self {
        Self::new()
    }
}

impl HookResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            overall_success: true,
            total_execution_time: Duration::ZERO,
        }
    }

    pub fn add_result(&mut self, result: HookResult) {
        self.overall_success = self.overall_success && result.exit_code == 0;
        self.total_execution_time += result.execution_time;
        self.results.push(result);
    }

    pub fn is_success(&self) -> bool {
        self.overall_success
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub hit_rate: f64,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_hit_time: Duration,
    pub average_miss_time: Duration,
    pub cache_size_bytes: u64,
    pub cached_entries: usize,
    pub warming_efficiency: f64,
    pub invalidation_rate: f64,
}

impl CacheStatistics {
    pub fn performance_grade(&self) -> PerformanceGrade {
        if self.hit_rate > 0.9 && self.average_hit_time < Duration::from_millis(50) {
            PerformanceGrade::Excellent
        } else if self.hit_rate > 0.8 && self.average_hit_time < Duration::from_millis(100) {
            PerformanceGrade::Good
        } else if self.hit_rate > 0.6 {
            PerformanceGrade::Acceptable
        } else {
            PerformanceGrade::Poor
        }
    }
}

/// Performance grade for cache system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

/// Error types specific to hot reload system
#[derive(Debug, thiserror::Error)]
pub enum HotReloadError {
    #[error("Cache error: {0}")]
    Cache(String),

    #[error("File tracking error: {0}")]
    FileTracking(String),

    #[error("Warming service error: {0}")]
    Warming(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

impl HotReloadError {
    pub fn is_cache_error(&self) -> bool {
        matches!(self, HotReloadError::Cache(_))
    }
}

/// Result type for hot reload operations
pub type HotReloadResult<T> = std::result::Result<T, HotReloadError>;