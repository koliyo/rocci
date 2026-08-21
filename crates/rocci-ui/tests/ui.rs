use rocci_ui::*;

#[test]
fn test_view_constructors_and_serialization() {
    let site = SiteView {
        title: "Rocci Docs".into(),
        description: "A fast framework".into(),
        base_url: "https://rocci.dev".into(),
        language: "en".into(),
        repository: "https://github.com/koliyo/rocci".into(),
        social_image: "/assets/og.png".into(),
        favicon: "/assets/favicon.svg".into(),
        apple_touch_icon: "/assets/apple-touch-icon.png".into(),
        subtitle: "Tools".into(),
        footer: "MIT License".into(),
    };

    let page = PageView {
        site,
        lanes: vec![
            LaneView {
                label: "Guide".into(),
                href: "/guide/".into(),
                current: true,
            },
            LaneView {
                label: "Reference".into(),
                href: "/reference/".into(),
                current: false,
            },
        ],
        sidebar: vec![NavItemView::new("Intro", "/guide/", "nav-link is-current")],
        route: "/guide/".into(),
        title: "Guide".into(),
        description: "Getting started".into(),
        layout: "docs".into(),
        published: "2026-08-18".into(),
        updated: "".into(),
        authors: vec!["Nils".into()],
        tags: vec!["guide".into()],
        collection: "".into(),
        collection_items: vec![CollectionItemView {
            route: "/news/post/".into(),
            title: "Post".into(),
            summary: "Summary".into(),
            published: "2026-08-18".into(),
            updated: "".into(),
            authors: vec!["Author".into()],
            tags: vec!["news".into()],
        }],
        outline: vec![
            OutlineView::new("installation", "Installation", "2"),
            OutlineView::new("configuration", "Configuration", "3"),
        ],
        breadcrumbs: vec![
            BreadcrumbView::new("Home", "/"),
            BreadcrumbView::new("Guide", "/guide/"),
        ],
        previous: NavItemView::default(),
        next: NavItemView::new("Next Chapter", "/guide/next/", "nav-link"),
        resources: ResourceView {
            stylesheet: "/assets/theme.css".into(),
            csp: "default-src 'self'".into(),
            canonical: "https://rocci.dev/guide/".into(),
            module_script: String::new(),
            chrome_script: "/assets/goto.js".into(),
            playground_css: String::new(),
        },
    };

    let json = serde_json::to_string(&page).unwrap();
    let deserialized: PageView = serde_json::from_str(&json).unwrap();
    assert_eq!(page, deserialized);
}

#[test]
fn goto_script_is_self_contained() {
    assert!(GOTO_SCRIPT.contains("window.__rocciGoto"));
    assert!(GOTO_SCRIPT.contains("/pages.json"));
    assert!(GOTO_SCRIPT.contains("/catalog.json"));
    assert!(GOTO_SCRIPT.contains("history.pushState"));
    assert!(GOTO_SCRIPT.contains("rocci-goto"));
    assert!(GOTO_SCRIPT.contains("data-rocci-goto-open"));
}

#[test]
fn test_escape_html() {
    assert_eq!(
        escape("<script>alert(\"hello\") & 'world'</script>"),
        "&lt;script&gt;alert(&quot;hello&quot;) &amp; &#39;world&#39;&lt;/script&gt;"
    );
}

#[test]
fn test_chrome_templates_compile() {
    for (name, src) in [
        ("Breadcrumbs.rocci", chrome::BREADCRUMBS),
        ("NavList.rocci", chrome::NAV_LIST),
        ("PageOutline.rocci", chrome::PAGE_OUTLINE),
    ] {
        let file = rocci_template::SourceFile::new(name, src);
        let compiled = rocci_template::compile(file, &rocci_template::LowerOptions::default());
        assert!(
            !compiled.has_errors(),
            "{name} compilation failed: {:?}",
            compiled.diagnostics
        );
        assert!(!compiled.roc.is_empty(), "{name} produced empty Roc");
    }
}
