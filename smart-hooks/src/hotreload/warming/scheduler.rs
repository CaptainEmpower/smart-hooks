//! Task scheduler for managing warming execution priorities

use super::{WarmingTask, WarmingTaskType};
use crate::hotreload::HotReloadResult;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

/// Task scheduler that manages warming task execution with priorities
pub struct TaskScheduler {
    /// Priority queue of warming tasks
    task_queue: Mutex<BinaryHeap<PrioritizedTask>>,
    /// Scheduling configuration
    config: SchedulerConfig,
}

/// Configuration for task scheduling
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum number of concurrent warming tasks
    pub max_concurrent_tasks: usize,
    /// Delay between task executions to avoid overwhelming system
    pub task_delay: Duration,
    /// Maximum age before tasks are considered stale
    pub max_task_age: Duration,
    /// Priority boost for manual tasks
    pub manual_task_boost: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 3,
            task_delay: Duration::from_millis(100),
            max_task_age: Duration::from_secs(3600), // 1 hour
            manual_task_boost: 0.2,
        }
    }
}

/// Wrapper for warming tasks with priority ordering
#[derive(Debug, Clone)]
struct PrioritizedTask {
    task: WarmingTask,
    effective_priority: f64,
    scheduled_at: SystemTime,
}

impl PrioritizedTask {
    fn new(task: WarmingTask, config: &SchedulerConfig) -> Self {
        let mut effective_priority = task.priority;

        // Boost priority for manual tasks
        if matches!(task.task_type, WarmingTaskType::Manual) {
            effective_priority += config.manual_task_boost;
        }

        // Age-based priority decay
        let age = task.created_at.elapsed().unwrap_or_default();
        let age_factor = 1.0 - (age.as_secs_f64() / config.max_task_age.as_secs_f64()).min(1.0);
        effective_priority *= age_factor;

        Self {
            task,
            effective_priority: effective_priority.min(1.0),
            scheduled_at: SystemTime::now(),
        }
    }

    fn is_stale(&self, max_age: Duration) -> bool {
        self.scheduled_at.elapsed().unwrap_or_default() > max_age
    }
}

// Implement ordering for priority queue (higher priority first)
impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.effective_priority
            .partial_cmp(&other.effective_priority)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.scheduled_at.cmp(&other.scheduled_at))
    }
}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.effective_priority == other.effective_priority
            && self.scheduled_at == other.scheduled_at
    }
}

impl Eq for PrioritizedTask {}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskScheduler {
    /// Create new task scheduler
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }

    /// Create scheduler with custom configuration
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            task_queue: Mutex::new(BinaryHeap::new()),
            config,
        }
    }

    /// Schedule a warming task
    pub async fn schedule_task(&self, task: WarmingTask) -> HotReloadResult<()> {
        let prioritized_task = PrioritizedTask::new(task, &self.config);

        tracing::debug!(
            "Scheduled warming task with priority {:.3}",
            prioritized_task.effective_priority
        );

        let mut queue = self.task_queue.lock().await;
        queue.push(prioritized_task);

        Ok(())
    }

    /// Get next task to execute (highest priority)
    pub async fn next_task(&self) -> Option<WarmingTask> {
        let mut queue = self.task_queue.lock().await;

        // Remove stale tasks
        let current_tasks: Vec<PrioritizedTask> = queue.drain().collect();
        for task in current_tasks {
            if !task.is_stale(self.config.max_task_age) {
                queue.push(task);
            }
        }

        // Get highest priority task
        queue.pop().map(|pt| pt.task)
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.task_queue.lock().await;
        queue.len()
    }

    /// Clear all scheduled tasks
    pub async fn clear_queue(&self) -> usize {
        let mut queue = self.task_queue.lock().await;
        let size = queue.len();
        queue.clear();
        size
    }

    /// Get tasks by priority level
    pub async fn get_tasks_by_priority(&self, min_priority: f64) -> Vec<WarmingTask> {
        let queue = self.task_queue.lock().await;
        queue
            .iter()
            .filter(|pt| pt.effective_priority >= min_priority)
            .map(|pt| pt.task.clone())
            .collect()
    }

    /// Update scheduler configuration
    pub fn update_config(&mut self, config: SchedulerConfig) {
        self.config = config;
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let queue = self.task_queue.lock().await;
        let tasks: Vec<&PrioritizedTask> = queue.iter().collect();

        let total_tasks = tasks.len();
        let high_priority_tasks = tasks
            .iter()
            .filter(|pt| pt.effective_priority > 0.8)
            .count();
        let medium_priority_tasks = tasks
            .iter()
            .filter(|pt| pt.effective_priority > 0.5 && pt.effective_priority <= 0.8)
            .count();
        let low_priority_tasks = tasks
            .iter()
            .filter(|pt| pt.effective_priority <= 0.5)
            .count();

        let average_priority = if total_tasks > 0 {
            tasks.iter().map(|pt| pt.effective_priority).sum::<f64>() / total_tasks as f64
        } else {
            0.0
        };

        let oldest_task_age = tasks
            .iter()
            .map(|pt| pt.scheduled_at.elapsed().unwrap_or_default())
            .max()
            .unwrap_or_default();

        SchedulerStats {
            total_tasks,
            high_priority_tasks,
            medium_priority_tasks,
            low_priority_tasks,
            average_priority,
            oldest_task_age,
        }
    }
}

