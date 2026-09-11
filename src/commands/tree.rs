use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::config::Config;

pub async fn run(config: &Config) -> Result<()> {
    let root = Path::new(&config.root);

    if !root.exists() {
        bail!("root directory does not exist: {}", root.display());
    }

    println!("{}", root.display());
    for line in tree_lines(root)? {
        println!("{line}");
    }

    Ok(())
}

/// Renders the directory tree rooted at `dir` as printable lines.
///
/// Only directories are listed and hidden ones are skipped. A directory that is
/// itself a Git repository is a leaf: the walk prints it but never descends
/// into it, so the tree follows the repository layout instead of a fixed depth
/// limit and hosts with nested namespaces (for example GitLab subgroups) are
/// expanded completely.
fn tree_lines(dir: &Path) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    collect(dir, "", &mut lines)?;
    Ok(lines)
}

fn collect(dir: &Path, prefix: &str, lines: &mut Vec<String>) -> Result<()> {
    if is_git_repo(dir) {
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| {
            !e.file_name()
                .to_string_lossy()
                .starts_with('.')
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name();
        lines.push(format!("{prefix}{connector}{}", name.to_string_lossy()));

        let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
        collect(&entry.path(), &child_prefix, lines)?;
    }

    Ok(())
}

/// A directory is a repository when it holds a `.git` entry, which covers both
/// ordinary clones (directory) and linked worktrees or submodules (file).
fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Throwaway directory that is removed once the test finishes.
    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            let unique = format!(
                "rt-tree-test-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            );
            let path = std::env::temp_dir().join(unique);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn dir(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(&path).unwrap();
            path
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn descends_past_three_levels_until_repos() {
        let tree = TempTree::new();
        tree.dir("github.com/acehinnnqru/rt/.git");
        tree.dir("github.com/acehinnnqru/rt/src");
        tree.dir("gitlab.com/group/subgroup/deeper/repo/.git");
        tree.dir("gitlab.com/group/.hidden/repo/.git");

        let lines = tree_lines(tree.path()).unwrap();

        assert_eq!(
            lines,
            vec![
                "├── github.com",
                "│   └── acehinnnqru",
                "│       └── rt",
                "└── gitlab.com",
                "    └── group",
                "        └── subgroup",
                "            └── deeper",
                "                └── repo",
            ]
        );
    }

    #[test]
    fn stops_at_a_repo_without_listing_its_contents() {
        let tree = TempTree::new();
        tree.dir("host/owner/repo/.git/objects");
        tree.dir("host/owner/repo/inner/deeper");

        let lines = tree_lines(tree.path()).unwrap();

        assert_eq!(lines, ["└── host", "    └── owner", "        └── repo"]);
    }

    #[test]
    fn stops_when_root_is_itself_a_repo() {
        let tree = TempTree::new();
        tree.dir(".git");
        tree.dir("host/owner/repo/.git");

        assert!(tree_lines(tree.path()).unwrap().is_empty());
    }

    #[test]
    fn skips_files_and_hidden_directories() {
        let tree = TempTree::new();
        tree.dir("host/owner/repo/.git");
        fs::write(tree.path().join("README.md"), "not a directory").unwrap();
        tree.dir(".cache/host/owner/repo");

        let lines = tree_lines(tree.path()).unwrap();

        assert_eq!(lines, ["└── host", "    └── owner", "        └── repo"]);
    }
}
