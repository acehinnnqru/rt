use clap::{arg, Command};

#[derive(Debug)]
pub(crate) struct Clone {}

impl Clone {
    pub fn new() -> Command {
        Command::new("clone")
            .about("Clones repositories")
            .arg(arg!(<REMOTE> "The remote to clone"))
            .arg_required_else_help(true)
    }
}