/// Statistics about the task scheduler
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// Total number of queued tasks
    pub total_tasks: usize,
    /// Number of high priority tasks (>0.8)
    pub high_priority_tasks: usize,
    /// Number of medium priority tasks (0.5-0.8)
    pub medium_priority_tasks: usize,
    /// Number of low priority tasks (<=0.5)
    pub low_priority_tasks: usize,
    /// Average priority of all tasks
    pub average_priority: f64,
    /// Age of the oldest task in queue
    pub oldest_task_age: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_task_scheduling() {
        let scheduler = TaskScheduler::new();

        // Create tasks with different priorities
        let high_priority_task = WarmingTask::new(
            vec![PathBuf::from("important.rs")],
            0.9,
            WarmingTaskType::Manual,
        );

        let low_priority_task = WarmingTask::new(
            vec![PathBuf::from("optional.rs")],
            0.3,
            WarmingTaskType::Predictive,
        );

        // Schedule tasks
        scheduler.schedule_task(low_priority_task).await.unwrap();
        scheduler
            .schedule_task(high_priority_task.clone())
            .await
            .unwrap();

        // High priority task should be returned first
        let next_task = scheduler.next_task().await.unwrap();
        assert_eq!(next_task.files, high_priority_task.files);
    }

    #[tokio::test]
    async fn test_priority_calculation() {
        let config = SchedulerConfig::default();

        let manual_task =
            WarmingTask::new(vec![PathBuf::from("test.rs")], 0.6, WarmingTaskType::Manual);

        let predictive_task = WarmingTask::new(
            vec![PathBuf::from("test.rs")],
            0.6,
            WarmingTaskType::Predictive,
        );

        let manual_prioritized = PrioritizedTask::new(manual_task, &config);
        let predictive_prioritized = PrioritizedTask::new(predictive_task, &config);

        // Manual task should have higher effective priority
        assert!(manual_prioritized.effective_priority > predictive_prioritized.effective_priority);
    }

    #[tokio::test]
    async fn test_queue_management() {
        let scheduler = TaskScheduler::new();

        let task = WarmingTask::new(
            vec![PathBuf::from("test.rs")],
            0.5,
            WarmingTaskType::Predictive,
        );

        assert_eq!(scheduler.queue_size().await, 0);

        scheduler.schedule_task(task).await.unwrap();
        assert_eq!(scheduler.queue_size().await, 1);

        let cleared = scheduler.clear_queue().await;
        assert_eq!(cleared, 1);
        assert_eq!(scheduler.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_scheduler_stats() {
        let scheduler = TaskScheduler::new();

        // Add tasks with different priorities
        let tasks = vec![
            WarmingTask::new(vec![PathBuf::from("high.rs")], 0.9, WarmingTaskType::Manual),
            WarmingTask::new(
                vec![PathBuf::from("medium.rs")],
                0.6,
                WarmingTaskType::Predictive,
            ),
            WarmingTask::new(
                vec![PathBuf::from("low.rs")],
                0.3,
                WarmingTaskType::Predictive,
            ),
        ];

        for task in tasks {
            scheduler.schedule_task(task).await.unwrap();
        }

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_tasks, 3);
        assert!(stats.average_priority > 0.0);
    }
}