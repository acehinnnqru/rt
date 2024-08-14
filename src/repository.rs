use std::str::FromStr;

use regex::Regex;

#[derive(Debug)]
pub struct Repository {
    pub platform: String,
    pub namespace: String,
    pub name: String,
}

impl Repository {
    fn new(platform: &str, namespace: &str, name: &str) -> Self {
        Self {
            platform: String::from_str(platform).unwrap(),
            namespace: String::from_str(namespace).unwrap(),
            name: String::from_str(name).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseRepositoryError;

impl FromStr for Repository {
    type Err = ParseRepositoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pattern = Regex::new(r"git@(.*):(.*)/(.*).git").unwrap();
        if !pattern.is_match(s) {
            pattern = Regex::new(r"http[s]?://(.*)/(.*)/(.*).git").unwrap();
            if !pattern.is_match(s) {
                unreachable!("invalid format of repository")
            }
        }

        let (_, [platform, namespace, name]) = pattern.captures(s).unwrap().extract();

        Ok(Repository::new(platform, namespace, name))
    }
}
