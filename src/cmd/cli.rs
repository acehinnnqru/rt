use crate::cmd::clone;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
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
