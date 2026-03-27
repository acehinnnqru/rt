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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn unique_temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("rt-clone-tests-{}-{nanos}", std::process::id()))
    }

    #[tokio::test]
    async fn returns_error_for_invalid_remote_url() {
        let root = unique_temp_root();
        std::fs::create_dir_all(&root).expect("should create temp root");

        let config = Config {
            root: root.to_string_lossy().into_owned(),
            ..Config::default()
        };

        let err = run(&config, "github.com")
            .await
            .expect_err("invalid remote URL should fail before cloning");
        assert!(err.to_string().contains("missing repo in short URL format"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn returns_error_when_destination_already_exists() {
        let root = unique_temp_root();
        let url = "https://github.com/acehinnnqru/rt.git";
        let remote = Remote::parse(url).expect("fixture URL should parse");
        let dest = mapping::repo_path(&root.to_string_lossy(), &remote);

        std::fs::create_dir_all(&dest).expect("should create destination fixture");

        let config = Config {
            root: root.to_string_lossy().into_owned(),
            ..Config::default()
        };

        let err = run(&config, url)
            .await
            .expect_err("existing destination should fail before cloning");
        assert!(err.to_string().contains("repository already exists at"));
        assert!(err.to_string().contains(&*dest.to_string_lossy()));

        let _ = std::fs::remove_dir_all(&root);
    }
}
