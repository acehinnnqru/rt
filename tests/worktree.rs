//! `rt wt add` and `rt wt remove`: creating and destroying checkouts.

mod common;

use common::{Fixture, assert_same_path};

#[test]
fn add_creates_a_worktree_for_a_remote_branch() {
    let fixture = Fixture::new("wt-add-remote");
    let main = fixture.bare();

    let stdout = fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let dev = fixture.worktree("dev");
    assert_same_path(&stdout, &dev);
    assert_eq!(fixture.ref_name(&dev, "HEAD"), "dev");
    assert_eq!(fixture.ref_name(&dev, "dev@{upstream}"), "origin/dev");
}

#[test]
fn add_works_from_a_nested_subdirectory() {
    let fixture = Fixture::new("wt-add-nested");
    let main = fixture.bare();
    let nested = main.join("src/deep");
    std::fs::create_dir_all(&nested).expect("should create the fixture directory");

    let stdout = fixture.rt_ok_in(&nested, &["wt", "add", "dev"]);

    assert_same_path(&stdout, &fixture.worktree("dev"));
}

#[test]
fn add_creates_a_branch_from_the_remote_default_when_it_is_new() {
    let fixture = Fixture::new("wt-add-new");
    let main = fixture.bare();

    let stdout = fixture.rt_ok_in(&main, &["wt", "add", "feature/login"]);

    let feature = fixture.worktree("feature/login");
    assert_same_path(&stdout, &feature);
    assert_eq!(fixture.ref_name(&feature, "HEAD"), "feature/login");
    assert_eq!(
        fixture
            .git(&feature, &["rev-list", "--count", "HEAD"])
            .trim(),
        "2"
    );
    assert!(
        fixture
            .branches()
            .iter()
            .any(|branch| branch == "feature/login"),
        "{:?}",
        fixture.branches()
    );
}

#[test]
fn add_fetches_a_branch_published_after_the_clone() {
    let fixture = Fixture::new("wt-add-fetch");
    let main = fixture.bare();
    fixture.publish_branch("later", "main");

    let stdout = fixture.rt_ok_in(&main, &["wt", "add", "later"]);

    let later = fixture.worktree("later");
    assert_same_path(&stdout, &later);
    assert_eq!(fixture.ref_name(&later, "later@{upstream}"), "origin/later");
}

#[test]
fn add_without_fetch_does_not_see_a_branch_published_after_the_clone() {
    let fixture = Fixture::new("wt-add-no-fetch");
    let main = fixture.bare();
    fixture.publish_branch("later", "main");

    let stdout = fixture.rt_ok_in(&main, &["wt", "add", "later", "--no-fetch"]);

    let later = fixture.worktree("later");
    assert_same_path(&stdout, &later);
    // Without the fetch `origin/later` is unknown, so this is a new branch off the
    // remote's default, not the commit that upstream published.
    assert_eq!(
        fixture.rev(&later, "HEAD"),
        fixture.rev(&fixture.repo(), "refs/remotes/origin/HEAD")
    );
    assert_eq!(fixture.ref_name(&later, "later@{upstream}"), "origin/main");
}

#[test]
fn add_starts_from_an_explicit_reference() {
    let fixture = Fixture::new("wt-add-from");
    let main = fixture.bare();

    let stdout = fixture.rt_ok_in(&main, &["wt", "add", "feature", "--from", "origin/dev"]);

    let feature = fixture.worktree("feature");
    assert_same_path(&stdout, &feature);
    assert_eq!(
        fixture.rev(&feature, "HEAD"),
        fixture.rev(&fixture.repo(), "refs/remotes/origin/dev")
    );
    // git's own autoSetupMerge rule: a start point that is a remote-tracking branch
    // becomes the new branch's upstream.
    assert_eq!(
        fixture.ref_name(&feature, "feature@{upstream}"),
        "origin/dev"
    );
}

#[test]
fn add_rejects_an_unknown_start_point() {
    let fixture = Fixture::new("wt-add-from-unknown");
    let main = fixture.bare();

    let output = fixture.rt_err_in(&main, &["wt", "add", "feature", "--from", "origin/nope"]);

    assert!(output.contains("is not a valid start point"), "{output}");
    assert!(!fixture.worktree("feature").exists());
}

