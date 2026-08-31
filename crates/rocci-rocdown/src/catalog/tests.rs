use std::collections::BTreeSet;

use crate::config::NavConfig;

use super::*;

fn page(id: &str, path: &str, hint: RouteHint, title: &str) -> SourcePage {
    SourcePage {
        id: id.to_string(),
        id_explicit: false,
        source_path: path.to_string(),
        route_hint: hint,
        aliases: Vec::new(),
        draft: false,
        suppress_unlisted_warning: false,
        layout: "docs".to_string(),
        published: String::new(),
        updated: String::new(),
        authors: Vec::new(),
        tags: Vec::new(),
        collection: String::new(),
        title: title.to_string(),
        description: String::new(),
        headings: Vec::new(),
        outgoing_links: Vec::new(),
        image_urls: Vec::new(),
        article_html: String::new(),
        island_css: String::new(),
        kind: crate::article::PageKind::Static,
        docs: crate::docs::PageDocs::default(),
    }
}

fn resolved(pages: &[SourcePage]) -> ResolveResult {
    resolve(pages, &ResolveOptions::default())
}

fn codes(result: &ResolveResult) -> Vec<&str> {
    result.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn derives_index_and_named_routes() {
    let result = resolved(&[
        page("guide", "guide.rocdown", RouteHint::Derived, "Guide"),
        page("index", "index.rocdown", RouteHint::Derived, "Home"),
    ]);
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert_eq!(result.site.pages[0].route, "/guide/");
    assert_eq!(result.site.pages[0].output_path, "guide/index.html");
    assert_eq!(result.site.pages[1].route, "/");
    assert_eq!(result.site.pages[1].output_path, "index.html");
    assert_eq!(result.site.pages[0].kind, crate::article::PageKind::Static);
}

#[test]
fn copies_page_kind_onto_resolved_pages() {
    let mut hydrate = page("widget", "widget.rocdown", RouteHint::Derived, "Widget");
    hydrate.kind = crate::article::PageKind::Hydrate;
    let mut live = page("counter", "counter.rocdown", RouteHint::Derived, "Counter");
    live.kind = crate::article::PageKind::Live;
    let result = resolved(&[hydrate, live]);
    let by_id: std::collections::BTreeMap<_, _> = result
        .site
        .pages
        .iter()
        .map(|page| (page.id.as_str(), page.kind))
        .collect();
    assert_eq!(by_id["widget"], crate::article::PageKind::Hydrate);
    assert_eq!(by_id["counter"], crate::article::PageKind::Live);
}

#[test]
fn derives_nested_index_routes() {
    assert_eq!(derived_route("guides/index"), "/guides/");
    assert_eq!(derived_route("guides/build"), "/guides/build/");
}

#[test]
fn explicit_id_is_independent_of_route() {
    let mut source = page(
        "guides.install",
        "install.rocdown",
        RouteHint::Explicit("/setup/".into()),
        "Install",
    );
    source.id_explicit = true;
    let result = resolved(&[source]);
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert_eq!(result.site.pages[0].id, "guides.install");
    assert_eq!(result.site.pages[0].route, "/setup/");
}

#[test]
fn duplicate_ids_are_errors() {
    let result = resolved(&[
        page("same", "a.rocdown", RouteHint::Derived, "A"),
        page("same", "b.rocdown", RouteHint::Derived, "B"),
    ]);
    assert!(codes(&result).contains(&"RD2001"));
    assert!(result.error_summary().contains("a.rocdown"));
    assert!(result.error_summary().contains("b.rocdown"));
}

#[test]
fn explicit_route_gets_trailing_slash_and_sorts_by_output() {
    let result = resolved(&[
        page("b", "b.rocdown", RouteHint::Explicit("/zeta".into()), "Z"),
        page("a", "a.rocdown", RouteHint::Explicit("/alpha/".into()), "A"),
    ]);
    assert_eq!(result.site.pages[0].output_path, "alpha/index.html");
    assert_eq!(result.site.pages[1].output_path, "zeta/index.html");
    assert_eq!(result.site.pages[1].route, "/zeta/");
}

#[test]
fn duplicate_routes_name_both_sources() {
    let result = resolved(&[
        page(
            "beta",
            "beta.rocdown",
            RouteHint::Explicit("/same/".into()),
            "Beta",
        ),
        page(
            "alpha",
            "alpha.rocdown",
            RouteHint::Explicit("/same/".into()),
            "Alpha",
        ),
    ]);
    let message = result.error_summary();
    assert!(codes(&result).contains(&"RD2002"));
    assert!(message.contains("duplicate route `/same/`"), "{message}");
    assert!(message.contains("alpha.rocdown"), "{message}");
    assert!(message.contains("beta.rocdown"), "{message}");
}

#[test]
fn aliases_collide_with_routes_and_each_other() {
    let mut old = page("old", "old.rocdown", RouteHint::Derived, "Old");
    old.aliases = vec!["/guide/".into()];
    let guide = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
    let result = resolved(&[old, guide]);
    assert!(codes(&result).contains(&"RD2003"));

    let mut a = page("a", "a.rocdown", RouteHint::Derived, "A");
    a.aliases = vec!["/legacy/".into()];
    let mut b = page("b", "b.rocdown", RouteHint::Derived, "B");
    b.aliases = vec!["/legacy/".into()];
    let result = resolved(&[a, b]);
    assert!(codes(&result).contains(&"RD2003"));
}

#[test]
fn case_insensitive_route_collision() {
    let result = resolved(&[
        page("a", "a.rocdown", RouteHint::Explicit("/Guide/".into()), "A"),
        page("b", "b.rocdown", RouteHint::Explicit("/guide/".into()), "B"),
    ]);
    assert!(codes(&result).contains(&"RD2006"));
}

#[test]
fn rejects_dotdot_and_relative_routes() {
    let result = resolved(&[page(
        "x",
        "x.rocdown",
        RouteHint::Explicit("/ok/../secret/".into()),
        "X",
    )]);
    assert!(result.error_summary().contains("(..)"));
    let result = resolved(&[page(
        "y",
        "y.rocdown",
        RouteHint::Explicit("relative".into()),
        "Y",
    )]);
    assert!(result.error_summary().contains("not absolute"));
}

#[test]
fn discovery_order_does_not_change_output_order() {
    let a = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
    let b = page("index", "index.rocdown", RouteHint::Derived, "Home");
    let forward = resolved(&[a.clone(), b.clone()]);
    let reverse = resolved(&[b, a]);
    assert_eq!(
        forward
            .site
            .pages
            .iter()
            .map(|p| p.output_path.as_str())
            .collect::<Vec<_>>(),
        reverse
            .site
            .pages
            .iter()
            .map(|p| p.output_path.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn resolves_configured_navigation_by_stable_id() {
    let pages = [
        page("guide", "guide.rocdown", RouteHint::Derived, "Guide"),
        page("index", "index.rocdown", RouteHint::Derived, "Home"),
    ];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Start".into(),
                items: vec!["index".into(), "guide".into()],
                directory: None,
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert_eq!(result.site.navigation[0].items[0].route, "/");
    assert_eq!(result.site.navigation[0].items[1].title, "Guide");
    let guide = result
        .site
        .pages
        .iter()
        .find(|page| page.id == "guide")
        .unwrap();
    assert_eq!(guide.previous.as_ref().unwrap().id, "index");
    let home = result
        .site
        .pages
        .iter()
        .find(|page| page.id == "index")
        .unwrap();
    assert!(home.next.is_some());
    assert!(result.site.unlisted.is_empty());
    assert_eq!(guide.breadcrumbs.last().unwrap().title, "Guide");
}

#[test]
fn examples_nav_skips_unstaged_page_ids() {
    let pages = [page(
        "examples/index",
        "examples/index.rocdown",
        RouteHint::Derived,
        "Examples",
    )];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Examples".into(),
                items: vec!["examples/index".into(), "examples/notes/index".into()],
                directory: Some("examples".into()),
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert!(!codes(&result).contains(&"RD2201"));
    assert_eq!(result.site.navigation[0].items.len(), 1);
    assert_eq!(result.site.navigation[0].items[0].id, "examples/index");
}

#[test]
fn linked_detail_is_unlisted_without_warning_noise() {
    let home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    let mut detail = page(
        "generated/detail",
        "generated/detail.rocdown",
        RouteHint::Derived,
        "Generated detail",
    );
    detail.suppress_unlisted_warning = true;
    let result = resolve(
        &[home, detail],
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Start".into(),
                items: vec!["index".into()],
                directory: None,
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );

    assert!(result.site.unlisted.contains(&"generated/detail".into()));
    assert!(!codes(&result).contains(&"RD2202"));
}

#[test]
fn authored_unlisted_page_still_warns() {
    let pages = [
        page("index", "index.rocdown", RouteHint::Derived, "Home"),
        page("orphan", "orphan.rocdown", RouteHint::Derived, "Orphan"),
    ];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Start".into(),
                items: vec!["index".into()],
                directory: None,
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );

    assert!(codes(&result).contains(&"RD2202"));
}

#[test]
fn indexless_listed_cluster_warns_rd2205() {
    let pages = [
        page(
            "docs/index",
            "docs/index.rocdown",
            RouteHint::Derived,
            "Overview",
        ),
        page(
            "docs/appendix/glossary",
            "docs/appendix/glossary.rocdown",
            RouteHint::Derived,
            "Glossary",
        ),
        page(
            "docs/appendix/roc-for-rocci",
            "docs/appendix/roc-for-rocci.rocdown",
            RouteHint::Derived,
            "Roc for Rocci",
        ),
        page(
            "docs/reference/contributor/checklist",
            "docs/reference/contributor/checklist.rocdown",
            RouteHint::Derived,
            "Checklist",
        ),
        page(
            "docs/reference/contributor/rocci-tree",
            "docs/reference/contributor/rocci-tree.rocdown",
            RouteHint::Derived,
            "Rocci tree",
        ),
    ];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Start".into(),
                items: vec![
                    "docs/index".into(),
                    "docs/appendix/glossary".into(),
                    "docs/appendix/roc-for-rocci".into(),
                    "docs/reference/contributor/checklist".into(),
                    "docs/reference/contributor/rocci-tree".into(),
                ],
                directory: None,
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "RD2205")
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings.iter().any(|m| m.contains("docs/appendix")));
    assert!(
        warnings
            .iter()
            .any(|m| m.contains("docs/reference/contributor"))
    );
}

#[test]
fn listed_index_clears_rd2205() {
    let pages = [
        page(
            "docs/appendix/index",
            "docs/appendix/index.rocdown",
            RouteHint::Derived,
            "Appendix",
        ),
        page(
            "docs/appendix/glossary",
            "docs/appendix/glossary.rocdown",
            RouteHint::Derived,
            "Glossary",
        ),
        page(
            "docs/appendix/roc-for-rocci",
            "docs/appendix/roc-for-rocci.rocdown",
            RouteHint::Derived,
            "Roc for Rocci",
        ),
    ];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Start".into(),
                items: vec![
                    "docs/appendix/index".into(),
                    "docs/appendix/glossary".into(),
                    "docs/appendix/roc-for-rocci".into(),
                ],
                directory: None,
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert!(!codes(&result).contains(&"RD2205"));
}

#[test]
fn directory_navigation_lists_index_first() {
    let pages = [
        page(
            "guides/build",
            "guides/build.rocdown",
            RouteHint::Derived,
            "Build",
        ),
        page(
            "guides/index",
            "guides/index.rocdown",
            RouteHint::Derived,
            "Guides",
        ),
        page("index", "index.rocdown", RouteHint::Derived, "Home"),
    ];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Guides".into(),
                items: Vec::new(),
                directory: Some("guides".into()),
                groups: Vec::new(),
            }],
            files: BTreeSet::new(),
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    let ids: Vec<_> = result.site.navigation[0]
        .items
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    assert_eq!(ids, ["guides/index", "guides/build"]);
    assert!(!result.site.unlisted.contains(&"index".to_string()));
    assert!(!codes(&result).contains(&"RD2202"));
    assert!(!result.has_errors());
}

