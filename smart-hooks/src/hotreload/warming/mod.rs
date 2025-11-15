//! Background warming module for proactive cache population
//!
//! This module provides intelligent cache warming through pattern learning and
//! predictive file change analysis. It's organized into focused sub-modules:
//!
//! - `task`: Core task types and statistics
//! - `service`: Background warming service implementation
//! - `patterns`: Pattern learning for prediction (existing)
//! - `scheduler`: Task scheduling logic (existing)

pub mod patterns;
pub mod scheduler;
pub mod service;
pub mod task;

// Re-export main types for external use
pub use patterns::PatternLearner;
pub use scheduler::TaskScheduler;
pub use service::BackgroundWarmingService;
pub use task::{WarmingStats, WarmingTask, WarmingTaskType};

// Preserve original public interface through re-exports
pub use service::BackgroundWarmingService as BackgroundWarmingServiceImpl;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::time::sleep;

    fn create_test_files() -> Vec<PathBuf> {
        vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("tests/test.rs"),
            PathBuf::from("Cargo.toml"),
        ]
    }

    #[tokio::test]
    async fn test_module_integration() {
        // Test that all components work together
        let service = BackgroundWarmingService::new().await.unwrap();
        let files = create_test_files();

        // Create and schedule tasks
        let task = WarmingTask::new(files.clone(), 0.8, WarmingTaskType::Predictive);
        assert!(matches!(task.task_type, WarmingTaskType::Predictive));

        // Schedule manual warming
        service.schedule_manual_warming(files).await.unwrap();
        assert_eq!(service.queue_size().await, 1);

        // Verify stats integration
        let stats = service.get_stats().await;
        assert_eq!(stats.queue_size, 1);
    }

    #[tokio::test]
    async fn test_task_types_integration() {
        let files = create_test_files();

        // Test all task types are properly integrated
        let _predictive = WarmingTask::new(files.clone(), 0.8, WarmingTaskType::Predictive);
        let _manual = WarmingTask::new(files.clone(), 1.0, WarmingTaskType::Manual);
        let _reactive = WarmingTask::new(files.clone(), 0.6, WarmingTaskType::Reactive);
        let _git_history = WarmingTask::new(files, 0.7, WarmingTaskType::GitHistory);

        // All types should be creatable without issues
        assert!(true);
    }

    #[tokio::test]
    async fn test_service_lifecycle_integration() {
        let service = BackgroundWarmingService::new().await.unwrap();
        let files = create_test_files();

        // Test full lifecycle
        assert!(!service.is_running());

        service.start().await.unwrap();
        assert!(service.is_running());

        service.schedule_manual_warming(files).await.unwrap();
        assert_eq!(service.queue_size().await, 1);

        let cleared = service.clear_queue().await;
        assert_eq!(cleared, 1);
        assert_eq!(service.queue_size().await, 0);

        service.stop().await;
        sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_stats_integration() {
        let service = BackgroundWarmingService::new().await.unwrap();
        let files = create_test_files();

        // Test stats tracking integration
        let initial_stats = service.get_stats().await;
        assert_eq!(initial_stats.efficiency(), 0.0);

        service.report_successful_prediction(&files).await;
        let updated_stats = service.get_stats().await;
        assert_eq!(updated_stats.successful_predictions, 1);

        service.report_wasted_effort(&files).await;
        let final_stats = service.get_stats().await;
        assert_eq!(final_stats.wasted_effort, 1);
    }

    #[tokio::test]
    async fn test_pattern_learner_integration() {
        let mut service = BackgroundWarmingService::new().await.unwrap();
        let files = create_test_files();

        // Test pattern learner integration
        let _pattern_learner = service.pattern_learner();

        // Test prediction scheduling (should not panic)
        let result = service.schedule_warming(&files).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_expiration_integration() {
        let files = create_test_files();
        let task = WarmingTask::new(files, 0.5, WarmingTaskType::Manual);

        // Test expiration functionality
        assert!(!task.is_expired(Duration::from_secs(3600)));

        sleep(Duration::from_millis(10)).await;
        assert!(task.is_expired(Duration::from_millis(5)));
    }

    #[tokio::test]
    async fn test_metadata_integration() {
        let files = create_test_files();
        let task = WarmingTask::new(files, 0.7, WarmingTaskType::GitHistory)
            .with_metadata("commit_id".to_string(), "abc123".to_string())
            .with_metadata("branch".to_string(), "main".to_string());

        assert_eq!(task.metadata.len(), 2);
        assert_eq!(task.metadata.get("commit_id"), Some(&"abc123".to_string()));
        assert_eq!(task.metadata.get("branch"), Some(&"main".to_string()));
    }

    #[tokio::test]
    async fn test_cross_module_functionality() {
        // Test that modules work together correctly
        let service = BackgroundWarmingService::new().await.unwrap();
        let files = create_test_files();

        // Create task from task module
        let task = WarmingTask::new(files.clone(), 0.9, WarmingTaskType::Predictive);

        // Use service from service module
        service.schedule_manual_warming(task.files).await.unwrap();

        // Check stats from task module via service
        let stats = service.get_stats().await;
        assert_eq!(stats.queue_size, 1);

        // Verify efficiency calculation from stats
        assert_eq!(stats.efficiency(), 0.0); // No executed tasks yet
    }
}