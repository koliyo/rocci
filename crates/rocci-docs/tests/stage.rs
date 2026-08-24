use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rocci_docs::{StageOptions, generate, load_catalog, stage, stage_with};

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
        "## Card used by the listing fixture.\n@component Card = |{ title }| {\n    @css { .card { padding: 1rem; } }\n    <div class=\"card\">{title}</div>\n}\n\n## GET the demo page.\n@get:view(\"/\") = || {\n    <html><body>ok</body></html>\n}\n",
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

fn mixed_site_catalog(root: &Path) {
    fixture_app(root);
    write(
        &root.join("hidden/index.rocdown"),
        "@page { layout: \"docs\", meta: { title: \"Hidden\" } }\n\n# Hidden\n",
    );
    write(
        &root.join("hidden/App.rocci"),
        "@component X = || { <p/> }\n",
    );
    write(
        &root.join("live/index.rocdown"),
        "@page { layout: \"docs\", meta: { title: \"Live\" } }\n\n# Live\n",
    );
    write(&root.join("live/App.rocci"), "@component X = || { <p/> }\n");
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

[[app]]
id = "hidden"
path = "hidden"
title = "Hidden"
summary = "Excluded from the public site"
entry = "App.rocci"
hosting = "docs"
site = false

[[app]]
id = "live"
path = "live"
title = "Live"
summary = "Live fixture"
entry = "App.rocci"
hosting = "live"
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
    assert_eq!(ids, ["live-counter", "datastar"]);
    assert!(!ids.contains(&"counter"));
    assert!(!ids.contains(&"snake"));
}

#[test]
fn site_false_is_omitted_from_staging_index_and_print_live() {
    let root = scratch("site-filter");
    mixed_site_catalog(&root);
    let catalog = load_catalog(&root.join("apps.toml")).unwrap();
    assert_eq!(
        rocci_docs::live_apps(&catalog)
            .iter()
            .map(|app| app.id.as_str())
            .collect::<Vec<_>>(),
        ["live"]
    );
    let out = root.join("out");
    generate(&root.join("apps.toml"), &out).unwrap();
    let files = collect_rel(&out);
    assert!(files.contains(&"listed/index.rocdown".into()));
    assert!(files.contains(&"live/index.rocdown".into()));
    assert!(!files.iter().any(|f| f.starts_with("hidden/")));
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("/examples/listed/"));
    assert!(index.contains("/examples/live/"));
    assert!(!index.contains("/examples/hidden/"));
    assert!(!index.contains("Hidden"));
}

#[test]
fn all_flag_stages_excluded_apps() {
    let root = scratch("site-all");
    mixed_site_catalog(&root);
    let catalog = load_catalog(&root.join("apps.toml")).unwrap();
    let out = root.join("out");
    stage_with(
        &catalog,
        &out,
        StageOptions {
            include_all: true,
            advertise_live: false,
        },
    )
    .unwrap();
    let files = collect_rel(&out);
    assert!(files.contains(&"hidden/index.rocdown".into()));
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("/examples/hidden/"));
}

#[test]
fn advertise_live_injects_launch_for_fixture_live_apps() {
    let root = scratch("launch");
    mixed_site_catalog(&root);
    let catalog = load_catalog(&root.join("apps.toml")).unwrap();

    let quiet = root.join("quiet");
    stage(&catalog, &quiet).unwrap();
    let quiet_listed = fs::read_to_string(quiet.join("listed/index.rocdown")).unwrap();
    let quiet_live = fs::read_to_string(quiet.join("live/index.rocdown")).unwrap();
    let quiet_index = fs::read_to_string(quiet.join("index.rocdown")).unwrap();
    assert!(!quiet_listed.contains(":link-card"));
    assert!(!quiet_live.contains(":link-card"));
    assert!(!quiet_index.contains("examples.rocci.dev"));
    assert!(quiet_index.contains("planned live"));

    let advertised = root.join("adv");
    stage_with(
        &catalog,
        &advertised,
        StageOptions {
            include_all: false,
            advertise_live: true,
        },
    )
    .unwrap();
    let live_page = fs::read_to_string(advertised.join("live/index.rocdown")).unwrap();
    assert!(live_page.contains(":link-card"));
    assert!(live_page.contains("title: \"Launch\""));
    assert!(live_page.contains("https://live.examples.rocci.dev"));
    let listed_page = fs::read_to_string(advertised.join("listed/index.rocdown")).unwrap();
    assert!(!listed_page.contains(":link-card"));
    assert!(!listed_page.contains("examples.rocci.dev"));
    let index = fs::read_to_string(advertised.join("index.rocdown")).unwrap();
    assert!(index.contains("| Launch |"));
    assert!(index.contains("https://live.examples.rocci.dev"));
    assert!(!index.contains("https://listed.examples.rocci.dev"));
    assert!(!index.contains("https://hidden.examples.rocci.dev"));
}

#[test]
fn advertise_live_uses_explicit_live_url() {
    let root = scratch("launch-url");
    write(
        &root.join("demo/index.rocdown"),
        "@page { layout: \"docs\", meta: { title: \"Demo\" } }\n\n# Demo\n",
    );
    write(&root.join("demo/App.rocci"), "@component X = || { <p/> }\n");
    write(
        &root.join("apps.toml"),
        r#"
[[app]]
id = "demo"
path = "demo"
title = "Demo"
summary = "Override"
entry = "App.rocci"
hosting = "live"
live_url = "https://rocci.dev/play/demo/"
"#,
    );
    let catalog = load_catalog(&root.join("apps.toml")).unwrap();
    let out = root.join("out");
    stage_with(
        &catalog,
        &out,
        StageOptions {
            include_all: false,
            advertise_live: true,
        },
    )
    .unwrap();
    let page = fs::read_to_string(out.join("demo/index.rocdown")).unwrap();
    assert!(page.contains(":link-card"));
    assert!(page.contains("https://rocci.dev/play/demo/"));
    assert!(!page.contains("https://demo.examples.rocci.dev"));
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("https://rocci.dev/play/demo/"));
}

