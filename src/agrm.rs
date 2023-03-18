use crate::{cmd::cli::Cli, settings::Settings};
use clap::crate_name;

pub static AGRM_NAME: &str = crate_name!();

pub fn run() -> ! {
    let cli = Cli::new();
    let settings = load_settings(&cli);

    cli.run(&settings)
}

fn load_settings(cli: &Cli) -> Settings {
    let s = cli
        .config
        .as_ref()
        .map_or_else(Settings::from_configs, Settings::from_file);

    let s = match s {
        Ok(s) => s,
        Err(e) => {
            error!("Loading settings error.");
            error!("Please check your configuration file.");
            error!("Error: {}", e);

            Settings::default()
        }
    };

    debug!("Settings: {:?}", s);
    s
}
