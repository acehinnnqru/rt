mod cmd;
mod logging;
pub mod settings;
use std::process;

use clap::Parser;
use cmd::cli::Cli;
use log::error;

fn main() {
    if let Err(e) = Cli::try_parse().and_then(try_run) {
        error!("{}", e);
        process::exit(1);
    }
}

fn try_run(c: Cli) -> Result<(), clap::Error> {
    c.run()
}
