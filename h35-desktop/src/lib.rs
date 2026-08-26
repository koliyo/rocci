//! Native window and webview host for hypermedia-driven desktop apps.

mod chrome;
mod dialog;
mod error;
mod events;
mod history;
mod icon;
mod menu;
mod preview;
mod source;
pub mod state;
mod types;
mod window;

use std::{env, fs, path::PathBuf};

pub use error::{Error, Result};
pub use events::{PreviewEvent, PreviewSink, ShellEvent};
pub use history::display_path;
pub use preview::{HostOptions, IpcHandler, NavigateHandler, preview};
pub use types::{WindowConfig, WindowEvent, WindowId};

pub(crate) fn web_context_dir(identifier: &str, window: &WindowId) -> PathBuf {
    let dir = env::temp_dir()
        .join("h35")
        .join(identifier)
        .join(window.as_str());
    if let Err(error) = fs::create_dir_all(&dir) {
        tracing::warn!(%error, path = %dir.display(), "failed to create webview data directory");
    }
    dir
}
