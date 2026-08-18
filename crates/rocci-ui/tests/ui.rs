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
            playground_css: String::new(),
        },
    };

    let json = serde_json::to_string(&page).unwrap();
    let deserialized: PageView = serde_json::from_str(&json).unwrap();
    assert_eq!(page, deserialized);
}

#[test]
fn test_escape_html() {
    assert_eq!(
        escape("<script>alert(\"hello\") & 'world'</script>"),
        "&lt;script&gt;alert(&quot;hello&quot;) &amp; &#39;world&#39;&lt;/script&gt;"
    );
}
