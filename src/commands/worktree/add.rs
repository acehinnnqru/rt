use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::git;
use crate::integrations;

use super::project::Project;

pub async fn run(config: &Config, branch: &str, from: Option<&str>, no_fetch: bool) -> Result<()> {
    if !git::succeeds(git::command().args(["check-ref-format", "--branch", branch])).await? {
        bail!("'{branch}' is not a valid git branch name");
    }

    let project = Project::current().await?;

    // Everything that can be rejected is rejected before a single directory is created.
    let path = project.worktree_path(branch)?;
    if let Some(entry) = project.find_branch(branch) {
        bail!(
            "branch '{branch}' is already checked out at {}",
            entry.path.display()
        );
    }
    if path.exists() {
        bail!("a worktree already exists at {}", path.display());
    }

    let local = git::ref_exists(&project.dir, &format!("refs/heads/{branch}")).await?;
    if from.is_some() && local {
        bail!("branch '{branch}' already exists; pick another name or drop --from");
    }

    fetch(&project, no_fetch).await?;

    let mut cmd = git::command_in(&project.dir);
    // `--quiet` keeps git's chatter off stdout, which carries only the created path.
    cmd.args(["worktree", "add", "--quiet"]);

    match start_point(&project, branch, from, local).await? {
        // The branch exists locally: check it out as it is. `wt add` creates a working
        // area, it does not move refs the user owns.
        StartPoint::Existing => {
            cmd.arg(&path).arg(branch);
        }
        StartPoint::Tracking(remote) => {
            cmd.args(["--track", "-b", branch]).arg(&path).arg(remote);
        }
        StartPoint::New(reference) => {
            cmd.args(["-b", branch]).arg(&path).arg(reference);
        }
    }

    git::run(&mut cmd, "git worktree add").await?;

    register(config, &path).await;
    println!("{}", path.display());
    Ok(())
}

/// Where a new worktree starts from.
enum StartPoint {
    /// The branch already exists locally: check it out as it is.
    Existing,
    /// `origin/<branch>` exists: create the branch tracking that tip.
    Tracking(String),
    /// Neither exists: create the branch from an explicit or remote-default reference.
    New(String),
}

async fn start_point(
    project: &Project,
    branch: &str,
    from: Option<&str>,
    local: bool,
) -> Result<StartPoint> {
    if let Some(reference) = from {
        if !git::ref_exists(&project.dir, &format!("{reference}^{{commit}}")).await? {
            bail!("'{reference}' is not a valid start point");
        }

        return Ok(StartPoint::New(reference.to_string()));
    }

    if local {
        return Ok(StartPoint::Existing);
    }

    let remote = format!("origin/{branch}");
    if git::ref_exists(&project.dir, &format!("refs/remotes/{remote}")).await? {
        return Ok(StartPoint::Tracking(remote));
    }

    let head = git::capture(
        git::command_in(&project.dir).args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"]),
        "git symbolic-ref refs/remotes/origin/HEAD",
    )
    .await
    .context("the remote has no default branch; pass --from <ref>")?;

    Ok(StartPoint::New(head.trim().to_string()))
}

/// Fetches `origin`, so a branch created upstream after the clone is usable without a
/// manual fetch: one object store serves every worktree, and this is the moment to update
/// it. A repository without an `origin` remote has nothing to fetch and is not an error.
async fn fetch(project: &Project, no_fetch: bool) -> Result<()> {
    if no_fetch {
        return Ok(());
    }

    let has_origin =
        git::succeeds(git::command_in(&project.dir).args(["config", "--get", "remote.origin.url"]))
            .await?;

    if !has_origin {
        return Ok(());
    }

    git::run(
        git::command_in(&project.dir).args(["fetch", "origin"]),
        "git fetch",
    )
    .await
    .context("fetching origin failed; pass --no-fetch to start from the local refs")
}

/// Registers the new checkout with the shell integrations, as `rt clone` does. A failing
/// hook is a warning: the worktree exists, and that is what the user asked for.
async fn register(config: &Config, path: &Path) {
    if config.integrations.zoxide
        && let Err(err) = integrations::zoxide::add(path).await
    {
        eprintln!("warning: zoxide: {err}");
    }

    if config.integrations.autojump
        && let Err(err) = integrations::autojump::add(path).await
    {
        eprintln!("warning: autojump: {err}");
    }
}
