use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub const PLATFORM_MAIN_ROC: &str = include_str!("../platform/main.roc");
pub const TARGET_HOST_O: &[u8] = include_bytes!("../platform/targets/wasm32/host.o");

/// Unpacks the embedded minimal Roc WebAssembly platform into the given directory.
pub fn stage_wasm_platform_into(target_dir: &Path) -> Result<()> {
    let platform_dir = target_dir.join("platform");
    let targets_dir = platform_dir.join("targets").join("wasm32");
    fs::create_dir_all(&targets_dir).with_context(|| {
        format!(
            "Failed to create platform targets directory at {:?}",
            targets_dir
        )
    })?;

    fs::write(platform_dir.join("main.roc"), PLATFORM_MAIN_ROC)
        .with_context(|| format!("Failed to write platform/main.roc at {:?}", platform_dir))?;

    fs::write(targets_dir.join("host.o"), TARGET_HOST_O)
        .with_context(|| format!("Failed to write targets/wasm32/host.o at {:?}", targets_dir))?;

    Ok(())
}
