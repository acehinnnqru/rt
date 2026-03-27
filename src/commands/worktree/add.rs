use anyhow::{bail, Context, Result};

use crate::config::Config;

pub async fn run(_config: &Config, branch: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_name = cwd
        .file_name()
        .context("could not determine repo name")?
        .to_string_lossy();
    let worktree_path = cwd
        .parent()
        .context("could not determine parent directory")?
        .join(format!("{repo_name}-{branch}"));

    let status = tokio::process::Command::new("git")
        .args(["worktree", "add"])
        .arg(&worktree_path)
        .arg(branch)
        .status()
        .await?;

    if !status.success() {
        bail!("git worktree add failed");
    }

    println!("{}", worktree_path.display());
    Ok(())
}
