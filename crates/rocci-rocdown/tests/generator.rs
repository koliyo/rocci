use std::collections::BTreeSet;
use std::path::Path;

use rocci_rocdown::{
    CompileOptions, IncludeOptions, NavConfig, PageDocs, PageHeading, ResolveOptions, RouteHint,
    SourceFile, SourcePage, compile, load_page_docs, render_article, resolve,
};

#[test]
fn test_article_html_components() {
    let doc_src = r#"# Components Guide

:note[title: "Important"] This is a callout note.

:tabs.begin[group: "languages", kind: "language"]
    :tab[id: "roc", label: "Roc"] {{
        ```roc
        main = "Hello"
        ```
    }}
    :tab[id: "rust", label: "Rust"] {{
        ```rust
        fn main() {}
        ```
    }}
:tabs.end

:details[summary: "Click to expand", open: True] Hidden content revealed.

:badge[label: "Beta", tone: "beta"]

:link-card[page: "/guides/install/", title: "Installation"]

:img[src: "/media/logo.png", alt: "Rocci Logo", width: "100", height: "100"]

Footnote reference.[^first]

[^first]: Footnote explanation.
"#;

    let source = SourceFile::new("guide.rocdown", doc_src);
    let compiled = compile(
        source,
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!compiled.has_errors(), "{:?}", compiled.diagnostics);

    let mut diagnostics = Vec::new();
    let root = Path::new(".");
    let page_docs = load_page_docs(
        source,
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(!diagnostics.iter().any(|d| d.is_error()), "{diagnostics:?}");

    let (segments, _fragments) =
        rocci_rocdown::plan_segments("guide", &page_docs.article, &Default::default());

    // Verify note aside widget
    let note = segments
        .iter()
        .find_map(|node| match node {
            rocci_rocdown::PlannedNode::Widget(widget) if widget.kind == "note" => Some(widget),
            _ => None,
        })
        .expect("missing note widget");
    assert_eq!(note.str_prop("title"), Some("Important"));

    // Verify tabs widget
    let tabs = segments
        .iter()
        .find_map(|node| match node {
            rocci_rocdown::PlannedNode::Widget(widget) if widget.kind == "tabs" => Some(widget),
            _ => None,
        })
        .expect("missing tabs widget");
    assert_eq!(tabs.children.len(), 2);
    let tab0 = match &tabs.children[0] {
        rocci_rocdown::PlannedNode::Widget(widget) => widget,
        _ => panic!("tab 0"),
    };
    let tab1 = match &tabs.children[1] {
        rocci_rocdown::PlannedNode::Widget(widget) => widget,
        _ => panic!("tab 1"),
    };
    assert_eq!(tab0.str_prop("label"), Some("Roc"));
    assert_eq!(tab1.str_prop("label"), Some("Rust"));

    // Verify details widget
    let details = segments
        .iter()
        .find_map(|node| match node {
            rocci_rocdown::PlannedNode::Widget(widget) if widget.kind == "details" => Some(widget),
            _ => None,
        })
        .expect("missing details widget");
    assert_eq!(details.str_prop("summary"), Some("Click to expand"));
    assert_eq!(details.bool_prop("open"), Some(true));

    // Verify badge widget
    let badge = segments
        .iter()
        .find_map(|node| match node {
            rocci_rocdown::PlannedNode::Widget(widget) if widget.kind == "badge" => Some(widget),
            _ => None,
        })
        .expect("missing badge widget");
    assert_eq!(badge.str_prop("label"), Some("Beta"));
    assert!(badge.str_prop("tone").is_none());

    // Verify link card widget
    let link_card = segments
        .iter()
        .find_map(|node| match node {
            rocci_rocdown::PlannedNode::Widget(widget) if widget.kind == "link-card" => {
                Some(widget)
            }
            _ => None,
        })
        .expect("missing link-card widget");
    assert_eq!(link_card.str_prop("title"), Some("Installation"));
    assert_eq!(link_card.str_prop("href"), Some("/guides/install/"));

    // Verify markdown projection and image / footnote rendering
    let html = render_article(&page_docs.article);
    assert!(
        html.contains("<img class=\"rd-image\"") && html.contains("src=\"/media/logo.png\""),
        "missing img: {html}"
    );
    assert!(
        html.contains("data-footnote-ref") && html.contains("aria-label=\"Footnotes\""),
        "missing footnote: {html}"
    );
}

#[test]
fn test_site_catalog_resolution() {
    let pages = vec![
        SourcePage {
            id: "index".to_string(),
            id_explicit: false,
            source_path: "index.rocdown".to_string(),
            route_hint: RouteHint::Derived,
            aliases: vec![],
            draft: false,
            suppress_unlisted_warning: false,
            layout: "home".to_string(),
            published: String::new(),
            updated: String::new(),
            authors: vec![],
            tags: vec![],
            collection: String::new(),
            title: "Home".to_string(),
            description: "Welcome to Rocci docs".to_string(),
            headings: vec![PageHeading {
                level: 1,
                id: "home".to_string(),
                text: "Home".to_string(),
            }],
            outgoing_links: vec!["/guide/".to_string()],
            image_urls: vec![],
            article_html: "<p>Welcome</p>".to_string(),
            island_css: String::new(),
            kind: rocci_rocdown::PageKind::Static,
            docs: PageDocs::default(),
        },
        SourcePage {
            id: "guide".to_string(),
            id_explicit: true,
            source_path: "guide.rocdown".to_string(),
            route_hint: RouteHint::Explicit("/guide/".to_string()),
            aliases: vec!["/getting-started/".to_string()],
            draft: false,
            suppress_unlisted_warning: false,
            layout: "docs".to_string(),
            published: String::new(),
            updated: String::new(),
            authors: vec![],
            tags: vec![],
            collection: String::new(),
            title: "Guide".to_string(),
            description: "User Guide".to_string(),
            headings: vec![PageHeading {
                level: 1,
                id: "guide".to_string(),
                text: "Guide".to_string(),
            }],
            outgoing_links: vec!["/".to_string()],
            image_urls: vec![],
            article_html: "<p>Guide text</p>".to_string(),
            island_css: String::new(),
            kind: rocci_rocdown::PageKind::Static,
            docs: PageDocs::default(),
        },
    ];

    let options = ResolveOptions {
        navigation: vec![NavConfig {
            label: "General".to_string(),
            items: vec!["index".to_string(), "guide".to_string()],
            directory: None,
            href: None,
            groups: Vec::new(),
        }],
        files: BTreeSet::new(),
    };

    let resolved = resolve(&pages, &options);
    assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);

    assert_eq!(resolved.site.pages.len(), 2);
    let home = resolved
        .site
        .pages
        .iter()
        .find(|p| p.id == "index")
        .expect("home page");
    let guide = resolved
        .site
        .pages
        .iter()
        .find(|p| p.id == "guide")
        .expect("guide page");

    assert_eq!(home.route, "/");
    assert_eq!(guide.route, "/guide/");
    assert_eq!(guide.aliases, vec!["/getting-started/"]);
    assert_eq!(resolved.site.navigation.len(), 1);
    assert_eq!(resolved.site.navigation[0].items.len(), 2);
}

