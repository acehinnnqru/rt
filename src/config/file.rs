use std::path::PathBuf;

use anyhow::{Context, Result};

use super::model::Config;

pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".rt.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;

    if path.exists() {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        config.root = expand_root(&config.root);
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

fn expand_root(root: &str) -> String {
    if (root.starts_with("~/") || root == "~") && let Some(home) = dirs::home_dir() {
        return root.replacen("~", &home.to_string_lossy(), 1);
    }
    if let Ok(home) = std::env::var("HOME") {
        return root.replace("{$HOME}", &home).replace("$HOME", &home);
    }
    root.to_string()
}
