//! Warming task types and statistics
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Task for warming cache with specific files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmingTask {
    /// Files to warm in cache
    pub files: Vec<PathBuf>,
    /// Priority of warming task (0.0 to 1.0)
    pub priority: f64,
    /// When this task was created
    pub created_at: SystemTime,
    /// Type of warming task
    pub task_type: WarmingTaskType,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmingTaskType {
    /// Predictive warming based on file patterns
    Predictive,
    /// Manual warming requested by user
    Manual,
    /// Warming triggered by file system events
    Reactive,
    /// Warming based on Git commit patterns
    GitHistory,
}

impl WarmingTask {
    /// Create new warming task
    pub fn new(files: Vec<PathBuf>, priority: f64, task_type: WarmingTaskType) -> Self {
        Self {
            files,
            priority,
            created_at: SystemTime::now(),
            task_type,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to task
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if task has expired
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed().unwrap_or_default() > ttl
    }
}

/// Statistics for warming service performance
#[derive(Debug, Clone, Default)]
pub struct WarmingStats {
    /// Total warming tasks executed
    pub tasks_executed: u64,
    /// Tasks that resulted in cache hits later
    pub successful_predictions: u64,
    /// Tasks that were never used
    pub wasted_effort: u64,
    /// Total time spent on warming
    pub total_warming_time: Duration,
    /// Average warming task execution time
    pub average_task_time: Duration,
    /// Current queue size
    pub queue_size: usize,
}

impl WarmingStats {
    /// Calculate warming efficiency (successful predictions / total tasks)
    pub fn efficiency(&self) -> f64 {
        if self.tasks_executed > 0 {
            self.successful_predictions as f64 / self.tasks_executed as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    fn create_test_files() -> Vec<PathBuf> {
        vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("tests/test.rs"),
            PathBuf::from("Cargo.toml"),
        ]
    }

    #[test]
    fn test_warming_task_creation() {
        let files = create_test_files();
        let task = WarmingTask::new(files.clone(), 0.8, WarmingTaskType::Predictive);

        assert_eq!(task.files, files);
        assert_eq!(task.priority, 0.8);
        assert!(matches!(task.task_type, WarmingTaskType::Predictive));
        assert!(task.metadata.is_empty());
    }

    #[test]
    fn test_warming_task_with_metadata() {
        let files = create_test_files();
        let task = WarmingTask::new(files, 0.9, WarmingTaskType::Manual)
            .with_metadata("source".to_string(), "user_request".to_string());

        assert_eq!(
            task.metadata.get("source"),
            Some(&"user_request".to_string())
        );
    }

    #[tokio::test]
    async fn test_warming_task_expiration() {
        let files = create_test_files();
        let task = WarmingTask::new(files, 0.7, WarmingTaskType::Reactive);

        // Should not be expired with reasonable TTL
        assert!(!task.is_expired(Duration::from_secs(3600)));

        // Should be expired with very short TTL
        sleep(Duration::from_millis(10)).await;
        assert!(task.is_expired(Duration::from_millis(5)));
    }

    #[test]
    fn test_warming_task_types() {
        let files = create_test_files();

        let predictive = WarmingTask::new(files.clone(), 0.8, WarmingTaskType::Predictive);
        assert!(matches!(predictive.task_type, WarmingTaskType::Predictive));

        let manual = WarmingTask::new(files.clone(), 1.0, WarmingTaskType::Manual);
        assert!(matches!(manual.task_type, WarmingTaskType::Manual));

        let reactive = WarmingTask::new(files.clone(), 0.6, WarmingTaskType::Reactive);
        assert!(matches!(reactive.task_type, WarmingTaskType::Reactive));

        let git_history = WarmingTask::new(files, 0.7, WarmingTaskType::GitHistory);
        assert!(matches!(git_history.task_type, WarmingTaskType::GitHistory));
    }

    #[test]
    fn test_stats_efficiency_calculation() {
        let mut stats = WarmingStats::default();

        // No executions
        assert_eq!(stats.efficiency(), 0.0);

        // Some successful predictions
        stats.tasks_executed = 10;
        stats.successful_predictions = 7;
        assert_eq!(stats.efficiency(), 0.7);

        // Perfect efficiency
        stats.successful_predictions = 10;
        assert_eq!(stats.efficiency(), 1.0);
    }

    #[test]
    fn test_warming_stats_time_tracking() {
        let mut stats = WarmingStats::default();

        // Add some execution time
        stats.total_warming_time = Duration::from_millis(1000);
        stats.tasks_executed = 10;

        // Calculate average (should be 100ms per task)
        stats.average_task_time = stats.total_warming_time / stats.tasks_executed as u32;
        assert_eq!(stats.average_task_time, Duration::from_millis(100));
    }

    #[test]
    fn test_task_metadata_functionality() {
        let files = create_test_files();
        let task = WarmingTask::new(files, 0.8, WarmingTaskType::Predictive)
            .with_metadata("trigger".to_string(), "file_change".to_string())
            .with_metadata("confidence".to_string(), "high".to_string());

        assert_eq!(task.metadata.len(), 2);
        assert_eq!(
            task.metadata.get("trigger"),
            Some(&"file_change".to_string())
        );
        assert_eq!(task.metadata.get("confidence"), Some(&"high".to_string()));
    }
}