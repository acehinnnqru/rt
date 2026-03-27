use std::path::Path;

use anyhow::{bail, Result};

pub async fn add(path: &Path) -> Result<()> {
    let status = tokio::process::Command::new("autojump")
        .arg("--add")
        .arg(path)
        .status()
        .await?;

    if !status.success() {
        bail!("autojump --add failed");
    }

    Ok(())
}
