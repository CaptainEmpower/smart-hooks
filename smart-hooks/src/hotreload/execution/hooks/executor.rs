//! Logic for executing individual hooks with smart-hooks integration

use super::super::super::{
    tracking::ChangeSet, HookResult, HookType, HotReloadError, HotReloadResult,
};
use std::time::SystemTime;
use tokio::process::Command;

/// Executes individual hooks with smart-hooks integration
pub struct HookExecutor;

impl HookExecutor {
    /// Execute a single hook
    pub async fn execute_single_hook(
        hook_type: &HookType,
        changes: &ChangeSet,
    ) -> HotReloadResult<HookResult> {
        let start_time = SystemTime::now();

        let (exit_code, stdout, stderr) = if cfg!(test) {
            // Use mock execution in tests for consistent results
            Self::execute_mock_hook(hook_type, changes).await?
        } else {
            match hook_type {
                HookType::Test => Self::execute_test_hook(changes).await?,
                HookType::Lint => Self::execute_lint_hook(changes).await?,
                HookType::Format => Self::execute_format_hook(changes).await?,
                HookType::Analysis => Self::execute_analysis_hook(changes).await?,
                HookType::Custom(name) => Self::execute_custom_hook(name, changes).await?,
            }
        };

        let execution_time = start_time.elapsed().unwrap_or_default();

        Ok(HookResult {
            hook_type: hook_type.clone(),
            exit_code,
            stdout,
            stderr,
            execution_time,
            timestamp: SystemTime::now(),
        })
    }

    /// Mock hook execution for tests (deterministic results)
    pub async fn execute_mock_hook(
        hook_type: &HookType,
        changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        // Simulate brief execution time for realistic testing
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let file_count = changes.all_files().len();
        let output = format!(
            "Mock {} hook executed on {} files",
            hook_type.as_str(),
            file_count
        );

        Ok((0, output, String::new()))
    }

    /// Execute test hook using smart-hooks test selection
    async fn execute_test_hook(changes: &ChangeSet) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("test")
            .arg("selective");

        // Add changed files to test command
        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| HotReloadError::Cache(format!("Failed to execute test hook: {}", e)))?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute lint hook using smart-hooks auto linting
    async fn execute_lint_hook(changes: &ChangeSet) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("lint")
            .arg("auto");

        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| HotReloadError::Cache(format!("Failed to execute lint hook: {}", e)))?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute format hook using smart-hooks auto formatting
    async fn execute_format_hook(changes: &ChangeSet) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("format")
            .arg("auto");

        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| HotReloadError::Cache(format!("Failed to execute format hook: {}", e)))?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute analysis hook using smart-hooks dependency analysis
    async fn execute_analysis_hook(changes: &ChangeSet) -> HotReloadResult<(i32, String, String)> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("smart-hooks")
            .arg("--")
            .arg("analyze")
            .arg("dependencies");

        for file in &changes.all_files() {
            cmd.arg(file.to_string_lossy().as_ref());
        }

        let output = cmd.output().await.map_err(|e| {
            HotReloadError::Cache(format!("Failed to execute analysis hook: {}", e))
        })?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    /// Execute custom hook
    async fn execute_custom_hook(
        hook_name: &str,
        _changes: &ChangeSet,
    ) -> HotReloadResult<(i32, String, String)> {
        // Custom hook execution - would be configurable
        let output = Command::new(hook_name).output().await.map_err(|e| {
            HotReloadError::Cache(format!(
                "Failed to execute custom hook {}: {}",
                hook_name, e
            ))
        })?;

        Ok((
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}