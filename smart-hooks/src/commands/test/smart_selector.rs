/// Smart test selector implementation
use anyhow::Result;
use std::process::Command;

pub async fn run(files: Vec<String>, with_bdd: bool, verbose: bool) -> Result<()> {
    // For now, delegate to the existing binary
    // TODO: Migrate the full implementation here
    
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "smart-test-selector", "--"]);
    
    if with_bdd {
        cmd.arg("--with-bdd");
    }
    
    if verbose {
        cmd.arg("--verbose");
    }
    
    cmd.args(files);
    
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Smart test selector failed"));
    }
    
    Ok(())
}