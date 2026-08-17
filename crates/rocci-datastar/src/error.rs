use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatastarError {
    #[error("invalid Datastar version: {0}")]
    InvalidVersion(String),

    #[error("failed to access cache directory: {0}")]
    CacheAccess(String),

    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to rename file from {from} to {to}: {source}")]
    RenameFile {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to download Datastar {tag}: {message}")]
    Download { tag: String, message: String },

    #[error("downloaded Datastar {tag} did not look like a JS bundle")]
    CorruptedBundle { tag: String },

    #[error("failed to deserialize signals: {0}")]
    SignalDeserialization(String),

    #[error("invalid SSE event: {0}")]
    InvalidSseEvent(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DatastarError>;
