//! Product-blind project browser library: protocol, discovery, registry, and picker.

mod client;
mod discovery;
mod fuzzy;
mod host;
mod paths;
mod picker;
mod protocol;
mod registry;

pub use client::AdapterClient;
pub use discovery::{PluginSpec, discover_plugins, load_plugin_manifest};
pub use fuzzy::{ScoreFields, fuzzy, score_entry};
pub use host::{Host, OpenRequest, Opened, Target};
pub use paths::{Paths, browser_dir};
pub use picker::{Picker, PickerAction, PickerOutcome, PickerStage};
pub use protocol::{
    Document, InitializeResult, ListDocumentsResult, OpenParams, OpenResult, PROTOCOL_VERSION,
    ProbeResult,
};
pub use registry::{Project, Registry};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn message(error: impl std::fmt::Display) -> Self {
        Self::Message(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