fn examples_nav_ids(text: &str) -> Vec<String> {
    let start = text.find("label = \"Examples\"").expect("Examples nav");
    let rest = &text[start..];
    let items_at = rest.find("items = [").expect("Examples items");
    let after = &rest[items_at + "items = [".len()..];
    let end = after.find(']').expect("Examples items close");
    after[..end]
        .split(',')
        .filter_map(|part| {
            let item = part.trim().trim_matches('"');
            if item.is_empty() {
                None
            } else {
                Some(item.to_string())
            }
        })
        .collect()
}

#[test]
fn examples_nav_matches_site_true_catalog_ids() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let catalog = load_catalog(&manifest.join("../../examples/rocci/apps.toml")).unwrap();
    let site_ids: Vec<&str> = catalog
        .apps
        .iter()
        .filter(|app| app.site)
        .map(|app| app.id.as_str())
        .collect();
    let excluded: Vec<&str> = catalog
        .apps
        .iter()
        .filter(|app| !app.site)
        .map(|app| app.id.as_str())
        .collect();
    let nav = fs::read_to_string(manifest.join("../../site/rocdown.toml")).unwrap();
    let items = examples_nav_ids(&nav);
    assert_eq!(items[0], "examples/index");
    let nav_ids: Vec<&str> = items
        .iter()
        .skip(1)
        .map(|item| {
            item.strip_prefix("examples/")
                .and_then(|rest| rest.strip_suffix("/index"))
                .unwrap_or(item)
        })
        .collect();
    let mut expected = site_ids.clone();
    expected.sort();
    let mut actual = nav_ids.clone();
    actual.sort();
    assert_eq!(actual, expected);
    for id in excluded {
        assert!(
            !nav_ids.contains(&id),
            "site = false id `{id}` must not remain in Examples nav"
        );
    }
}

#[test]
fn catalog_index_does_not_advertise_unserved_live_hostnames() {
    let catalog = load_catalog(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocci/apps.toml"),
    )
    .unwrap();
    let out = scratch("live-label");
    stage(&catalog, &out).unwrap();
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("planned live"));
    assert!(!index.contains("examples.rocci.dev"));
    assert!(!index.contains("| `live` |"));
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
    assert!(!files.contains(&"listed/source/index.rocdown".into()));
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
    assert!(page.contains("### `@component Card` · [#L2](#L2)"));
    assert!(page.contains("Card used by the listing fixture."));
    assert!(page.contains("### `@get:view(\"/\")` · [#L8](#L8)"));
    assert!(page.contains("GET the demo page."));
    assert!(!page.contains(".."));
    let index = fs::read_to_string(out.join("index.rocdown")).unwrap();
    assert!(index.contains("Examples Overview"));
    assert!(index.contains("/examples/listed/"));
    assert!(index.contains("`docs`"));
    assert!(index.contains("/docs/rocdown/"));
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
fn site_false_with_docs_loads_and_live_fails() {
    let docs = scratch("site-false-docs");
    write(
        &docs.join("apps.toml"),
        r#"
[[app]]
id = "local-only"
path = "local-only"
title = "Local only"
summary = "Docs, excluded from the public site"
entry = "App.rocci"
hosting = "docs"
site = false
"#,
    );
    write(&docs.join("local-only/index.rocdown"), "# Local\n");
    write(&docs.join("local-only/App.rocci"), "");
    let catalog = load_catalog(&docs.join("apps.toml")).unwrap();
    assert_eq!(catalog.apps.len(), 1);
    assert!(!catalog.apps[0].site);
    assert_eq!(catalog.apps[0].hosting, rocci_docs::Hosting::Docs);

    let live = scratch("site-false-live");
    write(
        &live.join("apps.toml"),
        r#"
[[app]]
id = "ghost-live"
path = "ghost-live"
title = "Ghost live"
summary = "Invalid live plus excluded"
entry = "App.rocci"
hosting = "live"
site = false
"#,
    );
    write(&live.join("ghost-live/index.rocdown"), "# Ghost\n");
    write(&live.join("ghost-live/App.rocci"), "");
    let err = load_catalog(&live.join("apps.toml"))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("hosting = \"live\" requires site = true"),
        "{err}"
    );
}

#[test]
fn unknown_catalog_key_fails() {
    let root = scratch("unknown-key");
    write(
        &root.join("apps.toml"),
        r#"
[[app]]
id = "x"
path = "x"
title = "X"
summary = "X"
entry = "X.rocci"
hosting = "docs"
siet = false
"#,
    );
    write(&root.join("x/index.rocdown"), "# X\n");
    write(&root.join("x/X.rocci"), "");
    let err = load_catalog(&root.join("apps.toml"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("siet") || err.contains("unknown"), "{err}");
}

#[test]
fn repo_catalog_defaults_site_true() {
    let catalog = load_catalog(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocci/apps.toml"),
    )
    .unwrap();
    assert!(!catalog.apps.is_empty());
    assert!(catalog.apps.iter().all(|app| app.site));
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
        site: true,
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
