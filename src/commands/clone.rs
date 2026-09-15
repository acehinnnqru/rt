use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::git;
use crate::integrations;
use crate::path::{mapping, remote::Remote};

/// A bare clone copies every branch head into `refs/heads/*` and creates neither
/// remote-tracking refs nor their configuration, which leaves `git fetch` with nothing
/// to update and `git pull` without an upstream. Fetching into `refs/remotes/origin/*`
/// instead keeps every worktree pullable.
const ORIGIN_FETCH_REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";

pub async fn run(config: &Config, url: &str, worktree: Option<&str>) -> Result<()> {
    let remote = Remote::parse(url)?;
    let dest = mapping::repo_path(&config.root, &remote);

    if dest.exists() {
        bail!("repository already exists at {}", dest.display());
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let checkout = match worktree {
        Some(requested) => clone_worktree(config, url, &dest, requested).await?,
        None => clone_plain(config, url, &dest).await?,
    };

    if config.integrations.zoxide {
        integrations::zoxide::add(&checkout).await?;
    }
    if config.integrations.autojump {
        integrations::autojump::add(&checkout).await?;
    }

    println!("{}", checkout.display());
    Ok(())
}

/// Ordinary clone: the repository and its checkout both live at the mapped path.
async fn clone_plain(config: &Config, url: &str, dest: &Path) -> Result<PathBuf> {
    let mut cmd = git::command();
    cmd.arg("clone");

    if config.clone.depth > 0 {
        cmd.arg("--depth").arg(config.clone.depth.to_string());
    }

    cmd.arg(url).arg(dest);

    git::run(&mut cmd, "git clone").await?;
    Ok(dest.to_path_buf())
}

/// Bare repository at the mapped path plus a branch linked as a worktree inside it, so
/// `A` holds the object store and `A/<branch>` is the checkout.
async fn clone_worktree(
    config: &Config,
    url: &str,
    dest: &Path,
    requested: &str,
) -> Result<PathBuf> {
    // git resolves the worktree path relative to `-C <dest>`, so keep it absolute.
    let dest = std::path::absolute(dest)?;

    // Everything that can be rejected is rejected before the repository exists.
    let (default, branch) = resolve_branches(url, requested).await?;
    if git::is_reserved_worktree_name(&branch) {
        bail!(
            "branch '{branch}' cannot be a worktree inside the repository: \
             '{}' is a git directory entry",
            branch.split('/').next().unwrap_or(&branch)
        );
    }

    let mut rollback = Rollback::new(&dest);

    git::run(
        // `--quiet` keeps git's chatter off stdout, which carries only the created path.
        git::command()
            .args(["init", "--bare", "--quiet"])
            .arg(&dest),
        "git init --bare",
    )
    .await?;
    git::run(
        git::command_in(&dest).args(["remote", "add", "origin", url]),
        "git remote add",
    )
    .await?;
    git::run(
        git::command_in(&dest).args(["config", "remote.origin.fetch", ORIGIN_FETCH_REFSPEC]),
        "git config remote.origin.fetch",
    )
    .await?;

    if config.clone.depth > 0 {
        eprintln!(
            "warning: ignoring clone.depth = {} for --worktree: shallow history breaks \
             merges between worktrees",
            config.clone.depth
        );
    }

    git::run(
        git::command_in(&dest).args(["fetch", "origin"]),
        "git fetch",
    )
    .await?;
    git::run(
        git::command_in(&dest).args(["symbolic-ref", "HEAD", &format!("refs/heads/{default}")]),
        "git symbolic-ref HEAD",
    )
    .await?;
    git::run(
        git::command_in(&dest).args([
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            &format!("refs/remotes/origin/{default}"),
        ]),
        "git symbolic-ref refs/remotes/origin/HEAD",
    )
    .await?;

    let checkout = dest.join(&branch);
    let mut cmd = git::command_in(&dest);
    // `--quiet` keeps git's chatter off stdout, which carries only the created path.
    cmd.args(["worktree", "add", "--quiet", "--track", "-b", &branch]);
    cmd.arg(&checkout).arg(format!("origin/{branch}"));
    git::run(&mut cmd, "git worktree add").await?;

    rollback.disarm();
    Ok(checkout)
}

/// The remote's default branch and the branch to link as a worktree: the requested one,
/// or the default branch when none was named.
async fn resolve_branches(url: &str, requested: &str) -> Result<(String, String)> {
    let default = default_branch(url).await?;
    let checkout = if requested.is_empty() {
        default.clone()
    } else {
        requested.to_string()
    };

    if !is_valid_branch_name(&checkout).await? {
        bail!("'{checkout}' is not a valid git branch name");
    }

    if checkout != default && !branch_exists(url, &checkout).await? {
        bail!("branch '{checkout}' does not exist on {url}");
    }

    Ok((default, checkout))
}

/// The branch the remote's `HEAD` points at, for example `main`.
async fn default_branch(url: &str) -> Result<String> {
    let refs = git::capture(
        git::command().args(["ls-remote", "--symref", url, "HEAD"]),
        "git ls-remote",
    )
    .await?;

    refs.lines()
        .find_map(|line| {
            let rest = line.strip_prefix("ref: ")?;
            let name = rest.split_whitespace().next()?;
            name.strip_prefix("refs/heads/").map(ToOwned::to_owned)
        })
        .with_context(|| {
            format!("could not determine the default branch of {url}; pass --worktree=<branch>")
        })
}

async fn branch_exists(url: &str, branch: &str) -> Result<bool> {
    let refname = format!("refs/heads/{branch}");
    let refs = git::capture(
        git::command().args(["ls-remote", "--heads", url, &refname]),
        "git ls-remote",
    )
    .await?;

    Ok(refs
        .lines()
        .any(|line| line.split_whitespace().nth(1) == Some(refname.as_str())))
}

async fn is_valid_branch_name(branch: &str) -> Result<bool> {
    git::succeeds(git::command().args(["check-ref-format", "--branch", branch])).await
}

/// Removes a half-built repository when a later step fails, so that the next `rt clone`
/// of the same URL is not blocked by leftovers.
struct Rollback {
    path: PathBuf,
    armed: bool,
}

impl Rollback {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for Rollback {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn config_with_root(root: &Path) -> Config {
        Config {
            root: root.to_string_lossy().into_owned(),
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn returns_error_for_invalid_remote_url() {
        let temp = TempDir::new("clone-url");
        let config = config_with_root(temp.path());

        let err = run(&config, "github.com", None)
            .await
            .expect_err("invalid remote URL should fail before cloning");
        assert!(err.to_string().contains("missing repo in short URL format"));
    }

    #[tokio::test]
    async fn returns_error_when_destination_already_exists() {
        let temp = TempDir::new("clone-exists");
        let url = "https://github.com/acehinnnqru/rt.git";
        let remote = Remote::parse(url).expect("fixture URL should parse");
        let dest = mapping::repo_path(&temp.path().to_string_lossy(), &remote);

        std::fs::create_dir_all(&dest).expect("should create destination fixture");

        let config = config_with_root(temp.path());
        let err = run(&config, url, None)
            .await
            .expect_err("existing destination should fail before cloning");
        assert!(err.to_string().contains("repository already exists at"));
        assert!(err.to_string().contains(&*dest.to_string_lossy()));
    }

    #[tokio::test]
    async fn returns_error_when_destination_already_exists_for_worktree_clone() {
        let temp = TempDir::new("clone-worktree-exists");
        let url = "https://github.com/acehinnnqru/rt.git";
        let remote = Remote::parse(url).expect("fixture URL should parse");
        let dest = mapping::repo_path(&temp.path().to_string_lossy(), &remote);

        std::fs::create_dir_all(&dest).expect("should create destination fixture");

        let config = config_with_root(temp.path());
        let err = run(&config, url, Some(""))
            .await
            .expect_err("existing destination should fail before cloning");
        assert!(err.to_string().contains("repository already exists at"));
    }
}
