//! Disk operation utilities for cache management
use crate::hotreload::{cache::CacheKey, HotReloadError, HotReloadResult};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Disk operations manager for cache files
pub struct DiskOpsManager {
    cache_dir: PathBuf,
}

impl DiskOpsManager {
    /// Create new disk operations manager
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get cache directory path
    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Generate file path for cache key
    pub fn cache_file_path(&self, key: &CacheKey) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", key.as_str()))
    }

    /// Ensure cache directory exists
    pub async fn ensure_cache_dir(&self) -> HotReloadResult<()> {
        fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(HotReloadError::Io)
    }

    /// Read cache file and return raw bytes
    pub async fn read_cache_file(&self, key: &CacheKey) -> HotReloadResult<Vec<u8>> {
        let file_path = self.cache_file_path(key);
        fs::read(&file_path).await.map_err(HotReloadError::Io)
    }

    /// Write data to cache file
    pub async fn write_cache_file(&self, key: &CacheKey, data: &[u8]) -> HotReloadResult<()> {
        self.ensure_cache_dir().await?;
        let file_path = self.cache_file_path(key);
        fs::write(&file_path, data)
            .await
            .map_err(HotReloadError::Io)
    }

    /// Remove cache file
    pub async fn remove_cache_file(&self, key: &CacheKey) -> HotReloadResult<()> {
        let file_path = self.cache_file_path(key);
        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(HotReloadError::Io)?;
        }
        Ok(())
    }

    /// Check if cache file exists
    pub async fn cache_file_exists(&self, key: &CacheKey) -> bool {
        let file_path = self.cache_file_path(key);
        file_path.exists()
    }

    /// Get size of cache file in bytes
    pub async fn cache_file_size(&self, key: &CacheKey) -> HotReloadResult<u64> {
        let file_path = self.cache_file_path(key);
        let metadata = fs::metadata(&file_path).await.map_err(HotReloadError::Io)?;
        Ok(metadata.len())
    }

    /// Clean up orphaned cache files (files without entries in tracker)
    pub async fn cleanup_orphaned_files(&self, valid_keys: &[CacheKey]) -> HotReloadResult<usize> {
        let valid_filenames: std::collections::HashSet<String> = valid_keys
            .iter()
            .map(|key| format!("{}.cache", key.as_str()))
            .collect();

        let mut removed_count = 0;
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(HotReloadError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(HotReloadError::Io)? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".cache") && !valid_filenames.contains(filename) {
                    if let Err(e) = fs::remove_file(&path).await {
                        tracing::warn!("Failed to remove orphaned cache file {:?}: {}", path, e);
                    } else {
                        removed_count += 1;
                        tracing::debug!("Removed orphaned cache file: {:?}", path);
                    }
                }
            }
        }

        Ok(removed_count)
    }

    /// Calculate total size of all cache files
    #[allow(dead_code)]
    pub async fn total_cache_size(&self) -> HotReloadResult<u64> {
        let mut total_size = 0u64;
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(HotReloadError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(HotReloadError::Io)? {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.ends_with(".cache") {
                        if let Ok(metadata) = fs::metadata(&path).await {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// List all cache files in directory
    pub async fn list_cache_files(&self) -> HotReloadResult<Vec<CacheKey>> {
        let mut cache_keys = Vec::new();
        let mut entries = fs::read_dir(&self.cache_dir)
            .await
            .map_err(HotReloadError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(HotReloadError::Io)? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(key_str) = filename.strip_suffix(".cache") {
                    cache_keys.push(CacheKey::new(key_str.to_string()));
                }
            }
        }

        Ok(cache_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_disk_ops() -> (DiskOpsManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let disk_ops = DiskOpsManager::new(temp_dir.path().to_path_buf());
        (disk_ops, temp_dir)
    }

    #[tokio::test]
    async fn test_cache_file_path_generation() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let key = CacheKey::new("test_key".to_string());
        let path = disk_ops.cache_file_path(&key);

        assert!(path.to_string_lossy().ends_with("test_key.cache"));
    }

    #[tokio::test]
    async fn test_ensure_cache_dir() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();

        let result = disk_ops.ensure_cache_dir().await;
        assert!(result.is_ok());
        assert!(disk_ops.cache_dir().exists());
    }

    #[tokio::test]
    async fn test_write_and_read_cache_file() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let key = CacheKey::new("test_key".to_string());
        let test_data = b"test cache data";

        // Write file
        let result = disk_ops.write_cache_file(&key, test_data).await;
        assert!(result.is_ok());

        // Check file exists
        assert!(disk_ops.cache_file_exists(&key).await);

        // Read file
        let read_result = disk_ops.read_cache_file(&key).await;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_data);
    }

    #[tokio::test]
    async fn test_remove_cache_file() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let key = CacheKey::new("test_key".to_string());
        let test_data = b"test cache data";

        // Write and verify file exists
        disk_ops.write_cache_file(&key, test_data).await.unwrap();
        assert!(disk_ops.cache_file_exists(&key).await);

        // Remove file
        let result = disk_ops.remove_cache_file(&key).await;
        assert!(result.is_ok());
        assert!(!disk_ops.cache_file_exists(&key).await);
    }

    #[tokio::test]
    async fn test_cache_file_size() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let key = CacheKey::new("test_key".to_string());
        let test_data = b"test cache data with specific length";

        disk_ops.write_cache_file(&key, test_data).await.unwrap();

        let size_result = disk_ops.cache_file_size(&key).await;
        assert!(size_result.is_ok());
        assert_eq!(size_result.unwrap(), test_data.len() as u64);
    }

    #[tokio::test]
    async fn test_list_cache_files() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let keys = vec![
            CacheKey::new("key1".to_string()),
            CacheKey::new("key2".to_string()),
            CacheKey::new("key3".to_string()),
        ];
        let test_data = b"test data";

        // Write cache files
        for key in &keys {
            disk_ops.write_cache_file(key, test_data).await.unwrap();
        }

        // List files
        let listed_keys = disk_ops.list_cache_files().await.unwrap();
        assert_eq!(listed_keys.len(), 3);

        // Check all keys are present (order may differ)
        for key in &keys {
            assert!(listed_keys.contains(key));
        }
    }

    #[tokio::test]
    async fn test_cleanup_orphaned_files() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let valid_keys = vec![
            CacheKey::new("valid1".to_string()),
            CacheKey::new("valid2".to_string()),
        ];
        let orphaned_keys = vec![
            CacheKey::new("orphan1".to_string()),
            CacheKey::new("orphan2".to_string()),
        ];
        let test_data = b"test data";

        // Write both valid and orphaned files
        for key in &valid_keys {
            disk_ops.write_cache_file(key, test_data).await.unwrap();
        }
        for key in &orphaned_keys {
            disk_ops.write_cache_file(key, test_data).await.unwrap();
        }

        // Cleanup orphaned files
        let cleanup_result = disk_ops.cleanup_orphaned_files(&valid_keys).await;
        assert!(cleanup_result.is_ok());
        assert_eq!(cleanup_result.unwrap(), 2); // 2 orphaned files removed

        // Verify valid files still exist
        for key in &valid_keys {
            assert!(disk_ops.cache_file_exists(key).await);
        }

        // Verify orphaned files are gone
        for key in &orphaned_keys {
            assert!(!disk_ops.cache_file_exists(key).await);
        }
    }

    #[tokio::test]
    async fn test_total_cache_size() {
        let (disk_ops, _temp_dir) = create_test_disk_ops();
        let keys = vec![
            CacheKey::new("key1".to_string()),
            CacheKey::new("key2".to_string()),
        ];
        let test_data1 = b"short";
        let test_data2 = b"much longer test data";

        // Write files of different sizes
        disk_ops
            .write_cache_file(&keys[0], test_data1)
            .await
            .unwrap();
        disk_ops
            .write_cache_file(&keys[1], test_data2)
            .await
            .unwrap();

        let total_size = disk_ops.total_cache_size().await.unwrap();
        let expected_size = (test_data1.len() + test_data2.len()) as u64;
        assert_eq!(total_size, expected_size);
    }
}