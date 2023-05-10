use std::process;

use crate::{clone, settings::Settings};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agrm")]
#[command(author, version, about, long_about = "A Git Repository Manager")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Self {
        match Self::try_parse() {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1)
            }
        }
    }

    pub fn run(self, settings: &Settings) -> ! {
        match self.command {
            Some(Commands::Clone(args)) => clone::run(settings, args),
            Some(Commands::Settings(_)) => {
                println!("{:#?}", settings);
                process::exit(0)
            }
            None => todo!(),
        }
    }
}

#[derive(Parser)]
struct Empty {}

#[derive(Subcommand)]
enum Commands {
    /// Clones repositories
    #[command(arg_required_else_help = true)]
    Clone(clone::CloneArgs),

    /// Show settings
    Settings(Empty),
}
