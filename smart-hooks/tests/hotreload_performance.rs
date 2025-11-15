//! Performance tests for hot reload functionality
//! Validates cache hit rates, execution times, and system efficiency

#[cfg(feature = "hotreload")]
mod hotreload_performance_tests {
    use smart_hooks::hotreload::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    /// Performance test configuration
    struct PerformanceTestConfig {
        pub target_cache_hit_rate: f64,
        pub max_cache_miss_time: Duration,
        pub max_cache_hit_time: Duration,
        pub min_cache_efficiency: f64,
    }

    impl Default for PerformanceTestConfig {
        fn default() -> Self {
            Self {
                target_cache_hit_rate: 0.80,                    // 80% hit rate target
                max_cache_miss_time: Duration::from_secs(10),   // 10s max for miss
                max_cache_hit_time: Duration::from_millis(100), // 100ms max for hit
                min_cache_efficiency: 0.85,                     // 85% efficiency minimum
            }
        }
    }

    /// Create a performance test project with many files
    async fn create_performance_test_project() -> (TempDir, HotReloadEngine) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Create large project structure for performance testing
        create_large_rust_project(&project_root, 20).unwrap();

        // Configure for performance testing
        let config = HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 500 * 1024 * 1024, // 500MB for large testing
            max_cache_entries: 50000,
            cache_ttl: HashMap::from([
                (HookType::Test, Duration::from_secs(3600)),    // 1 hour
                (HookType::Lint, Duration::from_secs(7200)),    // 2 hours
                (HookType::Format, Duration::from_secs(14400)), // 4 hours
            ]),
            min_confidence_threshold: 0.75,
            background_warming_enabled: true,
            fs_watching_enabled: false, // Disabled for deterministic tests
            cache_dir: Some(temp_dir.path().join(".perf-cache")),
        };

        // Use disk cache for realistic performance testing
        let cache_storage: Arc<dyn CacheStorage> = Arc::new(
            cache::disk_lru::DiskLruCache::new(
                temp_dir.path().join(".perf-cache"),
                config.max_cache_size_bytes,
                config.max_cache_entries,
            )
            .await
            .unwrap(),
        );

        let engine = HotReloadEngine::new(cache_storage, config, project_root)
            .await
            .unwrap();

        (temp_dir, engine)
    }

    fn create_large_rust_project(
        project_root: &std::path::Path,
        num_modules: usize,
    ) -> std::io::Result<()> {
        let src_dir = project_root.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create Cargo.toml
        fs::write(
            project_root.join("Cargo.toml"),
            r#"[package]
name = "performance-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
        )?;

        // Create main lib.rs that imports all modules
        let mut lib_content = String::new();
        lib_content.push_str("//! Performance test project with many modules\n\n");

        for i in 0..num_modules {
            lib_content.push_str(&format!("pub mod module_{};\n", i));
        }

        lib_content.push_str("\n");
        lib_content.push_str(&format!(
            r#"pub fn process_all_modules() -> Vec<String> {{
    vec![
{}
    ]
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_process_all_modules() {{
        let result = process_all_modules();
        assert_eq!(result.len(), {});
    }}
}}
"#,
            (0..num_modules)
                .map(|i| format!("        module_{}::process(),", i))
                .collect::<Vec<_>>()
                .join("\n"),
            num_modules
        ));

        fs::write(src_dir.join("lib.rs"), lib_content)?;

        // Create individual modules
        for i in 0..num_modules {
            let module_content = format!(
                r#"//! Module {} - Performance test module

use serde::{{Serialize, Deserialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data{} {{
    pub id: u32,
    pub name: String,
    pub values: Vec<i32>,
}}

impl Data{} {{
    pub fn new() -> Self {{
        Self {{
            id: {},
            name: "module_{}".to_string(),
            values: vec![1, 2, 3, 4, 5],
        }}
    }}

    pub fn compute(&self) -> i32 {{
        self.values.iter().sum::<i32>() + self.id as i32
    }}

    pub fn format_output(&self) -> String {{
        format!("{{}} [{}]: {{}}", self.name, self.compute())
    }}
}}

pub fn process() -> String {{
    let data = Data{}::new();
    data.format_output()
}}

pub fn heavy_computation() -> Vec<u64> {{
    // Simulate some computation work
    (0..1000).map(|x| (x * {} + 42) as u64).collect()
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_data_{}_creation() {{
        let data = Data{}::new();
        assert_eq!(data.id, {});
        assert_eq!(data.name, "module_{}");
    }}

    #[test]
    fn test_data_{}_computation() {{
        let data = Data{}::new();
        let result = data.compute();
        assert_eq!(result, 15 + {}); // sum of [1,2,3,4,5] + id
    }}

    #[test]
    fn test_process_function() {{
        let result = process();
        assert!(result.contains("module_{}"));
    }}

    #[test]
    fn test_heavy_computation() {{
        let result = heavy_computation();
        assert_eq!(result.len(), 1000);
        assert_eq!(result[0], 42);
        assert_eq!(result[1], {} + 42);
    }}
}}
"#,
                i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i
            );

            fs::write(src_dir.join(format!("module_{}.rs", i)), module_content)?;
        }

        Ok(())
    }

    /// Measure execution time with detailed timing breakdown
    async fn measure_execution_time<F, R>(operation: F) -> (R, Duration)
    where
        F: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        (result, duration)
    }

    #[tokio::test]
    async fn test_cache_hit_rate_performance() {
        let (temp_dir, mut engine) = create_performance_test_project().await;
        let config = PerformanceTestConfig::default();

        // Collect all module files for testing
        let src_dir = temp_dir.path().join("src");
        let mut test_files = Vec::new();

        for i in 0..20 {
            test_files.push(src_dir.join(format!("module_{}.rs", i)));
        }
        test_files.push(src_dir.join("lib.rs"));

        // Phase 1: Cold execution (should be cache misses)
        let mut miss_times = Vec::new();
        for file in &test_files[..5] {
            // Test first 5 files
            let (result, duration) =
                measure_execution_time(engine.execute_with_cache(&[file.clone()])).await;

            assert!(result.is_ok(), "Cold execution should succeed");
            miss_times.push(duration);
        }

        // Phase 2: Warm execution (should be cache hits)
        let mut hit_times = Vec::new();
        for file in &test_files[..5] {
            // Same 5 files
            let (result, duration) =
                measure_execution_time(engine.execute_with_cache(&[file.clone()])).await;

            assert!(result.is_ok(), "Warm execution should succeed");
            hit_times.push(duration);
        }

        // Performance Analysis
        let stats = engine.cache_stats().await.unwrap();

        // Verify cache hit rate meets target
        assert!(
            stats.hit_rate >= config.target_cache_hit_rate,
            "Cache hit rate {} should be >= {}",
            stats.hit_rate,
            config.target_cache_hit_rate
        );

        // Verify timing constraints
        let avg_miss_time = miss_times.iter().sum::<Duration>() / miss_times.len() as u32;
        let avg_hit_time = hit_times.iter().sum::<Duration>() / hit_times.len() as u32;

        assert!(
            avg_miss_time <= config.max_cache_miss_time,
            "Average cache miss time {:?} should be <= {:?}",
            avg_miss_time,
            config.max_cache_miss_time
        );

        assert!(
            avg_hit_time <= config.max_cache_hit_time,
            "Average cache hit time {:?} should be <= {:?}",
            avg_hit_time,
            config.max_cache_hit_time
        );

        // Performance improvement verification
        let speedup_ratio = avg_miss_time.as_millis() as f64 / avg_hit_time.as_millis() as f64;
        assert!(
            speedup_ratio >= 2.0,
            "Cache should provide at least 2x speedup, got {:.2}x",
            speedup_ratio
        );

        println!("Performance Results:");
        println!("  Cache Hit Rate: {:.1}%", stats.hit_rate * 100.0);
        println!("  Avg Cache Miss Time: {:?}", avg_miss_time);
        println!("  Avg Cache Hit Time: {:?}", avg_hit_time);
        println!("  Speedup Ratio: {:.2}x", speedup_ratio);
    }

    #[tokio::test]
    async fn test_multi_file_batch_performance() {
        let (temp_dir, mut engine) = create_performance_test_project().await;

        let src_dir = temp_dir.path().join("src");
        let batch_files: Vec<PathBuf> = (0..10)
            .map(|i| src_dir.join(format!("module_{}.rs", i)))
            .collect();

        // Measure batch execution performance
        let (result, duration) =
            measure_execution_time(engine.execute_with_cache(&batch_files)).await;

        assert!(result.is_ok(), "Batch execution should succeed");

        // Second execution should be much faster
        let (result2, cached_duration) =
            measure_execution_time(engine.execute_with_cache(&batch_files)).await;

        assert!(result2.is_ok(), "Cached batch execution should succeed");

        // Performance requirements for batch operations
        assert!(
            duration < Duration::from_secs(30),
            "Initial batch execution should complete within 30s, took {:?}",
            duration
        );

        assert!(
            cached_duration < Duration::from_secs(2),
            "Cached batch execution should complete within 2s, took {:?}",
            cached_duration
        );

        let batch_speedup = duration.as_millis() as f64 / cached_duration.as_millis() as f64;
        assert!(
            batch_speedup >= 5.0,
            "Batch caching should provide at least 5x speedup, got {:.2}x",
            batch_speedup
        );

        println!("Batch Performance Results:");
        println!("  Initial Batch Time: {:?}", duration);
        println!("  Cached Batch Time: {:?}", cached_duration);
        println!("  Batch Speedup: {:.2}x", batch_speedup);
    }

    #[tokio::test]
    async fn test_cache_efficiency_under_load() {
        let (temp_dir, mut engine) = create_performance_test_project().await;
        let config = PerformanceTestConfig::default();

        let src_dir = temp_dir.path().join("src");
        let test_files: Vec<PathBuf> = (0..15)
            .map(|i| src_dir.join(format!("module_{}.rs", i)))
            .collect();

        // Simulate high load with mixed patterns
        let mut total_operations = 0;
        let start_time = Instant::now();

        // Pattern 1: Sequential access (good for caching)
        for _ in 0..5 {
            for file in &test_files[..5] {
                let result = engine.execute_with_cache(&[file.clone()]).await;
                assert!(result.is_ok());
                total_operations += 1;
            }
        }

        // Pattern 2: Random access
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        for i in 0..20 {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            let file_idx = (hasher.finish() as usize) % test_files.len();

            let result = engine
                .execute_with_cache(&[test_files[file_idx].clone()])
                .await;
            assert!(result.is_ok());
            total_operations += 1;
        }

        // Pattern 3: Batch operations
        for _ in 0..5 {
            let batch = &test_files[..3];
            let result = engine.execute_with_cache(batch).await;
            assert!(result.is_ok());
            total_operations += 1;
        }

        let total_time = start_time.elapsed();
        let stats = engine.cache_stats().await.unwrap();

        // Performance requirements under load
        let ops_per_second = total_operations as f64 / total_time.as_secs_f64();
        assert!(
            ops_per_second >= 1.0,
            "Should handle at least 1 operation per second under load, got {:.2}",
            ops_per_second
        );

        assert!(
            stats.hit_rate >= config.min_cache_efficiency,
            "Cache efficiency {} should be >= {}",
            stats.hit_rate,
            config.min_cache_efficiency
        );

        assert_eq!(
            stats.cache_hits + stats.cache_misses,
            stats.total_requests,
            "Statistics should be consistent"
        );

        println!("Load Test Results:");
        println!("  Total Operations: {}", total_operations);
        println!("  Total Time: {:?}", total_time);
        println!("  Operations/sec: {:.2}", ops_per_second);
        println!("  Final Hit Rate: {:.1}%", stats.hit_rate * 100.0);
        println!("  Cache Requests: {}", stats.total_requests);
    }

    #[tokio::test]
    async fn test_cache_memory_efficiency() {
        let (temp_dir, mut engine) = create_performance_test_project().await;

        let src_dir = temp_dir.path().join("src");
        let test_files: Vec<PathBuf> = (0..20)
            .map(|i| src_dir.join(format!("module_{}.rs", i)))
            .collect();

        // Fill cache with various file combinations
        let mut file_combinations = Vec::new();

        // Single files
        for file in &test_files[..10] {
            file_combinations.push(vec![file.clone()]);
        }

        // File pairs
        for i in 0..5 {
            file_combinations.push(vec![test_files[i].clone(), test_files[i + 10].clone()]);
        }

        // Larger batches
        file_combinations.push(test_files[..5].to_vec());
        file_combinations.push(test_files[5..10].to_vec());

        // Execute all combinations to populate cache
        for files in &file_combinations {
            let result = engine.execute_with_cache(files).await;
            assert!(result.is_ok(), "Cache population should succeed");
        }

        let stats = engine.cache_stats().await.unwrap();

        // Memory efficiency requirements
        let bytes_per_entry = if stats.cached_entries > 0 {
            stats.cache_size_bytes / stats.cached_entries as u64
        } else {
            0
        };

        assert!(
            bytes_per_entry <= 1024 * 1024, // 1MB per entry max
            "Cache should use <= 1MB per entry, using {} bytes",
            bytes_per_entry
        );

        assert!(
            stats.cache_size_bytes <= 500 * 1024 * 1024, // 500MB total max
            "Total cache size should be <= 500MB, using {} bytes",
            stats.cache_size_bytes
        );

        // Test cache cleanup efficiency
        let initial_entries = stats.cached_entries;

        // Add more entries to potentially trigger LRU eviction
        for i in 0..10 {
            let extra_files = vec![test_files[i % test_files.len()].clone()];
            // Modify content to create new cache entries
            {
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .open(&extra_files[0])
                    .unwrap();
                writeln!(file, "\n// Cache test iteration {}", i).unwrap();
            }

            let result = engine.execute_with_cache(&extra_files).await;
            assert!(result.is_ok());
        }

        let final_stats = engine.cache_stats().await.unwrap();

        // Should still respect memory limits even with additional entries
        assert!(
            final_stats.cache_size_bytes <= 500 * 1024 * 1024,
            "Cache size should still respect limits after eviction"
        );

        println!("Memory Efficiency Results:");
        println!("  Bytes per Entry: {} KB", bytes_per_entry / 1024);
        println!(
            "  Total Cache Size: {} MB",
            stats.cache_size_bytes / (1024 * 1024)
        );
        println!("  Initial Entries: {}", initial_entries);
        println!("  Final Entries: {}", final_stats.cached_entries);
    }

    #[tokio::test]
    async fn test_concurrent_access_performance() {
        let (temp_dir, engine) = create_performance_test_project().await;

        // Wrap engine in Arc for concurrent access
        let engine = Arc::new(tokio::sync::Mutex::new(engine));

        let src_dir = temp_dir.path().join("src");
        let test_files: Vec<PathBuf> = (0..10)
            .map(|i| src_dir.join(format!("module_{}.rs", i)))
            .collect();

        let start_time = Instant::now();
        let mut handles = Vec::new();

        // Spawn concurrent tasks
        for task_id in 0..5 {
            let engine_clone = Arc::clone(&engine);
            let files_clone = test_files.clone();

            let handle = tokio::spawn(async move {
                let mut task_results = Vec::new();

                for i in 0..10 {
                    let file_idx = (task_id * 2 + i) % files_clone.len();
                    let test_file = &files_clone[file_idx];

                    let start = Instant::now();
                    {
                        let mut engine_guard = engine_clone.lock().await;
                        let result = engine_guard.execute_with_cache(&[test_file.clone()]).await;
                        assert!(result.is_ok(), "Concurrent execution should succeed");
                    }
                    let duration = start.elapsed();
                    task_results.push(duration);

                    // Small delay to allow interleaving
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }

                task_results
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut all_durations = Vec::new();
        for handle in handles {
            let task_durations = handle.await.unwrap();
            all_durations.extend(task_durations);
        }

        let total_time = start_time.elapsed();

        // Performance analysis
        let avg_duration = all_durations.iter().sum::<Duration>() / all_durations.len() as u32;
        let max_duration = all_durations.iter().max().unwrap();
        let min_duration = all_durations.iter().min().unwrap();

        // Concurrent access should not significantly degrade performance
        assert!(
            avg_duration <= Duration::from_secs(5),
            "Average concurrent operation time should be <= 5s, got {:?}",
            avg_duration
        );

        assert!(
            *max_duration <= Duration::from_secs(10),
            "Max concurrent operation time should be <= 10s, got {:?}",
            max_duration
        );

        let final_stats = {
            let engine_guard = engine.lock().await;
            engine_guard.cache_stats().await.unwrap()
        };

        // Should have good hit rate even with concurrent access
        assert!(
            final_stats.hit_rate >= 0.3,
            "Hit rate should be reasonable with concurrent access: {:.2}",
            final_stats.hit_rate
        );

        println!("Concurrent Access Results:");
        println!("  Total Operations: {}", all_durations.len());
        println!("  Total Time: {:?}", total_time);
        println!("  Avg Operation Time: {:?}", avg_duration);
        println!("  Min/Max Times: {:?} - {:?}", min_duration, max_duration);
        println!("  Final Hit Rate: {:.1}%", final_stats.hit_rate * 100.0);
    }
}

#[cfg(not(feature = "hotreload"))]
#[test]
fn test_hotreload_performance_feature_disabled() {
    // When hotreload feature is disabled, performance tests should still compile
    println!("Hot reload performance tests disabled - feature not enabled");
}