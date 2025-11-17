//! Prediction engine for file change patterns

use super::pattern_types::{ChangePrediction, CommitPattern, LearningConfig, PredictionReason};
use crate::hotreload::HotReloadResult;
use std::collections::HashMap;
use std::path::PathBuf;

/// Prediction engine for analyzing patterns and making predictions
pub struct PredictionEngine {
    /// File correlation matrix (how often files change together)
    file_correlation_matrix: HashMap<(PathBuf, PathBuf), f64>,
    /// Configuration for predictions
    config: LearningConfig,
}

impl PredictionEngine {
    /// Create new prediction engine with configuration
    pub fn new(config: LearningConfig) -> Self {
        Self {
            file_correlation_matrix: HashMap::new(),
            config,
        }
    }

    /// Update file correlation matrix based on commit patterns
    pub fn update_correlation_matrix(
        &mut self,
        commit_patterns: &[CommitPattern],
    ) -> HotReloadResult<()> {
        let mut co_occurrence_count: HashMap<(PathBuf, PathBuf), usize> = HashMap::new();
        let mut file_count: HashMap<PathBuf, usize> = HashMap::new();

        // Count co-occurrences
        for pattern in commit_patterns {
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
        commit_patterns: &[CommitPattern],
    ) -> HotReloadResult<Vec<ChangePrediction>> {
        let mut predictions = Vec::new();

        // Correlation-based predictions
        for trigger_file in trigger_files {
            let correlated_predictions = self.predict_correlated_files(trigger_file);
            predictions.extend(correlated_predictions);
        }

        // Pattern-based predictions
        let pattern_predictions = self
            .predict_from_patterns(trigger_files, commit_patterns)
            .await?;
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
        commit_patterns: &[CommitPattern],
    ) -> HotReloadResult<Vec<ChangePrediction>> {
        let mut predictions = Vec::new();
        let mut similar_patterns: HashMap<Vec<PathBuf>, usize> = HashMap::new();

        // Find patterns that contain any of the trigger files
        for pattern in commit_patterns {
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
        let total_patterns = commit_patterns.len() as f64;
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

    /// Clear correlation matrix (for testing or reset)
    pub fn clear_correlations(&mut self) {
        self.file_correlation_matrix.clear();
    }

    /// Get number of correlations in matrix
    pub fn correlation_count(&self) -> usize {
        self.file_correlation_matrix.len()
    }

    /// Get configuration
    pub fn get_config(&self) -> &LearningConfig {
        &self.config
    }
}

/// Statistics about file correlations
#[derive(Debug, Clone)]
pub struct CorrelationStats {
    pub total_correlations: usize,
    pub average_correlation: f64,
    pub high_correlation_count: usize,
}

impl CorrelationStats {
    /// Check if correlation data is reliable for predictions
    pub fn is_reliable(&self) -> bool {
        self.total_correlations >= 5 && self.average_correlation > 0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotreload::warming::patterns::pattern_types::{CommitCategory, CommitPattern};
    use std::time::SystemTime;

    fn create_test_patterns() -> Vec<CommitPattern> {
        vec![
            CommitPattern {
                files: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
                timestamp: SystemTime::now(),
                author: "test".to_string(),
                message_category: CommitCategory::Feature,
                commit_hash: "abc123".to_string(),
            },
            CommitPattern {
                files: vec![PathBuf::from("src/main.rs"), PathBuf::from("tests/test.rs")],
                timestamp: SystemTime::now(),
                author: "test".to_string(),
                message_category: CommitCategory::Test,
                commit_hash: "def456".to_string(),
            },
        ]
    }

    #[test]
    fn test_prediction_engine_creation() {
        let config = LearningConfig::default();
        let engine = PredictionEngine::new(config.clone());

        assert_eq!(engine.correlation_count(), 0);
        assert_eq!(engine.config.max_patterns, config.max_patterns);
    }

    #[test]
    fn test_correlation_matrix_update() {
        let config = LearningConfig::default();
        let mut engine = PredictionEngine::new(config);
        let patterns = create_test_patterns();

        let result = engine.update_correlation_matrix(&patterns);
        assert!(result.is_ok());
        assert!(engine.correlation_count() > 0);
    }

    #[tokio::test]
    async fn test_predict_correlated_files() {
        let config = LearningConfig {
            min_confidence: 0.3, // Lower threshold for tests
            ..LearningConfig::default()
        };
        let mut engine = PredictionEngine::new(config);
        let patterns = create_test_patterns();

        engine.update_correlation_matrix(&patterns).unwrap();
        let predictions = engine.predict_correlated_files(&PathBuf::from("src/main.rs"));

        // Should find correlations with src/lib.rs and tests/test.rs
        assert!(!predictions.is_empty());
    }

    #[test]
    fn test_clear_correlations() {
        let config = LearningConfig::default();
        let mut engine = PredictionEngine::new(config);
        let patterns = create_test_patterns();

        engine.update_correlation_matrix(&patterns).unwrap();
        assert!(engine.correlation_count() > 0);

        engine.clear_correlations();
        assert_eq!(engine.correlation_count(), 0);
    }

    #[test]
    fn test_correlation_stats() {
        let stats = CorrelationStats {
            total_correlations: 20,
            average_correlation: 0.6,
            high_correlation_count: 5,
        };

        assert!(stats.is_reliable());

        let unreliable_stats = CorrelationStats {
            total_correlations: 2,
            average_correlation: 0.2,
            high_correlation_count: 0,
        };

        assert!(!unreliable_stats.is_reliable());
    }
}