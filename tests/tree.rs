//! `rt tree` rendering of the bare layout.

mod common;

use common::{Fixture, URL};

#[test]
fn shows_a_bare_repository_with_its_worktrees_and_no_git_entries() {
    let fixture = Fixture::new("tree-bare");
    fixture.rt_ok(&["clone", URL, "--worktree"]);
    let main = fixture.worktree("main");
    fixture.rt_ok_in(&main, &["worktree", "add", "dev"]);

    let stdout = fixture.rt_ok(&["tree"]);
    let names: Vec<&str> = stdout.lines().skip(1).map(node_name).collect();

    assert_eq!(names, ["example.com", "acme", "widget", "dev", "main"]);
}

#[test]
fn shows_an_ordinary_clone_as_a_leaf() {
    let fixture = Fixture::new("tree-plain");
    fixture.rt_ok(&["clone", URL]);

    let stdout = fixture.rt_ok(&["tree"]);
    let names: Vec<&str> = stdout.lines().skip(1).map(node_name).collect();

    assert_eq!(names, ["example.com", "acme", "widget"]);
}

/// Strips the connectors from a rendered tree line.
fn node_name(line: &str) -> &str {
    line.trim_start_matches(|c: char| c == ' ' || "│├└─".contains(c))
}
