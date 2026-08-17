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
        },
    };

    let json = serde_json::to_string(&page).unwrap();
    let deserialized: PageView = serde_json::from_str(&json).unwrap();
    assert_eq!(page, deserialized);
}

#[test]
fn test_render_site_shell() {
    let page = PageView {
        site: SiteView {
            title: "Rocci".into(),
            subtitle: "Docs".into(),
            language: "en".into(),
            repository: "https://github.com/koliyo/rocci".into(),
            ..Default::default()
        },
        lanes: vec![LaneView {
            label: "Guide".into(),
            href: "/guide/".into(),
            current: true,
        }],
        sidebar: vec![NavItemView::new("Intro", "/guide/", "nav-link is-current")],
        route: "/guide/".into(),
        title: "Introduction".into(),
        breadcrumbs: vec![
            BreadcrumbView::new("Home", "/"),
            BreadcrumbView::new("Guide", "/guide/"),
        ],
        outline: vec![OutlineView::new("setup", "Setup", "2")],
        next: NavItemView::new("Next", "/guide/step2/", "nav-link"),
        ..Default::default()
    };

    let html = render_site_shell(&page, "<p>Welcome to Rocci.</p>");
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("<title>Introduction · Rocci</title>"));
    assert!(html.contains("class=\"brand\""));
    assert!(html.contains("class=\"lane-link is-current\""));
    assert!(html.contains("class=\"crumb-current\""));
    assert!(html.contains("<p>Welcome to Rocci.</p>"));
    assert!(html.contains("href=\"#setup\""));
    assert!(html.contains("href=\"/guide/step2/\""));
}

#[test]
fn test_render_stat_grid_and_badges() {
    let cards = vec![
        StatCardView::new("12", "Total Records"),
        StatCardView::new("3", "Action Required")
            .with_tone(StatTone::Action)
            .with_href("/review/"),
    ];
    let html = render_stat_grid(&cards);
    assert!(html.contains("class=\"okf-stat-grid\""));
    assert!(html.contains("Total Records"));
    assert!(html.contains("is-action"));
    assert!(html.contains("href=\"/review/\""));

    let badge = BadgeView::new("Human-Reviewed", BadgeTone::Human)
        .with_sub_label("human:nils @ 2026-08-17");
    let badge_html = render_badge(&badge);
    assert!(badge_html.contains("okf-trust-human"));
    assert!(badge_html.contains("human:nils @ 2026-08-17"));

    let alert = AlertView::new("Review Action Required", "Post-verification drift detected");
    let alert_html = render_alert_banner(&alert);
    assert!(alert_html.contains("okf-alert-banner alert-warning"));
    assert!(alert_html.contains("Post-verification drift detected"));
}

#[test]
fn test_template_syntax_parses() {
    let template = ROCCI_UI_TEMPLATE;
    assert!(template.contains("@component Breadcrumbs"));
    assert!(template.contains("@component Outline"));
    assert!(template.contains("@component Journey"));
    assert!(template.contains("@component StatCard"));
    assert!(template.contains("@component AlertBanner"));

    // Verify it compiles with rocci-template compile
    let source = rocci_template::SourceFile::new("RocciUi.rocci", template);
    let output = rocci_template::compile(source, &rocci_template::LowerOptions::default());
    let errors: Vec<_> = output.diagnostics.iter().filter(|d| d.is_error()).collect();
    assert!(
        errors.is_empty(),
        "RocciUi.rocci compile errors: {:?}",
        errors
    );
    assert!(!output.roc.is_empty());
    assert!(output.components.iter().any(|c| c.name == "breadcrumbs"));
    assert!(output.components.iter().any(|c| c.name == "outline"));
    assert!(output.components.iter().any(|c| c.name == "journey"));
    assert!(output.components.iter().any(|c| c.name == "statCard"));
    assert!(output.components.iter().any(|c| c.name == "alertBanner"));
}
