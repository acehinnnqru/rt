use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::config::Config;
use crate::git;
use crate::path::{mapping, remote::Remote};

pub async fn run(config: &Config, repo: Option<&str>, force: bool) -> Result<()> {
    let path = resolve(config, repo)?;

    if !path.exists() {
        bail!("path does not exist: {}", path.display());
    }

    // Absolute from here on, so that the prompt names what is about to go and the checkouts
    // git reports can be compared against it.
    let path = std::path::absolute(&path)?;
    let linked = git::is_linked_worktree(&path);
    // Nothing is asked under `--force`, so nothing has to be checked for it either: the
    // questions are the only reason to look, and a checkout whose git state cannot be read
    // has to stay deletable.
    let uncommitted = if force {
        None
    } else {
        uncommitted(&path).await?
    };

    if !force {
        for question in questions(&path, uncommitted.as_deref()) {
            if !confirm(&question)? {
                eprintln!("aborted");
                return Ok(());
            }
        }
    }

    // A linked worktree is a checkout of a repository, not a repository: removing the
    // directory alone would leave a dangling registration behind.
    if linked {
        let mut cmd = git::command_in(&path);
        cmd.args(["worktree", "remove"]);
        // Without `--force` git refuses a checkout with modified or untracked files; the
        // questions above have already covered that case.
        if force || uncommitted.is_some() {
            cmd.arg("--force");
        }
        cmd.arg(&path);

        git::run(&mut cmd, "git worktree remove").await?;
        println!("deleted worktree {}", path.display());
        return Ok(());
    }

    std::fs::remove_dir_all(&path)?;
    println!("deleted {}", path.display());
    Ok(())
}

/// The directory the command acts on: an existing path, a remote URL mapped through the
/// tree, or the current directory.
fn resolve(config: &Config, repo: Option<&str>) -> Result<PathBuf> {
    let Some(repo) = repo else {
        return Ok(std::env::current_dir()?);
    };

    // An existing path is used as it is: `rt delete <path>` has to work for absolute
    // paths, which would otherwise be mistaken for a remote URL.
    let candidate = PathBuf::from(repo);
    if candidate.exists() || !looks_like_remote(repo) {
        return Ok(candidate);
    }

    let remote = Remote::parse(repo)?;
    Ok(mapping::repo_path(&config.root, &remote))
}

/// Whether an argument that names nothing on disk should be read as a remote URL.
fn looks_like_remote(argument: &str) -> bool {
    argument.contains("://") || argument.contains('@') || argument.matches('/').count() >= 2
}

/// What has to be answered before `path` goes away: the path itself, always, and then the
/// checkout whose work is not in git yet — deleting is not undoable, so agreeing to the
/// path must not also mean agreeing to lose uncommitted work.
fn questions(path: &Path, uncommitted: Option<&Path>) -> Vec<String> {
    let mut questions = vec![format!("delete {}?", path.display())];

    if let Some(uncommitted) = uncommitted {
        questions.push(format!(
            "{} has uncommitted changes; delete anyway?",
            uncommitted.display()
        ));
    }

    questions
}

/// Asks a yes/no question on the terminal, where a default of no is what a stray newline
/// means.
///
/// Without a terminal there is nobody to ask, so the answer has to be given up front with
/// `--force` rather than assumed.
fn confirm(question: &str) -> Result<bool> {
    if !std::io::stdin().is_terminal() {
        bail!("refusing to delete without confirmation: pass --force");
    }

    ask(
        question,
        &mut std::io::stdin().lock(),
        &mut std::io::stderr().lock(),
    )
}

/// Writes the question, reads one line, and reports whether it means yes.
fn ask(question: &str, input: &mut impl BufRead, output: &mut impl Write) -> Result<bool> {
    write!(output, "{question} [y/N] ")?;
    output.flush()?;

    let mut answer = String::new();
    input.read_line(&mut answer)?;

    Ok(accepts(&answer))
}

