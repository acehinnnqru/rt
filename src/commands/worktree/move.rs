use anyhow::{Result, bail};

use crate::git;

use super::project::Project;

/// Moves a worktree to another directory. The branch keeps its name, so in the bare layout
/// a moved worktree no longer sits at `<repo>/<branch>`; every other command reads git's
/// registry, so only `add`'s naming rule is affected.
pub async fn run(branch: &str, destination: &str) -> Result<()> {
    let project = Project::current().await?;
    let source = project.resolve(branch)?.path.clone();
    let destination = std::path::absolute(destination)?;

    if source == destination {
        bail!("the worktree is already at {}", destination.display());
    }

    if let Some(current) = project.current_worktree().await
        && current.path == source
    {
        eprintln!(
            "warning: moving the worktree this command runs in; the current directory moves with it"
        );
    }

    git::run(
        git::command_in(&project.dir)
            .args(["worktree", "move"])
            .arg(&source)
            .arg(&destination),
        "git worktree move",
    )
    .await?;

    println!("{}", destination.display());
    Ok(())
}
