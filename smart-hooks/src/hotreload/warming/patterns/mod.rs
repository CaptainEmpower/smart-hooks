//! Pattern learning for predictive cache warming

use crate::hotreload::HotReloadResult;
use std::path::{Path, PathBuf};

mod git_analysis;
mod pattern_types;
mod prediction_engine;

pub use git_analysis::GitAnalyzer;
pub use pattern_types::{
    categorize_commit_message, ChangePrediction, CommitCategory, CommitPattern, LearningConfig,
    PatternStats, PredictionOutcome, PredictionReason,
};
pub use prediction_engine::{CorrelationStats, PredictionEngine};

/// Pattern learner that analyzes file change patterns for prediction
pub struct PatternLearner {
    /// Learned commit patterns from Git history
    commit_patterns: Vec<CommitPattern>,
    /// Success/failure tracking for learning
    prediction_outcomes: Vec<PredictionOutcome>,
    /// Learning configuration
    config: LearningConfig,
    /// Git analyzer for extracting patterns
    git_analyzer: GitAnalyzer,
    /// Prediction engine for correlations and predictions
    prediction_engine: PredictionEngine,
}

impl Default for PatternLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternLearner {
    /// Create new pattern learner
    pub fn new() -> Self {
        let config = LearningConfig::default();
        Self::with_config(config)
    }

    /// Create pattern learner with custom configuration
    pub fn with_config(config: LearningConfig) -> Self {
        let git_analyzer = GitAnalyzer::new(config.clone());
        let prediction_engine = PredictionEngine::new(config.clone());

        Self {
            commit_patterns: Vec::new(),
            prediction_outcomes: Vec::new(),
            config,
            git_analyzer,
            prediction_engine,
        }
    }

    /// Learn patterns from Git history
    pub async fn learn_from_git_history(&mut self, repo_path: &Path) -> HotReloadResult<usize> {
        // Validate repository first
        if !self.git_analyzer.validate_git_repository(repo_path).await? {
            return Err(crate::hotreload::HotReloadError::FileTracking(
                "Invalid Git repository".to_string(),
            ));
        }

        // Check if repository has sufficient history
        if !self.git_analyzer.has_sufficient_history(repo_path).await? {
            tracing::warn!("Repository has insufficient Git history for reliable pattern learning");
        }

        // Extract commit patterns
        self.commit_patterns = self.git_analyzer.extract_recent_commits(repo_path).await?;
        let pattern_count = self.commit_patterns.len();

        // Update correlation matrix in prediction engine
        self.prediction_engine
            .update_correlation_matrix(&self.commit_patterns)?;

        tracing::info!("Learned {} commit patterns from Git history", pattern_count);

        Ok(pattern_count)
    }

    /// Predict likely changes based on trigger files
    pub async fn predict_likely_changes(
        &self,
        trigger_files: &[PathBuf],
    ) -> HotReloadResult<Vec<ChangePrediction>> {
        self.prediction_engine
            .predict_likely_changes(trigger_files, &self.commit_patterns)
            .await
    }

    /// Record successful prediction for learning
    pub async fn record_successful_prediction(&mut self, files: &[PathBuf]) {
        let outcome = PredictionOutcome::new(files.to_vec(), files.to_vec(), true, 1.0);
        self.prediction_outcomes.push(outcome);
        tracing::debug!("Recorded successful prediction for {} files", files.len());
    }

    /// Record wasted effort for learning
    pub async fn record_wasted_effort(
        &mut self,
        predicted_files: &[PathBuf],
        actual_files: &[PathBuf],
    ) {
        let outcome =
            PredictionOutcome::new(predicted_files.to_vec(), actual_files.to_vec(), false, 0.5);
        self.prediction_outcomes.push(outcome);
        tracing::debug!("Recorded wasted effort for {} files", predicted_files.len());
    }

