use std::{fs, path::Path};

use anyhow::{Context, Result};

pub const HTML_ROC: &str = rocci_ui::HTML_ROC;
pub const PAGE_OUTLINE: &str = rocci_ui::chrome::PAGE_OUTLINE;
pub const OKF_THEME: &str = include_str!("../templates/OkfTheme.rocci");
pub const CONCEPT_META: &str = include_str!("../templates/ConceptMeta.rocci");
pub const REVIEW_QUEUE: &str = include_str!("../templates/ReviewQueue.rocci");
pub const OKF_BUILD_ROC: &str = include_str!("../runtime/OkfBuild.roc");

pub fn stage_into(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    fs::write(dir.join("Html.roc"), HTML_ROC)
        .with_context(|| format!("failed to write {}/Html.roc", dir.display()))?;
    fs::write(dir.join("OkfBuild.roc"), OKF_BUILD_ROC)
        .with_context(|| format!("failed to write {}/OkfBuild.roc", dir.display()))?;
    Ok(())
}
