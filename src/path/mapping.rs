use std::path::PathBuf;

use super::remote::Remote;

pub fn repo_path(root: &str, remote: &Remote) -> PathBuf {
    let mut path = PathBuf::from(root).join(layout_host(&remote.host));
    for segment in remote.namespace.iter() {
        path = path.join(segment);
    }
    path.join(&remote.repo)
}

fn layout_host(host: &str) -> &str {
    match host.rsplit_once(':') {
        Some((name, port)) if !name.is_empty() && port.chars().all(|ch| ch.is_ascii_digit()) => {
            name
        }
        _ => host,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(host: &str, namespace: &[&str], repo: &str) -> Remote {
        Remote {
            host: host.to_string(),
            namespace: namespace
                .iter()
                .map(|seg| (*seg).to_string())
                .collect::<Vec<String>>()
                .into(),
            repo: repo.to_string(),
        }
    }

    #[test]
    fn strips_port_from_layout_host() {
        let r = remote("gitlab.com:909", &["groups", "subgroups"], "name");
        let path = repo_path("/rt", &r);
        assert_eq!(path, PathBuf::from("/rt/gitlab.com/groups/subgroups/name"));
    }

    #[test]
    fn keeps_host_without_port() {
        let r = remote("go.googlesource.com", &[], "arch");
        let path = repo_path("/rt", &r);
        assert_eq!(path, PathBuf::from("/rt/go.googlesource.com/arch"));
    }
}
