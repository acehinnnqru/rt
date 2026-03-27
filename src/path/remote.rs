use anyhow::{bail, Context, Result};

pub struct Remote {
    pub host: String,
    pub owner: String,
    pub repo: String,
}

impl Remote {
    pub fn parse(url: &str) -> Result<Self> {
        // SSH: git@host:owner/repo.git
        if let Some(rest) = url.strip_prefix("git@") {
            let (host, path) = rest
                .split_once(':')
                .context("invalid SSH URL: missing ':' separator")?;
            let path = path.trim_end_matches(".git");
            let (owner, repo) = path
                .split_once('/')
                .context("invalid SSH URL: expected owner/repo")?;
            return Ok(Self {
                host: host.to_string(),
                owner: owner.to_string(),
                repo: repo.to_string(),
            });
        }

        // HTTPS: https://host/owner/repo.git
        if url.starts_with("https://") || url.starts_with("http://") {
            let url = url.trim_end_matches(".git");
            let without_scheme = url.split("://").nth(1).context("invalid HTTPS URL")?;
            let mut parts = without_scheme.splitn(3, '/');
            let host = parts.next().context("missing host")?;
            let owner = parts.next().context("missing owner")?;
            let repo = parts.next().context("missing repo")?;
            return Ok(Self {
                host: host.to_string(),
                owner: owner.to_string(),
                repo: repo.to_string(),
            });
        }

        // Short: host/owner/repo
        let url = url.trim_end_matches(".git");
        let mut parts = url.splitn(3, '/');
        let host = parts.next().context("missing host")?;
        let owner = parts.next().context("missing owner")?;
        let repo = parts
            .next()
            .context("missing repo in short URL format")?;

        if host.is_empty() || owner.is_empty() || repo.is_empty() {
            bail!("unrecognized URL format: {url}");
        }

        Ok(Self {
            host: host.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh_url() {
        let r = Remote::parse("git@github.com:acehinnnqru/rt.git").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "acehinnnqru");
        assert_eq!(r.repo, "rt");
    }

    #[test]
    fn parse_https_url() {
        let r = Remote::parse("https://github.com/acehinnnqru/rt.git").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "acehinnnqru");
        assert_eq!(r.repo, "rt");
    }

    #[test]
    fn parse_short_url() {
        let r = Remote::parse("github.com/acehinnnqru/rt").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "acehinnnqru");
        assert_eq!(r.repo, "rt");
    }
}
