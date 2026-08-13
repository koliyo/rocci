use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("backend failed: {0}")]
    Backend(String),
    #[error("window {0} not found")]
    WindowNotFound(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn config(error: impl fmt::Display) -> Self {
        Self::Config(error.to_string())
    }

    pub fn backend(error: impl fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }

    pub fn message(error: impl fmt::Display) -> Self {
        Self::Message(error.to_string())
    }
}
