use std::path::Path;

use crate::{config, git, integrations, repository::Repository};

pub fn print_config() {
    println!("agrm config:");
    println!("{:?}", config::config());
}

pub fn clone(root: &str, repo: &Repository) {
    let target_dir = Path::new(&root)
        .join(&repo.platform)
        .join(&repo.namespace)
        .join(&repo.name);

    let target_bare_dir = target_dir.join(".bare");

    println!(
        "\ntrying to clone into: {}\n",
        target_bare_dir.to_str().unwrap()
    );

    let repo_ssh = repo.ssh();
    git::clone_bare(&repo_ssh, &target_bare_dir);

    println!(
        "\ncloned {} into {}",
        repo_ssh,
        target_bare_dir.to_str().unwrap()
    );

    write_dot_git(&target_dir, "gitdir: ./.bare");
    println!("\nwrote stuff to {}/.git", target_dir.to_str().unwrap());

    if config::integrations::zoxide_enabled() {
        integrations::zoxide(&target_dir);
    }
}

fn write_dot_git(dir: &Path, content: &str) {
    std::fs::write(dir.join(".git"), content).unwrap();
}
