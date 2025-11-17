//! Content-based file hashing for change detection

use crate::hotreload::{HotReloadError, HotReloadResult};
use blake3::Hasher;
use std::path::Path;
use tokio::fs;

/// Content hasher using BLAKE3 for fast and secure hashing
pub struct ContentHasher {
    /// Buffer size for reading files (64KB)
    buffer_size: usize,
}

impl ContentHasher {
    /// Create new content hasher
    pub fn new() -> Self {
        Self {
            buffer_size: 64 * 1024, // 64KB buffer
        }
    }

    /// Create hasher with custom buffer size
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self { buffer_size }
    }

    /// Compute BLAKE3 hash of file contents
    pub async fn compute_file_hash(&self, file_path: &Path) -> HotReloadResult<String> {
        if !file_path.exists() {
            return Err(HotReloadError::FileTracking(format!(
                "File does not exist: {}",
                file_path.display()
            )));
        }

        let content = fs::read(file_path).await.map_err(|e| {
            HotReloadError::FileTracking(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let hash = blake3::hash(&content);
        Ok(hash.to_hex().to_string())
    }

    /// Compute hash of file contents streaming (for large files)
    pub async fn compute_file_hash_streaming(&self, file_path: &Path) -> HotReloadResult<String> {
        use tokio::io::{AsyncReadExt, BufReader};

        let file = fs::File::open(file_path).await.map_err(|e| {
            HotReloadError::FileTracking(format!(
                "Failed to open file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let mut reader = BufReader::new(file);
        let mut hasher = Hasher::new();
        let mut buffer = vec![0u8; self.buffer_size];

        loop {
            let bytes_read = reader.read(&mut buffer).await.map_err(|e| {
                HotReloadError::FileTracking(format!(
                    "Failed to read from file {}: {}",
                    file_path.display(),
                    e
                ))
            })?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Compute hash of multiple files combined
    pub async fn compute_combined_hash(&self, file_paths: &[&Path]) -> HotReloadResult<String> {
        let mut hasher = Hasher::new();

        for file_path in file_paths {
            let file_hash = self.compute_file_hash(file_path).await?;
            hasher.update(file_hash.as_bytes());

            // Include file path in hash to differentiate same content in different files
            hasher.update(file_path.to_string_lossy().as_bytes());
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Compute hash of string content
    pub fn compute_string_hash(&self, content: &str) -> String {
        blake3::hash(content.as_bytes()).to_hex().to_string()
    }

    /// Compute hash with file metadata (size, modified time)
    pub async fn compute_hash_with_metadata(&self, file_path: &Path) -> HotReloadResult<String> {
        let content_hash = self.compute_file_hash(file_path).await?;
        let metadata = fs::metadata(file_path).await.map_err(|e| {
            HotReloadError::FileTracking(format!(
                "Failed to read metadata for {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let mut hasher = Hasher::new();
        hasher.update(content_hash.as_bytes());

        // Include file size
        hasher.update(&metadata.len().to_le_bytes());

        // Include modified time
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                hasher.update(&duration.as_secs().to_le_bytes());
                hasher.update(&duration.subsec_nanos().to_le_bytes());
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Verify if file matches expected hash
    pub async fn verify_file_hash(
        &self,
        file_path: &Path,
        expected_hash: &str,
    ) -> HotReloadResult<bool> {
        let actual_hash = self.compute_file_hash(file_path).await?;
        Ok(actual_hash == expected_hash)
    }
}

impl Default for ContentHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_compute_file_hash() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let hasher = ContentHasher::new();
        let hash = hasher.compute_file_hash(temp_file.path()).await.unwrap();

        // Hash should be deterministic
        let hash2 = hasher.compute_file_hash(temp_file.path()).await.unwrap();
        assert_eq!(hash, hash2);

        // Different content should produce different hash
        let mut temp_file2 = NamedTempFile::new().unwrap();
        temp_file2.write_all(b"different content").unwrap();

        let hash3 = hasher.compute_file_hash(temp_file2.path()).await.unwrap();
        assert_ne!(hash, hash3);
    }

    #[tokio::test]
    async fn test_compute_string_hash() {
        let hasher = ContentHasher::new();

        let hash1 = hasher.compute_string_hash("test");
        let hash2 = hasher.compute_string_hash("test");
        let hash3 = hasher.compute_string_hash("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_combined_hash() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        temp_file1.write_all(b"content1").unwrap();

        let mut temp_file2 = NamedTempFile::new().unwrap();
        temp_file2.write_all(b"content2").unwrap();

        let hasher = ContentHasher::new();

        let combined_hash = hasher
            .compute_combined_hash(&[temp_file1.path(), temp_file2.path()])
            .await
            .unwrap();

        // Different order should produce different hash
        let combined_hash2 = hasher
            .compute_combined_hash(&[temp_file2.path(), temp_file1.path()])
            .await
            .unwrap();

        assert_ne!(combined_hash, combined_hash2);
    }
}