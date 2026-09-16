//! The repository a worktree command runs against.
//!
//! Every command starts here. The project is resolved by asking git for the common git
//! directory of the current directory, never by re-deriving a path from the working
//! directory or from `remote.origin.url`: the first misses subdirectories, and the second
//! jumps to a different repository when the same remote is cloned twice.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::git::{self, WorktreeEntry};

/// The repository that owns the current directory, plus the layout its worktrees use.
pub struct Project {
    /// Directory git commands run in: the bare repository `A`, or the main worktree of an
    /// ordinary clone.
    pub dir: PathBuf,
    /// Whether `dir` is a bare repository: the layout `rt clone --worktree` creates.
    pub bare: bool,
    /// Every entry git registers, the bare repository included.
    pub worktrees: Vec<WorktreeEntry>,
}

impl Project {
    /// Resolves the project from the current directory.
    pub async fn current() -> Result<Self> {
        let common = git::common_git_dir()
            .await
            .context("not inside a git repository: run this from a checkout managed by rt")?;

        let worktrees = git::worktrees(&common).await?;

        if git::is_bare_repository(&common).await? {
            return Ok(Self {
                dir: common,
                bare: true,
                worktrees,
            });
        }

        // An ordinary clone: its worktrees are siblings of the *main* worktree, which is
        // the first record git reports. `--show-toplevel` would name a linked worktree,
        // and the current directory may be a subdirectory of either.
        let main = worktrees
            .first()
            .context("git worktree list reported no worktree")?;

        Ok(Self {
            dir: main.path.clone(),
            bare: false,
            worktrees,
        })
    }

    /// Every entry that is a checkout: the bare repository is not one.
    pub fn checkouts(&self) -> impl Iterator<Item = &WorktreeEntry> {
        self.worktrees.iter().filter(|entry| entry.is_checkout())
    }

    /// The checkouts in the order `rt wt list` prints them: branch names alphabetically,
    /// entries without a branch last by path. `git worktree list` order is not a contract,
    /// and output that users script against has to be deterministic.
    pub fn sorted_checkouts(&self) -> Vec<&WorktreeEntry> {
        let mut entries: Vec<&WorktreeEntry> = self.checkouts().collect();

        entries.sort_by(|left, right| match (&left.branch, &right.branch) {
            (Some(left), Some(right)) => left.cmp(right),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.path.cmp(&right.path),
        });

        entries
    }

    /// The path a new worktree for `branch` gets: `<repo>/<branch>` in the bare layout,
    /// `<main>-<branch>` next to the main worktree of an ordinary clone.
    ///
    /// Fails for a branch whose first component collides with a git directory entry, but
    /// only in the bare layout, where the worktree path is a child of the git directory.
    pub fn worktree_path(&self, branch: &str) -> Result<PathBuf> {
        if self.bare {
            if git::is_reserved_worktree_name(branch) {
                bail!(
                    "branch '{branch}' cannot be a worktree inside the repository: \
                     '{}' is a git directory entry",
                    branch.split('/').next().unwrap_or(branch)
                );
            }

            return Ok(self.dir.join(branch));
        }

        let main = self
            .worktrees
            .first()
            .context("git worktree list reported no worktree")?;
        let name = main
            .path
            .file_name()
            .context("could not determine the repository name")?;
        let parent = main
            .path
            .parent()
            .context("could not determine the repository parent directory")?;

        Ok(parent.join(format!("{}-{branch}", name.to_string_lossy())))
    }

    /// The checkout that has `branch` checked out, wherever it lives.
    pub fn find_branch(&self, branch: &str) -> Option<&WorktreeEntry> {
        self.checkouts()
            .find(|entry| entry.branch.as_deref() == Some(branch))
    }

    /// The checkout registered at `path`, comparing paths as the filesystem spells them:
    /// on macOS a worktree created through `/var` is registered as `/private/var`.
    pub fn find_path(&self, path: &Path) -> Option<&WorktreeEntry> {
        let wanted = canonical(path);
        self.checkouts()
            .find(|entry| canonical(&entry.path) == wanted)
    }

    /// Resolves a branch name or a path to the checkout it names.
    pub fn resolve(&self, branch_or_path: &str) -> Result<&WorktreeEntry> {
        if let Some(entry) = self.find_branch(branch_or_path) {
            return Ok(entry);
        }

        if let Some(entry) = self.find_path(Path::new(branch_or_path)) {
            return Ok(entry);
        }

        bail!("no worktree for branch '{branch_or_path}'")
    }

    /// The checkout the command runs in, when it is one: inside the bare repository there
    /// is none.
    pub async fn current_worktree(&self) -> Option<&WorktreeEntry> {
        let toplevel = git::toplevel().await.ok().flatten()?;
        let wanted = canonical(&toplevel);

        self.checkouts()
            .find(|entry| canonical(&entry.path) == wanted)
    }
}

/// `path` with symlinks resolved, or unchanged when it does not exist.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn entry(path: &str, branch: Option<&str>) -> WorktreeEntry {
        WorktreeEntry {
            path: PathBuf::from(path),
            branch: branch.map(ToOwned::to_owned),
            bare: false,
            detached: branch.is_none(),
            locked: false,
            prunable: false,
        }
    }

    fn project(dir: &Path, bare: bool, worktrees: Vec<WorktreeEntry>) -> Project {
        Project {
            dir: dir.to_path_buf(),
            bare,
            worktrees,
        }
    }

    #[test]
    fn bare_layout_names_the_worktree_after_the_branch() {
        let temp = TempDir::new("project-bare");
        let repo = temp.path().join("A");
        let project = project(&repo, true, vec![]);

        assert_eq!(
            project.worktree_path("feature/login").unwrap(),
            repo.join("feature/login")
        );
        assert!(project.worktree_path("objects").is_err());
    }

    #[test]
    fn clone_layout_names_the_worktree_after_the_main_worktree() {
        let temp = TempDir::new("project-clone");
        let main = temp.path().join("r/example.com/acme/widget");
        let project = project(
            &main,
            false,
            vec![entry(&main.to_string_lossy(), Some("main"))],
        );

        assert_eq!(
            project.worktree_path("dev").unwrap(),
            temp.path().join("r/example.com/acme/widget-dev")
        );
    }

    #[test]
    fn finds_a_checkout_by_branch_or_by_path() {
        let temp = TempDir::new("project-find");
        let repo = temp.path().join("A");
        let project = project(
            &repo,
            true,
            vec![
                WorktreeEntry {
                    bare: true,
                    ..entry(&repo.to_string_lossy(), None)
                },
                entry(&repo.join("main").to_string_lossy(), Some("main")),
            ],
        );

        assert_eq!(project.find_branch("main").unwrap().path, repo.join("main"));
        assert!(project.find_branch("dev").is_none());
        assert_eq!(
            project
                .find_path(&repo.join("main"))
                .unwrap()
                .branch
                .as_deref(),
            Some("main")
        );
        assert!(project.checkouts().all(|entry| !entry.bare));
        assert!(project.resolve("main").is_ok());
        assert!(project.resolve("nope").is_err());
    }
}
