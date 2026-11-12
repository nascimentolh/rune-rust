//! The one error type used by the whole cache module.

use std::fmt;

#[derive(Debug)]
pub enum CacheError {
    Io(std::io::Error),
    /// The bytes don't match the expected format.
    Corrupt(String),
    /// The archive or file isn't in the cache.
    Missing(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

pub fn corrupt<T>(msg: impl Into<String>) -> Result<T> {
    Err(CacheError::Corrupt(msg.into()))
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheError::Io(e) => write!(f, "I/O error: {e}"),
            CacheError::Corrupt(m) => write!(f, "corrupt cache: {m}"),
            CacheError::Missing(m) => write!(f, "not found: {m}"),
        }
    }
}

impl From<std::io::Error> for CacheError {
    fn from(e: std::io::Error) -> Self {
        CacheError::Io(e)
    }
}
