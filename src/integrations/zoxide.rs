use std::path::Path;

use anyhow::{bail, Result};

pub async fn add(path: &Path) -> Result<()> {
    if !super::command_exists("zoxide") {
        bail!("zoxide command not found in PATH");
    }

    let status = tokio::process::Command::new("zoxide")
        .arg("add")
        .arg(path)
        .status()
        .await?;

    if !status.success() {
        bail!("zoxide add failed");
    }

    Ok(())
}
