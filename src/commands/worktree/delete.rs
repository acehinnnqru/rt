use anyhow::{bail, Context, Result};

use crate::config::Config;

pub async fn run(_config: &Config) -> Result<()> {
    let output = tokio::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .await?;

    if !output.status.success() {
        bail!("git worktree list failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let worktrees: Vec<&str> = stdout
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .collect();

    if worktrees.len() <= 1 {
        println!("no removable worktrees found");
        return Ok(());
    }

    println!("select a worktree to delete:");
    for (i, wt) in worktrees.iter().skip(1).enumerate() {
        println!("  {}: {wt}", i + 1);
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().context("invalid selection")?;

    if choice == 0 || choice > worktrees.len() - 1 {
        bail!("selection out of range");
    }

    let selected = worktrees[choice];

    let status = tokio::process::Command::new("git")
        .args(["worktree", "remove", selected])
        .status()
        .await?;

    if !status.success() {
        bail!("git worktree remove failed");
    }

    println!("deleted worktree: {selected}");
    Ok(())
}
