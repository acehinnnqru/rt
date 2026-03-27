use std::path::PathBuf;

use super::remote::Remote;

pub fn repo_path(root: &str, remote: &Remote) -> PathBuf {
    PathBuf::from(root)
        .join(&remote.host)
        .join(&remote.owner)
        .join(&remote.repo)
}
