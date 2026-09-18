//! `rt delete` in the bare layout: what it asks, what it removes, and what it keeps.

mod common;

use common::{Fixture, URL};

#[test]
fn removes_a_linked_worktree_and_its_registration() {
    let fixture = Fixture::new("delete-worktree");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");
    assert_eq!(fixture.registrations(), ["dev", "main"]);

    let stdout = fixture.rt_ok_in(&dev, &["delete", "--force"]);

    assert!(stdout.contains("deleted worktree"), "{stdout}");
    assert!(!dev.exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn asks_before_deleting() {
    let fixture = Fixture::new("delete-asks");
    let main = fixture.bare();

    // The questions are asked on a terminal; a pipe cannot answer them, so the answer has to
    // be given up front with `--force` instead of being assumed.
    let output = fixture.rt_err_in(&main, &["delete"]);

    assert!(output.contains("pass --force"), "{output}");
    assert!(fixture.repo().exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn discards_uncommitted_work_when_forced() {
    let fixture = Fixture::new("delete-dirty");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");
    std::fs::write(dev.join("scratch.txt"), "work in progress").expect("should write the file");

    let stdout = fixture.rt_ok_in(&dev, &["delete", "--force"]);

    assert!(stdout.contains("deleted worktree"), "{stdout}");
    assert!(!dev.exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn removes_the_bare_repository_with_every_worktree() {
    let fixture = Fixture::new("delete-repo");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let repo = fixture.repo().display().to_string();
    let stdout = fixture.rt_ok(&["delete", "--force", &repo]);

    assert!(stdout.contains("deleted"), "{stdout}");
    assert!(!fixture.repo().exists());
    assert!(!fixture.worktree("dev").exists());
}

#[test]
fn accepts_the_remote_url() {
    let fixture = Fixture::new("delete-url");
    fixture.bare();

    let stdout = fixture.rt_ok(&["delete", "--force", URL]);

    assert!(stdout.contains("deleted"), "{stdout}");
    assert!(!fixture.repo().exists());
}
