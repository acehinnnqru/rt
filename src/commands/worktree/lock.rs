use anyhow::Result;

use crate::git;

use super::project::Project;

/// Locks a worktree: git then refuses to prune or remove it until it is unlocked, which is
/// the guard for a checkout that must survive (a release branch, a bisect in progress).
pub async fn run_lock(branch: &str, reason: Option<&str>) -> Result<()> {
    let project = Project::current().await?;
    let path = project.resolve(branch)?.path.clone();

    let mut cmd = git::command_in(&project.dir);
    cmd.args(["worktree", "lock"]);
    if let Some(reason) = reason {
        cmd.arg("--reason").arg(reason);
    }
    cmd.arg(&path);

    git::run(&mut cmd, "git worktree lock").await?;
    println!("locked {}", path.display());
    Ok(())
}

pub async fn run_unlock(branch: &str) -> Result<()> {
    let project = Project::current().await?;
    let path = project.resolve(branch)?.path.clone();

    git::run(
        git::command_in(&project.dir)
            .args(["worktree", "unlock"])
            .arg(&path),
        "git worktree unlock",
    )
    .await?;

    println!("unlocked {}", path.display());
    Ok(())
}
