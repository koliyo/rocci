use std::collections::BTreeSet;
use std::path::Path;

use rocci_rocdown::{CompileOptions, SourceFile, compile};
use rocs::{
    IncludeOptions, NavConfig, PageDocs, PageHeading, ResolveOptions, RouteHint, SourcePage,
    load_page_docs, render_article, resolve,
};

#[test]
fn golden_article_html_components() {
    let doc_src = r#"# Components Guide

@docs note {
    title: "Important"
    This is a callout note.
}

@docs tabs {
    group: "languages"
    kind: "language"
    @docs tab {
        id: "roc"
        label: "Roc"
        ```roc
        main = "Hello"
        ```
    }
    @docs tab {
        id: "rust"
        label: "Rust"
        ```rust
        fn main() {}
        ```
    }
}

@docs details {
    summary: "Click to expand"
    open: true
    Hidden content revealed.
}

@docs badge {
    label: "Beta"
    tone: "beta"
}

@docs link-card {
    page: "/guides/install/"
    title: "Installation"
}

@img {
    src: "/media/logo.png"
    alt: "Rocci Logo"
    width: "100"
    height: "100"
}

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
            root: &root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(!diagnostics.iter().any(|d| d.is_error()), "{diagnostics:?}");

    let html = render_article(&page_docs.article);

    // Verify note aside rendering
    assert!(
        html.contains("class=\"rd-docs-aside rd-docs-note rd-docs-block\""),
        "missing note: {html}"
    );
    assert!(
        html.contains("This is a callout note."),
        "missing note body: {html}"
    );

    // Verify tabs rendering
    assert!(
        html.contains("class=\"rd-docs-tabs rd-docs-block\""),
        "missing tabs: {html}"
    );
    assert!(html.contains("Roc"), "missing roc tab: {html}");

    // Verify details rendering
    assert!(
        html.contains("<details class=\"rd-docs-details rd-docs-block\"")
            && html.contains("<summary class=\"rd-docs-summary\">Click to expand</summary>"),
        "missing details: {html}"
    );

    // Verify badge rendering
    assert!(
        html.contains("class=\"rd-docs-badge rd-docs-block\""),
        "missing badge: {html}"
    );
    assert!(html.contains("Beta"), "missing badge label: {html}");

    // Verify link card rendering
    assert!(
        html.contains("class=\"rd-docs-card rd-docs-link-card rd-docs-block\""),
        "missing link card: {html}"
    );

    // Verify img rendering
    assert!(
        html.contains("<img class=\"rd-image\"") && html.contains("src=\"/media/logo.png\""),
        "missing img: {html}"
    );

    // Verify footnotes
    assert!(
        html.contains("data-footnote-ref") && html.contains("aria-label=\"Footnotes\""),
        "missing footnote: {html}"
    );
}

#[test]
fn golden_site_catalog_resolution() {
    let pages = vec![
        SourcePage {
            id: "index".to_string(),
            id_explicit: false,
            source_path: "index.rocdown".to_string(),
            route_hint: RouteHint::Derived,
            aliases: vec![],
            draft: false,
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
            docs: PageDocs::default(),
        },
        SourcePage {
            id: "guide".to_string(),
            id_explicit: true,
            source_path: "guide.rocdown".to_string(),
            route_hint: RouteHint::Explicit("/guide/".to_string()),
            aliases: vec!["/getting-started/".to_string()],
            draft: false,
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
            docs: PageDocs::default(),
        },
    ];

    let options = ResolveOptions {
        navigation: vec![NavConfig {
            label: "General".to_string(),
            items: vec!["index".to_string(), "guide".to_string()],
            directory: None,
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
