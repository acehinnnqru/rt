use std::fmt::Display;

#[derive(Debug)]
pub enum CacheErr {
    Store,
    Parse(serde_json::Error),
    Write(cacache::Error),
    Other(String),
}

pub type Result<T> = std::result::Result<T, CacheErr>;

impl Display for CacheErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CacheErr::Store => write!(f, "Error storing data"),
            CacheErr::Parse(e) => write!(f, "Error parsing data: {}", e),
            CacheErr::Other(e) => write!(f, "Unknown error: {}", e),
            CacheErr::Write(e) => write!(f, "Error writing data: {}", e),
        }
    }
}

impl From<serde_json::Error> for CacheErr {
    fn from(e: serde_json::Error) -> Self {
        CacheErr::Parse(e)
    }
}

impl From<cacache::Error> for CacheErr {
    fn from(e: cacache::Error) -> Self {
        CacheErr::Write(e)
    }
}
