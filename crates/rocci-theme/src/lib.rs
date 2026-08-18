//! Color-scheme and font themes for Rocdown.
//!
//! A theme is a CSS file of `--rd-*` variables. Builtins ship in this
//! crate; named themes are loaded from `~/.rocci/themes`.

mod error;
mod resolve;
mod scheme;

pub use error::{Error, Result};
pub use resolve::{
    DEFAULT_THEME_ID, NONE_ID, PAPER_ID, ROCCI_ID, ResolvedTheme, ThemeOptions, ThemeOrigin,
    builtin_ids, discovered_ids, resolve, resolve_id,
};
pub use scheme::ColorSchemePolicy;

pub const TOC_SCRIPT: &str = rocci_ui::TOC_SCRIPT;
