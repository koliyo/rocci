//! Product-blind project browser library: protocol, discovery, registry, and picker.

pub mod adapter;
mod client;
mod discovery;
mod fuzzy;
mod host;
mod launcher;
pub mod overlay;
mod paths;
mod picker;
mod protocol;
mod registry;
mod session;

pub use adapter::{
    AdapterHandler, documents_from_pages_json, extract_http_url, inspector_url_for, serve_stdio,
    spawn_run_no_window,
};
pub use client::AdapterClient;
pub use discovery::{PluginSpec, discover_plugins, load_plugin_manifest};
pub use fuzzy::{ScoreFields, fuzzy, score_entry};
pub use host::{Host, OpenRequest, Opened, Target};
pub use launcher::{Launcher, launcher_html, spawn_launcher};
pub use paths::{Paths, browser_dir};
pub use picker::{Picker, PickerAction, PickerOutcome, PickerStage};
pub use protocol::{
    Document, InitializeResult, ListDocumentsResult, OpenParams, OpenResult, PROTOCOL_VERSION,
    ProbeResult,
};
pub use registry::{Project, Registry};
pub use session::{Session, SessionTable};

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
