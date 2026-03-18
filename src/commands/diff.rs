use anyhow::{Result, anyhow};
use std::process::Command;

/// Запускает git diff --staged и выводит результат в stdout.
pub fn run(staged: bool) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("--no-pager").arg("diff");
    if staged {
        cmd.arg("--staged");
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!(
            "git diff failed with exit code: {:?}",
            status.code()
        ));
    }
    Ok(())
}
