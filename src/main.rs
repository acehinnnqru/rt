mod cmd;
use cmd::cli::Cli;

fn main() {
    let cli = Cli::new();
    cli.run();
}