    /// Get statistics about learned patterns
    pub fn get_pattern_stats(&self) -> PatternStats {
        PatternStats::new(
            self.commit_patterns.len(),
            self.prediction_engine.correlation_count(),
            self.prediction_outcomes.len(),
            if self.commit_patterns.is_empty() {
                0.0
            } else {
                self.commit_patterns
                    .iter()
                    .map(|p| p.files.len())
                    .sum::<usize>() as f64
                    / self.commit_patterns.len() as f64
            },
        )
    }

    /// Get recent authors from Git history
    pub async fn get_recent_authors(&self, repo_path: &Path) -> HotReloadResult<Vec<String>> {
        self.git_analyzer.get_recent_authors(repo_path).await
    }

    /// Categorize commit message using static function
    pub fn categorize_commit_message(&self, message: &str) -> CommitCategory {
        categorize_commit_message(message)
    }

    /// Clear all learned patterns (for testing)
    pub fn clear_patterns(&mut self) {
        self.commit_patterns.clear();
        self.prediction_outcomes.clear();
        self.prediction_engine.clear_correlations();
    }

    /// Get learning configuration
    pub fn get_config(&self) -> &LearningConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_learner_creation() {
        let learner = PatternLearner::new();
        assert_eq!(learner.commit_patterns.len(), 0);
        assert!(learner.config.max_patterns > 0);
    }

    #[test]
    fn test_pattern_learner_with_config() {
        let config = LearningConfig {
            max_patterns: 500,
            min_confidence: 0.8,
            analysis_window_days: 14,
            learning_rate: 0.2,
        };

        let learner = PatternLearner::with_config(config.clone());
        assert_eq!(learner.config.max_patterns, 500);
        assert_eq!(learner.config.min_confidence, 0.8);
    }

    #[test]
    fn test_commit_categorization() {
        let learner = PatternLearner::new();

        assert_eq!(
            learner.categorize_commit_message("feat: add new feature"),
            CommitCategory::Feature
        );

        assert_eq!(
            learner.categorize_commit_message("fix: resolve bug in parser"),
            CommitCategory::BugFix
        );

        assert_eq!(
            learner.categorize_commit_message("refactor: clean up code"),
            CommitCategory::Refactor
        );

        assert_eq!(
            learner.categorize_commit_message("some random commit"),
            CommitCategory::Unknown
        );
    }

    #[tokio::test]
    async fn test_record_outcomes() {
        let mut learner = PatternLearner::new();
        let files = vec![PathBuf::from("test.rs")];

        learner.record_successful_prediction(&files).await;
        assert_eq!(learner.prediction_outcomes.len(), 1);

        learner.record_wasted_effort(&files, &[]).await;
        assert_eq!(learner.prediction_outcomes.len(), 2);
    }

    #[test]
    fn test_pattern_stats() {
        let learner = PatternLearner::new();
        let stats = learner.get_pattern_stats();

        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.file_correlations, 0);
        assert_eq!(stats.prediction_outcomes, 0);
        assert_eq!(stats.average_files_per_commit, 0.0);
    }

    #[test]
    fn test_clear_patterns() {
        let mut learner = PatternLearner::new();

        // Simulate some data
        learner.commit_patterns = vec![CommitPattern {
            files: vec![PathBuf::from("test.rs")],
            timestamp: std::time::SystemTime::now(),
            author: "test".to_string(),
            message_category: CommitCategory::Test,
            commit_hash: "test123".to_string(),
        }];

        learner.clear_patterns();
        assert_eq!(learner.commit_patterns.len(), 0);
        assert_eq!(learner.prediction_outcomes.len(), 0);
    }

    #[test]
    fn test_get_config() {
        let config = LearningConfig {
            max_patterns: 750,
            min_confidence: 0.85,
            analysis_window_days: 21,
            learning_rate: 0.15,
        };

        let learner = PatternLearner::with_config(config.clone());
        let retrieved_config = learner.get_config();

        assert_eq!(retrieved_config.max_patterns, 750);
        assert_eq!(retrieved_config.min_confidence, 0.85);
        assert_eq!(retrieved_config.analysis_window_days, 21);
        assert_eq!(retrieved_config.learning_rate, 0.15);
    }
}