//! Execution metrics tracking for hot reload performance monitoring

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Metrics for hot reload execution performance
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_executions: u64,
    pub total_execution_time: Duration,
    pub cache_hit_time: Duration,
    pub cache_miss_time: Duration,
}

impl ExecutionMetrics {
    /// Calculate cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (self.total_executions as f64)
        }
    }

    /// Calculate average execution time
    pub fn average_execution_time(&self) -> Duration {
        if self.total_executions == 0 {
            Duration::ZERO
        } else {
            self.total_execution_time / self.total_executions as u32
        }
    }

    /// Calculate average cache hit time
    pub fn average_cache_hit_time(&self) -> Duration {
        if self.cache_hits == 0 {
            Duration::ZERO
        } else {
            self.cache_hit_time / self.cache_hits as u32
        }
    }

    /// Calculate average cache miss time
    pub fn average_cache_miss_time(&self) -> Duration {
        if self.cache_misses == 0 {
            Duration::ZERO
        } else {
            self.cache_miss_time / self.cache_misses as u32
        }
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self, execution_time: Duration) {
        self.cache_hits += 1;
        self.total_executions += 1;
        self.cache_hit_time += execution_time;
        self.total_execution_time += execution_time;
    }

    /// Record a cache miss
    pub fn record_cache_miss(&mut self, execution_time: Duration) {
        self.cache_misses += 1;
        self.total_executions += 1;
        self.cache_miss_time += execution_time;
        self.total_execution_time += execution_time;
    }
}

/// Thread-safe metrics manager
pub struct MetricsManager {
    metrics: Arc<RwLock<ExecutionMetrics>>,
}

impl MetricsManager {
    /// Create new metrics manager
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ExecutionMetrics::default())),
        }
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> ExecutionMetrics {
        self.metrics.read().await.clone()
    }

    /// Update metrics with a closure
    pub async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut ExecutionMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut metrics);
    }

    /// Record cache hit
    pub async fn record_hit(&self, execution_time: Duration) {
        self.update_metrics(|m| m.record_cache_hit(execution_time))
            .await;
    }

    /// Record cache miss
    pub async fn record_miss(&self, execution_time: Duration) {
        self.update_metrics(|m| m.record_cache_miss(execution_time))
            .await;
    }

    /// Reset all metrics
    #[allow(dead_code)] // Used in tests and potentially in production
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = ExecutionMetrics::default();
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_execution_metrics_calculations() {
        let mut metrics = ExecutionMetrics::default();

        assert_eq!(metrics.hit_rate(), 0.0);
        assert_eq!(metrics.average_execution_time(), Duration::ZERO);

        // Record some hits and misses
        metrics.record_cache_hit(Duration::from_millis(100));
        metrics.record_cache_miss(Duration::from_millis(500));
        metrics.record_cache_hit(Duration::from_millis(50));

        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.total_executions, 3);

        assert!((metrics.hit_rate() - 0.666).abs() < 0.01); // ~66.7%

        let avg_hit_time = metrics.average_cache_hit_time();
        assert_eq!(avg_hit_time, Duration::from_millis(75)); // (100 + 50) / 2

        let avg_miss_time = metrics.average_cache_miss_time();
        assert_eq!(avg_miss_time, Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_metrics_manager() {
        let manager = MetricsManager::new();

        // Initially empty
        let initial = manager.get_metrics().await;
        assert_eq!(initial.total_executions, 0);

        // Record some operations
        manager.record_hit(Duration::from_millis(100)).await;
        manager.record_miss(Duration::from_millis(300)).await;

        let updated = manager.get_metrics().await;
        assert_eq!(updated.cache_hits, 1);
        assert_eq!(updated.cache_misses, 1);
        assert_eq!(updated.total_executions, 2);

        // Reset and verify
        manager.reset().await;
        let reset = manager.get_metrics().await;
        assert_eq!(reset.total_executions, 0);
    }

    #[tokio::test]
    async fn test_concurrent_metrics_updates() {
        let manager = Arc::new(MetricsManager::new());
        let mut handles = Vec::new();

        // Spawn multiple tasks updating metrics
        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                manager_clone.record_hit(Duration::from_millis(50)).await;
                manager_clone.record_miss(Duration::from_millis(200)).await;
            });
            handles.push(handle);
        }

        // Wait for all updates
        for handle in handles {
            handle.await.unwrap();
        }

        let final_metrics = manager.get_metrics().await;
        assert_eq!(final_metrics.cache_hits, 10);
        assert_eq!(final_metrics.cache_misses, 10);
        assert_eq!(final_metrics.total_executions, 20);
    }
}