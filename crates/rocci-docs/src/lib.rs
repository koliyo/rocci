//! Inventory cataloged Rocci apps and stage a Rocdown documentation tree.

mod catalog;
mod extract;
mod inventory;
mod stage;

pub use catalog::{AppEntry, Catalog, DocsError, Hosting, load_catalog};
pub use extract::{DeclDoc, declarations_markdown, documented_declarations};
pub use inventory::{PublishedFile, inventory_app, is_published_rel};
pub use stage::{StageReport, live_demo_url, stage};

use std::path::Path;

use anyhow::Result;

/// Apps whose catalog `hosting` is `live`, in catalog order.
pub fn live_apps(catalog: &Catalog) -> Vec<&AppEntry> {
    catalog
        .apps
        .iter()
        .filter(|app| app.hosting == Hosting::Live)
        .collect()
}

/// Load the catalog and write the staging tree.
pub fn generate(catalog_path: &Path, output: &Path) -> Result<StageReport> {
    let catalog = load_catalog(catalog_path)?;
    Ok(stage(&catalog, output)?)
}
