use crate::consts::AGRM;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    #[serde(default = "default_root_dir")]
    root: String,

    #[serde(default)]
    integrations: Integrations,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: default_root_dir(),
            integrations: Integrations::default(),
        }
    }
}

fn default_root_dir() -> String {
    String::from(Path::new(&homedir()).join(AGRM).to_str().unwrap())
}

fn read_config_file_in_queue(queue: Vec<PathBuf>) -> Config {
    let files = queue
        .iter()
        .filter(|path| path.exists())
        .collect::<Vec<&PathBuf>>();

    if files.is_empty() {
        return Config::default();
    }

    read_config_file(files[0])
}

fn read_config_file(filepath: &PathBuf) -> Config {
    println!("reading config file: {}", filepath.to_str().unwrap());
    let content = std::fs::read(filepath).unwrap();
    let conf: Config = toml::from_str(std::str::from_utf8(&content).unwrap()).unwrap();

    conf
}

impl Config {
    fn new() -> Self {
        read_config_file_in_queue(vec![
            Path::new(".").join(".agrm.toml"),
            Path::new(&homedir()).join(".agrm.toml"),
        ])
    }
}

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::new)
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Integrations {
    zoxide: bool,
}

impl Default for Integrations {
    fn default() -> Self {
        Self { zoxide: true }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

#[cfg(target_family = "windows")]
fn homedir() -> String {
    std::env::var("USERPROFILE").unwrap()
}

#[cfg(target_family = "unix")]
fn homedir() -> String {
    std::env::var("HOME").unwrap()
}

pub fn root() -> &'static str {
    &config().root
}

pub mod integrations {
    pub fn zoxide_enabled() -> bool {
        super::config().integrations.zoxide
    }
}
