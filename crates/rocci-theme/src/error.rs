use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("theme resolve: {0}")]
    Resolve(String),
    #[error("invalid theme configuration: {0}")]
    Config(String),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn unknown_theme(id: &str, searched: &[PathBuf]) -> Error {
    if searched.is_empty() {
        Error::Resolve(format!(
            "unknown theme `{id}`; expected none, paper, rocci, a CSS file, or a name in ~/.rocci/themes"
        ))
    } else {
        Error::Resolve(format!(
            "unknown theme `{id}`; searched {}",
            searched
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}
