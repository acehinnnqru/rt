use std::{path::Path, str::FromStr};

mod repository;
use repository::Repository;

mod cmd;
mod config;
mod git;
mod integrations;

fn help() {
    let usage = "
    agrm {{repository url/ssh}}

        `agrm` will clone a bare repository into a directory which named as `{{root}}/{{git platform}}/{{namespace}}/{{name}}/.bare`.

        The params in the directory name:

        - `root` is the root from config file `{{$HOME}}/.agrm.toml`
        - `git platform` means the target platform in the provided repository url/ssh.
        - `namespace` and `name` are also extract from the provided repository url/ssh.
    ";

    println!("{}", usage);
}

fn main() {
    let path = std::env::args().nth(1).expect("no repository given");

    if path.trim() == "--help" {
        help();
        return;
    }

    let repo = Repository::from_str(&path).unwrap();

    println!("\nparsed repository: {:?}", repo);

    config::init();

    let root = config::root();

    if root.trim().is_empty() {
        unreachable!("\ninvalide root setting")
    }

    println!("\nroot: {}\n", root);

    let target_dir = Path::new(&root)
        .join(&repo.platform)
        .join(&repo.namespace)
        .join(&repo.name);

    let ssh = format!("git@{}:{}/{}", repo.platform, repo.namespace, repo.name);

    let target_bare_dir = target_dir.join(".bare");

    println!(
        "\ntrying to clone into: {}\n",
        target_bare_dir.to_str().unwrap()
    );

    git::clone_bare(&ssh, &target_bare_dir);

    println!(
        "\ncloned {} into {}",
        ssh,
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
