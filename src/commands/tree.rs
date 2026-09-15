use std::fs;
use std::path::Path;

use anyhow::{Result, bail};

use crate::config::Config;
use crate::git;

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
/// itself a Git repository is a leaf: the walk prints it but never descends into it,
/// so the tree follows the repository layout instead of a fixed depth limit and hosts
/// with nested namespaces (for example GitLab subgroups) are expanded completely.
///
/// A bare repository is not a leaf but a container: its checkouts are linked
/// worktrees, so the walk descends into it while skipping the entries git keeps for
/// itself (`objects`, `refs`, `worktrees`, ...).
fn tree_lines(dir: &Path) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    collect(dir, "", &mut lines)?;
    Ok(lines)
}

fn collect(dir: &Path, prefix: &str, lines: &mut Vec<String>) -> Result<()> {
    if git::is_repo(dir) {
        return Ok(());
    }

    let bare = git::is_bare(dir);

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter(|e| !(bare && git::is_admin_entry(&e.file_name())))
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn descends_past_three_levels_until_repos() {
        let temp = TempDir::new("tree-levels");
        temp.dir("github.com/acehinnnqru/rt/.git");
        temp.dir("github.com/acehinnnqru/rt/src");
        temp.dir("gitlab.com/group/subgroup/deeper/repo/.git");
        temp.dir("gitlab.com/group/.hidden/repo/.git");

        let lines = tree_lines(temp.path()).unwrap();

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
        let temp = TempDir::new("tree-leaf");
        temp.dir("host/owner/repo/.git/objects");
        temp.dir("host/owner/repo/inner/deeper");

        let lines = tree_lines(temp.path()).unwrap();

        assert_eq!(lines, ["└── host", "    └── owner", "        └── repo"]);
    }

    #[test]
    fn stops_when_root_is_itself_a_repo() {
        let temp = TempDir::new("tree-root-repo");
        temp.dir(".git");
        temp.dir("host/owner/repo/.git");

        assert!(tree_lines(temp.path()).unwrap().is_empty());
    }

    #[test]
    fn skips_files_and_hidden_directories() {
        let temp = TempDir::new("tree-skip");
        temp.dir("host/owner/repo/.git");
        temp.write("README.md", "not a directory");
        temp.dir(".cache/host/owner/repo");

        let lines = tree_lines(temp.path()).unwrap();

        assert_eq!(lines, ["└── host", "    └── owner", "        └── repo"]);
    }

    #[test]
    fn lists_worktrees_of_a_bare_repo_without_git_entries() {
        let temp = TempDir::new("tree-bare");
        let bare = bare_repo(&temp, "host/owner/A");
        linked_worktree(&bare, "main");
        linked_worktree(&bare, "fix-login");

        let lines = tree_lines(temp.path()).unwrap();

        assert_eq!(
            lines,
            vec![
                "└── host",
                "    └── owner",
                "        └── A",
                "            ├── fix-login",
                "            └── main",
            ]
        );
    }

    #[test]
    fn lists_a_bare_repo_and_its_worktrees_next_to_an_ordinary_clone() {
        let temp = TempDir::new("tree-mixed");
        let bare = bare_repo(&temp, "host/owner/A");
        linked_worktree(&bare, "main");
        temp.dir("host/owner/B/.git");

        let lines = tree_lines(temp.path()).unwrap();

        assert_eq!(
            lines,
            vec![
                "└── host",
                "    └── owner",
                "        ├── A",
                "        │   └── main",
                "        └── B",
            ]
        );
    }

    /// Creates a bare repository fixture: the git directory with no `.git` entry.
    fn bare_repo(temp: &TempDir, relative: &str) -> PathBuf {
        let path = temp.dir(relative);
        std::fs::write(path.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        for entry in ["objects", "refs", "hooks", "info", "logs", "worktrees"] {
            std::fs::create_dir_all(path.join(entry)).unwrap();
        }
        std::fs::write(path.join("config"), "[core]\n\tbare = true\n").unwrap();
        path
    }

    /// Adds a linked worktree child, whose `.git` is a file rather than a directory.
    fn linked_worktree(bare: &Path, name: &str) {
        let path = bare.join(name);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(
            path.join(".git"),
            format!("gitdir: {}/worktrees/{name}\n", bare.display()),
        )
        .unwrap();
    }
}
