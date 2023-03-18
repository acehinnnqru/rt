use agrm_cmd as cmd;

mod settings;

use cmd::cli::Cli;

pub fn main() -> ! {
    init();

    let cli = Cli::new();
    let settings = settings::load(&cli);

    cli.run(&settings)
}

pub fn init() {}
