use std::path::PathBuf;

use anyhow::{bail, Result};

use crate::config::Config;
use crate::path::{mapping, remote::Remote};

pub async fn run(config: &Config, repo: Option<&str>) -> Result<()> {
    let path = match repo {
        Some(r) => {
            if r.contains("://") || r.contains('@') || r.matches('/').count() >= 2 {
                let remote = Remote::parse(r)?;
                mapping::repo_path(&config.root, &remote)
            } else {
                PathBuf::from(r)
            }
        }
        None => std::env::current_dir()?,
    };

    if !path.exists() {
        bail!("path does not exist: {}", path.display());
    }

    std::fs::remove_dir_all(&path)?;
    println!("deleted {}", path.display());
    Ok(())
}
