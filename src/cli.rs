use std::path::Path;

use crate::{config, consts::AGRM, git, integrations, repository::Repository};

fn long_about() -> String {
    "
`agrm` will clone a bare repository into a directory which named as `{root}/{git platform}/{namespace}/{name}/.bare`.

The params in the directory name:

- `root` is the root from config file `{$HOME}/.agrm.toml`
- `git platform` means the target platform in the provided repository url/ssh.
- `namespace` and `name` are also extract from the provided repository url/ssh.".to_string()
}

#[derive(clap::Parser)]
#[command(name = AGRM)]
#[command(bin_name = AGRM)]
#[command(version)]
#[command(about, long_about = long_about())]
pub struct Args {
    #[arg(value_parser = clap::value_parser!(Repository))]
    pub repository: Repository,
}

pub fn main(args: Args) {
    let root = config::root();

    println!("{:?}", config::root());

    if root.trim().is_empty() {
        unreachable!("\ninvalide root setting")
    }

    println!("\nroot: {}\n", root);

    let repo = args.repository;

    println!("\nparsed repository: {:?}", repo);

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
