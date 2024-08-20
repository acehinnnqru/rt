use std::path::Path;

mod repository;
use clap::Parser;
use repository::Repository;

mod cmd;
mod config;
mod consts;
mod git;
mod integrations;

fn long_about() -> String {
    "
`agrm` will clone a bare repository into a directory which named as `{root}/{git platform}/{namespace}/{name}/.bare`.

The params in the directory name:

- `root` is the root from config file `{$HOME}/.agrm.toml`
- `git platform` means the target platform in the provided repository url/ssh.
- `namespace` and `name` are also extract from the provided repository url/ssh.".to_string()
}

#[derive(clap::Parser)]
#[command(name = consts::AGRM)]
#[command(bin_name = consts::AGRM)]
#[command(version)]
#[command(about, long_about = long_about())]
struct Args {
    #[arg(value_parser = clap::value_parser!(Repository))]
    repository: Repository,
}

fn main() {
    let args = Args::parse();

    let repo = args.repository;

    println!("\nparsed repository: {:?}", repo);

    let root = config::root();

    println!("{:?}", config::root());

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