#[test]
fn resolves_nested_navigation_groups() {
    let pages = [
        page(
            "docs/index",
            "docs/index.rocdown",
            RouteHint::Derived,
            "Docs",
        ),
        page(
            "docs/tutorials/index",
            "docs/tutorials/index.rocdown",
            RouteHint::Derived,
            "Tutorials",
        ),
        page(
            "docs/tutorials/first-component",
            "docs/tutorials/first-component.rocdown",
            RouteHint::Derived,
            "Build your first component",
        ),
    ];
    let result = resolve(
        &pages,
        &ResolveOptions {
            navigation: vec![NavConfig {
                label: "Docs".into(),
                items: Vec::new(),
                directory: None,
                groups: vec![NavConfig {
                    label: "Tutorials".into(),
                    items: vec![
                        "docs/tutorials/index".into(),
                        "docs/tutorials/first-component".into(),
                    ],
                    directory: None,
                    groups: Vec::new(),
                }],
            }],
            files: BTreeSet::new(),
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert_eq!(result.site.navigation[0].label, "Docs");
    assert!(result.site.navigation[0].items.is_empty());
    assert_eq!(result.site.navigation[0].children[0].label, "Tutorials");
    assert_eq!(
        result.site.navigation[0].children[0].items[1].title,
        "Build your first component"
    );
    let tutorial = result
        .site
        .pages
        .iter()
        .find(|page| page.id == "docs/tutorials/first-component")
        .unwrap();
    assert!(
        tutorial
            .breadcrumbs
            .iter()
            .any(|crumb| crumb.title == "Tutorials")
    );
    assert!(
        !result
            .site
            .unlisted
            .contains(&"docs/tutorials/first-component".to_string())
    );
}

#[test]
fn rejects_broken_absolute_and_heading_links() {
    let mut home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    home.outgoing_links = vec!["/missing/".into(), "/guide/#nope".into()];
    let mut guide = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
    guide.headings.push(PageHeading {
        level: 2,
        id: "install".into(),
        text: "Install".into(),
    });
    let result = resolved(&[home, guide]);
    assert!(result.error_summary().contains("/missing/"));
    assert!(result.error_summary().contains("/guide/#nope"));
    assert!(codes(&result).contains(&"RD2101"));
    assert!(codes(&result).contains(&"RD2102"));
}

#[test]
fn accepts_include_source_line_anchor_links() {
    let mut source = page(
        "examples/styling/source/Styling-rocci",
        "examples/styling/source/Styling-rocci.rocdown",
        RouteHint::Derived,
        "Styling.rocci",
    );
    source.outgoing_links = vec![
        "#L4".into(),
        "/examples/styling/source/Styling-rocci/#L25".into(),
    ];
    source.article_html = concat!(
        "<pre class=\"rd-code-block rd-source-code\"><code>",
        "<span class=\"rd-source-line\" id=\"L4\">@get:view(\"/\")</span>\n",
        "<span class=\"rd-source-line\" id=\"L25\">@css { }</span>",
        "</code></pre>"
    )
    .into();
    let result = resolved(&[source]);
    assert!(!result.has_errors(), "{}", result.error_summary());
}

#[test]
fn rejects_missing_include_source_line_anchor() {
    let mut source = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
    source.outgoing_links = vec!["#L4".into()];
    let result = resolved(&[source]);
    assert!(result.error_summary().contains("#L4"));
    assert!(codes(&result).contains(&"RD2102"));
}

#[test]
fn accepts_valid_absolute_and_same_page_heading_links() {
    let mut home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    home.headings.push(PageHeading {
        level: 2,
        id: "start".into(),
        text: "Start".into(),
    });
    home.outgoing_links = vec!["/guide/#install".into(), "#start".into()];
    let mut guide = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
    guide.headings.push(PageHeading {
        level: 2,
        id: "install".into(),
        text: "Install".into(),
    });
    let result = resolved(&[home, guide]);
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert!(
        result
            .site
            .graph
            .iter()
            .any(|edge| edge.kind == EdgeKind::Heading && edge.target == "guide#install")
    );
}

#[test]
fn allows_missing_sibling_product_lanes() {
    let mut home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    home.outgoing_links = vec![
        "/examples/styling/".into(),
        "/rocdown/".into(),
        "/project/status/".into(),
        "/missing/".into(),
    ];
    let result = resolved(&[home]);
    assert!(result.error_summary().contains("/missing/"));
    assert!(!result.error_summary().contains("/examples/styling/"));
    assert!(!result.error_summary().contains("/rocdown/"));
    assert!(!result.error_summary().contains("/project/status/"));
    assert!(
        result
            .site
            .graph
            .iter()
            .any(|edge| edge.kind == EdgeKind::Asset && edge.raw == "/examples/styling/")
    );
}

#[test]
fn resolves_relative_wiki_and_asset_links() {
    let mut home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    home.outgoing_links = vec!["./guide.rocdown".into(), "Guide".into()];
    home.image_urls = vec!["/assets/og.png".into()];
    let guide = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
    let mut files = BTreeSet::new();
    files.insert("assets/og.png".into());
    let result = resolve(
        &[home, guide],
        &ResolveOptions {
            navigation: Vec::new(),
            files,
        },
    );
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert!(
        result
            .site
            .graph
            .iter()
            .any(|edge| edge.kind == EdgeKind::Page && edge.target == "guide")
    );
    assert!(
        result
            .site
            .graph
            .iter()
            .any(|edge| edge.kind == EdgeKind::Asset && edge.target == "/assets/og.png")
    );
}

#[test]
fn relative_link_from_nested_page() {
    let mut page_a = page(
        "guides/build",
        "guides/build.rocdown",
        RouteHint::Derived,
        "Build",
    );
    page_a.outgoing_links = vec!["../concepts/architecture.rocdown".into()];
    let page_b = page(
        "concepts/architecture",
        "concepts/architecture.rocdown",
        RouteHint::Derived,
        "Architecture",
    );
    let result = resolved(&[page_a, page_b]);
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert!(
        result
            .site
            .graph
            .iter()
            .any(|edge| edge.target == "concepts/architecture")
    );
}

#[test]
fn published_link_to_draft_is_an_error() {
    let mut home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    home.outgoing_links = vec!["/secret/".into()];
    let mut draft = page("secret", "secret.rocdown", RouteHint::Derived, "Secret");
    draft.draft = true;
    let result = resolved(&[home, draft]);
    assert!(codes(&result).contains(&"RD2104"));
}

#[test]
fn collects_independent_diagnostics_in_one_run() {
    let mut home = page("index", "index.rocdown", RouteHint::Derived, "Home");
    home.outgoing_links = vec!["/missing/".into()];
    let a = page("a", "a.rocdown", RouteHint::Explicit("/same/".into()), "A");
    let b = page("b", "b.rocdown", RouteHint::Explicit("/same/".into()), "B");
    let result = resolved(&[home, a, b]);
    assert!(codes(&result).contains(&"RD2101"));
    assert!(codes(&result).contains(&"RD2002"));
}

#[test]
fn hundred_page_fixture_resolves_internal_links() {
    let mut pages = Vec::new();
    for index in 0..100 {
        let id = format!("p{index:03}");
        let mut source = page(&id, &format!("{id}.rocdown"), RouteHint::Derived, &id);
        if index + 1 < 100 {
            source.outgoing_links = vec![format!("/p{:03}/", index + 1)];
        }
        pages.push(source);
    }
    let result = resolved(&pages);
    assert!(!result.has_errors(), "{}", result.error_summary());
    assert_eq!(result.site.pages.len(), 100);
    pages[50].outgoing_links = vec!["/nope/".into()];
    let result = resolved(&pages);
    assert!(result.has_errors());
    assert!(result.error_summary().contains("p050.rocdown"));
}
