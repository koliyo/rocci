use std::fs;
use std::path::Path;

use rocci_cli::inspect::{InspectPage, InspectSnapshot};
use rocci_cli::profile::ProfileSnapshot;

use crate::site::{LoadedSite, resolve_loaded};

#[derive(Debug, Clone)]
pub struct PageInspectDraft {
    pub source_path: String,
    pub source: String,
    pub ast: String,
    pub roc: String,
}

pub fn snapshot_from_loaded(
    loaded: &LoadedSite,
    output: &Path,
    profile: ProfileSnapshot,
) -> InspectSnapshot {
    let resolved = resolve_loaded(loaded);
    let mut pages = Vec::new();
    for draft in &loaded.inspect {
        let Some(page) = resolved
            .site
            .pages
            .iter()
            .find(|page| page.source_path == draft.source_path)
        else {
            continue;
        };
        let html_path = output.join(&page.output_path);
        let html = fs::read_to_string(&html_path).ok();
        let inspect = InspectPage::from_rocdown(
            &page.route,
            &draft.source_path,
            draft.source.clone(),
            draft.ast.clone(),
            draft.roc.clone(),
            html.clone(),
        );
        pages.push(inspect);
        for alias in &page.aliases {
            pages.push(InspectPage::from_rocdown(
                alias,
                &draft.source_path,
                draft.source.clone(),
                draft.ast.clone(),
                draft.roc.clone(),
                html.clone(),
            ));
        }
    }
    InspectSnapshot { pages, profile }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::load_site;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rocci-rocdown-inspect-{}-{}-{}",
            name,
            std::process::id(),
            nonce
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn snapshot_from_loaded_fills_rocdown_views() {
        let root = temp("site");
        std::fs::write(
            root.join("rocdown.toml"),
            "[site]\ntitle = \"Inspect\"\n\n[[nav]]\nlabel = \"Start\"\nitems = [\"index\"]\n",
        )
        .unwrap();
        std::fs::write(
            root.join("index.rocdown"),
            "# Hello inspect\n\nA paragraph with source.\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let home = resolved
            .site
            .pages
            .iter()
            .find(|page| page.route == "/")
            .unwrap();
        let output = temp("out");
        let dest = output.join(&home.output_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&dest, "<h1>Hello inspect</h1>").unwrap();

        let snapshot = snapshot_from_loaded(&loaded, &output, ProfileSnapshot::default());
        let page = snapshot.resolve(Some("/")).unwrap();
        assert_eq!(page.language, "rocdown");
        assert_eq!(page.path, "index.rocdown");
        assert!(page.source.contains("Hello inspect"), "{}", page.source);
        assert!(page.ast.contains("(rocdown"), "{}", page.ast);
        assert!(!page.roc.is_empty());
        assert!(page.capabilities.source.available);
        assert!(page.capabilities.ast.available);
        assert!(page.capabilities.roc.available);
        assert!(page.capabilities.html.available);
        assert!(page.html.contains("Hello inspect"), "{}", page.html);

        let (status, body) = snapshot.inspect_json(Some("/"));
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["language"], "rocdown");
        assert_eq!(value["capabilities"]["html"]["available"], true);
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(output);
    }
}
