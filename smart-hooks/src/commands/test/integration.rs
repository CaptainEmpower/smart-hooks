/// Integration testing implementation
use anyhow::Result;
use std::process::Command;

pub async fn run(force: bool, verbose: bool) -> Result<()> {
    // For now, delegate to the existing binary
    // TODO: Migrate the full implementation here

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "integration-tests-if-needed", "--"]);

    if force {
        cmd.arg("--force");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Integration tests failed"));
    }

    Ok(())
}