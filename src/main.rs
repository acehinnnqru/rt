mod cmd;
use cmd::cli::Cli;

fn main() {
    let matches = Cli::new().cmds.get_matches();
    match matches.subcommand() {
        Some(("clone", sub_matches)) => {
            println!(
                "Cloning {}",
                sub_matches.get_one::<String>("REMOTE").expect("required")
            );
        }
        _ => unreachable!(),
    }
}
