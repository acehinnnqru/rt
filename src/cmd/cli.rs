use clap::{Parser, Subcommand};
use super::clone::Clone;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}


impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn run(&self) {
        match &self.command {
            Some(_clone) => {
                println!("Clone a repository into a new directory")
            },
            None => todo!(),
        }
    }
}


#[derive(Subcommand)]
enum Commands {
    /// Clones repositories
   Clone(Clone)
}
