use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::git;
use crate::path::{mapping, remote::Remote};

pub async fn run(config: &Config, branch: &str) -> Result<()> {
    let worktree_path = match bare_repo_of_current(config).await? {
        Some(repo) => add_inside_bare_repo(&repo, branch).await?,
        None => add_next_to_current(branch).await?,
    };

    println!("{}", worktree_path.display());
    Ok(())
}

/// The bare repository that owns the current checkout, when it was cloned with
/// `rt clone --worktree`.
///
/// Resolved through the remote URL and the configured root rather than guessed from
/// the current directory, so `A/main` and `A/fix-login` both find `A`.
async fn bare_repo_of_current(config: &Config) -> Result<Option<PathBuf>> {
    // `git remote get-url` expands `url.*.insteadOf`, which would turn the URL that
    // `rt clone` mapped into something else entirely; read the configured value.
    let Ok(url) = git::capture(
        git::command().args(["config", "--get", "remote.origin.url"]),
        "git config --get remote.origin.url",
    )
    .await
    else {
        return Ok(None);
    };

    let Ok(remote) = Remote::parse(url.trim()) else {
        return Ok(None);
    };

    let repo = mapping::repo_path(&config.root, &remote);
    Ok(git::is_bare(&repo).then_some(repo))
}

/// `<repo>/<branch>`, the layout `rt clone --worktree` creates.
async fn add_inside_bare_repo(repo: &Path, branch: &str) -> Result<PathBuf> {
    if git::is_reserved_worktree_name(branch) {
        bail!(
            "branch '{branch}' cannot be a worktree inside the repository: \
             '{}' is a git directory entry",
            branch.split('/').next().unwrap_or(branch)
        );
    }

    let repo = std::path::absolute(repo)?;
    let worktree_path = repo.join(branch);

    let mut cmd = git::command_in(&repo);
    // `--quiet` keeps git's chatter off stdout, which carries only the created path.
    cmd.args(["worktree", "add", "--quiet"]);
    match start_point(&repo, branch).await? {
        StartPoint::RemoteBranch => {
            cmd.args(["--track", "-b", branch]);
            cmd.arg(&worktree_path).arg(format!("origin/{branch}"));
        }
        StartPoint::LocalBranch => {
            cmd.arg(&worktree_path).arg(branch);
        }
        StartPoint::RemoteHead(head) => {
            cmd.args(["-b", branch]);
            cmd.arg(&worktree_path).arg(head);
        }
    }

    git::run(&mut cmd, "git worktree add").await?;
    Ok(worktree_path)
}

/// Sibling of the current directory, the layout of an ordinary clone: `repo-branch`.
async fn add_next_to_current(branch: &str) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let repo_name = cwd
        .file_name()
        .context("could not determine repo name")?
        .to_string_lossy();
    let worktree_path = cwd
        .parent()
        .context("could not determine parent directory")?
        .join(format!("{repo_name}-{branch}"));

    let mut cmd = git::command();
    cmd.args(["worktree", "add", "--quiet"]);
    cmd.arg(&worktree_path).arg(branch);

    git::run(&mut cmd, "git worktree add").await?;
    Ok(worktree_path)
}

/// Where a new worktree branches off from.
enum StartPoint {
    /// `refs/remotes/origin/<branch>` exists: check it out as a tracking branch.
    RemoteBranch,
    /// The branch already exists locally: check it out as it is.
    LocalBranch,
    /// Neither exists: create the branch from the remote's default branch.
    RemoteHead(String),
}

async fn start_point(repo: &Path, branch: &str) -> Result<StartPoint> {
    if ref_exists(repo, &format!("refs/remotes/origin/{branch}")).await? {
        return Ok(StartPoint::RemoteBranch);
    }

    if ref_exists(repo, &format!("refs/heads/{branch}")).await? {
        return Ok(StartPoint::LocalBranch);
    }

    let head = git::capture(
        git::command_in(repo).args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"]),
        "git symbolic-ref refs/remotes/origin/HEAD",
    )
    .await?;

    Ok(StartPoint::RemoteHead(head.trim().to_string()))
}

async fn ref_exists(repo: &Path, refname: &str) -> Result<bool> {
    git::succeeds(git::command_in(repo).args(["rev-parse", "--verify", "--quiet", refname])).await
}
