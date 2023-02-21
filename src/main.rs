mod cmd;
use std::process;

use cmd::cli::Cli;
use clap::Parser;

fn main() {
    if let Err(e) = Cli::try_parse().and_then(try_run) {
        println!("{}", e);
        process::exit(1);
    }
}

fn try_run(c: Cli) -> Result<(), clap::Error> {
    c.run()
}
