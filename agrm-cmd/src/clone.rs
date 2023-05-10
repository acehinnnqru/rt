use clap::Args;
use std::thread;

use crate::settings::Settings;
extern crate clap;
use crate::helper;
use crate::manager::RepRoot;

#[derive(Args, Debug)]
pub struct CloneArgs {
    #[arg(short, long = "p", help = "The platform to clone from")]
    platform: String,

    #[arg(long = "ns", help = "The namespace to clone from")]
    namespace: Option<String>,

    #[arg(help = "The repositories to clone", value_parser, num_args = 1..)]
    repositories: Vec<String>,
}

pub fn run(_setting: &Settings, args: CloneArgs) -> ! {
    let rep_root: RepRoot = _setting.into();
    let _ = std::fs::create_dir_all(rep_root.get_root());

    let repos = helper::parse_repositories(args.namespace, args.repositories);

    let mut handles = vec![];

    repos.into_iter().for_each(|(ns, name)| {
        let rep_root = rep_root.clone();
        let platform = args.platform.clone();

        let handle = thread::spawn(move || {
            let target_path = rep_root.gen_target_path(&platform, &ns, &name);
            println!("Cloning {}:{}/{} to {:?}", &platform, ns, name, target_path);
            if let Err(e) = agrm_core::git::clone_default_branch(
                format!("https://{}/{}/{}", &platform, ns, name).as_str(),
                target_path.as_path().to_str().unwrap(),
            ) {
                eprintln!("{}", e);
            };
        });
        handles.push(handle);
    });

    handles.into_iter().for_each(|h| h.join().unwrap());

    std::process::exit(0)
}
