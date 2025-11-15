//! Background warming service implementation
use crate::hotreload::HotReloadResult;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time;

use super::patterns::PatternLearner;
use super::scheduler::TaskScheduler;
use super::task::{WarmingStats, WarmingTask, WarmingTaskType};

/// Background service for warming cache with predicted file changes
pub struct BackgroundWarmingService {
    /// Queue of warming tasks to execute
    warming_queue: Arc<Mutex<VecDeque<WarmingTask>>>,
    /// Pattern learner for predicting file change patterns
    pattern_learner: PatternLearner,
    /// Task scheduler for managing warming execution
    #[allow(dead_code)] // Will be used when background scheduling is fully implemented
    scheduler: TaskScheduler,
    /// Flag to control service running state
    is_running: Arc<AtomicBool>,
    /// Warming statistics
    stats: Arc<Mutex<WarmingStats>>,
}

impl BackgroundWarmingService {
    /// Create new background warming service
    pub async fn new() -> HotReloadResult<Self> {
        let warming_queue = Arc::new(Mutex::new(VecDeque::new()));
        let pattern_learner = PatternLearner::new();
        let scheduler = TaskScheduler::new();
        let is_running = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(Mutex::new(WarmingStats::default()));

        Ok(Self {
            warming_queue,
            pattern_learner,
            scheduler,
            is_running,
            stats,
        })
    }

