use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rocci_docs::{generate, load_catalog, stage};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("rocci-docs-{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn fixture_app(root: &Path) {
    write(
        &root.join("listed/index.rocdown"),
        "@page { layout: \"docs\", meta: { title: \"Listed\" } }\n\n# Listed\n",
    );
    write(
        &root.join("listed/extra.rocdown"),
        "@page { layout: \"docs\", meta: { title: \"Extra\" } }\n\n# Extra\n",
    );
    write(
        &root.join("listed/App.rocci"),
        "## Card used by the listing fixture.\n@component Card = |{ title }| {\n    @css { .card { padding: 1rem; } }\n    <div class=\"card\">{title}</div>\n}\n\n## GET the demo page.\n@view(\"/\") = || {\n    <html><body>ok</body></html>\n}\n",
    );
    write(&root.join("listed/README.md"), "local only\n");
    write(&root.join("listed/skip.db"), "not published");
    write(&root.join("listed/generated/Skip.roc"), "module []\n");
    write(&root.join("listed/assets/.gitkeep"), "");
    write(&root.join("listed/assets/ok.css"), ".ok { color: red; }\n");
    write(&root.join("listed/notes.txt"), "not listed\n");
    write(
        &root.join("unlisted/index.rocdown"),
        "@page { layout: \"docs\", meta: { title: \"Nope\" } }\n\n# Nope\n",
    );
    write(
        &root.join("unlisted/Hidden.rocci"),
        "@component X = || { <p/> }\n",
    );
    write(
        &root.join("apps.toml"),
        r#"
[[app]]
id = "listed"
path = "listed"
title = "Listed"
summary = "Fixture app"
entry = "App.rocci"
hosting = "docs"
"#,
    );
}

fn collect_rel(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                out.push(
                    path.strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn repo_catalog_live_ids_exclude_docs_only() {
    let catalog = load_catalog(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocci/apps.toml"),
    )
    .unwrap();
    let ids: Vec<_> = rocci_docs::live_apps(&catalog)
        .iter()
        .map(|app| app.id.as_str())
        .collect();
    assert_eq!(ids, ["live-counter", "datastar", "blocks"]);
    assert!(!ids.contains(&"counter"));
    assert!(!ids.contains(&"snake"));
    let blocks = catalog.apps.iter().find(|app| app.id == "blocks").unwrap();
    assert_eq!(
        rocci_docs::app_play_url(blocks),
        "https://rocci.dev/play/blocks/"
    );
}

#[test]
fn stages_expected_tree_and_skips_non_catalog() {
    let root = scratch("stage");
    fixture_app(&root);
    let out = root.join("out");
    generate(&root.join("apps.toml"), &out).unwrap();
    let files = collect_rel(&out);
    assert!(files.contains(&"index.rocdown".into()));
    assert!(files.contains(&"listed/index.rocdown".into()));
    assert!(files.contains(&"listed/extra.rocdown".into()));
    assert!(files.contains(&"listed/source/index.rocdown".into()));
    assert!(files.contains(&"listed/source/App-rocci.rocdown".into()));
    assert!(files.contains(&"listed/source/assets--ok-css.rocdown".into()));
    assert!(files.contains(&"listed/snippets/App.rocci".into()));
    assert!(files.contains(&"listed/snippets/assets/ok.css".into()));
    assert!(!files.iter().any(|f| f.contains("unlisted")));
    assert!(!files.iter().any(|f| f.contains("README")));
    assert!(!files.iter().any(|f| f.contains("skip.db")));
    assert!(!files.iter().any(|f| f.contains("generated")));
    assert!(!files.iter().any(|f| f.contains(".gitkeep")));
    assert!(!files.iter().any(|f| f.contains("notes.txt")));
    let page = fs::read_to_string(out.join("listed/source/App-rocci.rocdown")).unwrap();
    assert!(page.contains(":include[path: \"App.rocci\"]"));
    assert!(page.contains("## Declarations"));
    assert!(page.contains("### `@component Card`"));
    assert!(page.contains("Card used by the listing fixture."));
    assert!(page.contains("### `@view(\"/\")`"));
    assert!(page.contains("GET the demo page."));
    assert!(!page.contains(".."));
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("/examples/listed/"));
    assert!(index.contains("`docs`"));
    assert!(index.contains("/rocdown/"));
    assert!(!index.contains("rocdown run"));
}

#[test]
fn duplicate_ids_and_missing_docs_fail() {
    let root = scratch("errors");
    write(
        &root.join("apps.toml"),
        r#"
[[app]]
id = "dup"
path = "a"
title = "A"
summary = "A"
entry = "A.rocci"
hosting = "docs"

[[app]]
id = "dup"
path = "b"
title = "B"
summary = "B"
entry = "B.rocci"
hosting = "live"
"#,
    );
    write(&root.join("a/A.rocci"), "");
    write(&root.join("a/index.rocdown"), "# A\n");
    write(&root.join("b/B.rocci"), "");
    write(&root.join("b/index.rocdown"), "# B\n");
    let err = load_catalog(&root.join("apps.toml"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("duplicate app id"), "{err}");

    let missing = scratch("missing-docs");
    write(
        &missing.join("apps.toml"),
        r#"
[[app]]
id = "bare"
path = "bare"
title = "Bare"
summary = "No docs"
entry = "Bare.rocci"
hosting = "docs"
"#,
    );
    write(&missing.join("bare/Bare.rocci"), "");
    let err = load_catalog(&missing.join("apps.toml"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("missing index.rocdown"), "{err}");
}

#[test]
fn unknown_hosting_fails() {
    let root = scratch("hosting");
    write(
        &root.join("apps.toml"),
        r#"
[[app]]
id = "x"
path = "x"
title = "X"
summary = "X"
entry = "X.rocci"
hosting = "island"
"#,
    );
    write(&root.join("x/index.rocdown"), "# X\n");
    write(&root.join("x/X.rocci"), "");
    let err = load_catalog(&root.join("apps.toml"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("hosting") || err.contains("island"), "{err}");
}

#[test]
fn staging_is_deterministic() {
    let root = scratch("det");
    fixture_app(&root);
    let a = root.join("a");
    let b = root.join("b");
    let catalog = load_catalog(&root.join("apps.toml")).unwrap();
    stage(&catalog, &a).unwrap();
    stage(&catalog, &b).unwrap();
    assert_eq!(collect_rel(&a), collect_rel(&b));
    for rel in collect_rel(&a) {
        let left = fs::read(a.join(&rel)).unwrap();
        let right = fs::read(b.join(&rel)).unwrap();
        assert_eq!(left, right, "{rel}");
    }
}

#[test]
fn failed_stage_keeps_previous_tree() {
    let root = scratch("atomic");
    fixture_app(&root);
    let out = root.join("out");
    let mut catalog = load_catalog(&root.join("apps.toml")).unwrap();
    stage(&catalog, &out).unwrap();
    let previous = collect_rel(&out);
    catalog.apps.push(rocci_docs::AppEntry {
        id: "ghost".into(),
        path: "ghost".into(),
        title: "Ghost".into(),
        summary: "Missing on purpose".into(),
        entry: "Ghost.rocci".into(),
        hosting: rocci_docs::Hosting::Docs,
        files: Vec::new(),
        audience: String::new(),
        purpose: String::new(),
        complexity: String::new(),
        persistence: String::new(),
        support: String::new(),
        live_url: None,
    });
    assert!(stage(&catalog, &out).is_err());
    assert_eq!(collect_rel(&out), previous);
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("/examples/listed/"));
}
