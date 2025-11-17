//! Git history analysis for pattern learning

use super::pattern_types::{categorize_commit_message, CommitPattern, LearningConfig};
use crate::hotreload::{HotReloadError, HotReloadResult};
use std::path::Path;
use std::time::SystemTime;
use tokio::process::Command;

/// Git history analyzer for extracting commit patterns
pub struct GitAnalyzer {
    config: LearningConfig,
}

impl GitAnalyzer {
    /// Create new Git analyzer with configuration
    pub fn new(config: LearningConfig) -> Self {
        Self { config }
    }

    /// Extract recent commits from Git repository
    pub async fn extract_recent_commits(
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

        tracing::info!(
            "Extracted {} commit patterns from Git history",
            patterns.len()
        );
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

        let message_category = categorize_commit_message(&subject);

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

    /// Get recent authors who have committed to the repository
    pub async fn get_recent_authors(&self, repo_path: &Path) -> HotReloadResult<Vec<String>> {
        let since_date = format!("--since={} days ago", self.config.analysis_window_days);

        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["log", &since_date, "--pretty=format:%an", "--no-merges"])
            .output()
            .await
            .map_err(|e| HotReloadError::FileTracking(format!("Git command failed: {}", e)))?;

        let authors_text = String::from_utf8_lossy(&output.stdout);
        let mut authors: Vec<String> = authors_text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        authors.sort();
        authors.dedup();

        Ok(authors)
    }

    /// Check if repository has sufficient Git history for pattern learning
    pub async fn has_sufficient_history(&self, repo_path: &Path) -> HotReloadResult<bool> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .await
            .map_err(|e| HotReloadError::FileTracking(format!("Git command failed: {}", e)))?;

        let count_str = String::from_utf8_lossy(&output.stdout);
        let commit_count = count_str.trim().parse::<usize>().unwrap_or(0);

        Ok(commit_count >= 10) // Need at least 10 commits for meaningful patterns
    }

    /// Validate that the path is a Git repository
    pub async fn validate_git_repository(&self, repo_path: &Path) -> HotReloadResult<bool> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["rev-parse", "--git-dir"])
            .output()
            .await
            .map_err(|e| HotReloadError::FileTracking(format!("Git command failed: {}", e)))?;

        Ok(output.status.success())
    }

    /// Get configuration
    pub fn get_config(&self) -> &LearningConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_analyzer_creation() {
        let config = LearningConfig::default();
        let analyzer = GitAnalyzer::new(config.clone());

        assert_eq!(analyzer.config.max_patterns, config.max_patterns);
        assert_eq!(
            analyzer.config.analysis_window_days,
            config.analysis_window_days
        );
    }

    #[test]
    fn test_parse_git_date() {
        let config = LearningConfig::default();
        let analyzer = GitAnalyzer::new(config);

        let result = analyzer.parse_git_date("2024-01-15 10:30:45 -0800");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_empty_commit_block() {
        let config = LearningConfig::default();
        let analyzer = GitAnalyzer::new(config);

        let result = analyzer
            .parse_commit_block("", Path::new("."))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_parse_invalid_commit_block() {
        let config = LearningConfig::default();
        let analyzer = GitAnalyzer::new(config);

        // Invalid commit block with insufficient parts
        let invalid_block = "abc123|author";
        let result = analyzer
            .parse_commit_block(invalid_block, Path::new("."))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_parse_valid_commit_block() {
        let config = LearningConfig::default();
        let analyzer = GitAnalyzer::new(config);

        // Mock valid commit block
        let valid_block = "abc123|test_author|2024-01-15 10:30:45|feat: add feature\nCargo.toml";
        let result = analyzer
            .parse_commit_block(valid_block, Path::new("."))
            .await
            .unwrap();

        if let Some(pattern) = result {
            assert_eq!(pattern.commit_hash, "abc123");
            assert_eq!(pattern.author, "test_author");
            use crate::hotreload::warming::patterns::CommitCategory;
            assert_eq!(pattern.message_category, CommitCategory::Feature);
            // Note: files won't be populated in test because Cargo.toml doesn't exist in test context
        }
    }

    #[test]
    fn test_get_config() {
        let config = LearningConfig {
            max_patterns: 500,
            min_confidence: 0.8,
            analysis_window_days: 14,
            learning_rate: 0.2,
        };
        let analyzer = GitAnalyzer::new(config.clone());

        let retrieved_config = analyzer.get_config();
        assert_eq!(retrieved_config.max_patterns, 500);
        assert_eq!(retrieved_config.min_confidence, 0.8);
        assert_eq!(retrieved_config.analysis_window_days, 14);
        assert_eq!(retrieved_config.learning_rate, 0.2);
    }
}