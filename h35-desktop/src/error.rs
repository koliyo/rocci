use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn message(error: impl fmt::Display) -> Self {
        Self::Message(error.to_string())
    }
}
