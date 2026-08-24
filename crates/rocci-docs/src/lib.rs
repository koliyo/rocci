//! Inventory cataloged Rocci apps and stage a Rocdown documentation tree.

mod catalog;
mod extract;
mod inventory;
mod stage;

pub use catalog::{AppEntry, Catalog, DocsError, Hosting, load_catalog};
pub use extract::{DeclDoc, declarations_markdown, documented_declarations};
pub use inventory::{PublishedFile, inventory_app, is_published_rel};
pub use stage::{StageOptions, StageReport, app_play_url, live_demo_url, stage, stage_with};

use std::path::Path;

use anyhow::Result;

/// Apps whose catalog `hosting` is `live` and `site` is true, in catalog order.
pub fn live_apps(catalog: &Catalog) -> Vec<&AppEntry> {
    catalog
        .apps
        .iter()
        .filter(|app| app.hosting == Hosting::Live && app.site)
        .collect()
}

/// Load the catalog and write the staging tree (`site = true` rows only).
pub fn generate(catalog_path: &Path, output: &Path) -> Result<StageReport> {
    generate_with(catalog_path, output, false)
}

/// Load the catalog and write the staging tree.
///
/// When `include_all` is true, rows with `site = false` are staged too (local preview).
pub fn generate_with(catalog_path: &Path, output: &Path, include_all: bool) -> Result<StageReport> {
    let catalog = load_catalog(catalog_path)?;
    Ok(stage_with(
        &catalog,
        output,
        StageOptions {
            include_all,
            advertise_live: false,
        },
    )?)
}
