use anyhow::Result;

use crate::git::WorktreeEntry;

use super::project::Project;

pub async fn run(paths: bool) -> Result<()> {
    let project = Project::current().await?;

    for entry in project.sorted_checkouts() {
        if paths {
            println!("{}", entry.path.display());
        } else {
            println!("{}", row(entry));
        }
    }

    Ok(())
}

/// The default table row: tab-separated branch name and absolute path, followed by the
/// `locked` and `prunable` markers when they apply. The first two fields are stable.
fn row(entry: &WorktreeEntry) -> String {
    let mut line = format!("{}\t{}", entry.branch_label(), entry.path.display());

    if entry.locked {
        line.push_str("\tlocked");
    }
    if entry.prunable {
        line.push_str("\tprunable");
    }

    line
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn entry(branch: Option<&str>, locked: bool, prunable: bool) -> WorktreeEntry {
        WorktreeEntry {
            path: PathBuf::from("/r/A/dev"),
            branch: branch.map(ToOwned::to_owned),
            bare: false,
            detached: branch.is_none(),
            locked,
            prunable,
        }
    }

    #[test]
    fn renders_branch_path_and_markers() {
        assert_eq!(row(&entry(Some("dev"), false, false)), "dev\t/r/A/dev");
        assert_eq!(
            row(&entry(Some("dev"), true, false)),
            "dev\t/r/A/dev\tlocked"
        );
        assert_eq!(
            row(&entry(Some("dev"), true, true)),
            "dev\t/r/A/dev\tlocked\tprunable"
        );
        assert_eq!(row(&entry(None, false, false)), "(detached)\t/r/A/dev");
    }
}
