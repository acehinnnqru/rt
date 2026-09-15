use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::config::Config;
use crate::git;
use crate::path::{mapping, remote::Remote};

pub async fn run(config: &Config, repo: Option<&str>) -> Result<()> {
    let path = match repo {
        Some(r) => {
            // An existing path is used as it is: `rt delete <path>` has to work for
            // absolute paths, which would otherwise be mistaken for a remote URL.
            let candidate = PathBuf::from(r);
            if candidate.exists() {
                candidate
            } else if r.contains("://") || r.contains('@') || r.matches('/').count() >= 2 {
                let remote = Remote::parse(r)?;
                mapping::repo_path(&config.root, &remote)
            } else {
                candidate
            }
        }
        None => std::env::current_dir()?,
    };

    if !path.exists() {
        bail!("path does not exist: {}", path.display());
    }

    // A linked worktree is a checkout of a repository, not a repository: removing the
    // directory alone would leave a dangling registration behind and would discard
    // uncommitted work without asking.
    if git::is_linked_worktree(&path) {
        let path = std::path::absolute(&path)?;
        let mut cmd = git::command_in(&path);
        cmd.args(["worktree", "remove"]).arg(&path);

        git::run(&mut cmd, "git worktree remove").await?;
        println!("deleted worktree {}", path.display());
        return Ok(());
    }

    std::fs::remove_dir_all(&path)?;
    println!("deleted {}", path.display());
    Ok(())
}
