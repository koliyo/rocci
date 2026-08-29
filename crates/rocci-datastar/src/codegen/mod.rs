use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{DatastarError, Result};

pub const DATASTAR_ROC_TEMPLATE: &str = include_str!("../../../rocci-cli/runtime/Datastar.roc");

pub fn stage_datastar_roc(dest_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dest_dir).map_err(|source| DatastarError::CreateDir {
        path: dest_dir.to_path_buf(),
        source,
    })?;
    let dest = dest_dir.join("Datastar.roc");
    fs::write(&dest, DATASTAR_ROC_TEMPLATE).map_err(|source| DatastarError::WriteFile {
        path: dest.clone(),
        source,
    })?;
    Ok(dest)
}
