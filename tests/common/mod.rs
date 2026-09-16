//! Shared fixtures for the integration tests.
//!
//! `rt` is a binary crate, so these tests drive the built executable through
//! `CARGO_BIN_EXE_rt`. No test touches the developer's real `~/.rt.toml`: every fixture
//! gets its own `HOME`, its own root directory, and an origin repository that stays
//! offline.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

/// The `rt` executable built for this test run.
pub const RT: &str = env!("CARGO_BIN_EXE_rt");

/// Remote URL of the fixture origin.
///
/// `~/.gitconfig` rewrites `git@example.com:` to a local directory, so cloning needs no
/// network and the mapped path stays the realistic `{root}/example.com/acme/widget`.
pub const URL: &str = "git@example.com:acme/widget.git";

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Throwaway directory that is removed once it is dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "rt-it-{prefix}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("should create temp directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A repository to clone, plus the throwaway `HOME` and root that `rt` should use.
pub struct Fixture {
    temp: TempDir,
    work: PathBuf,
    origin: PathBuf,
    home: PathBuf,
    root: PathBuf,
}

impl Fixture {
    /// Creates an origin whose `main` and `dev` have diverged from the initial commit,
    /// plus the branches `objects` and `worktrees`, whose names collide with the
    /// entries git keeps inside a repository.
    pub fn new(prefix: &str) -> Self {
        let temp = TempDir::new(prefix);
        let home = temp.path().join("home");
        let root = temp.path().join("r");
        std::fs::create_dir_all(&home).expect("should create fixture home");
        std::fs::create_dir_all(&root).expect("should create fixture root");

        std::fs::write(
            home.join(".rt.toml"),
            format!("root = \"{}\"\n", root.display()),
        )
        .expect("should write fixture config");

        std::fs::write(
            home.join(".gitconfig"),
            format!(
                "[url \"file://{}/origin/\"]\n\tinsteadOf = git@example.com:\n",
                temp.path().display()
            ),
        )
        .expect("should write fixture gitconfig");

        let origin = temp.path().join("origin/acme/widget.git");
        std::fs::create_dir_all(origin.parent().expect("origin has a parent"))
            .expect("should create origin directory");
        git(
            &home,
            temp.path(),
            &[
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                "origin/acme/widget.git",
            ],
        );

        let work = temp.path().join("work");
        git(&home, temp.path(), &["init", "-q", "-b", "main", "work"]);
        git(
            &home,
            &work,
            &["commit", "-q", "--allow-empty", "-m", "initial"],
        );

        for branch in ["dev", "objects", "worktrees"] {
            git(&home, &work, &["branch", branch]);
        }

        git(&home, &work, &["switch", "-q", "dev"]);
        git(
            &home,
            &work,
            &["commit", "-q", "--allow-empty", "-m", "dev work"],
        );
        git(&home, &work, &["switch", "-q", "main"]);
        git(
            &home,
            &work,
            &["commit", "-q", "--allow-empty", "-m", "main work"],
        );
        git(
            &home,
            &work,
            &[
                "push",
                "-q",
                origin.to_str().expect("origin path is UTF-8"),
                "main",
                "dev",
                "objects",
                "worktrees",
            ],
        );

        Self {
            temp,
            work,
            origin,
            home,
            root,
        }
    }

    /// Where `rt` maps the fixture remote: `{root}/example.com/acme/widget`.
    pub fn repo(&self) -> PathBuf {
        self.root.join("example.com/acme/widget")
    }

    /// Clones the fixture origin into the bare layout and returns the default worktree.
    pub fn bare(&self) -> PathBuf {
        self.rt_ok(&["clone", URL, "--worktree"]);
        self.worktree("main")
    }

    /// A worktree inside the bare repository.
    pub fn worktree(&self, branch: &str) -> PathBuf {
        self.repo().join(branch)
    }

    /// Where `rt` maps `name`, for remotes that are not the fixture origin.
    pub fn mapped(&self, namespace: &str) -> PathBuf {
        self.root.join("example.com/acme").join(namespace)
    }

    /// Runs `rt` with the fixture home, from the fixture directory.
    pub fn rt(&self, args: &[&str]) -> Output {
        self.rt_in(self.temp.path(), args)
    }

    /// Runs `rt` with the fixture home, from `cwd`.
    pub fn rt_in(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(RT)
            .args(args)
            .current_dir(cwd)
            .env("HOME", &self.home)
            .output()
            .expect("rt should run")
    }

    /// Runs `rt`, asserts success, and returns its stdout.
    pub fn rt_ok(&self, args: &[&str]) -> String {
        self.rt_ok_in(self.temp.path(), args)
    }

