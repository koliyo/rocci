use std::fs;
use std::path::Path;

use okf::Bundle;
use rocci_cli::inspect::{InspectPage, InspectSnapshot};
use rocci_cli::profile::ProfileSnapshot;

pub fn from_bundle(
    root: &Path,
    bundle: &Bundle,
    output: &Path,
    profile: ProfileSnapshot,
) -> InspectSnapshot {
    let mut pages = Vec::new();
    for concept in &bundle.concepts {
        let route = format!("/{}/", concept.id.trim_matches('/'));
        let source_path = root.join(&concept.path);
        let source = fs::read_to_string(&source_path).unwrap_or_default();
        let html_path = output.join(&concept.id).join("index.html");
        let html = inspect_html(fs::read_to_string(&html_path).ok());
        pages.push(InspectPage::from_okf(&route, &concept.path, source, html));
    }
    if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
        let source_path = root.join(&index.path);
        let source = fs::read_to_string(&source_path).unwrap_or_default();
        let html = inspect_html(fs::read_to_string(output.join("index.html")).ok());
        pages.push(InspectPage::from_okf("/", &index.path, source, html));
    }
    for index in &bundle.indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        let source = fs::read_to_string(root.join(&index.path)).unwrap_or_default();
        let html =
            inspect_html(fs::read_to_string(output.join(collection).join("index.html")).ok());
        pages.push(InspectPage::from_okf(
            &format!("/{collection}/"),
            &index.path,
            source,
            html,
        ));
    }
    let review_html =
        inspect_html(fs::read_to_string(output.join("review").join("index.html")).ok());
    pages.push(InspectPage::from_okf(
        "/review/",
        "review",
        "# Knowledge Governance & Review Queue\n".to_string(),
        review_html,
    ));
    InspectSnapshot { pages, profile }
}

fn inspect_html(html: Option<String>) -> Option<String> {
    html.map(|body| body.replace("<script src=\"/__rocci_okf/reload.js\" defer></script>", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::build_review_site_pure_rust;
    use okf::{LoadOptions, Profile, load_timed};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rocci-okf-inspect-{}-{}-{}",
            name,
            std::process::id(),
            nonce
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn valid_rocci_concept(id: &str, body: &str) -> String {
        format!(
            "---\ntype: Architecture\ntitle: {id}\ndescription: Test concept {id}.\ntags: [domain/rocci, concern/architecture]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-08-17T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\n---\n\n# {id}\n\n{body}\n"
        )
    }

    #[test]
    fn snapshot_from_bundle_fills_markdown_and_html() {
        let root = temp("bundle");
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(
            root.join("hello.md"),
            valid_rocci_concept("Hello", "Inspect this concept body."),
        )
        .unwrap();
        let loaded = load_timed(
            &root,
            LoadOptions::new(Profile::Rocci).with_provenance(false),
        )
        .unwrap();
        assert!(
            !loaded.bundle.has_errors(),
            "{:?}",
            loaded.bundle.diagnostics
        );
        let output = temp("out");
        build_review_site_pure_rust(&loaded.bundle, &output).unwrap();

        let snapshot = from_bundle(&root, &loaded.bundle, &output, ProfileSnapshot::default());
        let concept = loaded.bundle.concepts.first().expect("concept");
        let route = format!("/{}/", concept.id);
        let page = snapshot.resolve(Some(&route)).unwrap();
        assert_eq!(page.language, "markdown");
        assert_eq!(page.path, concept.path);
        assert!(
            page.source.contains("Inspect this concept body."),
            "{}",
            page.source
        );
        assert!(page.capabilities.source.available);
        assert!(!page.capabilities.ast.available);
        assert!(
            page.capabilities
                .ast
                .reason
                .contains("not Rocci or Rocdown"),
            "{}",
            page.capabilities.ast.reason
        );
        assert!(!page.capabilities.roc.available);
        assert!(page.capabilities.html.available);
        assert!(page.html.contains("<html"), "{}", page.html);

        let (status, body) = snapshot.inspect_json(Some(&route));
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["language"], "markdown");
        assert_eq!(value["capabilities"]["ast"]["available"], false);
        assert_eq!(value["capabilities"]["roc"]["available"], false);
        assert_eq!(value["capabilities"]["html"]["available"], true);
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(output);
    }

    #[test]
    fn snapshot_covers_collection_indexes_and_review() {
        let root = temp("bundle-indexes");
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(
            root.join("hello.md"),
            valid_rocci_concept("Hello", "Inspect this concept body."),
        )
        .unwrap();
        for collection in [
            "architecture",
            "audits",
            "case-studies",
            "decisions",
            "design",
            "plans",
            "reference",
            "research",
            "status",
        ] {
            fs::create_dir_all(root.join(collection)).unwrap();
            fs::write(
                root.join(collection).join("index.md"),
                format!("# {collection}\n"),
            )
            .unwrap();
        }
        let loaded = load_timed(
            &root,
            LoadOptions::new(Profile::Rocci).with_provenance(false),
        )
        .unwrap();
        assert!(
            !loaded.bundle.has_errors(),
            "{:?}",
            loaded.bundle.diagnostics
        );
        let output = temp("out-indexes");
        build_review_site_pure_rust(&loaded.bundle, &output).unwrap();

        let snapshot = from_bundle(&root, &loaded.bundle, &output, ProfileSnapshot::default());
        for route in [
            "/architecture/",
            "/audits/",
            "/case-studies/",
            "/decisions/",
            "/design/",
            "/plans/",
            "/reference/",
            "/research/",
            "/status/",
            "/review/",
        ] {
            let (status, body) = snapshot.inspect_json(Some(route));
            assert_eq!(status, 200, "{route}: {body}");
            let value: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(
                value["capabilities"]["source"]["available"], true,
                "{route}"
            );
            assert_eq!(value["capabilities"]["ast"]["available"], false, "{route}");
            assert_eq!(value["capabilities"]["roc"]["available"], false, "{route}");
            assert_eq!(value["capabilities"]["html"]["available"], true, "{route}");
            assert!(
                value["html"].as_str().unwrap().contains("<html"),
                "{route} html"
            );
        }
        let plans = snapshot.resolve(Some("/plans/")).unwrap();
        assert_eq!(plans.path, "plans/index.md");
        assert!(plans.source.contains("# plans"), "{}", plans.source);
        assert!(!plans.html.contains("reload.js"), "{}", plans.html);
        let review = snapshot.resolve(Some("/review/")).unwrap();
        assert_eq!(review.path, "review");
        assert!(
            review.source.contains("Knowledge Governance"),
            "{}",
            review.source
        );
        assert!(!review.html.contains("reload.js"), "{}", review.html);
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(output);
    }
}
