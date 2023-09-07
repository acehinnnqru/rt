pub struct Repository {
    host: String,
    namespace: String,
    name: String,
}

impl Repository {
    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_full_name(&self) -> String {
        format!("{}/{}", self.get_namespace(), self.get_name())
    }

    pub fn get_full_name_with_host(&self) -> String {
        format!("{}/{}", self.get_host(), self.get_full_name())
    }

    pub fn get_ssh_url(&self) -> String {
        format!(
            "git@{}:{}/{}",
            self.get_host(),
            self.get_namespace(),
            self.get_name()
        )
    }

    pub fn get_https_url(&self) -> String {
        format!("https://{}", self.get_full_name_with_host())
    }
}

// Parse a repository string into a `Repository`.
// The format of a repository maybe like this:
// - ssh format: `git@<host>:<namespace>/<name>` or `git@<host>:<namespace>/<name>.git`
// - https format: `https://<host>/<namespace>/<name>` or `https://<host>/<namespace>/<name>.git`
// - simplely `<host>/<namespace>/<name>` or `<host>/<namespace>/<name>.git`
pub fn parse_repository(repo: String) -> Option<Repository> {
    match repo.find('@') {
        Some(_) => {
            // ssh format
            parse_ssh_format(&repo)
        }
        None => {
            // https format
            match repo.find("https://") {
                Some(_) => parse_https_format(&repo),
                None => parse_simple_format(&repo),
            }
        }
    }
}

fn parse_ssh_format(repo: &str) -> Option<Repository> {
    let mut split = repo.split('@');
    let _ = split.next().unwrap();
    let mut split = split.next().unwrap().split(':');
    let host = split.next().unwrap();
    let mut split = split.next().unwrap().split('/');
    let namespace = split.next().unwrap();
    let name = split.next().unwrap();
    Some(Repository {
        host: host.to_string(),
        namespace: namespace.to_string(),
        name: name.to_string(),
    })
}

fn parse_simple_format(repo: &str) -> Option<Repository> {
    let mut split = repo.split('/');
    let host = split.next().unwrap();
    let namespace = split.next().unwrap();
    let mut name = split.next().unwrap();
    if name.ends_with(".git") {
        name = &name[..name.len() - 4];
    }
    Some(Repository {
        host: host.to_string(),
        namespace: namespace.to_string(),
        name: name.to_string(),
    })
}

fn parse_https_format(repo: &str) -> Option<Repository> {
    let mut split = repo.split("https://");
    parse_simple_format(split.next().unwrap())
}
