use std::{path::Path, sync::OnceLock};

#[derive(serde::Deserialize)]
struct Config {
    #[serde(default = "default_root_dir")]
    root: String,

    #[serde(default)]
    integrations: Integrations,
}

fn default_root_dir() -> String {
    String::from(Path::new(&homedir()).join("agrm").to_str().unwrap())
}

impl Config {
    fn new() -> Self {
        let filepath = Path::new(&homedir()).join(".agrm.toml");
        println!("reading config file: {}", filepath.to_str().unwrap());
        let content = std::fs::read(&filepath).unwrap();
        let conf: Config = toml::from_str(std::str::from_utf8(&content).unwrap()).unwrap();

        conf
    }
}

pub fn init() {
    CONFIG.get_or_init(Config::new);
}

#[derive(serde::Deserialize)]
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

pub fn root() -> String {
    CONFIG.get().unwrap().root.clone()
}

pub mod integrations {
    use super::CONFIG;

    pub fn zoxide_enabled() -> bool {
        CONFIG.get().unwrap().integrations.zoxide
    }
}
