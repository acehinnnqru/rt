use clap::Args;

use crate::settings::Settings;
extern crate clap;

#[derive(Args, Debug)]
pub struct CloneArgs {
    #[arg(short, long, help = "The platform to clone from")]
    platform: String,
    #[arg(help = "The repositories to clone", value_parser, num_args = 1..)]
    repositories: Vec<String>,
}

pub fn run(_setting: &Settings, _args: CloneArgs) -> ! {
    todo!()
}
