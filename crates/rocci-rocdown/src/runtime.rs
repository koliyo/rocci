#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

#[cfg(not(target_arch = "wasm32"))]
use anyhow::{Context, Result};

pub const HTML: &str = include_str!("../runtime/Html.roc");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const DATASTAR: &str = include_str!("../runtime/Datastar.roc");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const BUILD: &str = include_str!("../runtime/RocdownBuild.roc");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const BUILD_WASM: &str = include_str!("../runtime/RocdownBuild.wasm.roc");
pub const THEME: &str = include_str!("../templates/RocdownTheme.rocci");
#[cfg(not(target_arch = "wasm32"))]
pub const BASE: &str = include_str!("../templates/RocdownBase.rocci");
#[cfg(not(target_arch = "wasm32"))]
pub const DOCS: &str = include_str!("../templates/DocsComponents.rocci");
#[cfg(not(target_arch = "wasm32"))]
pub const BLOCK_DEBUG: &str = include_str!("../templates/BlockDebug.rocci");
#[cfg(not(target_arch = "wasm32"))]
pub const BREADCRUMBS: &str = rocci_ui::chrome::BREADCRUMBS;
#[cfg(not(target_arch = "wasm32"))]
pub const NAV_LIST: &str = rocci_ui::chrome::NAV_LIST;
#[cfg(not(target_arch = "wasm32"))]
pub const PAGE_OUTLINE: &str = rocci_ui::chrome::PAGE_OUTLINE;

#[cfg(not(target_arch = "wasm32"))]
pub static PLAYGROUND_APP_JS: &[u8] = include_bytes!("../../../playground/dist/app.js");
#[cfg(not(target_arch = "wasm32"))]
pub static PLAYGROUND_WORKER_JS: &[u8] =
    include_bytes!("../../../playground/dist/compiler-worker.js");
#[cfg(not(target_arch = "wasm32"))]
pub static PLAYGROUND_STYLES_CSS: &[u8] = include_bytes!("../../../playground/dist/styles.css");
#[cfg(not(target_arch = "wasm32"))]
pub static PLAYGROUND_COMPILER_WASM: &[u8] =
    include_bytes!("../../../playground/dist/compiler.wasm");

pub const HTML_BINDINGS: &[&str] = &[
    "element",
    "void_element",
    "attribute",
    "boolean_attribute",
    "text",
    "fragment",
    "empty",
    "dangerously_include_unescaped_html",
    "render",
    "render_document",
    "render_fragment",
    "render_without_doc_type",
];

#[cfg(not(target_arch = "wasm32"))]
pub fn stage_into(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    fs::write(dir.join("Html.roc"), HTML)
        .with_context(|| format!("failed to write {}/Html.roc", dir.display()))?;
    fs::write(dir.join("RocdownBuild.roc"), BUILD)
        .with_context(|| format!("failed to write {}/RocdownBuild.roc", dir.display()))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn build_roc(is_wasm: bool) -> &'static str {
    if is_wasm { BUILD_WASM } else { BUILD }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn runtime_bytes() -> usize {
    HTML.len() + BUILD.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn stage_into_writes_string_html_not_platform_html() {
        let dir = env::temp_dir().join(format!("rocdown-runtime-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        stage_into(&dir).unwrap();
        let html = fs::read_to_string(dir.join("Html.roc")).unwrap();
        assert!(html.contains("Html := [].{"));
        assert!(!html.contains("import pf.Html"));
        assert!(!dir.join("Datastar.roc").exists());
        assert!(!DATASTAR.contains("import pf."));
        assert!(DATASTAR.contains("post ="));
        assert!(DATASTAR.contains("post_with ="));
        assert!(DATASTAR.contains("requestCancellation: 'disabled'"));
        assert!(dir.join("RocdownBuild.roc").is_file());
        assert!(!dir.join("RocdownModel.roc").exists());
        assert!(!dir.join("RocdownRoute.roc").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn html_declares_required_bindings() {
        for name in HTML_BINDINGS {
            assert!(
                HTML.contains(&format!("{name} =")),
                "rocdown Html.roc missing {name}"
            );
        }
    }
}
