use anyhow::{bail, Result};

use crate::config::Config;
use crate::integrations;
use crate::path::{mapping, remote::Remote};

pub async fn run(config: &Config, url: &str) -> Result<()> {
    let remote = Remote::parse(url)?;
    let dest = mapping::repo_path(&config.root, &remote);

    if dest.exists() {
        bail!("repository already exists at {}", dest.display());
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut cmd = tokio::process::Command::new("git");
    cmd.arg("clone");

    if config.clone.depth > 0 {
        cmd.arg("--depth").arg(config.clone.depth.to_string());
    }

    cmd.arg(url).arg(&dest);

    let status = cmd.status().await?;
    if !status.success() {
        bail!("git clone failed");
    }

    if config.integrations.zoxide {
        integrations::zoxide::add(&dest).await?;
    }
    if config.integrations.autojump {
        integrations::autojump::add(&dest).await?;
    }

    println!("{}", dest.display());
    Ok(())
}
