//! File system watcher for real-time change detection

use super::ChangeEvent;
use crate::hotreload::{HotReloadError, HotReloadResult};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

/// File system watcher for detecting real-time file changes
pub struct FileSystemWatcher {
    watcher: Option<RecommendedWatcher>,
    change_sender: mpsc::UnboundedSender<ChangeEvent>,
    project_root: std::path::PathBuf,
    watch_patterns: Vec<String>,
    ignore_patterns: Vec<String>,
}

impl FileSystemWatcher {
    /// Create new file system watcher
    pub fn new(
        change_sender: mpsc::UnboundedSender<ChangeEvent>,
        project_root: &Path,
    ) -> HotReloadResult<Self> {
        let watch_patterns = vec![
            "**/*.rs".to_string(),
            "**/*.ts".to_string(),
            "**/*.tsx".to_string(),
            "**/*.js".to_string(),
            "**/*.jsx".to_string(),
            "**/*.py".to_string(),
            "**/*.php".to_string(),
            "**/*.go".to_string(),
            "**/*.java".to_string(),
            "**/*.cs".to_string(),
            "**/Cargo.toml".to_string(),
            "**/package.json".to_string(),
            "**/requirements.txt".to_string(),
            "**/composer.json".to_string(),
            "**/go.mod".to_string(),
        ];

        let ignore_patterns = vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            "**/__pycache__/**".to_string(),
            "**/vendor/**".to_string(),
            "**/.venv/**".to_string(),
            "**/dist/**".to_string(),
            "**/build/**".to_string(),
            "**/*.log".to_string(),
            "**/.DS_Store".to_string(),
        ];

        Ok(Self {
            watcher: None,
            change_sender,
            project_root: project_root.to_path_buf(),
            watch_patterns,
            ignore_patterns,
        })
    }

    /// Start watching the project directory
    pub async fn start_watching(&mut self) -> HotReloadResult<()> {
        let sender = self.change_sender.clone();
        let ignore_patterns = self.ignore_patterns.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        let change_event = ChangeEvent::from(event.clone());

                        // Check if file should be ignored
                        if !Self::should_ignore(&change_event.path, &ignore_patterns) {
                            if let Err(e) = sender.send(change_event) {
                                tracing::warn!("Failed to send file change event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("File watcher error: {}", e);
                    }
                }
            })
            .map_err(|e| {
                HotReloadError::FileTracking(format!("Failed to create watcher: {}", e))
            })?;

        watcher
            .watch(&self.project_root, RecursiveMode::Recursive)
            .map_err(|e| {
                HotReloadError::FileTracking(format!("Failed to start watching: {}", e))
            })?;

        self.watcher = Some(watcher);

        tracing::info!(
            "File system watcher started for: {}",
            self.project_root.display()
        );

        Ok(())
    }

    /// Stop watching
    pub async fn stop_watching(&mut self) {
        if let Some(watcher) = self.watcher.take() {
            drop(watcher); // Dropping the watcher stops it
            tracing::info!("File system watcher stopped");
        }
    }

    /// Check if a file should be ignored based on patterns
    fn should_ignore(file_path: &Path, ignore_patterns: &[String]) -> bool {
        let path_str = file_path.to_string_lossy();

        for pattern in ignore_patterns {
            // Handle glob patterns
            if pattern.contains("**") {
                // Remove ** and get the remaining pattern
                let pattern_without_wildcard = pattern.replace("**/", "").replace("/**", "");

                // For patterns like **/*.log, check if path ends with .log
                if pattern.starts_with("**/") && pattern.contains("*") {
                    let extension_pattern = pattern.trim_start_matches("**/");
                    if extension_pattern.starts_with("*.") {
                        let extension = extension_pattern.trim_start_matches("*.");
                        if path_str.ends_with(&format!(".{}", extension)) {
                            return true;
                        }
                    }
                }

                // For patterns like **/target/**, check if path contains the directory
                if path_str.contains(&pattern_without_wildcard.trim_matches('/')) {
                    return true;
                }
            } else if path_str.ends_with(pattern) {
                return true;
            }
        }

        false
    }

    /// Add custom watch pattern
    pub fn add_watch_pattern(&mut self, pattern: String) {
        if !self.watch_patterns.contains(&pattern) {
            self.watch_patterns.push(pattern);
        }
    }

    /// Add custom ignore pattern  
    pub fn add_ignore_pattern(&mut self, pattern: String) {
        if !self.ignore_patterns.contains(&pattern) {
            self.ignore_patterns.push(pattern);
        }
    }

    /// Get current watch patterns
    pub fn watch_patterns(&self) -> &[String] {
        &self.watch_patterns
    }

    /// Get current ignore patterns
    pub fn ignore_patterns(&self) -> &[String] {
        &self.ignore_patterns
    }
}

