/// Selective unit testing implementation
use anyhow::Result;
use std::process::Command;

pub async fn run(files: Vec<String>, verbose: bool) -> Result<()> {
    // For now, delegate to the existing binary
    // TODO: Migrate the full implementation here
    
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "selective-unit-tests", "--"]);
    
    if verbose {
        cmd.arg("--verbose");
    }
    
    cmd.args(files);
    
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Selective unit tests failed"));
    }
    
    Ok(())
}