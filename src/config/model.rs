use serde::{Deserialize, Serialize};

fn default_root() -> String {
    dirs::home_dir()
        .map(|h| h.join("r").to_string_lossy().into_owned())
        .unwrap_or_else(|| "~/r".to_string())
}

fn default_depth() -> u32 {
    1
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default)]
    pub clone: CloneConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: default_root(),
            clone: CloneConfig::default(),
            integrations: IntegrationsConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CloneConfig {
    #[serde(default = "default_depth")]
    pub depth: u32,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            depth: default_depth(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct IntegrationsConfig {
    #[serde(default)]
    pub zoxide: bool,
    #[serde(default)]
    pub autojump: bool,
}
