mod cmd;
use std::process;

use cmd::cli::Cli;
use clap::Parser;

fn main() {
    if let Err(e) = Cli::try_parse().and_then(try_main) {
        println!("{}", e);
        process::exit(1);
    }
}

fn try_main(c: Cli) -> Result<(), clap::Error> {
    c.run()
}
