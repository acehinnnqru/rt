pub mod global;

use config::ConfigError;
use serde::Deserialize;

use crate::global::AGRM_NAME;
use crate::settings::global::GlobalSettings;

const CONFIG_FILE: &str = "agrm.toml";
const DOT_CONFIG_FILE: &str = ".agrm.toml";

pub type SettingsError = ConfigError;

#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct Settings {
    pub global: Option<GlobalSettings>,
}

#[cfg(target_family = "windows")]
fn config_path() -> Vec<String> {
    let app_data = std::env::var("AppData").unwrap();
    vec![
        format!("{}/Roaming/{}", app_data, CONFIG_FILE),
        format!("{}/{}", app_data, DOT_CONFIG_FILE),
        format!("{}/{}/{}", app_data, AGRM_NAME, DOT_CONFIG_FILE),
    ]
}

#[cfg(target_family = "unix")]
fn config_path() -> Vec<String> {
    let home = std::env::var("HOME").unwrap();
    vec![
        format!("{}/.config/{}/{}", home, AGRM_NAME, CONFIG_FILE),
        format!("{}/.config/{}", home, CONFIG_FILE),
        format!("{}/{}", home, DOT_CONFIG_FILE),
    ]
}

impl Settings {
    pub fn from_configs() -> Result<Settings, SettingsError> {
        Self::build_from(config_path())
    }

    pub fn from_file(path: String) -> Result<Settings, SettingsError> {
        let s = config::Config::builder().add_source(config::File::with_name(&path));
        match s.build() {
            Err(e) => Err(e),
            Ok(s) => s.try_deserialize(),
        }
    }

    fn build_from(paths: Vec<String>) -> Result<Settings, SettingsError> {
        let mut s = config::Config::builder();
        for path in paths {
            s = s.add_source(config::File::with_name(&path).required(false));
        }

        // s.build() won't fail because all sources are optional
        s.build().unwrap().try_deserialize()
    }
}
