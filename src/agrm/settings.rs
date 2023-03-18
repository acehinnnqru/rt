use agrm_cmd::cli::Cli;
use agrm_settings::Settings;

pub fn load(cli: &Cli) -> Settings {
    let s = cli.config.as_ref().map(Settings::from_file).unwrap();

    let s = match s {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Loading settings error.");
            eprintln!("Please check your configuration file.");

            Settings::default()
        }
    };

    s
}
