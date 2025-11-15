//! Background warming service for proactive cache population

use crate::hotreload::HotReloadResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time;

pub mod patterns;
pub mod scheduler;

pub use patterns::PatternLearner;
pub use scheduler::TaskScheduler;

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