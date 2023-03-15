pub mod global;

use config::ConfigError;
use log::{debug, error, info};
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
    vec![
        format!(
            "{}/AppData/Roaming/{}",
            std::env::var("HOME").unwrap(),
            CONFIG_FILE
        ),
        format!("{}/{}", std::env::var("HOME").unwrap(), DOT_CONFIG_FILE),
    ]
}

#[cfg(target_family = "unix")]
fn config_path() -> Vec<String> {
    debug!("Using unix config path");
    vec![
        format!("{}/.config/{}", std::env::var("HOME").unwrap(), CONFIG_FILE),
        format!("{}/{}", std::env::var("HOME").unwrap(), DOT_CONFIG_FILE),
    ]
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        Self::build_from(config_path())
    }
    pub fn from(path: String) -> Result<Self, ConfigError> {
        Self::build_from(vec![path])
    }
    fn build_from(paths: Vec<String>) -> Result<Self, ConfigError> {
        debug!("Loading configuration from: {:?}", paths);
        let mut s = config::Config::builder();
        for path in paths {
            s = s.add_source(config::File::with_name(&path));
        }
        match s.build() {
            Err(e) => {
                error!("Error loading configuration: {}", e);
                info!("Using default configuration");
                Ok(Self::default())
            }
            Ok(s) => s.try_deserialize(),
        }
    }
}
