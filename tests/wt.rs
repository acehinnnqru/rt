//! `rt wt list`, `path`, `prune`, `lock` / `unlock`, `move`, and how the command group
//! resolves the repository it acts on.

mod common;

use std::path::Path;

use common::{Fixture, URL, assert_same_path, canonical};

#[test]
fn list_shows_the_checkouts_sorted_and_without_the_bare_repository() {
    let fixture = Fixture::new("wt-list");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    fixture.rt_ok_in(&main, &["wt", "add", "feature/login"]);

    let stdout = fixture.rt_ok_in(&main, &["wt", "list"]);

    assert_eq!(
        relative_rows(&stdout, &fixture.repo()),
        ["dev\tdev", "feature/login\tfeature/login", "main\tmain"]
    );
}

#[test]
fn list_paths_prints_absolute_paths_only() {
    let fixture = Fixture::new("wt-list-paths");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let stdout = fixture.rt_ok_in(&main, &["wt", "list", "--paths"]);

    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            canonical(&fixture.worktree("dev")).to_string_lossy(),
            canonical(&fixture.worktree("main")).to_string_lossy(),
        ]
    );
}

#[test]
fn path_prints_the_worktree_of_a_branch() {
    let fixture = Fixture::new("wt-path");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let stdout = fixture.rt_ok_in(&main, &["wt", "path", "dev"]);

    assert_same_path(&stdout, &fixture.worktree("dev"));
}

#[test]
fn path_rejects_a_branch_without_a_worktree() {
    let fixture = Fixture::new("wt-path-missing");
    let main = fixture.bare();

    let output = fixture.rt_err_in(&main, &["wt", "path", "dev"]);

    assert!(output.contains("no worktree for branch 'dev'"), "{output}");
}

#[test]
fn prune_collects_a_worktree_whose_directory_is_gone() {
    let fixture = Fixture::new("wt-prune");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    std::fs::remove_dir_all(fixture.worktree("dev")).expect("should delete the checkout by hand");

    let listed = fixture.rt_ok_in(&main, &["wt", "list"]);
    assert!(listed.contains("prunable"), "{listed}");

    fixture.rt_ok_in(&main, &["wt", "prune"]);

    assert_eq!(fixture.registrations(), ["main"]);
    assert!(!listed_is_prunable(&fixture, &main));
}

#[test]
fn lock_records_a_reason_and_unlock_clears_it() {
    let fixture = Fixture::new("wt-lock");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let stdout = fixture.rt_ok_in(&main, &["wt", "lock", "dev", "--reason", "release freeze"]);

    assert!(stdout.contains("locked"), "{stdout}");
    assert!(
        fixture
            .git(&fixture.repo(), &["worktree", "list", "--porcelain"])
            .contains("locked release freeze")
    );
    assert!(
        fixture
            .rt_ok_in(&main, &["wt", "list"])
            .contains("\tlocked")
    );

    let stdout = fixture.rt_ok_in(&main, &["wt", "unlock", "dev"]);

    assert!(stdout.contains("unlocked"), "{stdout}");
    assert!(
        !fixture
            .rt_ok_in(&main, &["wt", "list"])
            .contains("\tlocked")
    );
}

#[test]
fn move_relocates_a_worktree_and_keeps_it_resolvable() {
    let fixture = Fixture::new("wt-move");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);
    let hotfix = fixture.repo().join("hotfix");

    let stdout = fixture.rt_ok_in(
        &main,
        &["wt", "move", "dev", hotfix.to_string_lossy().as_ref()],
    );

    assert_same_path(&stdout, &hotfix);
    assert!(!fixture.worktree("dev").exists());
    assert!(hotfix.join(".git").is_file());
    assert_same_path(&fixture.rt_ok_in(&main, &["wt", "path", "dev"]), &hotfix);
}

#[test]
fn add_in_an_ordinary_clone_creates_a_sibling_of_the_main_worktree() {
    let fixture = Fixture::new("wt-clone-layout");
    fixture.rt_ok(&["clone", URL]);
    let nested = fixture.repo().join("src/deep");
    std::fs::create_dir_all(&nested).expect("should create the fixture directory");

    let stdout = fixture.rt_ok_in(&nested, &["wt", "add", "dev"]);

    let sibling = fixture.mapped("widget-dev");
    assert_same_path(&stdout, &sibling);
    assert_eq!(fixture.ref_name(&sibling, "HEAD"), "dev");
}

#[test]
fn wt_and_worktree_are_the_same_command() {
    let fixture = Fixture::new("wt-alias");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "a", "dev"]);

    let short = fixture.rt_ok_in(&main, &["wt", "l"]);
    let long = fixture.rt_ok_in(&main, &["worktree", "list"]);

    assert_eq!(short, long);
    assert!(short.contains("dev"), "{short}");
}

#[test]
fn the_delete_spelling_still_removes_a_worktree() {
    let fixture = Fixture::new("wt-delete-alias");
    let main = fixture.bare();
    fixture.rt_ok_in(&main, &["wt", "add", "dev"]);

    let stdout = fixture.rt_ok_in(&main, &["worktree", "delete", "dev"]);

    assert!(stdout.contains("removed worktree"), "{stdout}");
    assert!(!fixture.worktree("dev").exists());
}

#[test]
fn every_command_fails_outside_a_repository() {
    let fixture = Fixture::new("wt-outside");
    let commands = [
        vec!["wt", "add", "dev"],
        vec!["wt", "list"],
        vec!["wt", "remove", "dev"],
        vec!["wt", "prune"],
        vec!["wt", "path", "dev"],
        vec!["wt", "lock", "dev"],
        vec!["wt", "unlock", "dev"],
        vec!["wt", "move", "dev", "out"],
        vec!["worktree", "list"],
    ];

    for args in &commands {
        let output = fixture.rt_err(args);
        assert!(
            output.contains("not inside a git repository"),
            "rt {args:?}: {output}"
        );
    }
}

/// The `list` rows with the repository prefix stripped, so the assertion does not depend on
/// the temp directory path.
fn relative_rows(stdout: &str, repo: &Path) -> Vec<String> {
    let repo = canonical(repo);
    stdout
        .lines()
        .map(|line| {
            let (branch, path) = line.split_once('\t').expect("two tab-separated fields");
            let path = Path::new(path)
                .strip_prefix(&repo)
                .unwrap_or_else(|_| panic!("{path} should be inside {}", repo.display()));
            format!("{branch}\t{}", path.display())
        })
        .collect()
}

/// Whether the checkout is still registered as prunable.
fn listed_is_prunable(fixture: &Fixture, cwd: &Path) -> bool {
    fixture.rt_ok_in(cwd, &["wt", "list"]).contains("prunable")
}