#[test]
fn add_checks_out_an_existing_local_branch_without_moving_it() {
    let fixture = Fixture::new("wt-add-local");
    let main = fixture.bare();
    let repo = fixture.repo();
    // A local `dev` that trails the one on the remote: the local ref wins and stays put.
    fixture.git(&repo, &["branch", "dev", "origin/main"]);
    let local = fixture.rev(&repo, "refs/heads/dev");

    let stdout = fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let dev = fixture.worktree("dev");
    assert_same_path(&stdout, &dev);
    assert_eq!(fixture.rev(&dev, "HEAD"), local);
    assert_eq!(fixture.rev(&repo, "refs/heads/dev"), local);
}

#[test]
fn add_rejects_an_existing_local_branch_with_from() {
    let fixture = Fixture::new("wt-add-from-existing");
    let main = fixture.bare();
    fixture.git(&fixture.repo(), &["branch", "feature", "origin/dev"]);

    let output = fixture.rt_err_in(&main, &["wt", "add", "feature", "--from", "origin/main"]);

    assert!(output.contains("already exists"), "{output}");
    assert!(!fixture.worktree("feature").exists());
}

#[test]
fn add_refuses_a_branch_that_is_already_checked_out() {
    let fixture = Fixture::new("wt-add-twice");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = common::canonical(&fixture.worktree("dev"));

    let output = fixture.rt_err_in(&main, &["wt", "add", "dev"]);

    assert!(output.contains("already checked out at"), "{output}");
    assert!(
        output.contains(&*dev.to_string_lossy()),
        "the existing checkout should be named: {output}"
    );
}

#[test]
fn add_rejects_a_branch_that_git_keeps_for_itself() {
    let fixture = Fixture::new("wt-add-reserved");
    let main = fixture.bare();

    let output = fixture.rt_err_in(&main, &["wt", "add", "objects"]);

    assert!(output.contains("git directory entry"), "{output}");
    assert!(
        !fixture.branches().iter().any(|branch| branch == "objects"),
        "{:?}",
        fixture.branches()
    );
}

#[test]
fn remove_takes_a_branch_name() {
    let fixture = Fixture::new("wt-remove-branch");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");

    let stdout = fixture.rt_ok_in(&main, &["wt", "rm", "dev"]);

    assert!(stdout.contains("removed worktree"), "{stdout}");
    assert!(!dev.exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn remove_takes_a_path() {
    let fixture = Fixture::new("wt-remove-path");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");

    let stdout = fixture.rt_ok_in(&main, &["wt", "remove", dev.to_string_lossy().as_ref()]);

    assert!(stdout.contains("removed worktree"), "{stdout}");
    assert!(!dev.exists());
}

#[test]
fn remove_rejects_an_unknown_target() {
    let fixture = Fixture::new("wt-remove-unknown");
    let main = fixture.bare();

    let output = fixture.rt_err_in(&main, &["wt", "rm", "nope"]);

    assert!(output.contains("no worktree for branch 'nope'"), "{output}");
}

#[test]
fn remove_refuses_a_dirty_worktree_unless_forced() {
    let fixture = Fixture::new("wt-remove-dirty");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let dev = fixture.worktree("dev");
    std::fs::write(dev.join("scratch.txt"), "work in progress").expect("should write the file");

    let output = fixture.rt_err_in(&main, &["wt", "rm", "dev"]);

    assert!(output.contains("git worktree remove failed"), "{output}");
    assert!(dev.exists());

    fixture.rt_ok_in(&main, &["wt", "rm", "dev", "--force"]);

    assert!(!dev.exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn remove_refuses_the_worktree_it_runs_in() {
    let fixture = Fixture::new("wt-remove-self");
    let main = fixture.bare();

    let output = fixture.rt_err_in(&main, &["wt", "rm", "main"]);

    assert!(
        output.contains("refusing to remove the worktree this command runs in"),
        "{output}"
    );
    assert!(main.exists());
    assert_eq!(fixture.registrations(), ["main"]);
}

#[test]
fn remove_without_a_terminal_asks_for_a_branch_name() {
    let fixture = Fixture::new("wt-remove-no-tty");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let output = fixture.rt_err_in(&main, &["wt", "rm"]);

    assert!(output.contains("pass a branch name"), "{output}");
    assert!(fixture.worktree("dev").exists());
}

#[test]
fn remove_without_arguments_reports_when_there_is_nothing_to_remove() {
    let fixture = Fixture::new("wt-remove-nothing");
    let main = fixture.bare();

    let stdout = fixture.rt_ok_in(&main, &["wt", "rm"]);

    assert!(stdout.contains("no removable worktrees found"), "{stdout}");
}
