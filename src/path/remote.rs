use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Namespace(Vec<String>);

impl Namespace {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

impl From<Vec<String>> for Namespace {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

pub struct Remote {
    pub host: String,
    pub namespace: Namespace,
    pub repo: String,
}

impl Remote {
    pub fn parse(url: &str) -> Result<Self> {
        // SSH URL: ssh://[user@]host[:port]/namespace/repo.git
        if let Some(rest) = url.strip_prefix("ssh://") {
            let (authority, path) = rest
                .split_once('/')
                .context("invalid SSH URL: missing repository path")?;
            let host = authority
                .rsplit_once('@')
                .map_or(authority, |(_, host)| host);
            let path = path.trim_end_matches(".git");
            let (namespace, repo) = split_namespace_repo(path);

            if host.is_empty() || repo.is_empty() {
                bail!("unrecognized URL format: {url}");
            }

            return Ok(Self {
                host: host.to_string(),
                namespace,
                repo: repo.to_owned(),
            });
        }

        // SSH: git@host:namespace/repo.git
        if let Some(rest) = url.strip_prefix("git@") {
            let (host, path) = rest
                .split_once(':')
                .context("invalid SSH URL: missing ':' separator")?;
            let path = path.trim_end_matches(".git");
            let (namespace, repo) = split_namespace_repo(path);
            return Ok(Self {
                host: host.to_string(),
                namespace,
                repo: repo.to_owned(),
            });
        }

        // HTTPS: https://host/namespace/repo.git or https://host/repo.git
        if url.starts_with("https://") || url.starts_with("http://") {
            let url = url.trim_end_matches(".git");
            let without_scheme = url.split("://").nth(1).context("invalid HTTPS URL")?;
            let mut parts = without_scheme.splitn(2, '/');
            let host = parts.next().context("missing host")?;
            let path = parts.next().context("missing repository path")?;
            let (namespace, repo) = split_namespace_repo(path);
            return Ok(Self {
                host: host.to_string(),
                namespace,
                repo: repo.to_owned(),
            });
        }

        // Short: host/namespace/repo or host/repo
        let url = url.trim_end_matches(".git");
        let mut parts = url.splitn(2, '/');
        let host = parts.next().context("missing host")?;
        let path = parts.next().context("missing repo in short URL format")?;
        let (namespace, repo) = split_namespace_repo(path);

        if host.is_empty() || repo.is_empty() {
            bail!("unrecognized URL format: {url}");
        }

        Ok(Self {
            host: host.to_string(),
            namespace,
            repo: repo.to_owned(),
        })
    }
}

fn split_namespace_repo(path: &str) -> (Namespace, &str) {
    match path.rsplit_once('/') {
        Some((path, repo)) => (
            path.split('/')
                .map(ToOwned::to_owned)
                .collect::<Vec<String>>()
                .into(),
            repo,
        ),
        None => (Namespace::default(), path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn ns(parts: &[&str]) -> Namespace {
        parts
            .iter()
            .map(|part| (*part).to_string())
            .collect::<Vec<String>>()
            .into()
    }

    #[test_case("git@github.com:acehinnnqru/rt.git", "github.com", ns(&["acehinnnqru"]), "rt"; "ssh")]
    #[test_case("https://github.com/acehinnnqru/rt.git", "github.com", ns(&["acehinnnqru"]), "rt"; "https")]
    #[test_case("https://go.googlesource.com/arch", "go.googlesource.com", Namespace::default(), "arch"; "https without owner")]
    #[test_case("github.com/acehinnnqru/rt", "github.com", ns(&["acehinnnqru"]), "rt"; "short")]
    #[test_case("go.googlesource.com/arch", "go.googlesource.com", Namespace::default(), "arch"; "short without owner")]
    #[test_case("https://gitlab.com/groups/subgroups/name.git", "gitlab.com", ns(&["groups", "subgroups"]), "name"; "nested owner")]
    #[test_case("ssh://git@gitlab.com:909/groups/subgroups/subgroups/name.git", "gitlab.com:909", ns(&["groups", "subgroups", "subgroups"]), "name"; "ssh scheme with port and nested owner")]
    #[test_case("https://gitlab.com:909/groups/subgroups/subgroups/name.git", "gitlab.com:909", ns(&["groups", "subgroups", "subgroups"]), "name"; "https scheme with port and nested owner")]
    fn parse_url(
        input: &'static str,
        host: &'static str,
        namespace: Namespace,
        repo: &'static str,
    ) {
        let r =
            Remote::parse(input).unwrap_or_else(|err| panic!("failed to parse `{}`: {err}", input));
        assert_eq!(r.host, host);
        assert_eq!(r.namespace, namespace);
        assert_eq!(r.repo, repo);
    }
}
