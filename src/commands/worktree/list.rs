use anyhow::{bail, Result};

use crate::config::Config;

pub async fn run(_config: &Config) -> Result<()> {
    let status = tokio::process::Command::new("git")
        .args(["worktree", "list"])
        .status()
        .await?;

    if !status.success() {
        bail!("git worktree list failed");
    }

    Ok(())
}
