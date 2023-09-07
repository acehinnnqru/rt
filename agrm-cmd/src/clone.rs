use clap::Args;
use std::path::Path;
use std::thread;

use crate::settings::Settings;
extern crate clap;
use crate::helper;

#[derive(Args, Debug)]
pub struct CloneArgs {
    #[arg(help = "The links of repositories to clone", value_parser, num_args = 1..)]
    repositories: Vec<String>,
}

pub fn run(_setting: &Settings, args: CloneArgs) -> ! {
    let global_root = _setting.global.clone().unwrap().root.unwrap().clone();
    let _ = std::fs::create_dir_all(global_root.as_str());

    let repos = args
        .repositories
        .into_iter()
        .map(helper::parse_repository)
        .collect::<Vec<_>>();

    let mut handles = vec![];

    repos.into_iter().for_each(|repo| {
        let global_root = global_root.clone();
        let handle = thread::spawn(move || {
            if let Some(repo) = repo {
                let target_path = Path::new(global_root.clone().as_str())
                    .join(repo.get_host())
                    .join(repo.get_namespace())
                    .join(repo.get_name());

                if let Err(e) = agrm_core::git::clone_default_branch(
                    repo.get_ssh_url().as_str(),
                    target_path.as_path().to_str().unwrap(),
                ) {
                    eprintln!("{}", e);
                };
            }
        });
        handles.push(handle);
    });

    handles.into_iter().for_each(|h| h.join().unwrap());

    std::process::exit(0)
}
