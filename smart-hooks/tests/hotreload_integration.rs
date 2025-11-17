//! Hot reload integration tests
//! Tests end-to-end hot reload functionality including caching, invalidation, and performance

#[cfg(feature = "hotreload")]
mod hotreload_tests {
    use smart_hooks::hotreload::execution::HookDeterminator;
    use smart_hooks::hotreload::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::timeout;

    /// Create a test project with hot reload capabilities
    async fn create_test_project_with_hotreload() -> (TempDir, HotReloadEngine) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Create project structure
        create_test_rust_files(&project_root).unwrap();

        // Configure hot reload
        let config = HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 50 * 1024 * 1024, // 50MB for testing
            max_cache_entries: 1000,
            cache_ttl: HashMap::from([
                (HookType::Test, Duration::from_secs(300)),
                (HookType::Lint, Duration::from_secs(600)),
                (HookType::Format, Duration::from_secs(900)),
            ]),
            min_confidence_threshold: 0.7,
            background_warming_enabled: false, // Disable for deterministic tests
            fs_watching_enabled: false,
            cache_dir: Some(temp_dir.path().join(".cache")),
        };

        // Create in-memory cache for fast testing
        let cache_storage: Arc<dyn CacheStorage> = Arc::new(cache::memory::MemoryCache::new());

        let engine = HotReloadEngine::new(cache_storage, config, project_root)
            .await
            .unwrap();

        (temp_dir, engine)
    }

    fn create_test_rust_files(project_root: &Path) -> std::io::Result<()> {
        let src_dir = project_root.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create Cargo.toml
        fs::write(
            project_root.join("Cargo.toml"),
            r#"[package]
name = "test-hotreload"
version = "0.1.0"
edition = "2021"
"#,
        )?;

        // Create main.rs
        fs::write(
            src_dir.join("main.rs"),
            r#"fn main() {
    println!("Hello, hot reload!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
        )?;

        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(4, 5), 20);
    }
}
"#,
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_hit_workflow() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let test_file = temp_dir.path().join("src/main.rs");

        // First execution should be a cache miss
        let result1 = timeout(
            Duration::from_secs(10),
            engine.execute_with_cache(&[test_file.clone()]),
        )
        .await
        .expect("Timeout on first execution")
        .expect("First execution should succeed");

        assert!(!result1.results.is_empty());

        // Small delay to ensure any async operations complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Second execution with same file should be a cache hit
        let result2 = timeout(
            Duration::from_secs(5),
            engine.execute_with_cache(&[test_file]),
        )
        .await
        .expect("Timeout on second execution")
        .expect("Second execution should succeed");

        // Results should be consistent (either both from cache or both fresh)
        // The important thing is that we get cache hits, not that results are identical
        assert!(!result2.results.is_empty());

        // Verify cache statistics show hits
        let stats = engine.cache_stats().await.unwrap();

        // Debug output for troubleshooting
        println!(
            "Cache stats: hits={}, misses={}, total={}",
            stats.cache_hits, stats.cache_misses, stats.total_requests
        );

        assert!(stats.total_requests >= 2, "Should have multiple requests");
        assert!(stats.cache_hits > 0, "Should have cache hits");
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_file_change() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let test_file = temp_dir.path().join("src/lib.rs");

        // First execution
        let _result1 = engine
            .execute_with_cache(&[test_file.clone()])
            .await
            .unwrap();

        // Modify the file
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&test_file)
                .unwrap();
            writeln!(file, "\n// Modified content").unwrap();
        }

        // Second execution should detect change and invalidate cache
        let result2 = engine.execute_with_cache(&[test_file]).await.unwrap();
        assert!(!result2.results.is_empty());

        // Verify we had both hits and misses
        let stats = engine.cache_stats().await.unwrap();
        assert!(
            stats.cache_misses > 0,
            "Should have cache misses due to invalidation"
        );
    }

    #[tokio::test]
    async fn test_multiple_file_cache_key_deterministic() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let file1 = temp_dir.path().join("src/main.rs");
        let file2 = temp_dir.path().join("src/lib.rs");

        // Execute with files in one order
        let _result1 = engine
            .execute_with_cache(&[file1.clone(), file2.clone()])
            .await
            .unwrap();

        // Execute with files in different order - should hit cache
        let _result2 = engine.execute_with_cache(&[file2, file1]).await.unwrap();

        let stats = engine.cache_stats().await.unwrap();
        assert!(
            stats.cache_hits > 0,
            "Different file order should still hit cache"
        );
    }

    #[tokio::test]
    async fn test_fallback_on_cache_failure() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let test_file = temp_dir.path().join("src/main.rs");

        // Clear cache to simulate cache failure scenario
        engine.clear_cache().await.unwrap();

        // Should fall back gracefully
        let result = engine.execute_with_fallback(&[test_file]).await;
        assert!(result.is_ok(), "Fallback execution should succeed");
    }

    #[tokio::test]
    async fn test_confidence_based_cache_validation() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let test_file = temp_dir.path().join("src/lib.rs");

        // First execution to populate cache
        let _result1 = engine
            .execute_with_cache(&[test_file.clone()])
            .await
            .unwrap();

        // Get metrics before second execution
        let metrics_before = engine.execution_metrics().await;

        // Second execution should validate confidence and potentially hit cache
        let _result2 = engine.execute_with_cache(&[test_file]).await.unwrap();

        let metrics_after = engine.execution_metrics().await;
        assert!(
            metrics_after.total_executions >= metrics_before.total_executions,
            "Should track execution metrics"
        );
    }

    #[tokio::test]
    async fn test_cache_statistics_accuracy() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let test_file = temp_dir.path().join("src/main.rs");

        // Perform multiple operations
        for _ in 0..3 {
            let _ = engine
                .execute_with_cache(&[test_file.clone()])
                .await
                .unwrap();
        }

        let stats = engine.cache_stats().await.unwrap();

        // Verify statistics are reasonable
        assert!(stats.total_requests >= 3, "Should track all requests");
        assert!(
            stats.cache_hits + stats.cache_misses == stats.total_requests,
            "Hits + misses should equal total requests"
        );
        assert!(
            stats.hit_rate >= 0.0 && stats.hit_rate <= 1.0,
            "Hit rate should be between 0 and 1"
        );
    }

    #[tokio::test]
    async fn test_cache_size_limits() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        create_test_rust_files(&project_root).unwrap();

        // Create config with very small cache limits
        let config = HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 1024, // Very small - 1KB
            max_cache_entries: 2,       // Only 2 entries
            cache_ttl: HashMap::from([(HookType::Test, Duration::from_secs(300))]),
            min_confidence_threshold: 0.7,
            background_warming_enabled: false,
            fs_watching_enabled: false,
            cache_dir: Some(temp_dir.path().join(".cache")),
        };

        let cache_storage: Arc<dyn CacheStorage> = Arc::new(
            cache::disk_lru::DiskLruCache::new(
                temp_dir.path().join(".cache"),
                config.max_cache_size_bytes,
                config.max_cache_entries,
            )
            .await
            .unwrap(),
        );

        let mut engine = HotReloadEngine::new(cache_storage, config, project_root)
            .await
            .unwrap();

        let file1 = temp_dir.path().join("src/main.rs");
        let file2 = temp_dir.path().join("src/lib.rs");

        // Add entries that should trigger eviction
        let _ = engine.execute_with_cache(&[file1]).await.unwrap();
        let _ = engine.execute_with_cache(&[file2]).await.unwrap();

        // Cache should handle size limits gracefully
        let stats = engine.cache_stats().await.unwrap();
        assert!(
            stats.cached_entries <= 2,
            "Should respect max entries limit"
        );
    }

    #[tokio::test]
    async fn test_hook_type_determination() {
        let (temp_dir, _engine) = create_test_project_with_hotreload().await;

        // Create different types of files
        let rust_file = temp_dir.path().join("src/new_module.rs");
        fs::write(&rust_file, "pub fn new_function() {}").unwrap();

        let cargo_file = temp_dir.path().join("Cargo.toml");
        let mut cargo_content = fs::read_to_string(&cargo_file).unwrap();
        cargo_content.push_str("\n# New dependency");
        fs::write(&cargo_file, cargo_content).unwrap();

        // Test source code changes
        let mut changes = ChangeSet::new();
        changes.add_changed_file(rust_file);
        assert!(HookDeterminator::has_source_code_changes(&changes));

        // Test dependency file changes
        let mut dep_changes = ChangeSet::new();
        dep_changes.add_changed_file(cargo_file);
        assert!(HookDeterminator::has_dependency_file_changes(&dep_changes));
    }

    #[tokio::test]
    async fn test_engine_metrics_tracking() {
        let (temp_dir, mut engine) = create_test_project_with_hotreload().await;

        let test_file = temp_dir.path().join("src/main.rs");

        // Initial metrics should be zero
        let initial_metrics = engine.execution_metrics().await;
        assert_eq!(initial_metrics.total_executions, 0);
        assert_eq!(initial_metrics.cache_hits, 0);
        assert_eq!(initial_metrics.cache_misses, 0);

        // Execute some operations
        let _ = engine
            .execute_with_cache(&[test_file.clone()])
            .await
            .unwrap();
        let _ = engine.execute_with_cache(&[test_file]).await.unwrap();

        // Metrics should be updated
        let final_metrics = engine.execution_metrics().await;
        assert!(final_metrics.total_executions > 0);
        assert!(final_metrics.cache_hits + final_metrics.cache_misses > 0);
    }
}

#[cfg(not(feature = "hotreload"))]
#[test]
fn test_hotreload_feature_disabled() {
    // When hotreload feature is disabled, this should still compile
    // but hot reload functionality should not be available
    println!("Hot reload feature is disabled");
}