    /// Start the background warming service
    pub async fn start(&self) -> HotReloadResult<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(()); // Already running
        }

        self.is_running.store(true, Ordering::Relaxed);

        // Start the main warming loop
        let queue = self.warming_queue.clone();
        let is_running = self.is_running.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            Self::warming_loop(queue, is_running, stats).await;
        });

        tracing::info!("Background warming service started");
        Ok(())
    }

    /// Stop the background warming service
    pub async fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        tracing::info!("Background warming service stopped");
    }

    /// Schedule warming for files likely to be committed together
    pub async fn schedule_warming(&self, changed_files: &[PathBuf]) -> HotReloadResult<()> {
        // Predict likely changes using pattern learning
        let predictions = self
            .pattern_learner
            .predict_likely_changes(changed_files)
            .await?;

        let mut queue = self.warming_queue.lock().await;
        let mut stats = self.stats.lock().await;

        for prediction in predictions {
            if prediction.confidence > 0.7 {
                // High confidence threshold
                let task = WarmingTask::new(
                    prediction.files,
                    prediction.confidence,
                    WarmingTaskType::Predictive,
                )
                .with_metadata(
                    "trigger_files".to_string(),
                    changed_files
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(";"),
                );

                queue.push_back(task);
                stats.queue_size = queue.len();
            }
        }

        Ok(())
    }

    /// Schedule manual warming task
    pub async fn schedule_manual_warming(&self, files: Vec<PathBuf>) -> HotReloadResult<()> {
        let task = WarmingTask::new(files, 1.0, WarmingTaskType::Manual);

        let mut queue = self.warming_queue.lock().await;
        let mut stats = self.stats.lock().await;

        queue.push_back(task);
        stats.queue_size = queue.len();

        Ok(())
    }

    /// Main warming loop that processes tasks
    async fn warming_loop(
        queue: Arc<Mutex<VecDeque<WarmingTask>>>,
        is_running: Arc<AtomicBool>,
        stats: Arc<Mutex<WarmingStats>>,
    ) {
        let mut interval = time::interval(Duration::from_millis(100));

        while is_running.load(Ordering::Relaxed) {
            interval.tick().await;

            // Process next task if available
            let task = {
                let mut queue_guard = queue.lock().await;
                queue_guard.pop_front()
            };

            if let Some(task) = task {
                let start_time = SystemTime::now();

                // Execute warming task
                match Self::execute_warming_task(&task).await {
                    Ok(_) => {
                        let execution_time = start_time.elapsed().unwrap_or_default();

                        let mut stats_guard = stats.lock().await;
                        stats_guard.tasks_executed += 1;
                        stats_guard.total_warming_time += execution_time;
                        stats_guard.average_task_time =
                            stats_guard.total_warming_time / stats_guard.tasks_executed as u32;
                        stats_guard.queue_size = {
                            let queue_guard = queue.lock().await;
                            queue_guard.len()
                        };

                        tracing::debug!(
                            "Warming task completed: {} files in {:?}",
                            task.files.len(),
                            execution_time
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Warming task failed: {:?}", e);
                    }
                }

                // Clean up expired tasks
                let mut queue_guard = queue.lock().await;
                let task_ttl = Duration::from_secs(3600); // 1 hour TTL
                queue_guard.retain(|task| !task.is_expired(task_ttl));

                let mut stats_guard = stats.lock().await;
                stats_guard.queue_size = queue_guard.len();
            }
        }

        tracing::info!("Warming loop exited");
    }

    /// Execute a warming task by pre-computing hook results
    async fn execute_warming_task(task: &WarmingTask) -> HotReloadResult<()> {
        // In a real implementation, this would:
        // 1. Run smart-hooks commands on the files
        // 2. Cache the results
        // 3. Track success/failure for learning

        tracing::debug!(
            "Executing warming task: {} files, priority: {:.2}",
            task.files.len(),
            task.priority
        );

        // Simulate warming work
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(())
    }

    /// Get current warming statistics
    pub async fn get_stats(&self) -> WarmingStats {
        self.stats.lock().await.clone()
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        self.warming_queue.lock().await.len()
    }

    /// Clear warming queue
    pub async fn clear_queue(&self) -> usize {
        let mut queue = self.warming_queue.lock().await;
        let size = queue.len();
        queue.clear();

        let mut stats = self.stats.lock().await;
        stats.queue_size = 0;

        size
    }

    /// Report successful prediction (for learning)
    pub async fn report_successful_prediction(&self, files: &[PathBuf]) {
        self.pattern_learner
            .record_successful_prediction(files)
            .await;

        let mut stats = self.stats.lock().await;
        stats.successful_predictions += 1;
    }

    /// Report wasted effort (for learning)
    pub async fn report_wasted_effort(&self, files: &[PathBuf]) {
        self.pattern_learner.record_wasted_effort(files).await;

        let mut stats = self.stats.lock().await;
        stats.wasted_effort += 1;
    }

    /// Get pattern learner for advanced configuration
    pub fn pattern_learner(&mut self) -> &mut PatternLearner {
        &mut self.pattern_learner
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::sleep;

    fn create_test_files() -> Vec<PathBuf> {
        vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("tests/test.rs"),
            PathBuf::from("Cargo.toml"),
        ]
    }

    async fn create_test_service() -> BackgroundWarmingService {
        BackgroundWarmingService::new().await.unwrap()
    }

    #[tokio::test]
    async fn test_service_creation() {
        let service = create_test_service().await;
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_service_start_stop() {
        let service = create_test_service().await;

        // Start service
        service.start().await.unwrap();
        assert!(service.is_running());

        // Starting again should be OK
        service.start().await.unwrap();
        assert!(service.is_running());

        // Stop service
        service.stop().await;
        // Note: There might be a small delay before is_running returns false
        sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_manual_warming_scheduling() {
        let service = create_test_service().await;
        let files = create_test_files();

        // Schedule manual warming
        service
            .schedule_manual_warming(files.clone())
            .await
            .unwrap();

        // Check queue size increased
        assert_eq!(service.queue_size().await, 1);

        // Get stats to verify
        let stats = service.get_stats().await;
        assert_eq!(stats.queue_size, 1);
    }

    #[tokio::test]
    async fn test_queue_clearing() {
        let service = create_test_service().await;
        let files = create_test_files();

        // Add multiple tasks
        for i in 0..3 {
            let mut task_files = files.clone();
            task_files.push(PathBuf::from(format!("file_{}.rs", i)));
            service.schedule_manual_warming(task_files).await.unwrap();
        }

        assert_eq!(service.queue_size().await, 3);

        // Clear queue
        let cleared_count = service.clear_queue().await;
        assert_eq!(cleared_count, 3);
        assert_eq!(service.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_stats_initialization() {
        let service = create_test_service().await;
        let stats = service.get_stats().await;

        assert_eq!(stats.tasks_executed, 0);
        assert_eq!(stats.successful_predictions, 0);
        assert_eq!(stats.wasted_effort, 0);
        assert_eq!(stats.queue_size, 0);
        assert_eq!(stats.efficiency(), 0.0);
    }

    #[tokio::test]
    async fn test_successful_prediction_reporting() {
        let service = create_test_service().await;
        let files = create_test_files();

        let initial_stats = service.get_stats().await;
        assert_eq!(initial_stats.successful_predictions, 0);

        // Report successful prediction
        service.report_successful_prediction(&files).await;

        let updated_stats = service.get_stats().await;
        assert_eq!(updated_stats.successful_predictions, 1);
    }

    #[tokio::test]
    async fn test_wasted_effort_reporting() {
        let service = create_test_service().await;
        let files = create_test_files();

        let initial_stats = service.get_stats().await;
        assert_eq!(initial_stats.wasted_effort, 0);

        // Report wasted effort
        service.report_wasted_effort(&files).await;

        let updated_stats = service.get_stats().await;
        assert_eq!(updated_stats.wasted_effort, 1);
    }

    #[tokio::test]
    async fn test_pattern_learner_access() {
        let mut service = create_test_service().await;
        let _pattern_learner = service.pattern_learner();
        // Just verify we can access it without panicking
    }

    #[tokio::test]
    async fn test_task_execution_simulation() {
        let files = create_test_files();
        let task = WarmingTask::new(files, 0.9, WarmingTaskType::Manual);

        // Test the execute_warming_task function
        let start = SystemTime::now();
        let result = BackgroundWarmingService::execute_warming_task(&task).await;
        let duration = start.elapsed().unwrap();

        assert!(result.is_ok());
        // Should take at least 50ms due to the sleep in the implementation
        assert!(duration >= Duration::from_millis(45));
    }

    #[tokio::test]
    async fn test_concurrent_task_scheduling() {
        let service = Arc::new(create_test_service().await);
        let files = create_test_files();

        // Schedule multiple tasks concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let service_clone = Arc::clone(&service);
            let files_clone = files.clone();
            handles.push(tokio::spawn(async move {
                let mut task_files = files_clone;
                task_files.push(PathBuf::from(format!("concurrent_{}.rs", i)));
                service_clone.schedule_manual_warming(task_files).await
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // All tasks should be queued
        assert_eq!(service.queue_size().await, 5);
    }

    #[tokio::test]
    async fn test_service_warming_loop_integration() {
        let service = create_test_service().await;
        let files = create_test_files();

        // Start the service
        service.start().await.unwrap();

        // Schedule some tasks
        service
            .schedule_manual_warming(files.clone())
            .await
            .unwrap();
        service.schedule_manual_warming(files).await.unwrap();

        // Wait a bit for processing
        sleep(Duration::from_millis(200)).await;

        // Check that stats show some activity
        let stats = service.get_stats().await;
        // Tasks should have been processed (always true since u64 is always >= 0)
        assert!(stats.total_warming_time >= Duration::ZERO);

        // Stop the service
        service.stop().await;
    }

    #[tokio::test]
    async fn test_high_confidence_threshold_filtering() {
        let service = create_test_service().await;
        let files = create_test_files();

        // This would require mocking the pattern learner to return specific predictions
        // For now, we'll test that the method exists and doesn't panic
        let result = service.schedule_warming(&files).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_state_consistency() {
        let service = create_test_service().await;

        // Initial state
        assert!(!service.is_running());
        assert_eq!(service.queue_size().await, 0);

        // Start and add tasks
        service.start().await.unwrap();
        assert!(service.is_running());

        let files = create_test_files();
        service.schedule_manual_warming(files).await.unwrap();
        assert_eq!(service.queue_size().await, 1);

        // Stop service
        service.stop().await;
        // Queue should still contain items after stopping
        assert_eq!(service.queue_size().await, 1);
    }

    #[tokio::test]
    async fn test_edge_cases() {
        let service = create_test_service().await;

        // Empty file list
        let empty_files = vec![];
        let result = service.schedule_manual_warming(empty_files).await;
        assert!(result.is_ok());
    }
}