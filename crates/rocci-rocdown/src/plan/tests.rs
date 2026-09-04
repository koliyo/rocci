use super::theme::roc_fn_name;
use super::*;
use crate::build::tests::{lock_roc, skip_without_roc};
use crate::site::{InspectKind, inspect, load_site, resolve_loaded};
use std::{env, fs, path::PathBuf, process::Command};

#[test]
fn views_roc_is_staged_with_the_runtime() {
    let views = include_str!("../../runtime/Views.roc");
    assert!(
        views.contains("NavGroupView := {"),
        "Views.roc must name NavGroupView as a nominal type"
    );
    assert!(
        views.contains("children : List(NavGroupView)"),
        "NavGroupView.children must be recursive"
    );
    assert!(
        views.contains("Page(a) : {"),
        "Page must stay parametric over segments"
    );
    assert!(
        views.contains("Views := [].{"),
        "types live in the Views module"
    );
    let staged = env::temp_dir().join(format!("rocdown-views-stage-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staged);
    crate::runtime::stage_into(&staged).unwrap();
    assert!(staged.join("Views.roc").is_file());
    assert!(staged.join("RocdownBuild.roc").is_file());
    let _ = fs::remove_dir_all(&staged);
}

#[test]
fn missing_nav_group_children_names_the_field() {
    if skip_without_roc() {
        return;
    }
    let _lock = lock_roc();
    let dir = env::temp_dir().join(format!("rocdown-nav-group-missing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("Views.roc"),
        include_str!("../../runtime/Views.roc"),
    )
    .unwrap();
    fs::write(
        dir.join("main.roc"),
        format!(
            "\
app [main!] {{ pf: platform \"{}\" }}

import Views

main! = |_| {{
    group : Views.NavGroupView
    group = {{
        title: \"Lang\",
        href: \"/l/\",
        open: False,
        items: [],
    }}
    _ = group
    Ok({{}})
}}
",
            crate::BASIC_CLI_PLATFORM
        ),
    )
    .unwrap();
    let output = Command::new("roc")
        .arg("check")
        .arg("main.roc")
        .current_dir(&dir)
        .output()
        .expect("roc check");
    let _ = fs::remove_dir_all(&dir);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "expected a type error, got success:\n{text}"
    );
    assert!(
        !text.contains("malformed type")
            && !text.contains("undeclared type")
            && !text.contains("expected function arrow"),
        "Views.roc and the fixture must parse; got:\n{text}"
    );
    let named = text.contains("children") || text.contains("NavGroup");
    assert!(
        named,
        "diagnostic must name children or NavGroup, not only List.iter:\n{text}"
    );
}

#[test]
fn document_title_adds_the_brand_exactly_once() {
    assert_eq!(document_title("Guide", "Rocci"), "Guide · Rocci");
    assert_eq!(
        document_title("Contributing to Rocci", "Rocci"),
        "Contributing to Rocci"
    );
    assert_eq!(
        document_title("Rocci · Native interfaces", "Rocci"),
        "Rocci · Native interfaces"
    );
}

fn temp(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("rocdown-plan-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn nav_item(id: &str, title: &str, route: &str) -> catalog::NavItem {
    catalog::NavItem {
        id: id.into(),
        title: title.into(),
        route: route.into(),
    }
}

fn nav_section(label: &str, items: Vec<catalog::NavItem>, children: Vec<NavSection>) -> NavSection {
    NavSection {
        label: label.into(),
        items,
        children,
    }
}

#[test]
fn sidebar_lists_all_flat_sections_and_opens_current() {
    let navigation = vec![
        nav_section(
            "Start",
            vec![
                nav_item("index", "Home", "/"),
                nav_item("start/install", "Install", "/start/install/"),
            ],
            vec![],
        ),
        nav_section(
            "Tutorials",
            vec![
                nav_item("tutorials/index", "Tutorials", "/tutorials/"),
                nav_item(
                    "tutorials/first-component",
                    "Build your first component",
                    "/tutorials/first-component/",
                ),
            ],
            vec![],
        ),
    ];
    let (lanes, sidebar) = lanes_and_sidebar(&navigation, Some("tutorials/first-component"));
    assert!(lanes.is_empty());
    assert_eq!(sidebar.len(), 2);
    assert!(!sidebar[0].open);
    assert!(sidebar[1].open);
    assert_eq!(sidebar[1].href, "/tutorials/");
    assert_eq!(sidebar[1].items[0].title, "Overview");
    assert_eq!(sidebar[1].items[0].href, "/tutorials/");
    assert_eq!(sidebar[1].items[1].title, "Build your first component");
    assert!(sidebar[1].items[1].class_name.contains("is-current"));
}

fn source_page(id: &str, title: &str, route: &str) -> ResolvedPage {
    ResolvedPage {
        id: id.into(),
        source_path: format!("{id}.rocdown"),
        kind: PageKind::Static,
        title: title.into(),
        description: String::new(),
        layout: "docs".into(),
        published: String::new(),
        updated: String::new(),
        authors: Vec::new(),
        tags: Vec::new(),
        collection: String::new(),
        headings: Vec::new(),
        outgoing_links: Vec::new(),
        article_html: String::new(),
        island_css: String::new(),
        island_html: Vec::new(),
        route: route.into(),
        output_path: String::new(),
        aliases: Vec::new(),
        draft: false,
        suppress_unlisted_warning: true,
        unlisted: true,
        breadcrumbs: Vec::new(),
        previous: None,
        next: None,
        article: Vec::new(),
        examples: Vec::new(),
        includes: Vec::new(),
        docs_kinds: Vec::new(),
    }
}

#[test]
fn selected_example_lists_source_tree_below_the_example() {
    let navigation = vec![nav_section(
        "Examples",
        vec![
            nav_item("examples/index", "Examples", "/examples/"),
            nav_item("examples/blocks/index", "Rocci Blocks", "/examples/blocks/"),
            nav_item("examples/snake/index", "Snake", "/examples/snake/"),
        ],
        vec![],
    )];
    let pages = [
        source_page(
            "examples/blocks/source/backend--Blocks-rocci",
            "backend/Blocks.rocci",
            "/examples/blocks/source/backend--Blocks-rocci/",
        ),
        source_page(
            "examples/snake/source/Snake-rocci",
            "Snake.rocci",
            "/examples/snake/source/Snake-rocci/",
        ),
    ];
    let (_, mut sidebar) = lanes_and_sidebar(
        &navigation,
        Some("examples/blocks/source/backend--Blocks-rocci"),
    );
    attach_example_source_tree(
        &mut sidebar,
        Some("examples/blocks/source/backend--Blocks-rocci"),
        &pages,
    );
    assert_eq!(sidebar.len(), 4);
    assert_eq!(sidebar[0].title, "Examples");
    assert!(sidebar[0].items.is_empty());
    assert_eq!(sidebar[1].title, "Rocci Blocks");
    assert_eq!(sidebar[1].href, "/examples/blocks/");
    assert_eq!(sidebar[2].title, "Source");
    assert!(sidebar[2].open);
    assert_eq!(sidebar[2].items.len(), 1);
    assert_eq!(sidebar[2].items[0].title, "backend/Blocks.rocci");
    assert!(sidebar[2].items[0].class_name.contains("is-current"));
    assert_eq!(sidebar[3].title, "Snake");
    assert!(sidebar[3].items.is_empty());
    assert!(sidebar_has_current(
        &sidebar,
        "/examples/blocks/source/backend--Blocks-rocci/"
    ));
}

#[test]
fn examples_index_does_not_attach_a_source_tree() {
    let navigation = vec![nav_section(
        "Examples",
        vec![nav_item("examples/index", "Examples", "/examples/")],
        vec![],
    )];
    let pages = [source_page(
        "examples/blocks/source/App-rocci",
        "App.rocci",
        "/examples/blocks/source/App-rocci/",
    )];
    let (_, mut sidebar) = lanes_and_sidebar(&navigation, Some("examples/index"));
    attach_example_source_tree(&mut sidebar, Some("examples/index"), &pages);
    assert_eq!(sidebar.len(), 1);
    assert_eq!(sidebar[0].title, "Examples");
    assert_eq!(sidebar[0].items.len(), 1);
}

#[test]
fn nested_groups_keep_lanes_and_current_docs_sidebar() {
    let navigation = vec![
        nav_section(
            "Docs",
            vec![],
            vec![
                nav_section(
                    "Tutorials",
                    vec![
                        nav_item("docs/tutorials/index", "Tutorials", "/docs/tutorials/"),
                        nav_item(
                            "docs/tutorials/first-component",
                            "Build your first component",
                            "/docs/tutorials/first-component/",
                        ),
                    ],
                    vec![],
                ),
                nav_section(
                    "Status",
                    vec![nav_item("docs/status", "Status", "/docs/status/")],
                    vec![],
                ),
            ],
        ),
        nav_section(
            "News",
            vec![nav_item("news/index", "News", "/news/")],
            vec![],
        ),
    ];
    let (lanes, sidebar) = lanes_and_sidebar(&navigation, Some("docs/tutorials/first-component"));
    assert_eq!(lanes.len(), 2);
    assert!(lanes[0].current);
    assert!(!lanes[1].current);
    assert_eq!(sidebar[0].title, "Tutorials");
    assert!(sidebar[0].open);
    assert_eq!(sidebar[1].title, "Status");
    assert_eq!(sidebar[1].items.len(), 1);
    assert_eq!(sidebar[1].items[0].title, "Status");
    assert!(sidebar[0].children.is_empty());
}

#[test]
fn language_index_nests_descendants_and_opens_ancestors() {
    let navigation = vec![nav_section(
        "Reference",
        vec![
            nav_item("docs/reference/index", "Reference", "/docs/reference/"),
            nav_item(
                "docs/reference/language/index",
                "Rocci language reference",
                "/docs/reference/language/",
            ),
            nav_item(
                "docs/reference/language/file-structure",
                "File structure and Roc regions",
                "/docs/reference/language/file-structure/",
            ),
            nav_item(
                "docs/reference/runtime",
                "Runtime and HTTP",
                "/docs/reference/runtime/",
            ),
            nav_item(
                "docs/reference/contributor/index",
                "Contributor",
                "/docs/reference/contributor/",
            ),
            nav_item(
                "docs/reference/contributor/rocci-tree",
                "Rocci tree appendix",
                "/docs/reference/contributor/rocci-tree/",
            ),
        ],
        vec![],
    )];
    let (_, sidebar) =
        lanes_and_sidebar(&navigation, Some("docs/reference/language/file-structure"));
    assert_eq!(sidebar.len(), 1);
    assert!(sidebar[0].open);
    assert_eq!(sidebar[0].href, "/docs/reference/");
    assert_eq!(sidebar[0].items.len(), 1);
    assert_eq!(sidebar[0].items[0].title, "Overview");
    assert_eq!(sidebar[0].items[0].href, "/docs/reference/");
    assert_eq!(sidebar[0].children.len(), 3);
    assert_eq!(sidebar[0].children[0].title, "Rocci language reference");
    assert_eq!(sidebar[0].children[0].href, "/docs/reference/language/");
    assert!(sidebar[0].children[0].open);
    assert_eq!(sidebar[0].children[0].items[0].title, "Overview");
    assert_eq!(
        sidebar[0].children[0].items[0].href,
        "/docs/reference/language/"
    );
    assert_eq!(
        sidebar[0].children[0].items[1].title,
        "File structure and Roc regions"
    );
    assert!(
        sidebar[0].children[0].items[1]
            .class_name
            .contains("is-current")
    );
    assert_eq!(sidebar[0].children[1].title, "Runtime and HTTP");
    assert_eq!(sidebar[0].children[1].href, "/docs/reference/runtime/");
    assert!(sidebar[0].children[1].items.is_empty());
    assert_eq!(sidebar[0].children[2].title, "Contributor");
    assert_eq!(sidebar[0].children[2].href, "/docs/reference/contributor/");
    assert_eq!(sidebar[0].children[2].items[0].title, "Overview");
    assert_eq!(
        sidebar[0].children[2].items[0].href,
        "/docs/reference/contributor/"
    );
    assert_eq!(sidebar[0].children[2].items[1].title, "Rocci tree appendix");
    assert!(sidebar_has_current(
        &sidebar,
        "/docs/reference/language/file-structure/"
    ));
}

#[test]
fn explicit_nested_groups_stay_inside_the_parent() {
    let navigation = vec![nav_section(
        "Docs",
        vec![],
        vec![nav_section(
            "Reference",
            vec![nav_item(
                "docs/reference/index",
                "Reference",
                "/docs/reference/",
            )],
            vec![nav_section(
                "Language",
                vec![
                    nav_item(
                        "docs/reference/language/index",
                        "Rocci language reference",
                        "/docs/reference/language/",
                    ),
                    nav_item(
                        "docs/reference/language/tags",
                        "Tags and fragments",
                        "/docs/reference/language/tags/",
                    ),
                ],
                vec![],
            )],
        )],
    )];
    let (lanes, sidebar) = lanes_and_sidebar(&navigation, Some("docs/reference/language/tags"));
    assert_eq!(lanes.len(), 1);
    assert_eq!(sidebar.len(), 1);
    assert_eq!(sidebar[0].title, "Reference");
    assert!(sidebar[0].open);
    assert_eq!(sidebar[0].href, "/docs/reference/");
    assert_eq!(sidebar[0].items[0].title, "Overview");
    assert_eq!(sidebar[0].items[0].href, "/docs/reference/");
    assert_eq!(sidebar[0].children.len(), 1);
    assert_eq!(sidebar[0].children[0].title, "Language");
    assert_eq!(sidebar[0].children[0].href, "/docs/reference/language/");
    assert!(sidebar[0].children[0].open);
    assert_eq!(sidebar[0].children[0].items[0].title, "Overview");
    assert_eq!(sidebar[0].children[0].items[1].title, "Tags and fragments");
}

#[test]
fn appendix_without_index_stays_flat() {
    let navigation = vec![nav_section(
        "Start",
        vec![
            nav_item("docs/index", "Overview", "/docs/"),
            nav_item("docs/install", "Install", "/docs/install/"),
            nav_item(
                "docs/appendix/glossary",
                "Glossary",
                "/docs/appendix/glossary/",
            ),
            nav_item(
                "docs/appendix/roc-for-rocci",
                "Roc for Rocci",
                "/docs/appendix/roc-for-rocci/",
            ),
        ],
        vec![],
    )];
    let (_, sidebar) = lanes_and_sidebar(&navigation, Some("docs/appendix/glossary"));
    assert_eq!(sidebar.len(), 1);
    assert_eq!(sidebar[0].href, "/docs/");
    assert_eq!(sidebar[0].items.len(), 4);
    assert!(sidebar[0].children.is_empty());
    assert_eq!(sidebar[0].items[0].title, "Overview");
    assert_eq!(sidebar[0].items[0].href, "/docs/");
    assert_eq!(sidebar[0].items[1].title, "Install");
    assert_eq!(sidebar[0].items[2].title, "Glossary");
}

#[test]
fn templates_index_still_peels_when_title_matches_label() {
    let navigation = vec![nav_section(
        "Templates",
        vec![
            nav_item("docs/templates/index", "Templates", "/docs/templates/"),
            nav_item(
                "docs/templates/components",
                "Components",
                "/docs/templates/components/",
            ),
        ],
        vec![],
    )];
    let (_, sidebar) = lanes_and_sidebar(&navigation, Some("docs/templates/components"));
    assert_eq!(sidebar[0].href, "/docs/templates/");
    assert_eq!(sidebar[0].items.len(), 2);
    assert_eq!(sidebar[0].items[0].title, "Overview");
    assert_eq!(sidebar[0].items[0].href, "/docs/templates/");
    assert_eq!(sidebar[0].items[1].title, "Components");
}

#[test]
fn single_page_lane_is_an_expandable_current_group() {
    let navigation = vec![nav_section(
        "FAQ",
        vec![nav_item("faq/index", "FAQ", "/faq/")],
        vec![],
    )];
    let (_, sidebar) = lanes_and_sidebar(&navigation, Some("faq/index"));
    assert_eq!(sidebar.len(), 1);
    assert!(sidebar[0].open);
    assert!(sidebar[0].href.is_empty());
    assert_eq!(sidebar[0].items.len(), 1);
    assert!(sidebar_has_current(&sidebar, "/faq/"));
}

fn write_site(root: &Path) {
    fs::create_dir_all(root.join("assets/icons")).unwrap();
    fs::write(root.join("assets/og.png"), b"og-bytes").unwrap();
    fs::write(
        root.join("assets/favicon.svg"),
        b"<svg xmlns='http://www.w3.org/2000/svg'/>",
    )
    .unwrap();
    fs::write(root.join("assets/apple-touch-icon.png"), b"touch-bytes").unwrap();
    fs::write(root.join("assets/icons/logo.png"), b"logo-bytes").unwrap();
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"
social_image = "/assets/og.png"
favicon = "/assets/favicon.svg"
apple_touch_icon = "/assets/apple-touch-icon.png"
subtitle = "Tools"
footer = "Experimental."

[[nav]]
label = "Start"
items = ["index", "guide"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n![og](/assets/og.png)\n\nSee the [guide](/guide/).\n",
    )
    .unwrap();
    fs::write(
        root.join("guide.rocdown"),
        "@page {\n    aliases: [\"/old-guide/\"],\n    meta: { title: \"Guide\" },\n}\n\n# Guide\n\n## Details\n\nLogo: ![logo](/assets/icons/logo.png)\n",
    )
    .unwrap();
}

#[test]
fn playground_styles_do_not_override_host_root_tokens() {
    let css = include_str!("../../../../playground/src/styles.css");
    assert!(
        !css.contains(":root {"),
        "playground CSS must inherit host theme tokens when embedded"
    );
    assert!(
        !css.contains("#faf9f6") && !css.contains("#161413") && !css.contains("#e64b2f"),
        "standalone playground must not keep the old warm palette\n{css}"
    );
    assert!(css.contains("html:has(body > #playground-root)"), "{css}");
    assert!(css.contains("background: var(--code)"), "{css}");
}

#[test]
fn default_csp_is_strict_and_stable() {
    assert_eq!(
        DEFAULT_CSP,
        "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'"
    );
    assert!(!DEFAULT_CSP.contains("unsafe-eval"));
    assert!(!DEFAULT_CSP.contains("unsafe-inline"));
}

#[test]
fn playground_layout_gets_hashed_assets_session_and_csp() {
    let root = temp("playground-layout");
    write_site(&root);
    fs::write(
        root.join("playground.rocdown"),
        "@page {\n    layout: \"playground\",\n    route: \"/playground/\",\n    meta: { title: \"Playground\" },\n}\n\n# Playground\n",
    )
    .unwrap();
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"

[[nav]]
label = "Start"
items = ["index", "guide"]

[[nav]]
label = "Playground"
items = ["playground"]
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

    let playground = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/playground")
        .expect("playground page");
    let guide = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/guide")
        .expect("guide page");

    assert_eq!(playground.view.layout, "playground");
    assert_eq!(playground.view.resources.csp, PLAYGROUND_CSP);
    assert!(
        playground
            .view
            .resources
            .module_script
            .contains("playground-app."),
        "{}",
        playground.view.resources.module_script
    );
    assert!(
        playground
            .view
            .resources
            .playground_css
            .contains("playground-styles."),
        "{}",
        playground.view.resources.playground_css
    );
    assert!(
        playground
            .view
            .resources
            .playground_session
            .contains("playground-session."),
        "{}",
        playground.view.resources.playground_session
    );
    assert!(playground.view.sidebar.is_empty());

    assert_eq!(guide.view.resources.csp, DEFAULT_CSP);
    assert!(guide.view.resources.module_script.is_empty());
    assert!(guide.view.resources.playground_css.is_empty());
    assert!(guide.view.resources.playground_session.is_empty());

    let logical: Vec<_> = planned
        .assets
        .iter()
        .map(|asset| asset.logical_path.as_str())
        .collect();
    for expected in [
        "/assets/playground-app.js",
        "/assets/playground-worker.js",
        "/assets/playground-styles.css",
        "/assets/compiler.wasm",
        "/assets/playground-session.json",
    ] {
        assert!(logical.contains(&expected), "{logical:?}");
    }

    let session = planned
        .assets
        .iter()
        .find(|asset| asset.logical_path == "/assets/playground-session.json")
        .expect("session asset");
    let worker = planned
        .assets
        .iter()
        .find(|asset| asset.logical_path == "/assets/playground-worker.js")
        .expect("worker asset");
    let wasm = planned
        .assets
        .iter()
        .find(|asset| asset.logical_path == "/assets/compiler.wasm")
        .expect("wasm asset");
    let parsed: serde_json::Value = serde_json::from_slice(&session.bytes).unwrap();
    assert_eq!(parsed["mode"], "wasm");
    assert_eq!(parsed["html_runtime"]["available"], false);
    assert_eq!(parsed["compiler_wasm_url"], wasm.hashed_url);
    assert_eq!(parsed["worker_url"], worker.hashed_url);
    let languages: Vec<&str> = parsed["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| doc["language"].as_str().unwrap())
        .collect();
    assert!(languages.contains(&"rocci"), "{languages:?}");
    assert!(languages.contains(&"rocdown"), "{languages:?}");

    let roc = planned.pages_roc();
    assert!(roc.contains("playground_session: "));
    assert!(
        !guide.view.resources.csp.contains("wasm-unsafe-eval"),
        "{}",
        guide.view.resources.csp
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn playground_examples_manifest_loads_checked_in_sources() {
    let root = temp("playground-examples");
    write_site(&root);
    fs::create_dir_all(root.join("playground")).unwrap();
    fs::write(
        root.join("playground/index.rocdown"),
        "@page {\n    layout: \"playground\",\n    route: \"/playground/\",\n    meta: { title: \"Playground\" },\n}\n\n# Playground\n",
    )
    .unwrap();
    fs::write(
        root.join("sample.rocci"),
        "@component Hello = |_| {\n    <p>hi</p>\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("sample.rocdown"),
        "# Sample\n\nFrom the manifest.\n",
    )
    .unwrap();
    fs::write(
        root.join("playground/examples.toml"),
        r#"
[[example]]
id = "hello"
file = "sample.rocci"
language = "rocci"

[[example]]
id = "sample"
file = "sample.rocdown"
language = "rocdown"
"#,
    )
    .unwrap();
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"

[[nav]]
label = "Playground"
items = ["playground/index"]
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let session = planned
        .assets
        .iter()
        .find(|asset| asset.logical_path == "/assets/playground-session.json")
        .expect("session");
    let parsed: serde_json::Value = serde_json::from_slice(&session.bytes).unwrap();
    assert_eq!(parsed["selected_document"], "hello");
    assert_eq!(
        parsed["documents"][0]["source"],
        "@component Hello = |_| {\n    <p>hi</p>\n}\n"
    );
    assert_eq!(
        parsed["documents"][1]["source"],
        "# Sample\n\nFrom the manifest.\n"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn site_without_playground_omits_playground_assets() {
    let root = temp("no-playground");
    write_site(&root);
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    assert!(planned.assets.iter().all(|asset| {
        !asset.logical_path.contains("playground") && !asset.logical_path.contains("compiler.wasm")
    }));
    let home = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/")
        .unwrap();
    assert!(home.view.resources.playground_session.is_empty());
    assert!(!home.view.resources.csp.contains("wasm-unsafe-eval"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn hashed_names_are_deterministic_and_keep_directories() {
    let first = hashed_asset("icons/logo.png", b"logo-bytes");
    let second = hashed_asset("icons/logo.png", b"logo-bytes");
    assert_eq!(first.output_path, second.output_path);
    assert!(first.output_path.starts_with("assets/icons/logo."));
    assert!(first.output_path.ends_with(".png"));
    assert_ne!(first.output_path, "assets/icons/logo.png");
    let other = hashed_asset("icons/logo.png", b"other");
    assert_ne!(first.output_path, other.output_path);
}

#[test]
fn plan_rewrites_article_html_and_social_image() {
    let root = temp("rewrite");
    write_site(&root);
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let home = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/")
        .unwrap();
    assert!(home.article_html.contains("/assets/og."));
    assert!(!home.article_html.contains("/assets/og.png"));
    assert!(home.view.site.social_image.starts_with("/assets/og."));
    assert_ne!(home.view.site.social_image, "/assets/og.png");
    assert!(home.view.site.favicon.starts_with("/assets/favicon."));
    assert_ne!(home.view.site.favicon, "/assets/favicon.svg");
    assert!(
        home.view
            .site
            .apple_touch_icon
            .starts_with("/assets/apple-touch-icon.")
    );
    assert_ne!(
        home.view.site.apple_touch_icon,
        "/assets/apple-touch-icon.png"
    );
    let guide = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/guide")
        .unwrap();
    assert!(guide.article_html.contains("/assets/icons/logo."));
    assert!(!guide.article_html.contains("/assets/icons/logo.png"));
    assert_eq!(
        guide
            .view
            .breadcrumbs
            .iter()
            .map(|crumb| crumb.title.as_str())
            .collect::<Vec<_>>(),
        ["Rocci", "Guide"]
    );
    assert!(home.view.breadcrumbs.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn plan_lists_404_stylesheet_redirects_and_discovery() {
    let root = temp("artifacts");
    write_site(&root);
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let artifacts = planned.artifacts();
    let kinds: Vec<_> = artifacts.iter().map(|item| item.kind).collect();
    assert!(kinds.contains(&"not_found"));
    assert!(kinds.contains(&"stylesheet"));
    assert!(kinds.contains(&"redirect"));
    assert!(kinds.contains(&"llms"));
    assert!(kinds.contains(&"pages"));
    assert!(kinds.contains(&"sitemap"));
    assert!(kinds.contains(&"robots"));
    let pages_json = planned
        .files
        .iter()
        .find(|file| file.output_path == "pages.json")
        .unwrap();
    assert_eq!(pages_json.route, "/pages.json");
    let listed: serde_json::Value = serde_json::from_str(&pages_json.contents).unwrap();
    assert!(listed.as_array().unwrap().iter().any(|page| {
        page["route"] == "/guide" && page["title"] == "Guide" && page["kind"] == "static"
    }));
    assert!(artifacts.iter().any(|item| item.output_path == "404.html"));
    assert!(
        artifacts
            .iter()
            .any(|item| item.output_path == "old-guide/index.html")
    );
    assert!(planned.assets.iter().any(|asset| asset.kind == "stylesheet"
        && String::from_utf8_lossy(&asset.bytes).contains("forced-colors")));
    let recovery = planned
        .pages
        .iter()
        .find(|page| page.output_path == "404.html")
        .unwrap();
    assert_eq!(recovery.view.resources.csp, DEFAULT_CSP);
    assert_eq!(
        recovery
            .view
            .breadcrumbs
            .iter()
            .map(|crumb| crumb.title.as_str())
            .collect::<Vec<_>>(),
        ["Rocci", "Page not found"]
    );
    assert!(!recovery.view.sidebar.is_empty());
    let roc = planned.pages_roc();
    let not_found = roc.find("output_path: \"404.html\"").unwrap();
    let guide = roc.find("output_path: \"guide/index.html\"").unwrap();
    let index = roc.find("output_path: \"index.html\"").unwrap();
    assert!(not_found < guide);
    assert!(guide < index);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn inspect_artifacts_uses_the_plan() {
    let root = temp("inspect");
    write_site(&root);
    let json = inspect(&root, InspectKind::Artifacts, None).unwrap();
    assert!(json.contains("404.html"), "{json}");
    assert!(json.contains("old-guide/index.html"), "{json}");
    assert!(json.contains("theme."), "{json}");
    let report: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(report["datastar"], false);
    assert!(report["service_routes"].as_array().unwrap().is_empty());
    assert!(
        report["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["kind"] == "static" && page["route"] == "/"),
        "{json}"
    );
    assert!(
        report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["output_path"] == "pages.json"),
        "{json}"
    );
    assert!(
        !report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["output_path"] == "islands.json"),
        "{json}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pages_roc_is_stable_for_body_only_edits() {
    let root = temp("hash-body");
    write_site(&root);
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    let first = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let first_roc = first.pages_roc();

    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n![og](/assets/og.png)\n\nSee the [guide](/guide/).\n\nExtra paragraph.\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    let second = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    assert_eq!(first_roc, second.pages_roc());
    let home = |plan: &BuildPlan| {
        plan.pages
            .iter()
            .find(|page| page.view.route == "/")
            .unwrap()
            .article_html
            .clone()
    };
    assert_ne!(home(&first), home(&second));

    fs::write(
        root.join("index.rocdown"),
        "# Home changed\n\n![og](/assets/og.png)\n\nSee the [guide](/guide/).\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    let third = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    assert_ne!(first_roc, third.pages_roc());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pages_roc_is_stable_for_docs_body_only_edits() {
    let root = temp("hash-docs-body");
    write_site(&root);
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:note[title: \"Watch\"] First body.\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let first = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let first_roc = first.pages_roc();
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:note[title: \"Watch\"] Second body, still a note.\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    let second = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    assert_eq!(first_roc, second.pages_roc());
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:note[title: \"Changed\"] Second body, still a note.\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    let third = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    assert_ne!(first_roc, third.pages_roc());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pages_roc_emits_typed_widget_tags_not_segment_bag() {
    let root = temp("typed-props");
    write_site(&root);
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:note[title: \"Watch\"] Body text.\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let roc = planned.pages_roc();
    assert!(roc.contains("import Views"), "{roc}");
    assert!(roc.contains("pages : List(Views.Page(_))"), "{roc}");
    let after_previous = roc
        .split_once("previous: {")
        .map(|(_, rest)| rest)
        .unwrap_or("");
    assert!(
        after_previous.contains("class_name:"),
        "previous/next must emit NavItemView.class_name\n{roc}"
    );
    assert!(roc.contains("HtmlFile({ path:"), "{roc}");
    assert!(roc.contains("Note({"), "{roc}");
    assert!(roc.contains("title: \"Watch\""), "{roc}");
    assert!(roc.contains("child_count:"), "{roc}");
    assert!(!roc.contains("tab_id"), "{roc}");
    assert!(!roc.contains("kind: \"note\""), "{roc}");
    let home = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/")
        .unwrap();
    assert!(
        home.segments
            .iter()
            .any(|node| node.widget_kind() == Some("note"))
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pages_roc_emits_tab_ids_on_typed_children() {
    let root = temp("tab-ids");
    write_site(&root);
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n",
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let roc = planned.pages_roc();
    assert!(roc.contains("Tabs({"), "{roc}");
    assert!(roc.contains("group: \"os\""), "{roc}");
    assert!(roc.contains("kind: \"platform\""), "{roc}");
    assert!(roc.contains("Tab({"), "{roc}");
    assert!(roc.contains("id: \"mac\""), "{roc}");
    assert!(roc.contains("id: \"linux\""), "{roc}");
    assert!(roc.contains("label: \"macOS\""), "{roc}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn hybrid_pages_keep_static_widgets_and_islands_in_authored_order() {
    let root = temp("dual-apply");
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Dual"
base_url = "https://rocci.dev"

[[nav]]
label = "Start"
items = ["index", "widgets"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:note[title: \"Watch\"] Body text.\n",
    )
    .unwrap();
    fs::write(
        root.join("widgets.rocdown"),
        r#"
@page {
route: "/widgets/",
meta: { title: "Widgets" },
}

@component
FeatureCount = |_| {
<p class="feature-count">3 core ideas</p>
}

# Widgets

:card-grid.begin
:link-card[href: "/", title: "Start", summary: "First path."]
:link-card[href: "/widgets/", title: "Project", summary: "Second path."]
:card-grid.end

@render FeatureCount({})
"#,
    )
    .unwrap();
    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let home_page = resolved
        .site
        .pages
        .iter()
        .find(|page| page.route == "/")
        .unwrap();
    let widgets_page = resolved
        .site
        .pages
        .iter()
        .find(|page| page.route == "/widgets")
        .unwrap();
    assert_eq!(home_page.kind, PageKind::Static);
    assert_eq!(widgets_page.kind, PageKind::Hydrate);
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let roc = planned.pages_roc();
    assert!(roc.contains("Note({"), "{roc}");
    assert!(roc.contains("HtmlFile({ path:"), "{roc}");
    let home = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/")
        .unwrap();
    assert!(
        home.segments
            .iter()
            .any(|node| node.widget_kind() == Some("note")),
        "{:?}",
        home.segments
    );
    let widgets = planned
        .pages
        .iter()
        .find(|page| page.view.route == "/widgets")
        .unwrap();
    let card_segment = widgets
        .segments
        .iter()
        .position(|node| node.widget_kind() == Some("card-grid"))
        .expect("card-grid segment");
    assert!(
        card_segment + 1 < widgets.segments.len(),
        "{:?}",
        widgets.segments
    );
    assert!(
        widgets
            .fragments
            .iter()
            .any(|(_, html)| html.contains(crate::islands::PLACEHOLDER)),
        "{:?}",
        widgets.fragments
    );
    let roc = planned.pages_roc();
    let card_at = roc.find("CardGrid({").expect("card-grid in generated Roc");
    let island_at = roc[card_at..]
        .find("HtmlFile({ path:")
        .map(|offset| card_at + offset)
        .expect("island fragment after card-grid");
    assert!(card_at < island_at, "{roc}");
    assert!(widgets.view.resources.module_script.is_empty());
    assert!(widgets.view.resources.chrome_script.contains("goto."));
    assert!(rocci_ui::chrome_script().contains("__rocciCopy"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_local_theme_compiles_and_is_staged() {
    let root = temp("custom-theme");
    write_site(&root);
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
@component SiteShell = |view, content| {
<html>
    <head>
        <title>{view.title} - Custom Theme</title>
    </head>
    <body>
        <header>Custom Header</header>
        <main>{content}</main>
    </body>
</html>
}
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

    let type_names: Vec<_> = planned
        .theme_modules
        .iter()
        .map(|m| m.type_name.as_str())
        .collect();
    assert!(type_names.contains(&"SiteShell"));
    assert!(type_names.contains(&"RocdownTheme"));
    assert!(type_names.contains(&"DocsComponents"));
    assert!(type_names.contains(&"BlockPainters"));
    assert!(type_names.contains(&"RocdownBase"));

    let css = planned
        .theme_modules
        .iter()
        .flat_map(|module| module.styles.iter())
        .map(|style| style.css.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(css.contains("--canvas"), "{css}");
    assert!(css.contains(".rd-header-1"), "{css}");
    assert!(
        !css.contains("data-rocci-css~=\"RocdownBase"),
        "base article CSS must apply without a document stamp\n{css}"
    );

    let site_shell = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "SiteShell")
        .unwrap();
    assert!(
        !site_shell.roc.contains("Html.text(content)"),
        "{}",
        site_shell.roc
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn block_pack_binds_note_to_pack_module() {
    let root = temp("block-pack-overlay");
    write_site(&root);
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Html

@component SiteShell = |view, content| {
<html>
    <body>{content}</body>
</html>
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Note = |{ title }, content|
<section data-test-note data-title={title}>{content}</section>
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let painters = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "BlockPainters")
        .unwrap();
    assert!(
        painters.roc.contains("Blocks.note(props, content)"),
        "{}",
        painters.roc
    );
    assert!(
        painters.roc.contains("DocsComponents.tip(props, content)"),
        "{}",
        painters.roc
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn generated_dispatcher_covers_widget_kinds_without_editing_runtime() {
    let arms = widget_kind_render_arms();
    for spec in crate::registry::KINDS
        .iter()
        .filter(|kind| kind.paints_as_widget())
    {
        let painter = roc_fn_name(spec.component);
        assert!(
            arms.contains(&format!("{}(seg)", spec.component)),
            "generated dispatcher missing tag `{}`: {arms}",
            spec.component
        );
        assert!(
            arms.contains(&format!("BlockPainters.{painter}")),
            "generated dispatcher missing `{painter}`: {arms}"
        );
    }
    let src = include_str!("../../runtime/RocdownBuild.roc");
    assert!(
        src.contains("# rocci-widget-kind-arms"),
        "runtime dispatcher should splice generated arms"
    );
    assert!(
        !src.contains("Note(seg)"),
        "builtin widget arms should not be handwritten in RocdownBuild.roc"
    );
    assert!(src.contains("render_forest!"), "{src}");
    assert!(src.contains("HtmlFile"), "{src}");
}

fn write_shell(root: &Path) {
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Html

@component SiteShell = |view, content| {
<html>
    <body>{content}</body>
</html>
}
"#,
    )
    .unwrap();
}

#[test]
fn blocks_pack_path_replaces_theme_blocks_convention() {
    let root = temp("blocks-pack-path");
    write_site(&root);
    write_shell(&root);
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Note = |{ title }, content|
<section data-convention-note>{content}</section>
"#,
    )
    .unwrap();
    fs::write(
        root.join("theme/AltPack.rocci"),
        r#"
import Html

@component Note = |{ title }, content|
<section data-pack-note>{content}</section>
"#,
    )
    .unwrap();
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"

[blocks]
pack = "theme/AltPack.rocci"
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let painters = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "BlockPainters")
        .unwrap();
    assert!(
        painters.roc.contains("AltPack.note(props, content)"),
        "{}",
        painters.roc
    );
    assert!(
        !painters.roc.contains("Blocks.note(props, content)"),
        "{}",
        painters.roc
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn blocks_override_remaps_kind_to_pack_component() {
    let root = temp("blocks-override");
    write_site(&root);
    write_shell(&root);
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Callout = |{ title }, content|
<section data-callout data-title={title}>{content}</section>
"#,
    )
    .unwrap();
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"

[blocks.override]
note = "Callout"
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let painters = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "BlockPainters")
        .unwrap();
    assert!(
        painters
            .roc
            .contains("note = |props, content|\n        Blocks.callout(props, content)"),
        "{}",
        painters.roc
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn pack_custom_kind_is_planned_and_dispatched() {
    let root = temp("pack-custom-callout");
    write_site(&root);
    write_shell(&root);
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Callout = |{ tone ?? "note" }, content|
<aside data-test-callout data-tone={tone}>{content}</aside>
"#,
    )
    .unwrap();
    fs::write(
        root.join("index.rocdown"),
        "# Home\n\n:callout[tone: \"warn\"] {{\n    Watch this.\n}}\n",
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let roc = planned.pages_roc();
    assert!(roc.contains("Callout({"), "{roc}");
    assert!(roc.contains("tone: \"warn\""), "{roc}");
    assert!(
        planned
            .widget_render_arms
            .contains("BlockPainters.callout({ tone: seg.tone }, body)"),
        "{}",
        planned.widget_render_arms
    );
    assert!(
        planned
            .widget_render_arms
            .contains("BlockPainters.note({ title: seg.title }, body)"),
        "{}",
        planned.widget_render_arms
    );
    let painters = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "BlockPainters")
        .unwrap();
    assert!(
        painters.roc.contains("callout = |props, content|"),
        "{}",
        painters.roc
    );
    assert!(
        painters.roc.contains("Blocks.callout(props, content)"),
        "{}",
        painters.roc
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn pack_reserved_helper_name_fails_site_load() {
    let root = temp("pack-reserved-page");
    write_site(&root);
    write_shell(&root);
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Page = |{ title }, content|
<div>{content}</div>
"#,
    )
    .unwrap();

    let err = load_site(&root).unwrap_err().to_string();
    assert!(
        err.contains("reserved name") && err.contains("helpers must not live in the block pack"),
        "{err}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn blocks_override_unknown_component_fails_theme_compile() {
    let root = temp("blocks-override-missing");
    write_site(&root);
    write_shell(&root);
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"

[blocks.override]
note = "Missing"
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let err = plan(&loaded.root, &loaded.config, &resolved.site)
        .unwrap_err()
        .to_string();
    assert!(err.contains("unknown component `Missing`"), "{err}");

    let _ = fs::remove_dir_all(root);
}

fn write_incomplete_docs(root: &Path) {
    write_shell(root);
    fs::write(
        root.join("theme/DocsComponents.rocci"),
        r#"
import Html

@component Tip = |{ title }, content|
<p data-stub-tip data-title={title}>{content}</p>
"#,
    )
    .unwrap();
}

#[test]
fn missing_painter_errors_unless_debug_or_preview() {
    let root = temp("missing-painter");
    write_site(&root);
    write_incomplete_docs(&root);

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let err = plan(&loaded.root, &loaded.config, &resolved.site)
        .unwrap_err()
        .to_string();
    assert!(err.contains("no renderer bound for kind `note`"), "{err}");

    let preview = plan_preview(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let painters = preview
        .theme_modules
        .iter()
        .find(|module| module.type_name == "BlockPainters")
        .unwrap();
    assert!(
        painters.roc.contains("BlockDebug.debug({ kind: \"note\""),
        "{}",
        painters.roc
    );
    assert!(
        painters.roc.contains("data-rocci-block-debug")
            || preview
                .theme_modules
                .iter()
                .any(|module| module.type_name == "BlockDebug"
                    && (module.src.contains("data-rocci-block-debug")
                        || module.roc.contains("data-rocci-block-debug"))),
        "debug component missing data-rocci-block-debug"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_painter_binds_debug_when_flag_set() {
    let root = temp("missing-painter-debug");
    write_site(&root);
    write_incomplete_docs(&root);
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"

[blocks]
debug = true
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let painters = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "BlockPainters")
        .unwrap();
    assert!(
        painters.roc.contains("BlockDebug.debug({ kind: \"note\""),
        "{}",
        painters.roc
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_layout_article_slot_is_html_body_param() {
    let root = temp("layout-body-param");
    write_site(&root);
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Layouts

@component SiteShell = |view, content| {
<html>
    <body>
        <Layouts.Home view={view}>{content}</Layouts.Home>
    </body>
</html>
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("theme/Layouts.rocci"),
        r#"
@component Home = |{ view }, content| {
<article class="article">{content}</article>
}
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
    let layouts = planned
        .theme_modules
        .iter()
        .find(|module| module.type_name == "Layouts")
        .unwrap();
    assert!(
        !layouts.roc.contains("Html.text(content)"),
        "{}",
        layouts.roc
    );
    assert!(layouts.roc.contains("content"), "{}", layouts.roc);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn named_layouts_and_collection_metadata_are_propagated() {
    let root = temp("layouts-meta");
    write_site(&root);
    fs::write(
        root.join("guide.rocdown"),
        r#"
@page {
layout: "plain",
published: "2026-08-18",
updated: "2026-08-19",
authors: ["Nils", "Collaborator"],
tags: ["guide", "release"],
collection: "guides",
summary: "A plain guide without docs sidebar",
}

# Guide

Content here.
"#,
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

    let guide = planned
        .pages
        .iter()
        .find(|p| p.view.route == "/guide")
        .unwrap();
    assert_eq!(guide.view.layout, "plain");
    assert_eq!(guide.view.published, "2026-08-18");
    assert_eq!(guide.view.updated, "2026-08-19");
    assert_eq!(guide.view.authors, vec!["Nils", "Collaborator"]);
    assert_eq!(guide.view.tags, vec!["guide", "release"]);
    assert_eq!(guide.view.collection, "guides");
    assert_eq!(guide.view.description, "A plain guide without docs sidebar");

    let roc = planned.pages_roc();
    assert!(roc.contains("layout: \"plain\""));
    assert!(roc.contains("published: \"2026-08-18\""));
    assert!(roc.contains("authors: ["));
    assert!(roc.contains("\"Nils\""));
    assert!(roc.contains("collection: \"guides\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn unknown_layout_returns_rd2007_diagnostic() {
    let root = temp("bad-layout");
    write_site(&root);
    fs::write(
        root.join("guide.rocdown"),
        "@page {\n    layout: \"nonexistent_layout\"\n}\n\n# Guide\n",
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(resolved.has_errors());
    assert!(
        resolved.diagnostics.iter().any(
            |d| d.code == "RD2007" && d.message.contains("unknown layout `nonexistent_layout`")
        )
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn collection_sorting_and_feed_generation_in_plan() {
    let root = temp("news-collection");
    write_site(&root);
    fs::create_dir_all(root.join("news")).unwrap();
    fs::write(
        root.join("news/index.rocdown"),
        "@page {\n    layout: \"news-index\",\n}\n\n# News\n",
    )
    .unwrap();
    fs::write(
        root.join("news/older.rocdown"),
        "@page {\n    layout: \"news-post\",\n    published: \"2026-08-10\",\n    collection: \"news\",\n    summary: \"Older post\",\n}\n\n# Older\n",
    )
    .unwrap();
    fs::write(
        root.join("news/newer.rocdown"),
        "@page {\n    layout: \"news-post\",\n    published: \"2026-08-18\",\n    collection: \"news\",\n    summary: \"Newer post\",\n}\n\n# Newer\n",
    )
    .unwrap();
    fs::write(
        root.join("news/draft.rocdown"),
        "@page {\n    draft: True,\n    layout: \"news-post\",\n    published: \"2026-08-20\",\n    collection: \"news\",\n}\n\n# Draft\n",
    )
    .unwrap();

    let loaded = load_site(&root).unwrap();
    let resolved = resolve_loaded(&loaded);
    assert!(!resolved.has_errors(), "{}", resolved.error_summary());
    let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

    let news_index = planned
        .pages
        .iter()
        .find(|p| p.view.route == "/news/")
        .unwrap();
    assert_eq!(news_index.view.collection_items.len(), 2);
    assert_eq!(news_index.view.collection_items[0].title, "Newer");
    assert_eq!(news_index.view.collection_items[0].published, "2026-08-18");
    assert_eq!(news_index.view.collection_items[1].title, "Older");
    assert_eq!(news_index.view.collection_items[1].published, "2026-08-10");

    let home = planned.pages.iter().find(|p| p.view.route == "/").unwrap();
    assert_eq!(home.view.collection_items.len(), 2);
    assert_eq!(home.view.collection_items[0].title, "Newer");

    let feed = planned
        .files
        .iter()
        .find(|f| f.output_path == "news/feed.xml")
        .unwrap();
    assert_eq!(feed.kind, "feed");
    assert!(feed.contents.contains("<title>Newer</title>"));
    assert!(feed.contents.contains("<title>Older</title>"));
    assert!(!feed.contents.contains("Draft"));

    let roc = planned.pages_roc();
    assert!(roc.contains("collection_items: ["));
    assert!(roc.contains("title: \"Newer\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn builtin_theme_keeps_phone_menu_and_table_wrap() {
    let theme = runtime::THEME;
    assert!(theme.contains("id=\"site-nav\""));
    assert!(theme.contains("class=\"mobile-menu\""));
    assert!(theme.contains("@media (max-width: 70rem)"));
    assert!(theme.contains("@media (max-width: 48rem)"));
    assert!(theme.contains(".mobile-menu { position: relative; display: block"));
    assert!(theme.contains("100dvh"));
    assert!(theme.contains("env(safe-area-inset-top"));
    assert!(theme.contains(
        "min-height: calc(100vh - var(--header-height) - var(--rocci-chrome-bottom, 0px))"
    ));
    assert!(
        theme.contains(
            "height: calc(100vh - var(--header-height) - var(--rocci-chrome-bottom, 0px))"
        )
    );
    assert!(theme.contains(
        "max-height: calc(100vh - var(--header-height) - env(safe-area-inset-top, 0px) - var(--rocci-chrome-bottom, 0px))"
    ));
    let panel = theme.find("class=\"mobile-panel\"").expect("mobile panel");
    let details_end = theme[panel..].find("</details>").expect("details close");
    assert!(
        theme[panel..panel + details_end].contains("PageOutline.pageOutline"),
        "phone menu must include on-this-page links"
    );
    assert!(runtime::BASE.contains("rd-table-wrap"));
    assert!(runtime::BASE.contains("overflow-x: auto"));
    assert!(!runtime::BASE.contains("overflow: hidden"));
}
