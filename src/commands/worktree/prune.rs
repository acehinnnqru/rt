use anyhow::Result;

use crate::git;

use super::project::Project;

/// Collects registrations whose directory is gone: worktrees deleted with `rm -rf` leave
/// one behind, and `rt wt list` marks it `prunable` first.
pub async fn run() -> Result<()> {
    let project = Project::current().await?;

    // `-v` names what git collected; git writes those lines to stderr, so stdout stays the
    // scriptable stream it is for every other command.
    git::run(
        git::command_in(&project.dir).args(["worktree", "prune", "-v"]),
        "git worktree prune",
    )
    .await
}
