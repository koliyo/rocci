//! Rocs is a static documentation generator built on Rocci.
//!
//! Rust owns discovery, the page catalog, routes, and static Markdown rendering.
//! Rocci owns the site shell, compiled once per build. Dynamic `@render` / Rocci
//! islands stay on the Roc path and are not part of the static catalog yet.

mod article;
mod build;
mod catalog;
mod runtime;

pub use build::{BuildReport, build, discover_rocdown, pages_source};
pub use runtime::{HTML, HTML_BINDINGS, THEME, runtime_bytes, stage_into};

pub const BASIC_CLI_PLATFORM: &str = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst";
pub const STAGING_ENV: &str = "ROCS_STAGING";
