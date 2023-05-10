pub mod global;
pub mod local;

use config::ConfigError;
use serde::Deserialize;

use crate::global::GlobalSettings;

pub(crate) const AGRM_NAME: &str = "agrm";

pub type SettingsError = ConfigError;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Settings {
    pub global: Option<GlobalSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            global: Some(GlobalSettings::default()),
        }
    }
}

impl Settings {
    fn merge(self, target: Settings) -> Settings {
        match self.global {
            None => target,
            Some(global) => Settings {
                global: Some(global.merge(target.global)),
            },
        }
    }

    pub fn from_multi_files(configs: Vec<String>) -> Result<Settings, SettingsError> {
        match Self::build_from(configs) {
            Ok(s) => Ok(Self::default().merge(s)),
            Err(e) => Err(e),
        }
    }

    pub fn from_file(path: &str) -> Result<Settings, SettingsError> {
        let s = config::Config::builder().add_source(config::File::with_name(path));
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
