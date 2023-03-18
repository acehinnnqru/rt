use agrm_settings::Settings;
use agrm_cmd::cli::Cli;

pub fn load(cli: &Cli) -> Settings {
    let s = cli.config.as_ref().map(Settings::from_file).unwrap();

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
