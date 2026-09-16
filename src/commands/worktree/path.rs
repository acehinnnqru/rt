use anyhow::{Context, Result};

use super::project::Project;

pub async fn run(branch: &str) -> Result<()> {
    let project = Project::current().await?;
    let entry = project
        .find_branch(branch)
        .with_context(|| format!("no worktree for branch '{branch}'"))?;

    println!("{}", entry.path.display());
    Ok(())
}
