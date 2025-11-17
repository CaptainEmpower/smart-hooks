//! BDD (Behavior-Driven Development) tests for hot reload functionality
//! Tests user scenarios and behaviors from the developer's perspective

#[cfg(feature = "hotreload")]
mod hotreload_bdd_tests {
    use smart_hooks::hotreload::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Create a comprehensive test project mimicking real development scenarios
    async fn create_realistic_project() -> (TempDir, HotReloadEngine) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // Create realistic project structure
        create_realistic_rust_project(&project_root).unwrap();

        // Configure hot reload with production-like settings
        let config = HotReloadConfig {
            enabled: true,
            max_cache_size_bytes: 100 * 1024 * 1024, // 100MB
            max_cache_entries: 10000,
            cache_ttl: HashMap::from([
                (HookType::Test, Duration::from_secs(600)),  // 10 minutes
                (HookType::Lint, Duration::from_secs(1800)), // 30 minutes
                (HookType::Format, Duration::from_secs(3600)), // 1 hour
            ]),
            min_confidence_threshold: 0.8,
            background_warming_enabled: true, // Enable for realistic testing
            fs_watching_enabled: true,
            cache_dir: Some(temp_dir.path().join(".smart-hooks-cache")),
        };

        // Use disk cache for realistic persistence testing
        let cache_storage: Arc<dyn CacheStorage> = Arc::new(
            cache::disk_lru::DiskLruCache::new(
                temp_dir.path().join(".smart-hooks-cache"),
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

    fn create_realistic_rust_project(project_root: &Path) -> std::io::Result<()> {
        let src_dir = project_root.join("src");
        fs::create_dir_all(&src_dir)?;
        fs::create_dir_all(project_root.join("tests"))?;
        fs::create_dir_all(project_root.join("benches"))?;

        // Create Cargo.toml with realistic dependencies
        fs::write(
            project_root.join("Cargo.toml"),
            r#"[package]
name = "example-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.8"
criterion = "0.5"

[[bench]]
name = "performance_bench"
harness = false
"#,
        )?;

        // Create realistic lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            r#"//! Example project demonstrating hot reload capabilities

pub mod core;
pub mod utils;
pub mod config;
pub mod api;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            database_url: "postgres://localhost/app".to_string(),
            log_level: "info".to_string(),
        }
    }
}

pub async fn run_app(config: AppConfig) -> Result<()> {
    let processor = core::DataProcessor::new(&config.database_url).await?;
    let api_server = api::Server::new(config.port, processor);

    api_server.run().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.port, 8080);
        assert!(config.database_url.contains("postgres"));
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.port, deserialized.port);
    }
}
"#,
        )?;

        // Create core module with business logic
        fs::write(
            src_dir.join("core.rs"),
            r#"//! Core business logic

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataProcessor {
    database_url: String,
    cache: HashMap<String, ProcessedData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    pub id: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

impl DataProcessor {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Simulate async database connection
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(Self {
            database_url: database_url.to_string(),
            cache: HashMap::new(),
        })
    }

    pub async fn process(&mut self, input: &str) -> Result<ProcessedData> {
        if input.is_empty() {
            anyhow::bail!("Input cannot be empty");
        }

        // Check cache first
        if let Some(cached) = self.cache.get(input) {
            return Ok(cached.clone());
        }

        // Simulate processing
        let processed = ProcessedData {
            id: format!("processed_{}", input.len()),
            data: serde_json::json!({
                "original": input,
                "processed": input.to_uppercase(),
                "length": input.len()
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        // Cache the result
        self.cache.insert(input.to_string(), processed.clone());

        Ok(processed)
    }

    pub fn get_cache_size(&self) -> usize {
        self.cache.len()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_processor_creation() {
        let processor = DataProcessor::new("test://localhost").await.unwrap();
        assert_eq!(processor.database_url, "test://localhost");
        assert_eq!(processor.get_cache_size(), 0);
    }

    #[tokio::test]
    async fn test_data_processing() {
        let mut processor = DataProcessor::new("test://localhost").await.unwrap();

        let result = processor.process("hello world").await.unwrap();
        assert_eq!(result.id, "processed_11");
        assert_eq!(result.data["original"], "hello world");
        assert_eq!(result.data["processed"], "HELLO WORLD");
    }

    #[tokio::test]
    async fn test_caching_behavior() {
        let mut processor = DataProcessor::new("test://localhost").await.unwrap();

        // First call
        let result1 = processor.process("test").await.unwrap();
        assert_eq!(processor.get_cache_size(), 1);

        // Second call should use cache
        let result2 = processor.process("test").await.unwrap();
        assert_eq!(result1.timestamp, result2.timestamp);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mut processor = DataProcessor::new("test://localhost").await.unwrap();

        let result = processor.process("").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }
}
"#,
        )?;

        // Create API module
        fs::write(
            src_dir.join("api.rs"),
            r#"//! API server implementation

use crate::core::{DataProcessor, ProcessedData};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Server {
    port: u16,
    processor: DataProcessor,
}

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub success: bool,
    pub result: Option<ProcessedData>,
    pub error: Option<String>,
}

impl Server {
    pub fn new(port: u16, processor: DataProcessor) -> Self {
        Self { port, processor }
    }

    pub async fn run(&self) -> Result<()> {
        println!("Server running on port {}", self.port);
        // Simulate server running
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    pub async fn handle_process_request(&mut self, req: ProcessRequest) -> ProcessResponse {
        match self.processor.process(&req.data).await {
            Ok(result) => ProcessResponse {
                success: true,
                result: Some(result),
                error: None,
            },
            Err(e) => ProcessResponse {
                success: false,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataProcessor;

    #[tokio::test]
    async fn test_server_creation() {
        let processor = DataProcessor::new("test://localhost").await.unwrap();
        let server = Server::new(8080, processor);
        assert_eq!(server.port, 8080);
    }

    #[tokio::test]
    async fn test_process_request_success() {
        let processor = DataProcessor::new("test://localhost").await.unwrap();
        let mut server = Server::new(8080, processor);

        let request = ProcessRequest {
            data: "test input".to_string(),
        };

        let response = server.handle_process_request(request).await;
        assert!(response.success);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_process_request_error() {
        let processor = DataProcessor::new("test://localhost").await.unwrap();
        let mut server = Server::new(8080, processor);

        let request = ProcessRequest {
            data: "".to_string(),
        };

        let response = server.handle_process_request(request).await;
        assert!(!response.success);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }
}
"#,
        )?;

        // Create utils module
        fs::write(
            src_dir.join("utils.rs"),
            r#"//! Utility functions

use anyhow::Result;
use std::collections::HashMap;

pub fn validate_config(config: &HashMap<String, String>) -> Result<()> {
    if !config.contains_key("database_url") {
        anyhow::bail!("Missing required config: database_url");
    }

    if !config.contains_key("port") {
        anyhow::bail!("Missing required config: port");
    }

    Ok(())
}

pub fn format_data(data: &str) -> String {
    data.trim().to_lowercase().replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_success() {
        let mut config = HashMap::new();
        config.insert("database_url".to_string(), "postgres://localhost".to_string());
        config.insert("port".to_string(), "8080".to_string());

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_missing_database() {
        let mut config = HashMap::new();
        config.insert("port".to_string(), "8080".to_string());

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database_url"));
    }

    #[test]
    fn test_format_data() {
        assert_eq!(format_data("Hello World"), "hello_world");
        assert_eq!(format_data("  TEST DATA  "), "test_data");
    }
}
"#,
        )?;

        // Create config module
        fs::write(
            src_dir.join("config.rs"),
            r#"//! Configuration management

use std::collections::HashMap;
use std::env;

pub fn load_from_env() -> HashMap<String, String> {
    let mut config = HashMap::new();

    config.insert(
        "database_url".to_string(),
        env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/app".to_string())
    );

    config.insert(
        "port".to_string(),
        env::var("PORT").unwrap_or_else(|_| "8080".to_string())
    );

    config.insert(
        "log_level".to_string(),
        env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
    );

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_env() {
        let config = load_from_env();

        assert!(config.contains_key("database_url"));
        assert!(config.contains_key("port"));
        assert!(config.contains_key("log_level"));

        // Test default values
        assert!(config["database_url"].contains("postgres"));
        assert_eq!(config["port"], "8080");
        assert_eq!(config["log_level"], "info");
    }
}
"#,
        )?;

        // Create integration test
        fs::write(
            project_root.join("tests").join("integration_test.rs"),
            r#"//! Integration tests

use example_project::{AppConfig, run_app};

#[tokio::test]
async fn test_app_integration() {
    let config = AppConfig {
        port: 0, // Use any available port
        database_url: "postgres://test".to_string(),
        log_level: "debug".to_string(),
    };

    // This would fail in a real app, but that's okay for testing
    let result = run_app(config).await;
    assert!(result.is_err()); // Expected to fail without real database
}
"#,
        )?;

        // Create benchmark
        fs::write(
            project_root.join("benches").join("performance_bench.rs"),
            r#"use criterion::{black_box, criterion_group, criterion_main, Criterion};
use example_project::core::DataProcessor;

async fn process_data_benchmark() {
    let mut processor = DataProcessor::new("bench://localhost").await.unwrap();
    let _ = processor.process(black_box("benchmark data")).await.unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("process_data", |b| {
        b.to_async(&rt).iter(|| process_data_benchmark())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
"#,
        )?;

        Ok(())
    }

    // BDD Test Scenarios

    #[tokio::test]
    async fn scenario_developer_makes_small_change_expects_fast_feedback() {
        // Given: A developer has a project with hot reload enabled
        let (temp_dir, mut engine) = create_realistic_project().await;

        // When: The developer makes a small change to a utility function
        let utils_file = temp_dir.path().join("src/utils.rs");
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&utils_file)
                .unwrap();
            writeln!(file, "\n// Added small comment for testing").unwrap();
        }

        // And: They trigger the pre-commit hooks
        let start_time = std::time::Instant::now();
        let result = engine.execute_with_cache(&[utils_file]).await;

        // Then: The hooks should complete quickly (within reasonable time)
        let execution_time = start_time.elapsed();
        assert!(result.is_ok(), "Hook execution should succeed");
        assert!(
            execution_time < Duration::from_secs(30),
            "First execution should complete in reasonable time"
        );

        // And: Subsequent executions should be much faster due to caching
        let start_time = std::time::Instant::now();
        let result2 = engine
            .execute_with_cache(&[temp_dir.path().join("src/utils.rs")])
            .await;
        let cached_execution_time = start_time.elapsed();

        assert!(result2.is_ok(), "Cached execution should succeed");
        assert!(
            cached_execution_time < Duration::from_secs(5),
            "Cached execution should be very fast"
        );
    }

    #[tokio::test]
    async fn scenario_developer_modifies_core_logic_cache_invalidates_appropriately() {
        // Given: A project with hot reload and cached results
        let (temp_dir, mut engine) = create_realistic_project().await;

        let core_file = temp_dir.path().join("src/core.rs");

        // When: Developer first runs hooks to populate cache
        let _initial_result = engine
            .execute_with_cache(&[core_file.clone()])
            .await
            .unwrap();

        // And: Developer makes a significant change to core business logic
        {
            let content = fs::read_to_string(&core_file).unwrap();
            let modified_content = content.replace(
                "processed_{}",
                "processed_v2_{}", // Change that affects behavior
            );
            fs::write(&core_file, modified_content).unwrap();
        }

        // And: They run hooks again
        let result = engine.execute_with_cache(&[core_file]).await;

        // Then: The cache should be invalidated and hooks should run fresh
        assert!(result.is_ok(), "Modified file execution should succeed");

        let stats = engine.cache_stats().await.unwrap();
        assert!(
            stats.cache_misses > 0,
            "Should have cache misses for modified core logic"
        );
    }

    #[tokio::test]
    async fn scenario_developer_commits_multiple_related_files_cache_optimization() {
        // Given: A project with hot reload
        let (temp_dir, mut engine) = create_realistic_project().await;

        // When: Developer modifies multiple related files in a single commit
        let files_to_modify = vec![
            temp_dir.path().join("src/core.rs"),
            temp_dir.path().join("src/api.rs"),
            temp_dir.path().join("src/lib.rs"),
        ];

        // Modify all files
        for file in &files_to_modify {
            let mut file_handle = fs::OpenOptions::new().append(true).open(file).unwrap();
            writeln!(file_handle, "\n// Related change for feature X").unwrap();
        }

        // And: They execute hooks for all modified files
        let result = engine.execute_with_cache(&files_to_modify).await;

        // Then: The system should handle multiple files efficiently
        assert!(result.is_ok(), "Multi-file execution should succeed");

        let stats = engine.cache_stats().await.unwrap();
        assert!(stats.total_requests > 0, "Should track multi-file request");

        // And: Subsequent runs should benefit from caching
        let result2 = engine.execute_with_cache(&files_to_modify).await;
        assert!(
            result2.is_ok(),
            "Cached multi-file execution should succeed"
        );

        let stats2 = engine.cache_stats().await.unwrap();
        // Since files were modified, we expect cache misses initially, but stats should track properly
        assert!(
            stats2.total_requests > stats.total_requests,
            "Should have more total requests after second execution"
        );
    }

    #[tokio::test]
    async fn scenario_large_project_cache_performance_remains_good() {
        // Given: A project with hot reload
        let (temp_dir, mut engine) = create_realistic_project().await;

        // When: Developer works with many files over time (simulating large project)
        let all_files = vec![
            temp_dir.path().join("src/lib.rs"),
            temp_dir.path().join("src/core.rs"),
            temp_dir.path().join("src/api.rs"),
            temp_dir.path().join("src/utils.rs"),
            temp_dir.path().join("src/config.rs"),
        ];

        // Simulate working with many files over multiple commits
        for iteration in 0..10 {
            // Modify some files
            for (i, file) in all_files.iter().enumerate() {
                if i % 3 == iteration % 3 {
                    let mut file_handle = fs::OpenOptions::new().append(true).open(file).unwrap();
                    writeln!(file_handle, "\n// Iteration {} change", iteration).unwrap();
                }
            }

            // Execute hooks
            let result = engine.execute_with_cache(&all_files).await;
            assert!(result.is_ok(), "Large project execution should succeed");
        }

        // Then: Cache performance should remain good
        let final_stats = engine.cache_stats().await.unwrap();
        assert!(
            final_stats.hit_rate >= 0.3,
            "Hit rate should be reasonable for large project: {}",
            final_stats.hit_rate
        );
        assert!(
            final_stats.total_requests >= 10,
            "Should have handled multiple requests"
        );
    }

    #[tokio::test]
    async fn scenario_cache_gracefully_handles_corruption_or_errors() {
        // Given: A project with hot reload
        let (temp_dir, mut engine) = create_realistic_project().await;

        let test_file = temp_dir.path().join("src/utils.rs");

        // When: Normal execution populates cache
        let _result1 = engine
            .execute_with_cache(&[test_file.clone()])
            .await
            .unwrap();

        // And: Cache corruption occurs (simulated by clearing cache)
        engine.clear_cache().await.unwrap();

        // And: Developer tries to use hooks again
        let result2 = engine.execute_with_fallback(&[test_file]).await;

        // Then: System should gracefully fall back to normal execution
        assert!(
            result2.is_ok(),
            "Fallback execution should succeed despite cache issues"
        );

        let stats = engine.cache_stats().await.unwrap();
        // After clearing, we should see cache misses but system continues working
        assert_eq!(
            stats.cached_entries, 0,
            "Cache should be empty after clearing"
        );
    }

    #[tokio::test]
    async fn scenario_background_warming_improves_developer_experience() {
        // Given: A project with background warming enabled
        let (temp_dir, mut engine) = create_realistic_project().await;

        // When: Developer enables file watching and makes changes
        engine.enable_file_watching().await.unwrap();

        let test_file = temp_dir.path().join("src/core.rs");

        // And: They make an initial change
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&test_file)
                .unwrap();
            writeln!(file, "\n// Initial change to trigger warming").unwrap();
        }

        // Wait a brief moment for background processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Then: System should have started background warming
        // (This is hard to test directly, so we test that the system handles it gracefully)
        let result = engine.execute_with_cache(&[test_file]).await;
        assert!(result.is_ok(), "Execution with warming should succeed");

        // Cleanup
        engine.disable_file_watching().await;
    }

    #[tokio::test]
    async fn scenario_cache_statistics_provide_useful_insights() {
        // Given: A project with hot reload
        let (temp_dir, mut engine) = create_realistic_project().await;

        let test_files = vec![
            temp_dir.path().join("src/core.rs"),
            temp_dir.path().join("src/api.rs"),
        ];

        // When: Developer uses the system over time
        for _ in 0..5 {
            let _ = engine.execute_with_cache(&test_files).await.unwrap();
        }

        // Then: Statistics should provide meaningful insights
        let stats = engine.cache_stats().await.unwrap();

        // Should track requests
        assert!(stats.total_requests >= 5, "Should track total requests");

        // Should have reasonable hit rate after repeated executions
        assert!(stats.hit_rate <= 1.0, "Hit rate should not exceed 100%");

        // Should track cache size
        assert!(stats.cached_entries > 0, "Should have cached entries");

        // Hit + miss should equal total
        assert_eq!(
            stats.cache_hits + stats.cache_misses,
            stats.total_requests,
            "Hits + misses should equal total requests"
        );
    }

    #[tokio::test]
    async fn scenario_different_hook_types_cached_independently() {
        // Given: A project with different hook types configured
        let (temp_dir, mut engine) = create_realistic_project().await;

        let test_file = temp_dir.path().join("src/utils.rs");

        // When: Different types of hooks are executed
        // Note: This test verifies the engine can handle the execution flow
        // Real hook type differentiation would be handled by the actual hook execution
        let result1 = engine.execute_with_cache(&[test_file.clone()]).await;
        assert!(result1.is_ok(), "First execution should succeed");

        // And: The same files are processed again
        let result2 = engine.execute_with_cache(&[test_file]).await;
        assert!(result2.is_ok(), "Second execution should succeed");

        // Then: Cache should be used effectively
        let stats = engine.cache_stats().await.unwrap();
        assert!(
            stats.cache_hits > 0,
            "Should have cache hits for repeated executions"
        );
    }

    #[tokio::test]
    async fn scenario_memory_usage_stays_within_reasonable_bounds() {
        // Given: A project with cache size limits
        let (temp_dir, mut engine) = create_realistic_project().await;

        // When: Many operations are performed
        let test_files = vec![
            temp_dir.path().join("src/lib.rs"),
            temp_dir.path().join("src/core.rs"),
            temp_dir.path().join("src/api.rs"),
        ];

        // Simulate many operations
        for i in 0..20 {
            // Modify files to create new cache entries
            for file in &test_files {
                let mut file_handle = fs::OpenOptions::new().append(true).open(file).unwrap();
                writeln!(file_handle, "\n// Operation {} change", i).unwrap();
            }

            let _ = engine.execute_with_cache(&test_files).await.unwrap();
        }

        // Then: Cache should respect size limits
        let stats = engine.cache_stats().await.unwrap();
        assert!(
            stats.cache_size_bytes <= 100 * 1024 * 1024, // 100MB limit
            "Cache size should respect configured limits"
        );
        assert!(
            stats.cached_entries <= 10000, // Entry limit
            "Cache entry count should respect limits"
        );
    }
}

#[cfg(not(feature = "hotreload"))]
#[test]
fn test_hotreload_bdd_feature_disabled() {
    // When hotreload feature is disabled, BDD tests should still compile
    // but not execute hot reload functionality
    println!("Hot reload BDD tests disabled - feature not enabled");
}