/// Whether an answer means yes: only an explicit `y` or `yes` does, so a stray newline — or
/// the end of the input — declines.
fn accepts(answer: &str) -> bool {
    let answer = answer.trim();

    answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes")
}

/// The checkout under `path` that holds work which is not in git, if there is one.
///
/// A checkout of a project has to be asked of git, since a linked worktree keeps its own
/// index. A bare repository has none of its own, but the worktrees inside it do — the layout
/// `rt clone --worktree` creates — and they go away with it. A checkout that `rt wt move`
/// put somewhere else stays on disk, so its uncommitted work is not this deletion's business.
async fn uncommitted(path: &Path) -> Result<Option<PathBuf>> {
    if git::is_repo(path) {
        return Ok(dirty(path).await?.then(|| path.to_path_buf()));
    }

    if !git::is_bare(path) {
        return Ok(None);
    }

    let root = git::canonical(path);

    for entry in git::worktrees(path).await? {
        if entry.prunable || !entry.is_checkout() {
            continue;
        }

        if entry.path.starts_with(&root) && dirty(&entry.path).await? {
            return Ok(Some(entry.path));
        }
    }

    Ok(None)
}

/// Whether `dir` has staged, modified or untracked files.
async fn dirty(dir: &Path) -> Result<bool> {
    let status = git::capture(
        git::command_in(dir).args(["status", "--porcelain"]),
        "git status --porcelain",
    )
    .await?;

    Ok(!status.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn only_an_explicit_yes_accepts() {
        assert!(accepts("y"));
        assert!(accepts("Y\n"));
        assert!(accepts("yes"));
        assert!(accepts("YES\n"));

        assert!(!accepts(""));
        assert!(!accepts("\n"));
        assert!(!accepts("n"));
        assert!(!accepts("no"));
        assert!(!accepts("yep"));
    }

    #[test]
    fn asks_the_question_and_reads_the_answer() {
        let mut output = Vec::new();

        let accepted = ask(
            "delete /r/A?",
            &mut std::io::Cursor::new(b"y\n"),
            &mut output,
        )
        .expect("asking should work");

        assert!(accepted);
        assert_eq!(
            String::from_utf8(output).expect("the prompt is UTF-8"),
            "delete /r/A? [y/N] "
        );
    }

    #[test]
    fn asks_again_before_losing_uncommitted_work() {
        let temp = TempDir::new("delete-questions");
        let repo = temp.path().join("r/A");
        let checkout = repo.join("main");

        assert_eq!(
            questions(&repo, None),
            [format!("delete {}?", repo.display())]
        );

        let asked = questions(&repo, Some(&checkout));
        assert_eq!(asked.len(), 2);
        assert_eq!(asked[0], format!("delete {}?", repo.display()));
        assert_eq!(
            asked[1],
            format!(
                "{} has uncommitted changes; delete anyway?",
                checkout.display()
            )
        );
    }

    #[test]
    fn arguments_are_read_as_remotes_only_when_they_look_like_one() {
        assert!(looks_like_remote("git@example.com:acme/widget.git"));
        assert!(looks_like_remote("https://example.com/acme/widget"));
        assert!(looks_like_remote("example.com/acme/widget"));
        // A slash-heavy argument is a remote too, which is why an argument that exists on
        // disk is taken as a path before this is ever asked: `rt delete /abs/path` has to
        // delete that path, not parse it.
        assert!(looks_like_remote("/r/example.com/acme/widget"));

        assert!(!looks_like_remote("widget"));
        assert!(!looks_like_remote("./widget"));
    }

    #[tokio::test]
    async fn a_plain_directory_has_nothing_git_could_lose() {
        let temp = TempDir::new("delete-plain");
        let dir = temp.dir("scratch");
        std::fs::write(dir.join("notes.txt"), "work in progress\n").expect("should write the file");

        assert_eq!(uncommitted(&dir).await.expect("the check should run"), None);
    }
}
