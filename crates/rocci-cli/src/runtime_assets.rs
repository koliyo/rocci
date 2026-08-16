use std::{fs, path::Path};

use anyhow::{Context, Result};

const HTML: &str = include_str!("../runtime/Html.roc");
const DATASTAR: &str = include_str!("../runtime/Datastar.roc");

pub fn stage_into(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    fs::write(dir.join("Html.roc"), HTML)
        .with_context(|| format!("failed to write {}/Html.roc", dir.display()))?;
    fs::write(dir.join("Datastar.roc"), DATASTAR)
        .with_context(|| format!("failed to write {}/Datastar.roc", dir.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn stage_into_writes_html_and_datastar() {
        let dir = env::temp_dir().join(format!("rocci-runtime-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        stage_into(&dir).unwrap();
        let html = fs::read_to_string(dir.join("Html.roc")).unwrap();
        let datastar = fs::read_to_string(dir.join("Datastar.roc")).unwrap();
        assert!(html.contains("Html := [].{"));
        assert!(html.contains("import pf.Html as PlatformHtml"));
        assert!(datastar.contains("Datastar := [].{"));
        assert!(datastar.contains("patch_elements"));
        let _ = fs::remove_dir_all(&dir);
    }
}
