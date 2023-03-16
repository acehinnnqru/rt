pub mod global;

use serde::Deserialize;

use crate::settings::global::GlobalSettings;

const CONFIG_FILE: &str = "agrm.toml";
const DOT_CONFIG_FILE: &str = ".agrm.toml";

#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct Settings {
    pub global: GlobalSettings,
}

#[cfg(target_family = "windows")]
fn config_path() -> Vec<String> {
    debug!("Using windows config path");
    let app_data = std::env::var("AppData").unwrap();
    vec![
        format!("{}/Roaming/{}", app_data, CONFIG_FILE),
        format!("{}/{}", app_data, DOT_CONFIG_FILE),
        format!("{}/{}", app_data, DOT_CONFIG_FILE),
    ]
}

#[cfg(target_family = "unix")]
fn config_path() -> Vec<String> {
    debug!("Using unix config path");
    let home = std::env::var("HOME").unwrap();
    vec![
        format!("{}/.config/{}/{}", home, "agrm", CONFIG_FILE),
        format!("{}/.config/{}", home, CONFIG_FILE),
        format!("{}/{}", home, DOT_CONFIG_FILE),
    ]
}

impl Settings {
    pub fn new() -> Self {
        Self::build_from(config_path())
    }

    pub fn from(path: String) -> Self {
        Self::build_from(vec![path])
    }

    fn build_from(paths: Vec<String>) -> Self {
        debug!("Loading configuration from: {:?}", paths);
        let mut s = config::Config::builder();
        for path in paths {
            s = s.add_source(config::File::with_name(&path).required(false));
        }
        match s.build() {
            Err(e) => {
                error!("Error loading configuration: {}", e);
                info!("Using default configuration");
                Self::default()
            }
            Ok(s) => s.try_deserialize().unwrap_or(Self::default()),
        }
    }
}