#[test]
fn test_site_with_mounts() {
    let temp_root = std::env::temp_dir().join(format!("rocdown-mount-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_root);
    let site_dir = temp_root.join("site");
    let docs_dir = temp_root.join("docs");
    std::fs::create_dir_all(&site_dir).unwrap();
    std::fs::create_dir_all(&docs_dir).unwrap();

    std::fs::write(
        site_dir.join("rocdown.toml"),
        r#"
[site]
title = "Mount Test"

[[mount]]
source = "../docs"
prefix = "docs"
layout = "docs"

[[nav]]
label = "Main"
items = ["index"]

[[nav]]
label = "Docs"
items = ["docs/getting-started"]
"#,
    )
    .unwrap();

    std::fs::write(
        site_dir.join("index.rocdown"),
        r#"
@page {
    meta: {
        title: "Welcome",
    },
}

# Welcome

Read our [Getting Started guide](/docs/getting-started/).
"#,
    )
    .unwrap();

    std::fs::write(
        docs_dir.join("getting-started.rocdown"),
        r#"
@page {
    aliases: ["/getting-started/"],
    meta: {
        title: "Getting Started",
    },
}

# Getting Started

Back to [Home](/) or [Relative Home](../).
"#,
    )
    .unwrap();

    let loaded = rocci_rocdown::load_site(&site_dir).unwrap();
    assert_eq!(loaded.sources.len(), 2);

    let result = rocci_rocdown::resolve_loaded(&loaded);
    assert!(!result.has_errors(), "{:?}", result.diagnostics);

    let home = result.site.pages.iter().find(|p| p.id == "index").unwrap();
    let docs_page = result
        .site
        .pages
        .iter()
        .find(|p| p.id == "docs/getting-started")
        .unwrap();

    assert_eq!(home.route, "/");
    assert_eq!(docs_page.route, "/docs/getting-started/");
    assert_eq!(docs_page.layout, "docs");
    assert_eq!(docs_page.aliases, vec!["/getting-started/"]);

    assert_eq!(result.site.navigation.len(), 2);
    assert_eq!(
        result.site.navigation[1].items[0].id,
        "docs/getting-started"
    );
    assert_eq!(
        result.site.navigation[1].items[0].route,
        "/docs/getting-started/"
    );

    let _ = std::fs::remove_dir_all(&temp_root);
}
