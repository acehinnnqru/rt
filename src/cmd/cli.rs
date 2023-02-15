use clap::Command;

use super::clone::Clone;

#[derive(Debug)]
pub struct Cli {
    pub cmds: Command,
}

impl Cli {
    pub fn new() -> Self {
        Cli { cmds: default() }
    }
}

fn default() -> Command {
    Command::new("agrm")
        .about("A git repositories manager for local usage.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Clone::new())
}
