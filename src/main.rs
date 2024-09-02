mod repository;
use clap::Parser;

mod cli;
mod cmd;
mod config;
mod consts;
mod exec;
mod git;
mod integrations;

fn main() {
    let args = cli::Args::parse();

    cli::main(args);
}
