//! What `rt` knows about git itself.
//!
//! Two kinds of knowledge live here: the on-disk layout of a git directory, which
//! `rt tree` walks past and `rt clone --worktree` creates, and the thin command
//! helpers the commands share.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result, bail};
use tokio::process::Command;

/// Entries a git directory may contain for its own bookkeeping, from
/// gitrepository-layout(5).
///
/// A linked worktree lives *inside* a bare repository, so a worktree directory name
/// must not collide with any of these.
pub const ADMIN_ENTRIES: &[&str] = &[
    "HEAD",
    "branches",
    "commondir",
    "config",
    "config.worktree",
    "description",
    "gitdir",
    "hooks",
    "index",
    "info",
    "logs",
    "modules",
    "objects",
    "packed-refs",
    "refs",
    "remotes",
    "shallow",
    "worktrees",
];

/// Whether `name` is one of git's own directory entries.
pub fn is_admin_entry(name: &OsStr) -> bool {
    name.to_str()
        .is_some_and(|name| ADMIN_ENTRIES.contains(&name))
}

/// An ordinary clone, a linked worktree or a submodule: the directory holds a `.git`
/// entry, either a directory (ordinary clone) or a file (worktree, submodule).
pub fn is_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// A bare repository has no `.git` entry; the directory *is* the git directory.
pub fn is_bare(dir: &Path) -> bool {
    dir.join("HEAD").is_file() && dir.join("objects").is_dir() && dir.join("refs").is_dir()
}

/// A linked worktree created by `git worktree add`: `.git` is a file pointing at
/// `<common dir>/worktrees/<id>`. A submodule also has a `.git` file, but it points
/// into `modules/`, so it is not matched here.
pub fn is_linked_worktree(dir: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(dir.join(".git")) else {
        return false;
    };
    let Some(gitdir) = contents.trim().strip_prefix("gitdir:") else {
        return false;
    };
    Path::new(gitdir.trim())
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|parent| parent == OsStr::new("worktrees"))
}

/// Whether `name` can be a worktree directory inside a bare repository.
///
/// Only the first component is compared: `feature/login` becomes `<repo>/feature/login`,
/// and only the top level shares a namespace with git's own entries.
pub fn is_reserved_worktree_name(name: &str) -> bool {
    match name.split('/').next() {
        Some(first) => first.is_empty() || first.starts_with('.') || ADMIN_ENTRIES.contains(&first),
        None => true,
    }
}

/// A `git` command to run in the current directory.
pub fn command() -> Command {
    Command::new("git")
}

/// A `git -C <dir>` command.
pub fn command_in(dir: &Path) -> Command {
    let mut cmd = command();
    cmd.arg("-C").arg(dir);
    cmd
}

/// Runs `cmd` to completion, failing with `what` when git does not succeed.
pub async fn run(cmd: &mut Command, what: &str) -> Result<()> {
    let status = cmd
        .status()
        .await
        .with_context(|| format!("failed to run {what}"))?;

    if !status.success() {
        bail!("{what} failed");
    }

    Ok(())
}

/// Runs `cmd` and returns its standard output, reporting git's stderr on failure.
pub async fn capture(cmd: &mut Command, what: &str) -> Result<String> {
    let output = cmd
        .output()
        .await
        .with_context(|| format!("failed to run {what}"))?;

    if !output.status.success() {
        bail!(
            "{what} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Whether `cmd` exits successfully, with its output discarded.
pub async fn succeeds(cmd: &mut Command) -> Result<bool> {
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    Ok(cmd.status().await?.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn tells_ordinary_clones_worktrees_and_bare_repositories_apart() {
        let temp = TempDir::new("git-layout");

        let clone = temp.dir("clone");
        fs::create_dir_all(clone.join(".git")).expect("should create .git");
        assert!(is_repo(&clone));
        assert!(!is_bare(&clone));
        assert!(!is_linked_worktree(&clone));

        let bare = temp.dir("bare");
        fs::write(bare.join("HEAD"), "ref: refs/heads/main\n").expect("should write HEAD");
        fs::create_dir_all(bare.join("objects")).expect("should create objects");
        fs::create_dir_all(bare.join("refs")).expect("should create refs");
        assert!(is_bare(&bare));
        assert!(!is_repo(&bare));

        let worktree = temp.dir("bare/main");
        fs::write(worktree.join(".git"), "gitdir: /tmp/bare/worktrees/main\n")
            .expect("should write the gitdir file");
        assert!(is_repo(&worktree));
        assert!(is_linked_worktree(&worktree));
    }

    #[test]
    fn a_submodule_is_not_a_linked_worktree() {
        let temp = TempDir::new("git-submodule");
        let submodule = temp.dir("super/sub");
        fs::write(
            submodule.join(".git"),
            "gitdir: /tmp/super/.git/modules/sub\n",
        )
        .expect("should write the gitdir file");

        assert!(is_repo(&submodule));
        assert!(!is_linked_worktree(&submodule));
    }

    #[test]
    fn reserved_names_cover_git_entries_and_only_the_first_component() {
        assert!(is_reserved_worktree_name("objects"));
        assert!(is_reserved_worktree_name("worktrees"));
        assert!(is_reserved_worktree_name("objects/nested"));
        assert!(is_reserved_worktree_name(".hidden"));
        assert!(is_reserved_worktree_name(""));

        assert!(!is_reserved_worktree_name("main"));
        assert!(!is_reserved_worktree_name("feature/login"));
        assert!(!is_reserved_worktree_name("refs-backup"));
    }
}
