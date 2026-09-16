//! `rt delete` in the bare layout: what it removes, and what it refuses to remove.

mod common;

use common::{Fixture, URL};

#[test]
fn removes_a_linked_worktree_and_its_registration() {
    let fixture = Fixture::new("delete-worktree");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");
    assert_eq!(fixture.registrations(), ["dev", "main"]);

    let stdout = fixture.rt_ok_in(&dev, &["delete"]);

    assert!(stdout.contains("deleted worktree"), "{stdout}");
    assert!(!dev.exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn refuses_to_discard_uncommitted_work() {
    let fixture = Fixture::new("delete-dirty");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");
    std::fs::write(dev.join("scratch.txt"), "work in progress").expect("should write the file");

    let output = fixture.rt_err_in(&dev, &["delete"]);

    assert!(output.contains("git worktree remove failed"), "{output}");
    assert!(dev.exists());
    assert_eq!(fixture.registrations(), ["dev", "main"]);
}

#[test]
fn removes_the_bare_repository_with_every_worktree() {
    let fixture = Fixture::new("delete-repo");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let stdout = fixture.rt_ok(&["delete", fixture.repo().display().to_string().as_str()]);

    assert!(stdout.contains("deleted"), "{stdout}");
    assert!(!fixture.repo().exists());
    assert!(!fixture.worktree("dev").exists());
}

#[test]
fn accepts_the_remote_url() {
    let fixture = Fixture::new("delete-url");
    fixture.bare();

    let stdout = fixture.rt_ok(&["delete", URL]);

    assert!(stdout.contains("deleted"), "{stdout}");
    assert!(!fixture.repo().exists());
}
