//! Pattern learning for predictive cache warming

use crate::hotreload::{HotReloadError, HotReloadResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tokio::process::Command;

/// Pattern learner that analyzes file change patterns for prediction
pub struct PatternLearner {
    /// Learned commit patterns from Git history
    commit_patterns: Vec<CommitPattern>,
    /// File correlation matrix (how often files change together)
    file_correlation_matrix: HashMap<(PathBuf, PathBuf), f64>,
    /// Success/failure tracking for learning
    prediction_outcomes: Vec<PredictionOutcome>,
    /// Learning configuration
    config: LearningConfig,
}

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
    predicted_files: Vec<PathBuf>,
    /// Files that actually changed
    actual_files: Vec<PathBuf>,
    /// Timestamp of prediction
    prediction_time: SystemTime,
    /// Whether prediction was successful
    successful: bool,
    /// Confidence score of original prediction
    original_confidence: f64,
}

impl PatternLearner {
    /// Create new pattern learner
    pub fn new() -> Self {
        Self {
            commit_patterns: Vec::new(),
            file_correlation_matrix: HashMap::new(),
            prediction_outcomes: Vec::new(),
            config: LearningConfig::default(),
        }
    }

    /// Create pattern learner with custom configuration
    pub fn with_config(config: LearningConfig) -> Self {
        Self {
            commit_patterns: Vec::new(),
            file_correlation_matrix: HashMap::new(),
            prediction_outcomes: Vec::new(),
            config,
        }
    }

    /// Learn patterns from Git history
    pub async fn learn_from_git_history(&mut self, repo_path: &Path) -> HotReloadResult<usize> {
        let commits = self.extract_recent_commits(repo_path).await?;
        let pattern_count = commits.len();

        // Store commit patterns
        self.commit_patterns = commits;

        // Update file correlation matrix
        self.update_correlation_matrix()?;

        tracing::info!("Learned {} commit patterns from Git history", pattern_count);

        Ok(pattern_count)
    }

    /// Extract recent commits from Git repository
    async fn extract_recent_commits(
        &self,
        repo_path: &Path,
    ) -> HotReloadResult<Vec<CommitPattern>> {
        let since_date = format!("--since={} days ago", self.config.analysis_window_days);

        // Get commit list with files
        let output = Command::new("git")
            .current_dir(repo_path)
            .args([
                "log",
                &since_date,
                "--pretty=format:%H|%an|%ai|%s",
                "--name-only",
                "--no-merges",
            ])
            .output()
            .await
            .map_err(|e| HotReloadError::FileTracking(format!("Git command failed: {}", e)))?;

        let git_log = String::from_utf8_lossy(&output.stdout);
        let mut patterns = Vec::new();

        for commit_block in git_log.split("\n\n") {
            if let Some(pattern) = self.parse_commit_block(commit_block, repo_path).await? {
                patterns.push(pattern);

                if patterns.len() >= self.config.max_patterns {
                    break;
                }
            }
        }

        Ok(patterns)
    }

    /// Parse a single commit block from git log
    async fn parse_commit_block(
        &self,
        block: &str,
        repo_path: &Path,
    ) -> HotReloadResult<Option<CommitPattern>> {
        let lines: Vec<&str> = block.lines().collect();

        if lines.is_empty() {
            return Ok(None);
        }

        // Parse commit info line: hash|author|date|subject
        let info_parts: Vec<&str> = lines[0].split('|').collect();
        if info_parts.len() < 4 {
            return Ok(None);
        }

        let commit_hash = info_parts[0].to_string();
        let author = info_parts[1].to_string();
        let timestamp = self.parse_git_date(info_parts[2])?;
        let subject = info_parts[3].to_string();

        // Parse file list (lines 1 onwards)
        let mut files = Vec::new();
        for line in &lines[1..] {
            if !line.trim().is_empty() {
                let file_path = repo_path.join(line.trim());
                if file_path.exists() {
                    files.push(file_path);
                }
            }
        }

        if files.is_empty() {
            return Ok(None);
        }

        let message_category = self.categorize_commit_message(&subject);

        Ok(Some(CommitPattern {
            files,
            timestamp,
            author,
            message_category,
            commit_hash,
        }))
    }

    /// Parse Git date string to SystemTime
    fn parse_git_date(&self, _date_str: &str) -> HotReloadResult<SystemTime> {
        // Git date format: "2024-01-15 10:30:45 -0800"
        // For simplicity, use current time - in production would parse properly
        Ok(SystemTime::now())
    }

