//! File change tracking for intelligent cache invalidation

use crate::hotreload::{HotReloadError, HotReloadResult};
use crate::dependency::DependencyGraph;
use notify::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, RwLock};

pub mod content_hash;
pub mod fs_watcher;

pub use content_hash::ContentHasher;
pub use fs_watcher::FileSystemWatcher;

/// File change event from file system watching
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub path: PathBuf,
    pub event_type: ChangeEventType,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum ChangeEventType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

impl From<Event> for ChangeEvent {
    fn from(event: Event) -> Self {
        let timestamp = SystemTime::now();
        let path = event.paths.first().cloned().unwrap_or_default();

        let event_type = match event.kind {
            notify::EventKind::Create(_) => ChangeEventType::Created,
            notify::EventKind::Modify(_) => ChangeEventType::Modified,
            notify::EventKind::Remove(_) => ChangeEventType::Deleted,
            _ => ChangeEventType::Modified, // Default to modified
        };

        ChangeEvent {
            path,
            event_type,
            timestamp,
        }
    }
}

/// Set of changes detected in project files
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// Files that were directly modified
    pub changed_files: Vec<PathBuf>,
    /// Files affected by dependency relationships
    pub affected_files: Vec<PathBuf>,
    /// Tests that should be run due to changes
    pub affected_tests: Vec<PathBuf>,
    /// Confidence score for the change analysis
    pub confidence: f64,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            changed_files: Vec::new(),
            affected_files: Vec::new(),
            affected_tests: Vec::new(),
            confidence: 1.0,
        }
    }

    pub fn add_changed_file(&mut self, file: PathBuf) {
        if !self.changed_files.contains(&file) {
            self.changed_files.push(file);
        }
    }

    pub fn extend_affected(&mut self, affected: Vec<PathBuf>) {
        for file in affected {
            if !self.affected_files.contains(&file) {
                self.affected_files.push(file);
            }
        }
    }

    pub fn all_files(&self) -> Vec<PathBuf> {
        let mut all_files = self.changed_files.clone();
        all_files.extend(self.affected_files.clone());
        all_files.sort();
        all_files.dedup();
        all_files
    }

    pub fn is_empty(&self) -> bool {
        self.changed_files.is_empty() && self.affected_files.is_empty()
    }
}

/// File change tracker with content hashing and dependency analysis
pub struct FileChangeTracker {
    /// Content hashes for fast change detection
    content_hashes: Arc<RwLock<HashMap<PathBuf, String>>>,
    /// Smart-hooks dependency graph
    dependency_graph: Option<DependencyGraph>,
    /// File system watcher for real-time events
    fs_watcher: Option<FileSystemWatcher>,
    /// Change event channel
    change_sender: Option<mpsc::UnboundedSender<ChangeEvent>>,
    /// Content hasher utility
    pub hasher: ContentHasher,
    /// Project root directory
    project_root: PathBuf,
}

