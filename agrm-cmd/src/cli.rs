use std::process;

use crate::{clone, settings::Settings};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = "A Git Repository Manager")]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn new() -> Self {
        match Self::try_parse() {
            Ok(cli) => cli,
            Err(e) => {
                println!("{}", e);
                process::exit(1)
            }
        }
    }

    pub fn run(self, settings: &Settings) -> ! {
        match self.command {
            Some(Commands::Clone(args)) => clone::run(settings, args),
            None => todo!(),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Clones repositories
    #[command(arg_required_else_help = true)]
    Clone(clone::CloneArgs),
}