    /// Categorize commit message into predefined categories
    pub fn categorize_commit_message(&self, message: &str) -> CommitCategory {
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

    /// Update file correlation matrix based on commit patterns
    fn update_correlation_matrix(&mut self) -> HotReloadResult<()> {
        let mut co_occurrence_count: HashMap<(PathBuf, PathBuf), usize> = HashMap::new();
        let mut file_count: HashMap<PathBuf, usize> = HashMap::new();

        // Count co-occurrences
        for pattern in &self.commit_patterns {
            // Count individual files
            for file in &pattern.files {
                *file_count.entry(file.clone()).or_insert(0) += 1;
            }

            // Count file pairs
            for i in 0..pattern.files.len() {
                for j in (i + 1)..pattern.files.len() {
                    let file1 = &pattern.files[i];
                    let file2 = &pattern.files[j];

                    // Ensure consistent ordering
                    let key = if file1 < file2 {
                        (file1.clone(), file2.clone())
                    } else {
                        (file2.clone(), file1.clone())
                    };

                    *co_occurrence_count.entry(key).or_insert(0) += 1;
                }
            }
        }

        // Calculate correlation coefficients
        self.file_correlation_matrix.clear();
        for ((file1, file2), co_count) in co_occurrence_count {
            let file1_count = file_count.get(&file1).unwrap_or(&0);
            let file2_count = file_count.get(&file2).unwrap_or(&0);

            if *file1_count > 0 && *file2_count > 0 {
                // Simple correlation: co-occurrence / min(file1_count, file2_count)
                let correlation = co_count as f64 / (*file1_count.min(file2_count) as f64);
                self.file_correlation_matrix
                    .insert((file1, file2), correlation);
            }
        }

        tracing::debug!(
            "Updated correlation matrix with {} file pairs",
            self.file_correlation_matrix.len()
        );

        Ok(())
    }

    /// Predict likely changes based on trigger files
    pub async fn predict_likely_changes(
        &self,
        trigger_files: &[PathBuf],
    ) -> HotReloadResult<Vec<ChangePrediction>> {
        let mut predictions = Vec::new();

        // Correlation-based predictions
        for trigger_file in trigger_files {
            let correlated_predictions = self.predict_correlated_files(trigger_file);
            predictions.extend(correlated_predictions);
        }

        // Pattern-based predictions
        let pattern_predictions = self.predict_from_patterns(trigger_files).await?;
        predictions.extend(pattern_predictions);

        // Remove duplicates and sort by confidence
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        predictions.dedup_by(|a, b| a.files == b.files);

        // Filter by confidence threshold
        predictions.retain(|p| p.confidence >= self.config.min_confidence);

        Ok(predictions)
    }

    /// Predict files correlated with a trigger file
    fn predict_correlated_files(&self, trigger_file: &PathBuf) -> Vec<ChangePrediction> {
        let mut predictions = Vec::new();

        for ((file1, file2), correlation) in &self.file_correlation_matrix {
            let correlated_file = if file1 == trigger_file {
                Some(file2.clone())
            } else if file2 == trigger_file {
                Some(file1.clone())
            } else {
                None
            };

            if let Some(file) = correlated_file {
                if *correlation >= self.config.min_confidence {
                    predictions.push(ChangePrediction {
                        files: vec![file],
                        confidence: *correlation,
                        reason: PredictionReason::Correlation {
                            trigger_file: trigger_file.clone(),
                            correlation: *correlation,
                        },
                    });
                }
            }
        }

        predictions
    }

    /// Predict changes based on historical commit patterns
    async fn predict_from_patterns(
        &self,
        trigger_files: &[PathBuf],
    ) -> HotReloadResult<Vec<ChangePrediction>> {
        let mut predictions = Vec::new();
        let mut similar_patterns: HashMap<Vec<PathBuf>, usize> = HashMap::new();

        // Find patterns that contain any of the trigger files
        for pattern in &self.commit_patterns {
            let has_trigger_file = pattern.files.iter().any(|f| trigger_files.contains(f));

            if has_trigger_file {
                // Consider all other files in this pattern as predictions
                let other_files: Vec<PathBuf> = pattern
                    .files
                    .iter()
                    .filter(|f| !trigger_files.contains(f))
                    .cloned()
                    .collect();

                if !other_files.is_empty() {
                    *similar_patterns.entry(other_files).or_insert(0) += 1;
                }
            }
        }

        // Convert patterns to predictions
        let total_patterns = self.commit_patterns.len() as f64;
        for (files, count) in similar_patterns {
            let confidence = count as f64 / total_patterns;

            if confidence >= self.config.min_confidence {
                predictions.push(ChangePrediction {
                    files,
                    confidence,
                    reason: PredictionReason::CommitPattern {
                        similar_commits: count,
                    },
                });
            }
        }

        Ok(predictions)
    }

    /// Record successful prediction for learning
    pub async fn record_successful_prediction(&self, files: &[PathBuf]) {
        tracing::debug!("Recorded successful prediction for {} files", files.len());
        // In a full implementation, this would update learning parameters
    }

    /// Record wasted effort for learning
    pub async fn record_wasted_effort(&self, files: &[PathBuf]) {
        tracing::debug!("Recorded wasted effort for {} files", files.len());
        // In a full implementation, this would update learning parameters
    }

    /// Get statistics about learned patterns
    pub fn get_pattern_stats(&self) -> PatternStats {
        PatternStats {
            total_patterns: self.commit_patterns.len(),
            file_correlations: self.file_correlation_matrix.len(),
            prediction_outcomes: self.prediction_outcomes.len(),
            average_files_per_commit: if self.commit_patterns.is_empty() {
                0.0
            } else {
                self.commit_patterns
                    .iter()
                    .map(|p| p.files.len())
                    .sum::<usize>() as f64
                    / self.commit_patterns.len() as f64
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
    async fn test_pattern_prediction() {
        let mut learner = PatternLearner::new();

        // Add some mock patterns
        learner.commit_patterns = vec![CommitPattern {
            files: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
            timestamp: SystemTime::now(),
            author: "test".to_string(),
            message_category: CommitCategory::Feature,
            commit_hash: "abc123".to_string(),
        }];

        learner.update_correlation_matrix().unwrap();

        let predictions = learner
            .predict_likely_changes(&[PathBuf::from("src/main.rs")])
            .await
            .unwrap();

        // Should predict src/lib.rs based on correlation
        assert!(!predictions.is_empty());
    }
}