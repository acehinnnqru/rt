//! `rt worktree add` and `rt delete` inside the bare layout.

mod common;

use common::{Fixture, URL, assert_same_path};

#[test]
fn add_creates_a_branch_from_the_remote_default_when_it_is_new() {
    let fixture = Fixture::new("wt-add-new");
    fixture.rt_ok(&["clone", URL, "--worktree"]);
    let main = fixture.worktree("main");

    let stdout = fixture.rt_ok_in(&main, &["worktree", "add", "feature/login"]);

    let feature = fixture.worktree("feature/login");
    assert_same_path(&stdout, &feature);
    assert_eq!(
        fixture
            .git(&feature, &["rev-parse", "--abbrev-ref", "HEAD"])
            .trim(),
        "feature/login"
    );
    assert_eq!(
        fixture
            .git(&feature, &["rev-list", "--count", "HEAD"])
            .trim(),
        "2"
    );
    assert!(
        fixture.registered().iter().any(|name| name == "login"),
        "{:?}",
        fixture.registered()
    );
}

#[test]
fn delete_removes_a_linked_worktree_and_its_registration() {
    let fixture = Fixture::new("wt-delete");
    fixture.rt_ok(&["clone", URL, "--worktree"]);
    let main = fixture.worktree("main");
    fixture.rt_ok_in(&main, &["worktree", "add", "dev"]);
    let dev = fixture.worktree("dev");
    assert_eq!(fixture.registrations(), ["dev", "main"]);

    let stdout = fixture.rt_ok_in(&dev, &["delete"]);

    assert!(stdout.contains("deleted worktree"), "{stdout}");
    assert!(!dev.exists());
    assert_eq!(fixture.registrations(), ["main"]);
    assert!(
        !fixture.registered().iter().any(|name| name == "dev"),
        "dev is still registered: {:?}",
        fixture.registered()
    );
}

#[test]
fn delete_refuses_to_discard_uncommitted_work() {
    let fixture = Fixture::new("wt-delete-dirty");
    fixture.rt_ok(&["clone", URL, "--worktree"]);
    let main = fixture.worktree("main");
    fixture.rt_ok_in(&main, &["worktree", "add", "dev"]);
    let dev = fixture.worktree("dev");
    std::fs::write(dev.join("scratch.txt"), "work in progress").expect("should write the file");

    let output = fixture.rt_err_in(&dev, &["delete"]);

    assert!(output.contains("git worktree remove failed"), "{output}");
    assert!(dev.exists());
    assert_eq!(fixture.registrations(), ["dev", "main"]);
}

#[test]
fn delete_removes_the_bare_repository_with_every_worktree() {
    let fixture = Fixture::new("wt-delete-repo");
    fixture.rt_ok(&["clone", URL, "--worktree"]);
    let main = fixture.worktree("main");
    fixture.rt_ok_in(&main, &["worktree", "add", "dev"]);

    let stdout = fixture.rt_ok(&["delete", fixture.repo().display().to_string().as_str()]);

    assert!(stdout.contains("deleted"), "{stdout}");
    assert!(!fixture.repo().exists());
    assert!(!fixture.worktree("dev").exists());
}

#[test]
fn delete_accepts_the_remote_url() {
    let fixture = Fixture::new("wt-delete-url");
    fixture.rt_ok(&["clone", URL, "--worktree"]);

    let stdout = fixture.rt_ok(&["delete", URL]);

    assert!(stdout.contains("deleted"), "{stdout}");
    assert!(!fixture.repo().exists());
}
