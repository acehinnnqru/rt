use std::{path::Path, str::FromStr};

mod repository;
use repository::Repository;

mod cmd;
mod config;
mod git;
mod integrations;

fn main() {
    let path = std::env::args().nth(1).expect("no repository given");

    let repo = Repository::from_str(&path).unwrap();

    println!("\nparsed repository: {:?}\n", repo);

    config::init();

    let root = config::root();

    if root.trim().is_empty() {
        unreachable!("invalide root setting")
    }

    println!("root: {}", root);

    let target_dir = Path::new(&root)
        .join(&repo.platform)
        .join(&repo.namespace)
        .join(&repo.name);

    let ssh = format!("git@{}:{}/{}", repo.platform, repo.namespace, repo.name);

    git::clone_bare(&ssh, &target_dir.join(".bare"));

    write_dot_git(&target_dir, "gitdir: ./.bare");

    if config::integrations::zoxide_enabled() {
        integrations::zoxide(&target_dir);
    }
}

fn write_dot_git(dir: &Path, content: &str) {
    std::fs::write(dir.join(".git"), content).unwrap();
}
