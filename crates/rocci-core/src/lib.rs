//! Shared types for the Rocci desktop runtime.
//!
//! This crate has no HTTP or webview dependency. Configuration, window
//! sessions, lifecycle events, and the backend factory contract live here so
//! `rocci-wry` and the CLI can evolve independently.

mod backend;
mod config;
mod error;
mod event;
mod session;
mod state;

pub use backend::{Backend, ExternalBackend, RunningBackend, join_origin};
pub use config::{
    AppConfig, AssetConfig, BundleConfig, BundleResource, Config, DatastarAsset, DevelopmentConfig,
    HttpConfig, SecurityConfig, WindowConfig, parse_datastar_version,
};
pub use error::{Error, Result};
pub use event::{AppEvent, EventHook, ExitHook, Hooks, SetupHook, WindowEvent};
pub use session::{Session, SessionStore, WindowId};
pub use state::ManagedState;