/// Debounced file watcher that groups rapid changes
pub struct DebouncedFileWatcher {
    base_watcher: FileSystemWatcher,
    debounce_duration: Duration,
}

impl DebouncedFileWatcher {
    /// Create new debounced watcher
    pub fn new(
        change_sender: mpsc::UnboundedSender<ChangeEvent>,
        project_root: &Path,
        debounce_duration: Duration,
    ) -> HotReloadResult<Self> {
        let base_watcher = FileSystemWatcher::new(change_sender, project_root)?;

        Ok(Self {
            base_watcher,
            debounce_duration,
        })
    }

    /// Start watching with debouncing
    pub async fn start_watching_debounced(
        &mut self,
    ) -> HotReloadResult<mpsc::UnboundedReceiver<Vec<ChangeEvent>>> {
        let (debounced_sender, debounced_receiver) = mpsc::unbounded_channel();
        let (raw_sender, mut raw_receiver) = mpsc::unbounded_channel();

        // Replace sender in base watcher
        self.base_watcher.change_sender = raw_sender;
        self.base_watcher.start_watching().await?;

        // Start debouncing task
        let debounce_duration = self.debounce_duration;
        tokio::spawn(async move {
            let mut pending_events: Vec<ChangeEvent> = Vec::new();
            let mut debounce_timer = time::interval(debounce_duration);

            loop {
                tokio::select! {
                    // New event received
                    event = raw_receiver.recv() => {
                        match event {
                            Some(change_event) => {
                                pending_events.push(change_event);
                            }
                            None => break, // Channel closed
                        }
                    }

                    // Timer tick - send accumulated events
                    _ = debounce_timer.tick() => {
                        if !pending_events.is_empty() {
                            let events_to_send = std::mem::take(&mut pending_events);
                            if let Err(_) = debounced_sender.send(events_to_send) {
                                break; // Receiver dropped
                            }
                        }
                    }
                }
            }
        });

        Ok(debounced_receiver)
    }

    /// Stop watching
    pub async fn stop_watching(&mut self) {
        self.base_watcher.stop_watching().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::time;

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let (sender, _receiver) = mpsc::unbounded_channel();

        let watcher = FileSystemWatcher::new(sender, temp_dir.path());
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_should_ignore_patterns() {
        let ignore_patterns = vec![
            "**/target/**".to_string(),
            "**/.git/**".to_string(),
            "**/*.log".to_string(),
        ];

        assert!(FileSystemWatcher::should_ignore(
            Path::new("project/target/debug/main"),
            &ignore_patterns
        ));

        assert!(FileSystemWatcher::should_ignore(
            Path::new("project/.git/config"),
            &ignore_patterns
        ));

        assert!(FileSystemWatcher::should_ignore(
            Path::new("app.log"),
            &ignore_patterns
        ));

        assert!(!FileSystemWatcher::should_ignore(
            Path::new("src/main.rs"),
            &ignore_patterns
        ));
    }

    #[tokio::test]
    async fn test_debounced_watcher() {
        let temp_dir = TempDir::new().unwrap();
        let (sender, _receiver) = mpsc::unbounded_channel();

        let mut debounced_watcher =
            DebouncedFileWatcher::new(sender, temp_dir.path(), Duration::from_millis(100)).unwrap();

        // This is a basic test - in real usage, we'd test the debouncing behavior
        // by creating/modifying files and checking the grouped events
        assert!(debounced_watcher.start_watching_debounced().await.is_ok());
    }
}