use std::process;

use crate::cmd::clone;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = "A Git Repository Manager")]
pub(crate) struct Cli {
    #[arg(short, long)]
    pub config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn new() -> Option<Self> {
        match Self::try_parse() {
            Ok(cli) => Some(cli),
            Err(e) => {
                error!("Parsing parameters error.");
                error!("Please read the usage below.");
                println!();
                println!("{}", e);
                process::exit(1)
            }
        }
    }

    pub fn run(self) -> ! {
        match self.command {
            Some(Commands::Clone(args)) => clone::run(args),
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
