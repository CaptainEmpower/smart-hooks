/// Claude BDD selector implementation  
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub async fn run(
    project_dir: Option<PathBuf>,
    context: Option<String>,
    dry_run: bool,
    output: String,
    confidence_threshold: f32,
) -> Result<()> {
    // For now, delegate to the existing binary
    // TODO: Migrate the full implementation here
    
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "claude-bdd-selector", "--features", "claude-ai", "--"]);
    
    if let Some(dir) = project_dir {
        cmd.args(["--project-dir", &dir.to_string_lossy()]);
    }
    
    if let Some(ctx) = context {
        cmd.args(["--context", &ctx]);
    }
    
    if dry_run {
        cmd.arg("--dry-run");
    }
    
    cmd.args(["--output", &output]);
    cmd.args(["--confidence-threshold", &confidence_threshold.to_string()]);
    
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Claude BDD selector failed"));
    }
    
    Ok(())
}