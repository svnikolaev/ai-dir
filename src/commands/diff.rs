use anyhow::{Result, anyhow};
use std::process::Command;

/// Проверяет, находимся ли мы внутри git-репозитория.
fn is_in_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Запускает git diff --staged и выводит результат в stdout.
pub fn run(staged: bool) -> Result<()> {
    if !is_in_git_repo() {
        return Err(anyhow!("Not inside a git repository"));
    }

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