    /// Runs `rt` in `cwd`, asserts success, and returns its stdout.
    pub fn rt_ok_in(&self, cwd: &Path, args: &[&str]) -> String {
        let output = self.rt_in(cwd, args);
        assert!(
            output.status.success(),
            "rt {args:?} failed:\n{}\n{}",
            text(&output.stdout),
            text(&output.stderr)
        );
        text(&output.stdout)
    }

    /// Runs `rt` in `cwd`, asserts failure, and returns its combined output.
    pub fn rt_err_in(&self, cwd: &Path, args: &[&str]) -> String {
        let output = self.rt_in(cwd, args);
        assert!(
            !output.status.success(),
            "rt {args:?} unexpectedly succeeded:\n{}",
            text(&output.stdout)
        );
        format!("{}{}", text(&output.stdout), text(&output.stderr))
    }

    /// Runs `rt`, asserts failure, and returns its combined output.
    pub fn rt_err(&self, args: &[&str]) -> String {
        self.rt_err_in(self.temp.path(), args)
    }

    /// Runs git inside the fixture, asserting that it succeeds.
    pub fn git(&self, dir: &Path, args: &[&str]) -> String {
        git(&self.home, dir, args)
    }

    /// Whether git inside the fixture succeeds, with its output discarded.
    pub fn git_ok(&self, dir: &Path, args: &[&str]) -> bool {
        git_output(&self.home, dir, args).status.success()
    }

    /// The commit `rev` resolves to inside `dir`, without the trailing newline.
    pub fn rev(&self, dir: &Path, rev: &str) -> String {
        self.git(dir, &["rev-parse", rev]).trim().to_string()
    }

    /// The short name `rev` resolves to inside `dir`, for example `dev` or `origin/dev`.
    pub fn ref_name(&self, dir: &Path, rev: &str) -> String {
        self.git(dir, &["rev-parse", "--abbrev-ref", rev])
            .trim()
            .to_string()
    }

    /// Adds a commit to `main` upstream and pushes it, so the remote moves on.
    pub fn commit_upstream(&self, message: &str) {
        git(
            &self.home,
            &self.work,
            &["commit", "-q", "--allow-empty", "-m", message],
        );
        git(
            &self.home,
            &self.work,
            &[
                "push",
                "-q",
                self.origin.to_str().expect("origin path is UTF-8"),
                "main",
            ],
        );
    }

    /// Creates `name` upstream on top of `start` and pushes it: a branch that exists on the
    /// remote but not yet in any clone of it.
    pub fn publish_branch(&self, name: &str, start: &str) {
        self.git(&self.work, &["switch", "-q", "-c", name, start]);
        self.git(&self.work, &["commit", "-q", "--allow-empty", "-m", name]);
        self.git(
            &self.work,
            &[
                "push",
                "-q",
                self.origin.to_str().expect("origin path is UTF-8"),
                name,
            ],
        );
        self.git(&self.work, &["switch", "-q", "main"]);
    }

    /// Directory names of every worktree git registers, the bare repository included.
    pub fn registered(&self) -> Vec<String> {
        self.git(&self.repo(), &["worktree", "list", "--porcelain"])
            .lines()
            .filter_map(|line| line.strip_prefix("worktree "))
            .map(|path| {
                Path::new(path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }

    /// Branch names git registers as worktrees, the bare repository excluded.
    pub fn branches(&self) -> Vec<String> {
        self.git(&self.repo(), &["worktree", "list", "--porcelain"])
            .lines()
            .filter_map(|line| line.strip_prefix("branch refs/heads/"))
            .map(ToOwned::to_owned)
            .collect()
    }

    /// The worktree registry of the bare repository: one directory per linked worktree.
    pub fn registrations(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.repo().join("worktrees"))
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

/// Asserts that the path `rt` printed is `expected`.
///
/// Both sides are canonicalized first: the temp directory is reached through a symlink
/// on macOS (`/var` is `/private/var`), and paths keep their double separators.
pub fn assert_same_path(printed: &str, expected: &Path) {
    let printed = canonical(Path::new(printed.trim()));
    let expected = canonical(expected);
    assert_eq!(printed, expected);
}

/// `path` with symlinks resolved, as git spells the paths it reports.
pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// Runs git hermetically: the fixture home, a fixed identity and no commit signing.
fn git(home: &Path, dir: &Path, args: &[&str]) -> String {
    let output = git_output(home, dir, args);

    assert!(
        output.status.success(),
        "git {args:?} failed:\n{}",
        text(&output.stderr)
    );

    text(&output.stdout)
}

/// Runs git with the fixture home and a fixed identity, without judging the exit status.
fn git_output(home: &Path, dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args([
            "-c",
            "user.email=rt@example.com",
            "-c",
            "user.name=rt",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .env("HOME", home)
        .output()
        .expect("git should run")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
