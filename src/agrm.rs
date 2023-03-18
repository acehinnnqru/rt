use crate::{cmd::cli::Cli, settings::Settings};
use clap::crate_name;

pub static AGRM_NAME: &str = crate_name!();

pub fn run() -> ! {
    init();

    let cli = Cli::new();
    let settings = load_settings(&cli);

    cli.run(&settings)
}

pub fn init() {
    init_logger();
}

fn init_logger() {
    let env = env_logger::Env::default().filter_or("AGRM_LOG", "info");

    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .init();
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