impl FileChangeTracker {
    /// Create new file change tracker
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            content_hashes: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: None,
            fs_watcher: None,
            change_sender: None,
            hasher: ContentHasher::new(),
            project_root,
        }
    }

    /// Initialize with dependency graph for smarter change detection
    pub fn with_dependency_graph(mut self, dependency_graph: DependencyGraph) -> Self {
        self.dependency_graph = Some(dependency_graph);
        self
    }

    /// Start file system watching for real-time change detection
    pub async fn start_watching(
        &mut self,
    ) -> HotReloadResult<mpsc::UnboundedReceiver<ChangeEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut fs_watcher = FileSystemWatcher::new(tx.clone(), &self.project_root)?;
        fs_watcher.start_watching().await?;

        self.fs_watcher = Some(fs_watcher);
        self.change_sender = Some(tx);

        Ok(rx)
    }

    /// Stop file system watching
    pub async fn stop_watching(&mut self) {
        if let Some(mut watcher) = self.fs_watcher.take() {
            watcher.stop_watching().await;
        }
        self.change_sender = None;
    }

    /// Detect changes in specified files and compute affected file set
    pub async fn detect_changes(&mut self, files: &[PathBuf]) -> HotReloadResult<ChangeSet> {
        let mut changes = ChangeSet::new();

        for file in files {
            if self.has_file_changed(file).await? {
                changes.add_changed_file(file.clone());

                // Use dependency graph to find affected files
                if let Some(ref dependency_graph) = self.dependency_graph {
                    let affected = self
                        .find_affected_files_via_dependencies(file, dependency_graph)
                        .await?;
                    changes.extend_affected(affected);
                }
            }
        }

        // Update hash cache for all processed files
        self.update_hash_cache(files).await?;

        Ok(changes)
    }

    /// Check if a single file has changed based on content hash
    pub async fn has_file_changed(&self, file: &Path) -> HotReloadResult<bool> {
        let current_hash = self.hasher.compute_file_hash(file).await?;
        let cached_hash = self.content_hashes.read().await.get(file).cloned();

        Ok(cached_hash.map(|h| h != current_hash).unwrap_or(true))
    }

    /// Update content hash cache for files
    async fn update_hash_cache(&self, files: &[PathBuf]) -> HotReloadResult<()> {
        let mut hash_cache = self.content_hashes.write().await;

        for file in files {
            let hash = self.hasher.compute_file_hash(file).await?;
            hash_cache.insert(file.clone(), hash);
        }

        Ok(())
    }

    /// Find affected files using dependency analysis
    async fn find_affected_files_via_dependencies(
        &self,
        _changed_file: &Path,
        _dependency_graph: &DependencyGraph,
    ) -> HotReloadResult<Vec<PathBuf>> {
        // This would integrate with existing smart-hooks dependency analysis
        // For now, return empty vector - will be implemented when integrating
        // with the dependency analysis system
        Ok(Vec::new())
    }

    /// Get current content hash for a file
    pub async fn get_content_hash(&self, file: &Path) -> HotReloadResult<Option<String>> {
        Ok(self.content_hashes.read().await.get(file).cloned())
    }

    /// Preload content hashes for files
    pub async fn preload_hashes(&mut self, files: &[PathBuf]) -> HotReloadResult<()> {
        let mut hash_cache = self.content_hashes.write().await;

        for file in files {
            if !hash_cache.contains_key(file) {
                let hash = self.hasher.compute_file_hash(file).await?;
                hash_cache.insert(file.clone(), hash);
            }
        }

        Ok(())
    }

    /// Clear hash cache for specific files
    pub async fn invalidate_hashes(&self, files: &[PathBuf]) {
        let mut hash_cache = self.content_hashes.write().await;

        for file in files {
            hash_cache.remove(file);
        }
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let hash_cache = self.content_hashes.read().await;
        (hash_cache.len(), hash_cache.capacity())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    async fn create_test_tracker() -> (FileChangeTracker, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let tracker = FileChangeTracker::new(temp_dir.path().to_path_buf());
        (tracker, temp_dir)
    }

    fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_detect_changes_with_new_file() {
        let (mut tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        let changes = tracker.detect_changes(&[file_path.clone()]).await.unwrap();

        assert_eq!(changes.changed_files.len(), 1);
        assert_eq!(changes.changed_files[0], file_path);
        assert!(changes.affected_files.is_empty());
    }

    #[tokio::test]
    async fn test_detect_changes_with_unchanged_file() {
        let (mut tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        // First detection should see it as changed (new file)
        let changes1 = tracker.detect_changes(&[file_path.clone()]).await.unwrap();
        assert_eq!(changes1.changed_files.len(), 1);

        // Second detection should see no changes
        let changes2 = tracker.detect_changes(&[file_path]).await.unwrap();
        assert!(changes2.changed_files.is_empty());
    }

    #[tokio::test]
    async fn test_detect_changes_with_modified_file() {
        let (mut tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        // Initial detection
        tracker.detect_changes(&[file_path.clone()]).await.unwrap();

        // Modify the file
        std::fs::write(&file_path, "fn main() { println!(\"changed\"); }").unwrap();

        // Should detect the change
        let changes = tracker.detect_changes(&[file_path.clone()]).await.unwrap();
        assert_eq!(changes.changed_files.len(), 1);
        assert_eq!(changes.changed_files[0], file_path);
    }

    #[tokio::test]
    async fn test_has_file_changed() {
        let (tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        // New file should be detected as changed
        let changed = tracker.has_file_changed(&file_path).await.unwrap();
        assert!(changed);
    }

    #[tokio::test]
    async fn test_preload_hashes() {
        let (mut tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        // Preload hashes
        tracker.preload_hashes(&[file_path.clone()]).await.unwrap();

        // File should not be detected as changed after preload
        let changed = tracker.has_file_changed(&file_path).await.unwrap();
        assert!(!changed);
    }

    #[tokio::test]
    async fn test_invalidate_hashes() {
        let (mut tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        // Preload and then invalidate
        tracker.preload_hashes(&[file_path.clone()]).await.unwrap();
        tracker.invalidate_hashes(&[file_path.clone()]).await;

        // File should be detected as changed after invalidation
        let changed = tracker.has_file_changed(&file_path).await.unwrap();
        assert!(changed);
    }

    #[tokio::test]
    async fn test_get_content_hash() {
        let (tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        // Initially no hash
        let hash = tracker.get_content_hash(&file_path).await.unwrap();
        assert!(hash.is_none());

        // After computing hash
        let computed_hash = tracker.hasher.compute_file_hash(&file_path).await.unwrap();
        {
            let mut cache = tracker.content_hashes.write().await;
            cache.insert(file_path.clone(), computed_hash.clone());
        }

        let retrieved_hash = tracker.get_content_hash(&file_path).await.unwrap();
        assert_eq!(retrieved_hash, Some(computed_hash));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let (mut tracker, temp_dir) = create_test_tracker().await;

        let file_path = create_test_file(temp_dir.path(), "test.rs", "fn main() {}");

        let (initial_len, _) = tracker.cache_stats().await;
        assert_eq!(initial_len, 0);

        tracker.preload_hashes(&[file_path]).await.unwrap();

        let (after_len, _) = tracker.cache_stats().await;
        assert_eq!(after_len, 1);
    }

    #[tokio::test]
    async fn test_changeset_operations() {
        let mut changeset = ChangeSet::new();

        assert!(changeset.is_empty());

        let file1 = PathBuf::from("src/file1.rs");
        let file2 = PathBuf::from("src/file2.rs");

        changeset.add_changed_file(file1.clone());
        changeset.extend_affected(vec![file2.clone()]);

        assert!(!changeset.is_empty());
        assert_eq!(changeset.changed_files.len(), 1);
        assert_eq!(changeset.affected_files.len(), 1);

        let all_files = changeset.all_files();
        assert_eq!(all_files.len(), 2);
        assert!(all_files.contains(&file1));
        assert!(all_files.contains(&file2));
    }

    #[tokio::test]
    async fn test_changeset_deduplication() {
        let mut changeset = ChangeSet::new();

        let file1 = PathBuf::from("src/file1.rs");

        // Add same file multiple times
        changeset.add_changed_file(file1.clone());
        changeset.add_changed_file(file1.clone());
        changeset.extend_affected(vec![file1.clone()]);

        // Should be deduplicated
        let all_files = changeset.all_files();
        assert_eq!(all_files.len(), 1);
        assert_eq!(all_files[0], file1);
    }
}