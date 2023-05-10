use git2::build::RepoBuilder;
use git2::Repository;
use std::path::Path;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum GitError {
    RepoNotFound,
    Clone(git2::Error),
    Init,
    Other(String),
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoNotFound => write!(f, "Repository not found"),
            Self::Clone(e) => write!(f, "Clone error: {}", e),
            Self::Init => write!(f, "Init error"),
            Self::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<String> for GitError {
    fn from(v: String) -> Self {
        Self::Other(v)
    }
}

type Result<T> = std::result::Result<T, GitError>;

pub fn clone_bare(url: &str, target: &str) -> Result<()> {
    match RepoBuilder::new().bare(true).clone(url, Path::new(target)) {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::Clone(e)),
    }
}

pub fn clone_default_branch(url: &str, target: &str) -> Result<()> {
    match Repository::clone(url, Path::new(target)) {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::Clone(e)),
    }
}

pub fn clone(url: &str, branch: &str, target: &str) -> Result<()> {
    match RepoBuilder::new()
        .branch(branch)
        .clone(url, Path::new(target))
    {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::Clone(e)),
    }
}
