//! What `rt` knows about git itself.
//!
//! Two kinds of knowledge live here: the on-disk layout of a git directory, which
//! `rt tree` walks past and `rt clone --worktree` creates, and the thin command
//! helpers the commands share.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
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

/// One record of `git worktree list --porcelain`: a checkout git knows about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeEntry {
    /// Absolute path of the checkout.
    pub path: PathBuf,
    /// Short name of the branch checked out there; `None` when bare or detached.
    pub branch: Option<String>,
    /// The bare repository itself rather than a checkout.
    pub bare: bool,
    /// A checkout with a detached HEAD.
    pub detached: bool,
    /// Locked, so `git worktree prune` and `git worktree remove` refuse it.
    pub locked: bool,
    /// Registered but gone from disk; `git worktree prune` collects it.
    pub prunable: bool,
}

impl WorktreeEntry {
    /// Whether git registers a checkout here, as opposed to the bare repository.
    pub fn is_checkout(&self) -> bool {
        !self.bare
    }

    /// The branch name, or the placeholder `rt` prints for an entry without one.
    pub fn branch_label(&self) -> &str {
        self.branch.as_deref().unwrap_or("(detached)")
    }
}

/// Every checkout git registers in `dir`, the bare repository included.
pub async fn worktrees(dir: &Path) -> Result<Vec<WorktreeEntry>> {
    let output = capture(
        command_in(dir).args(["worktree", "list", "--porcelain"]),
        "git worktree list",
    )
    .await?;

    Ok(parse_worktrees(&output))
}

/// Parses `git worktree list --porcelain`: blank-line separated records, each starting
/// with a `worktree <path>` line followed by its attributes.
fn parse_worktrees(output: &str) -> Vec<WorktreeEntry> {
    let mut entries: Vec<WorktreeEntry> = Vec::new();

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            entries.push(WorktreeEntry {
                path: PathBuf::from(path),
                branch: None,
                bare: false,
                detached: false,
                locked: false,
                prunable: false,
            });
            continue;
        }

        let Some(entry) = entries.last_mut() else {
            continue;
        };

        if let Some(refname) = line.strip_prefix("branch ") {
            entry.branch = Some(
                refname
                    .strip_prefix("refs/heads/")
                    .unwrap_or(refname)
                    .to_string(),
            );
        } else if line == "bare" {
            entry.bare = true;
        } else if line == "detached" {
            entry.detached = true;
        } else if line == "locked" || line.starts_with("locked ") {
            entry.locked = true;
        } else if line == "prunable" || line.starts_with("prunable ") {
            entry.prunable = true;
        }
    }

    entries
}

/// Whether `dir` is a bare repository, asked of git rather than guessed from the filesystem:
/// the `.git` directory of an ordinary clone holds `HEAD`, `objects/` and `refs/` too, so
/// [`is_bare`] cannot tell the two apart.
pub async fn is_bare_repository(dir: &Path) -> Result<bool> {
    let answer = capture(
        command_in(dir).args(["rev-parse", "--is-bare-repository"]),
        "git rev-parse --is-bare-repository",
    )
    .await?;

    Ok(answer.trim() == "true")
}

/// The common git directory of the current directory: the bare repository for every
/// worktree of the bare layout, `.git` for an ordinary clone.
pub async fn common_git_dir() -> Result<PathBuf> {
    let path = capture(
        command().args(["rev-parse", "--path-format=absolute", "--git-common-dir"]),
        "git rev-parse --git-common-dir",
    )
    .await?;

    Ok(PathBuf::from(path.trim()))
}

/// The root of the checkout the current directory belongs to; `None` outside a work tree,
/// which is the case inside a bare repository.
pub async fn toplevel() -> Result<Option<PathBuf>> {
    let output = command()
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .await
        .context("failed to run git rev-parse --show-toplevel")?;

    if !output.status.success() {
        return Ok(None);
    }

    Ok(Some(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    )))
}

/// Whether `refname` resolves in `dir`.
pub async fn ref_exists(dir: &Path, refname: &str) -> Result<bool> {
    succeeds(command_in(dir).args(["rev-parse", "--verify", "--quiet", refname])).await
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

    #[test]
    fn parses_the_porcelain_registry() {
        let output = "\
worktree /r/A
bare

worktree /r/A/dev
HEAD 1111111111111111111111111111111111111111
branch refs/heads/dev
locked release freeze

worktree /r/A/main
HEAD 2222222222222222222222222222222222222222
branch refs/heads/main

";

        let entries = parse_worktrees(output);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].path, Path::new("/r/A"));
        assert!(entries[0].bare);
        assert!(!entries[0].is_checkout());
        assert_eq!(entries[0].branch, None);

        assert_eq!(entries[1].path, Path::new("/r/A/dev"));
        assert_eq!(entries[1].branch.as_deref(), Some("dev"));
        assert_eq!(entries[1].branch_label(), "dev");
        assert!(entries[1].locked);
        assert!(!entries[1].bare);

        assert_eq!(entries[2].branch.as_deref(), Some("main"));
        assert!(!entries[2].locked);
    }

    #[test]
    fn parses_detached_and_prunable_entries() {
        let output = "\
worktree /r/A/main
HEAD 1111111111111111111111111111111111111111
detached
prunable gitdir file points to non-existent location

";

        let entries = parse_worktrees(output);

        assert_eq!(entries.len(), 1);
        assert!(entries[0].detached);
        assert!(entries[0].prunable);
        assert_eq!(entries[0].branch_label(), "(detached)");
    }

    #[test]
    fn ignores_attribute_lines_without_a_record() {
        assert!(parse_worktrees("HEAD 1111111111111111111111111111111111111111\n").is_empty());
    }
}
