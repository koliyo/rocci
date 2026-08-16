use std::{fs, path::Path};

use anyhow::{Context, Result};

pub const HTML: &str = include_str!("../runtime/Html.roc");
const BUILD: &str = include_str!("../runtime/RocsBuild.roc");
pub const THEME: &str = include_str!("../templates/RocsTheme.rocci");

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

pub fn stage_into(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    fs::write(dir.join("Html.roc"), HTML)
        .with_context(|| format!("failed to write {}/Html.roc", dir.display()))?;
    fs::write(dir.join("RocsBuild.roc"), BUILD)
        .with_context(|| format!("failed to write {}/RocsBuild.roc", dir.display()))?;
    Ok(())
}

pub fn runtime_bytes() -> usize {
    HTML.len() + BUILD.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn stage_into_writes_string_html_not_platform_html() {
        let dir = env::temp_dir().join(format!("rocs-runtime-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        stage_into(&dir).unwrap();
        let html = fs::read_to_string(dir.join("Html.roc")).unwrap();
        assert!(html.contains("Html := [].{"));
        assert!(!html.contains("import pf.Html"));
        assert!(!dir.join("Datastar.roc").exists());
        assert!(dir.join("RocsBuild.roc").is_file());
        assert!(!dir.join("RocsModel.roc").exists());
        assert!(!dir.join("RocsRoute.roc").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn html_declares_required_bindings() {
        for name in HTML_BINDINGS {
            assert!(
                HTML.contains(&format!("{name} =")),
                "rocs Html.roc missing {name}"
            );
        }
    }
}
