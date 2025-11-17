//! Pattern learning data structures and types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Configuration for pattern learning
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Maximum number of commit patterns to analyze
    pub max_patterns: usize,
    /// Minimum confidence threshold for predictions
    pub min_confidence: f64,
    /// Time window for analyzing recent patterns (days)
    pub analysis_window_days: i64,
    /// Learning rate for updating correlations
    pub learning_rate: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            max_patterns: 1000,
            min_confidence: 0.7,
            analysis_window_days: 30,
            learning_rate: 0.1,
        }
    }
}

/// Commit pattern extracted from Git history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPattern {
    /// Files modified in this commit
    pub files: Vec<PathBuf>,
    /// Commit timestamp
    pub timestamp: SystemTime,
    /// Commit author
    pub author: String,
    /// Categorized commit message
    pub message_category: CommitCategory,
    /// Hash of the commit
    pub commit_hash: String,
}

/// Categories of commit messages for pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommitCategory {
    Feature,
    BugFix,
    Refactor,
    Documentation,
    Test,
    Configuration,
    Dependencies,
    Unknown,
}

/// Prediction of likely file changes
#[derive(Debug, Clone)]
pub struct ChangePrediction {
    /// Files likely to be changed together
    pub files: Vec<PathBuf>,
    /// Confidence in this prediction (0.0 to 1.0)
    pub confidence: f64,
    /// Reason for this prediction
    pub reason: PredictionReason,
}

/// Reason for a specific prediction
#[derive(Debug, Clone)]
pub enum PredictionReason {
    /// Based on file correlation patterns
    Correlation {
        trigger_file: PathBuf,
        correlation: f64,
    },
    /// Based on historical commit patterns
    CommitPattern { similar_commits: usize },
    /// Based on directory/module relationships
    ModuleRelationship { module: String },
    /// Combination of multiple factors
    Combined,
}

/// Outcome of a prediction for learning feedback
#[derive(Debug, Clone)]
pub struct PredictionOutcome {
    /// Files that were predicted
    #[allow(dead_code)] // Used for learning feedback (future implementation)
    pub predicted_files: Vec<PathBuf>,
    /// Files that actually changed
    #[allow(dead_code)] // Used for learning feedback (future implementation)
    pub actual_files: Vec<PathBuf>,
    /// Timestamp of prediction
    #[allow(dead_code)] // Used for learning feedback (future implementation)
    pub prediction_time: SystemTime,
    /// Whether prediction was successful
    #[allow(dead_code)] // Used for learning feedback (future implementation)
    pub successful: bool,
    /// Confidence score of original prediction
    #[allow(dead_code)] // Used for learning feedback (future implementation)
    pub original_confidence: f64,
}

impl PredictionOutcome {
    /// Create new prediction outcome record
    pub fn new(
        predicted_files: Vec<PathBuf>,
        actual_files: Vec<PathBuf>,
        successful: bool,
        original_confidence: f64,
    ) -> Self {
        Self {
            predicted_files,
            actual_files,
            prediction_time: SystemTime::now(),
            successful,
            original_confidence,
        }
    }
}

/// Statistics about learned patterns
#[derive(Debug, Clone)]
pub struct PatternStats {
    pub total_patterns: usize,
    pub file_correlations: usize,
    pub prediction_outcomes: usize,
    pub average_files_per_commit: f64,
}

impl PatternStats {
    /// Create new pattern statistics
    pub fn new(
        total_patterns: usize,
        file_correlations: usize,
        prediction_outcomes: usize,
        average_files_per_commit: f64,
    ) -> Self {
        Self {
            total_patterns,
            file_correlations,
            prediction_outcomes,
            average_files_per_commit,
        }
    }

    /// Check if enough patterns have been learned for reliable predictions
    pub fn has_sufficient_data(&self) -> bool {
        self.total_patterns >= 10 && self.file_correlations >= 5
    }

    /// Get prediction quality score based on statistics
    pub fn prediction_quality(&self) -> f64 {
        if self.total_patterns == 0 {
            return 0.0;
        }

        let pattern_factor = (self.total_patterns as f64).min(100.0) / 100.0;
        let correlation_factor = (self.file_correlations as f64).min(50.0) / 50.0;

        (pattern_factor + correlation_factor) / 2.0
    }
}

/// Categorize commit message into predefined categories
pub fn categorize_commit_message(message: &str) -> CommitCategory {
    let message_lower = message.to_lowercase();

    if message_lower.contains("feat") || message_lower.contains("feature") {
        CommitCategory::Feature
    } else if message_lower.contains("fix") || message_lower.contains("bug") {
        CommitCategory::BugFix
    } else if message_lower.contains("refactor") || message_lower.contains("refact") {
        CommitCategory::Refactor
    } else if message_lower.contains("doc") || message_lower.contains("readme") {
        CommitCategory::Documentation
    } else if message_lower.contains("test") {
        CommitCategory::Test
    } else if message_lower.contains("config") || message_lower.contains("env") {
        CommitCategory::Configuration
    } else if message_lower.contains("dep")
        || message_lower.contains("cargo")
        || message_lower.contains("package")
    {
        CommitCategory::Dependencies
    } else {
        CommitCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_config_default() {
        let config = LearningConfig::default();
        assert_eq!(config.max_patterns, 1000);
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.analysis_window_days, 30);
        assert_eq!(config.learning_rate, 0.1);
    }

    #[test]
    fn test_commit_categorization() {
        assert_eq!(
            categorize_commit_message("feat: add new feature"),
            CommitCategory::Feature
        );

        assert_eq!(
            categorize_commit_message("fix: resolve bug in parser"),
            CommitCategory::BugFix
        );

        assert_eq!(
            categorize_commit_message("refactor: clean up code"),
            CommitCategory::Refactor
        );

        assert_eq!(
            categorize_commit_message("some random commit"),
            CommitCategory::Unknown
        );
    }

    #[test]
    fn test_pattern_stats_quality() {
        let good_stats = PatternStats::new(100, 50, 20, 4.0);
        assert!(good_stats.prediction_quality() > 0.8);

        let empty_stats = PatternStats::new(0, 0, 0, 0.0);
        assert_eq!(empty_stats.prediction_quality(), 0.0);
    }

    #[test]
    fn test_prediction_outcome_creation() {
        let predicted = vec![PathBuf::from("test1.rs")];
        let actual = vec![PathBuf::from("test1.rs"), PathBuf::from("test2.rs")];

        let outcome = PredictionOutcome::new(predicted.clone(), actual.clone(), true, 0.9);

        assert_eq!(outcome.predicted_files, predicted);
        assert_eq!(outcome.actual_files, actual);
        assert!(outcome.successful);
        assert_eq!(outcome.original_confidence, 0.9);
    }
}