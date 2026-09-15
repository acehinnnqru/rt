//! `rt clone --worktree`: a bare repository with a linked worktree inside it.
//!
//! The cases mirror `docs/design/clone-worktree.md` §11.

mod common;

use common::{Fixture, URL, assert_same_path};

#[test]
fn creates_a_bare_repository_with_the_default_worktree() {
    let fixture = Fixture::new("wt-clone");
    let stdout = fixture.rt_ok(&["clone", URL, "--worktree"]);
    let repo = fixture.repo();
    let main = fixture.worktree("main");

    assert_same_path(&stdout, &main);
    assert_eq!(
        fixture
            .git(&repo, &["rev-parse", "--is-bare-repository"])
            .trim(),
        "true"
    );
    assert!(!repo.join(".git").exists());
    assert!(main.join(".git").is_file());
    assert_eq!(
        fixture
            .git(&main, &["rev-parse", "--abbrev-ref", "HEAD"])
            .trim(),
        "main"
    );
    assert_eq!(
        fixture
            .git(&main, &["rev-parse", "--abbrev-ref", "main@{upstream}"])
            .trim(),
        "origin/main"
    );
    assert_eq!(
        fixture
            .git(&main, &["config", "--get", "remote.origin.url"])
            .trim(),
        URL
    );
}

#[test]
fn can_pull_after_the_remote_moves() {
    let fixture = Fixture::new("wt-pull");
    fixture.rt_ok(&["clone", URL, "--worktree"]);
    fixture.commit_upstream("second");

    let main = fixture.worktree("main");
    fixture.git(&main, &["pull", "--ff-only"]);

    // The fixture publishes two commits on `main`, plus the one added above.
    assert_eq!(
        fixture.git(&main, &["rev-list", "--count", "HEAD"]).trim(),
        "3"
    );
}

#[test]
fn keeps_every_branch_usable_at_the_default_depth() {
    let fixture = Fixture::new("wt-branches");
    fixture.rt_ok(&["clone", URL, "--worktree"]);

    // clone.depth defaults to 1, which must not degrade into --single-branch.
    let refs = fixture.git(
        &fixture.repo(),
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/remotes/origin",
        ],
    );
    for branch in ["origin/main", "origin/dev", "origin/objects"] {
        assert!(
            refs.lines().any(|line| line == branch),
            "expected {branch} in:\n{refs}"
        );
    }

    let main = fixture.worktree("main");
    let stdout = fixture.rt_ok_in(&main, &["worktree", "add", "dev"]);

    let dev = fixture.worktree("dev");
    assert_same_path(&stdout, &dev);
    assert_eq!(
        fixture
            .git(&dev, &["rev-parse", "--abbrev-ref", "HEAD"])
            .trim(),
        "dev"
    );
    assert_eq!(
        fixture
            .git(&dev, &["rev-parse", "--abbrev-ref", "dev@{upstream}"])
            .trim(),
        "origin/dev"
    );
}

#[test]
fn worktrees_can_be_merged_despite_the_default_depth() {
    let fixture = Fixture::new("wt-merge");
    fixture.rt_ok(&["clone", URL, "--worktree"]);

    // `main` and `dev` diverged, so merging them needs history, not just branch tips.
    let main = fixture.worktree("main");
    fixture.rt_ok_in(&main, &["worktree", "add", "dev"]);
    assert!(
        !fixture
            .git(&main, &["merge-base", "origin/main", "origin/dev"])
            .trim()
            .is_empty(),
        "the merge base must be present"
    );

    fixture.git(&main, &["merge", "--no-edit", "origin/dev"]);

    // initial, main work, dev work, merge commit.
    assert_eq!(
        fixture.git(&main, &["rev-list", "--count", "HEAD"]).trim(),
        "4"
    );
}

#[test]
fn accepts_an_explicit_branch() {
    let fixture = Fixture::new("wt-explicit");
    let stdout = fixture.rt_ok(&["clone", URL, "--worktree=dev"]);

    assert_same_path(&stdout, &fixture.worktree("dev"));
    assert_eq!(
        fixture
            .git(
                &fixture.worktree("dev"),
                &["rev-parse", "--abbrev-ref", "HEAD"]
            )
            .trim(),
        "dev"
    );
    // The bare repository keeps pointing at the remote's default branch.
    assert_eq!(
        fixture
            .git(&fixture.repo(), &["symbolic-ref", "--short", "HEAD"])
            .trim(),
        "main"
    );
}

#[test]
fn rejects_a_branch_that_is_not_on_the_remote() {
    let fixture = Fixture::new("wt-missing");
    let output = fixture.rt_err(&["clone", URL, "--worktree=nope"]);

    assert!(output.contains("does not exist on"), "{output}");
    assert!(!fixture.repo().exists(), "nothing should be created");
}

#[test]
fn rejects_a_branch_that_git_keeps_for_itself() {
    let fixture = Fixture::new("wt-reserved");
    let output = fixture.rt_err(&["clone", URL, "--worktree=objects"]);

    assert!(output.contains("git directory entry"), "{output}");
    assert!(!fixture.repo().exists(), "nothing should be created");
}

#[test]
fn removes_a_half_built_repository_when_the_remote_is_unreachable() {
    let fixture = Fixture::new("wt-unreachable");
    let output = fixture.rt_err(&["clone", "git@example.com:acme/missing.git", "--worktree"]);

    assert!(output.contains("git ls-remote failed"), "{output}");
    assert!(
        !fixture.mapped("missing").exists(),
        "a failed clone must not leave a repository behind"
    );
}

#[test]
fn a_plain_clone_is_still_an_ordinary_clone() {
    let fixture = Fixture::new("plain");
    let stdout = fixture.rt_ok(&["clone", URL]);
    let repo = fixture.repo();

    assert_same_path(&stdout, &repo);
    assert!(repo.join(".git").is_dir());
    assert_eq!(
        fixture
            .git(&repo, &["rev-parse", "--is-bare-repository"])
            .trim(),
        "false"
    );
    // clone.depth still applies here.
    assert_eq!(
        fixture.git(&repo, &["rev-list", "--count", "HEAD"]).trim(),
        "1"
    );
}
