use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::git::{self, WorktreeEntry};

use super::project::Project;

pub async fn run(branch: Option<&str>, force: bool) -> Result<()> {
    let project = Project::current().await?;
    let current = project.current_worktree().await;

    let target = match branch {
        Some(argument) => project.resolve(argument)?.path.clone(),
        None => match pick(&project, current)? {
            Some(path) => path,
            None => {
                println!("no removable worktrees found");
                return Ok(());
            }
        },
    };

    // Measured: `git worktree remove <current>` exits 0 and deletes the directory the
    // caller's shell is standing in. `rt delete` is the command that means "remove what I
    // am standing in"; this one manages the set from outside.
    if let Some(current) = current
        && current.path == target
    {
        bail!(
            "refusing to remove the worktree this command runs in: run rt delete from inside \
             it, or run rt wt remove from another checkout"
        );
    }

    let mut cmd = git::command_in(&project.dir);
    // Without `--force` git refuses a worktree with modified or untracked files, which is
    // the safer default: `--force` discards uncommitted work.
    cmd.args(["worktree", "remove"]);
    if force {
        cmd.arg("--force");
    }
    cmd.arg(&target);

    git::run(&mut cmd, "git worktree remove").await?;
    println!("removed worktree {}", target.display());
    Ok(())
}

/// The interactive fallback: nothing was named, so offer the worktrees that can go. The
/// menu goes to stderr, keeping stdout to the one line the command prints on success.
fn pick(project: &Project, current: Option<&WorktreeEntry>) -> Result<Option<PathBuf>> {
    let removable: Vec<&WorktreeEntry> = project
        .sorted_checkouts()
        .into_iter()
        .filter(|entry| current.is_none_or(|current| current.path != entry.path))
        .collect();

    if removable.is_empty() {
        return Ok(None);
    }

    if !std::io::stdin().is_terminal() {
        bail!("pass a branch name: rt wt remove <branch>");
    }

    eprintln!("select a worktree to remove:");
    for (index, entry) in removable.iter().enumerate() {
        eprintln!(
            "  {}: {}\t{}",
            index + 1,
            entry.branch_label(),
            entry.path.display()
        );
    }

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;

    choose(answer.trim(), &removable).map(Some)
}

/// Reads an answer to the menu: a number, or a branch name.
fn choose(answer: &str, removable: &[&WorktreeEntry]) -> Result<PathBuf> {
    if answer.is_empty() {
        bail!("no worktree selected");
    }

    if let Ok(number) = answer.parse::<usize>() {
        if number == 0 || number > removable.len() {
            bail!("selection out of range: {number}");
        }

        return Ok(removable[number - 1].path.clone());
    }

    removable
        .iter()
        .find(|entry| entry.branch.as_deref() == Some(answer))
        .map(|entry| entry.path.clone())
        .with_context(|| format!("no worktree for branch '{answer}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str) -> WorktreeEntry {
        WorktreeEntry {
            path: PathBuf::from(format!("/r/A/{name}")),
            branch: Some(name.to_string()),
            bare: false,
            detached: false,
            locked: false,
            prunable: false,
        }
    }

    #[test]
    fn accepts_a_number_or_a_branch_name() {
        let dev = entry("dev");
        let main = entry("main");
        let removable = vec![&dev, &main];

        assert_eq!(choose("1", &removable).unwrap(), dev.path);
        assert_eq!(choose("2", &removable).unwrap(), main.path);
        assert_eq!(choose("main", &removable).unwrap(), main.path);
    }

    #[test]
    fn rejects_an_empty_or_out_of_range_answer() {
        let dev = entry("dev");
        let removable = vec![&dev];

        assert!(choose("", &removable).is_err());
        assert!(choose("0", &removable).is_err());
        assert!(choose("2", &removable).is_err());
        assert!(choose("nope", &removable).is_err());
    }
}
