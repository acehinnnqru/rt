use regex::bytes::Regex;
use std::str::FromStr;

fn main() {
    let repo = std::env::args().nth(1).expect("no repository given");

    println!("repository: {:?}", Repository::from_str(&repo));
}

#[derive(Debug)]
struct Repository {
    platform: String,
    namespace: String,
    name: String,
}

impl Repository {
    fn new(platform: &[u8], namespace: &[u8], name: &[u8]) -> Self {
        Self {
            platform: String::from_utf8(platform.to_vec()).unwrap(),
            namespace: String::from_utf8(namespace.to_vec()).unwrap(),
            name: String::from_utf8(name.to_vec()).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseRepositoryError;

impl FromStr for Repository {
    type Err = ParseRepositoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pattern = Regex::new(r"git@(.*):(.*)/(.*).git").unwrap();
        let sbytes = s.as_bytes();
        if !pattern.is_match(sbytes) {
            pattern = Regex::new(r"http[s]?://(.*)/(.*)/(.*).git").unwrap();
            if !pattern.is_match(sbytes) {
                unreachable!("invalid format of repository")
            }
        }

        let (_, [platform, namespace, name]) = pattern.captures(sbytes).unwrap().extract();

        Ok(Repository::new(platform, namespace, name))
    }
}
