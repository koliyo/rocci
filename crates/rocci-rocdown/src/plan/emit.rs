use serde::Serialize;

use crate::article::PageKind;
use crate::catalog::{NavSection, ResolvedPage};
use crate::config::SiteConfig;
use crate::service::IslandRoute;
use rocci_ui::{
    BreadcrumbView, CollectionItemView, NavGroupView, NavItemView, PageView, ResourceView, SiteView,
};

use super::nav::{find_route_id, lanes_and_sidebar};
use super::{PlannedFile, PlannedPage, PublishPage, document_title};

#[derive(Debug, Clone, Serialize)]
struct PageIndexEntry<'a> {
    title: &'a str,
    route: &'a str,
    path: &'a str,
    kind: PageKind,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    datastar: bool,
    #[serde(skip_serializing_if = "str::is_empty")]
    description: &'a str,
}

pub(crate) fn not_found_page(
    site: &SiteView,
    navigation: &[NavSection],
    stylesheet: &str,
    csp: &str,
    chrome_script: &str,
) -> PlannedPage {
    let home = find_route_id(navigation, "/");
    let (lanes, _) = lanes_and_sidebar(navigation, home);
    let mut recovery_items = lanes
        .iter()
        .map(|lane| NavItemView::new(&lane.label, &lane.href, "nav-link nav-child"))
        .collect::<Vec<_>>();
    for (title, route) in [
        ("Start with Rocci", "/docs/"),
        ("Project status", "/project/status/"),
    ] {
        if !recovery_items.iter().any(|item| item.href == route) {
            recovery_items.push(NavItemView::new(title, route, "nav-link nav-child"));
        }
    }
    let sidebar = vec![NavGroupView::new("Recover", "", true, recovery_items)];
    PlannedPage {
        article_path: "articles/NotFound.html".into(),
        output_path: "404.html".into(),
        article_html: not_found_html(),
        fragments: vec![("articles/NotFound.html".into(), not_found_html())],
        segments: vec![crate::docs::PlannedNode::Html {
            path: "articles/NotFound.html".into(),
        }],
        view: PageView {
            site: site.clone(),
            lanes,
            sidebar,
            route: "/404.html".into(),
            title: "Page not found".into(),
            document_title: document_title("Page not found", &site.title),
            description: "This page does not exist.".into(),
            layout: "not-found".into(),
            published: String::new(),
            updated: String::new(),
            authors: Vec::new(),
            tags: Vec::new(),
            collection: String::new(),
            collection_items: Vec::new(),
            outline: Vec::new(),
            breadcrumbs: vec![
                BreadcrumbView::new(&site.title, "/"),
                BreadcrumbView::new("Page not found", "/404.html"),
            ],
            previous: NavItemView::default(),
            next: NavItemView::default(),
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: csp.to_string(),
                canonical: String::new(),
                module_script: String::new(),
                chrome_script: chrome_script.to_string(),
                playground_css: String::new(),
                playground_session: String::new(),
            },
        },
    }
}

fn not_found_html() -> String {
    String::from(
        "<h1 class=\"rd-header-1\">Page not found</h1>\n\
<p class=\"rd-paragraph\">This URL is not part of the current Rocci manual. \
The stack-first docs moved several academy routes; use the links below or \
<strong>Go to</strong> (Cmd/Ctrl+K) to open a live page.</p>\n\
<p class=\"rd-paragraph\" id=\"rocci-not-found-hint\" hidden></p>\n\
<ul class=\"rd-list\">\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/docs/\">Documentation home</a></p></li>\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/docs/five-minutes/\">Rocci in five minutes</a> — preview a component</p></li>\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/docs/install/\">Install Rocci</a></p></li>\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/\">Site home</a></p></li>\n\
</ul>\n",
    )
}

pub(crate) fn discovery_files(
    config: &SiteConfig,
    pages: &[ResolvedPage],
    news_items: &[CollectionItemView],
    service_routes: &[IslandRoute],
) -> Vec<PlannedFile> {
    let mut files = Vec::new();
    let mut llms = format!("# {}\n\n{}\n\n", config.site.title, config.site.description);
    for page in pages {
        let url = format!("{}{}", config.site.base_url, page.route);
        if page.description.is_empty() {
            llms.push_str(&format!("- [{}]({url})\n", page.title));
        } else {
            llms.push_str(&format!(
                "- [{}]({url}): {}\n",
                page.title, page.description
            ));
        }
    }
    files.push(PlannedFile {
        kind: "llms",
        route: "/llms.txt".into(),
        output_path: "llms.txt".into(),
        contents: llms,
    });
    files.push(PlannedFile {
        kind: "pages",
        route: "/pages.json".into(),
        output_path: "pages.json".into(),
        contents: pages_json(pages),
    });
    if pages.iter().any(|page| page.kind == PageKind::Live) {
        files.push(PlannedFile {
            kind: "islands",
            route: "/islands.json".into(),
            output_path: "islands.json".into(),
            contents: islands_json(&config.http.service_origin, pages, service_routes),
        });
    }
    if !config.site.base_url.is_empty() {
        let mut sitemap = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        for page in pages {
            sitemap.push_str("  <url><loc>");
            sitemap.push_str(&escape_xml(&format!(
                "{}{}",
                config.site.base_url, page.route
            )));
            sitemap.push_str("</loc></url>\n");
        }
        sitemap.push_str("</urlset>\n");
        files.push(PlannedFile {
            kind: "sitemap",
            route: "/sitemap.xml".into(),
            output_path: "sitemap.xml".into(),
            contents: sitemap,
        });
        files.push(PlannedFile {
            kind: "robots",
            route: "/robots.txt".into(),
            output_path: "robots.txt".into(),
            contents: format!(
                "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
                config.site.base_url
            ),
        });
        if !news_items.is_empty() {
            files.push(PlannedFile {
                kind: "feed",
                route: "/news/feed.xml".into(),
                output_path: "news/feed.xml".into(),
                contents: atom_feed(config, news_items),
            });
        }
    }
    files.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    files
}

fn pages_json(pages: &[ResolvedPage]) -> String {
    let mut entries: Vec<PageIndexEntry<'_>> = pages
        .iter()
        .map(|page| PageIndexEntry {
            title: &page.title,
            route: &page.route,
            path: &page.source_path,
            kind: page.kind,
            datastar: page.kind == PageKind::Live,
            description: &page.description,
        })
        .collect();
    entries.sort_by(|left, right| left.route.cmp(right.route));
    match serde_json::to_string_pretty(&entries) {
        Ok(json) => format!("{json}\n"),
        Err(_) => "[]\n".into(),
    }
}

fn islands_json(service_origin: &str, pages: &[ResolvedPage], routes: &[IslandRoute]) -> String {
    #[derive(Serialize)]
    struct IslandsPage<'a> {
        id: &'a str,
        route: &'a str,
        kind: PageKind,
    }
    #[derive(Serialize)]
    struct IslandsFile<'a> {
        service_origin: &'a str,
        pages: Vec<IslandsPage<'a>>,
        routes: &'a [IslandRoute],
    }
    let mut island_pages: Vec<IslandsPage<'_>> = pages
        .iter()
        .filter(|page| page.kind == PageKind::Live)
        .map(|page| IslandsPage {
            id: &page.id,
            route: &page.route,
            kind: page.kind,
        })
        .collect();
    island_pages.sort_by(|left, right| left.route.cmp(right.route).then(left.id.cmp(right.id)));
    let file = IslandsFile {
        service_origin,
        pages: island_pages,
        routes,
    };
    match serde_json::to_string_pretty(&file) {
        Ok(json) => format!("{json}\n"),
        Err(_) => "{\n  \"service_origin\": \"\",\n  \"pages\": [],\n  \"routes\": []\n}\n".into(),
    }
}

pub(crate) fn publish_pages(pages: &[ResolvedPage]) -> Vec<PublishPage> {
    let mut entries: Vec<PublishPage> = pages
        .iter()
        .map(|page| PublishPage {
            id: page.id.clone(),
            route: page.route.clone(),
            kind: page.kind,
            datastar: page.kind == PageKind::Live,
            output_path: page.output_path.clone(),
        })
        .collect();
    entries.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then_with(|| left.id.cmp(&right.id))
    });
    entries
}

fn atom_feed(config: &SiteConfig, news_items: &[CollectionItemView]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n",
    );
    xml.push_str(&format!(
        "  <title>{} News</title>\n",
        escape_xml(&config.site.title)
    ));
    let feed_url = format!("{}/news/feed.xml", config.site.base_url);
    let site_news_url = format!("{}/news/", config.site.base_url);
    xml.push_str(&format!(
        "  <link href=\"{}\" rel=\"self\" />\n",
        escape_xml(&feed_url)
    ));
    xml.push_str(&format!(
        "  <link href=\"{}\" />\n",
        escape_xml(&site_news_url)
    ));
    xml.push_str(&format!("  <id>{}</id>\n", escape_xml(&site_news_url)));

    let latest_date = news_items
        .first()
        .map(|item| {
            if !item.updated.is_empty() {
                item.updated.as_str()
            } else if !item.published.is_empty() {
                item.published.as_str()
            } else {
                "2026-01-01"
            }
        })
        .unwrap_or("2026-01-01");
    xml.push_str(&format!("  <updated>{}T00:00:00Z</updated>\n", latest_date));

    for item in news_items {
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", escape_xml(&item.title)));
        let entry_url = format!("{}{}", config.site.base_url, item.route);
        xml.push_str(&format!(
            "    <link href=\"{}\" />\n",
            escape_xml(&entry_url)
        ));
        xml.push_str(&format!("    <id>{}</id>\n", escape_xml(&entry_url)));
        let pub_date = if !item.published.is_empty() {
            &item.published
        } else {
            "2026-01-01"
        };
        xml.push_str(&format!(
            "    <published>{}T00:00:00Z</published>\n",
            pub_date
        ));
        let upd_date = if !item.updated.is_empty() {
            &item.updated
        } else {
            pub_date
        };
        xml.push_str(&format!("    <updated>{}T00:00:00Z</updated>\n", upd_date));
        if !item.summary.is_empty() {
            xml.push_str(&format!(
                "    <summary>{}</summary>\n",
                escape_xml(&item.summary)
            ));
        }
        for author in &item.authors {
            xml.push_str(&format!(
                "    <author><name>{}</name></author>\n",
                escape_xml(author)
            ));
        }
        xml.push_str("  </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}

pub(crate) fn redirect_html(target: &str) -> String {
    let target = escape_xml(target);
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>Redirect</title>\n<link rel=\"canonical\" href=\"{target}\">\n<meta http-equiv=\"refresh\" content=\"0; url={target}\">\n</head>\n<body>\n<p>Moved to <a href=\"{target}\">{target}</a>.</p>\n</body>\n</html>\n"
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) fn pages_roc(pages: &[PlannedPage]) -> String {
    let mut pages = pages.to_vec();
    pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    let mut out = String::from("RocdownPages := [].{\n    pages = [\n");
    for page in &pages {
        out.push_str("        {\n            article_path: ");
        push_roc_string(&mut out, &page.article_path);
        out.push_str(",\n            output_path: ");
        push_roc_string(&mut out, &page.output_path);
        out.push_str(",\n            segments: ");
        push_nodes(&mut out, &page.segments, 3);
        out.push_str(
            ",\n            view: {\n                site: {\n                    title: ",
        );
        push_roc_string(&mut out, &page.view.site.title);
        out.push_str(",\n                    description: ");
        push_roc_string(&mut out, &page.view.site.description);
        out.push_str(",\n                    base_url: ");
        push_roc_string(&mut out, &page.view.site.base_url);
        out.push_str(",\n                    language: ");
        push_roc_string(&mut out, &page.view.site.language);
        out.push_str(",\n                    repository: ");
        push_roc_string(&mut out, &page.view.site.repository);
        out.push_str(",\n                    social_image: ");
        push_roc_string(&mut out, &page.view.site.social_image);
        out.push_str(",\n                    favicon: ");
        push_roc_string(&mut out, &page.view.site.favicon);
        out.push_str(",\n                    apple_touch_icon: ");
        push_roc_string(&mut out, &page.view.site.apple_touch_icon);
        out.push_str(",\n                    subtitle: ");
        push_roc_string(&mut out, &page.view.site.subtitle);
        out.push_str(",\n                    footer: ");
        push_roc_string(&mut out, &page.view.site.footer);
        out.push_str("\n                },\n                lanes: [\n");
        for lane in &page.view.lanes {
            out.push_str("                    { label: ");
            push_roc_string(&mut out, &lane.label);
            out.push_str(", href: ");
            push_roc_string(&mut out, &lane.href);
            out.push_str(", current: ");
            out.push_str(if lane.current { "True" } else { "False" });
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                sidebar: [\n");
        for group in &page.view.sidebar {
            emit_nav_group(&mut out, group, "                    ", true);
        }
        out.push_str("                ],\n                route: ");
        push_roc_string(&mut out, &page.view.route);
        out.push_str(",\n                title: ");
        push_roc_string(&mut out, &page.view.title);
        out.push_str(",\n                document_title: ");
        push_roc_string(&mut out, &page.view.document_title);
        out.push_str(",\n                description: ");
        push_roc_string(&mut out, &page.view.description);
        out.push_str(",\n                layout: ");
        push_roc_string(&mut out, &page.view.layout);
        out.push_str(",\n                published: ");
        push_roc_string(&mut out, &page.view.published);
        out.push_str(",\n                updated: ");
        push_roc_string(&mut out, &page.view.updated);
        out.push_str(",\n                authors: [\n");
        for author in &page.view.authors {
            out.push_str("                    ");
            push_roc_string(&mut out, author);
            out.push_str(",\n");
        }
        out.push_str("                ],\n                tags: [\n");
        for tag in &page.view.tags {
            out.push_str("                    ");
            push_roc_string(&mut out, tag);
            out.push_str(",\n");
        }
        out.push_str("                ],\n                collection: ");
        push_roc_string(&mut out, &page.view.collection);
        out.push_str(",\n                collection_items: [\n");
        for item in &page.view.collection_items {
            out.push_str("                    {\n                        route: ");
            push_roc_string(&mut out, &item.route);
            out.push_str(",\n                        title: ");
            push_roc_string(&mut out, &item.title);
            out.push_str(",\n                        summary: ");
            push_roc_string(&mut out, &item.summary);
            out.push_str(",\n                        published: ");
            push_roc_string(&mut out, &item.published);
            out.push_str(",\n                        updated: ");
            push_roc_string(&mut out, &item.updated);
            out.push_str(",\n                        authors: [\n");
            for author in &item.authors {
                out.push_str("                            ");
                push_roc_string(&mut out, author);
                out.push_str(",\n");
            }
            out.push_str("                        ],\n                        tags: [\n");
            for tag in &item.tags {
                out.push_str("                            ");
                push_roc_string(&mut out, tag);
                out.push_str(",\n");
            }
            out.push_str("                        ]\n                    },\n");
        }
        out.push_str("                ],\n                outline: [\n");
        for heading in &page.view.outline {
            out.push_str("                    { id: ");
            push_roc_string(&mut out, &heading.id);
            out.push_str(", title: ");
            push_roc_string(&mut out, &heading.title);
            out.push_str(", level: ");
            push_roc_string(&mut out, &heading.level);
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                breadcrumbs: [\n");
        for crumb in &page.view.breadcrumbs {
            out.push_str("                    { title: ");
            push_roc_string(&mut out, &crumb.title);
            out.push_str(", href: ");
            push_roc_string(&mut out, &crumb.href);
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                previous: { title: ");
        push_roc_string(&mut out, &page.view.previous.title);
        out.push_str(", href: ");
        push_roc_string(&mut out, &page.view.previous.href);
        out.push_str(" },\n                next: { title: ");
        push_roc_string(&mut out, &page.view.next.title);
        out.push_str(", href: ");
        push_roc_string(&mut out, &page.view.next.href);
        out.push_str(" },\n                resources: {\n                    stylesheet: ");
        push_roc_string(&mut out, &page.view.resources.stylesheet);
        out.push_str(",\n                    csp: ");
        push_roc_string(&mut out, &page.view.resources.csp);
        out.push_str(",\n                    canonical: ");
        push_roc_string(&mut out, &page.view.resources.canonical);
        out.push_str(",\n                    module_script: ");
        push_roc_string(&mut out, &page.view.resources.module_script);
        out.push_str(",\n                    chrome_script: ");
        push_roc_string(&mut out, &page.view.resources.chrome_script);
        out.push_str(",\n                    playground_css: ");
        push_roc_string(&mut out, &page.view.resources.playground_css);
        out.push_str(",\n                    playground_session: ");
        push_roc_string(&mut out, &page.view.resources.playground_session);
        out.push_str("\n                }\n            }\n        },\n");
    }
    out.push_str("    ]\n}\n");
    out
}

fn push_nodes(out: &mut String, nodes: &[crate::docs::PlannedNode], indent: usize) {
    let mut flat = Vec::new();
    collect_flat(nodes, &mut flat);
    out.push_str("[\n");
    for node in flat {
        for _ in 0..indent + 1 {
            out.push_str("    ");
        }
        push_node(out, node);
        out.push_str(",\n");
    }
    for _ in 0..indent {
        out.push_str("    ");
    }
    out.push(']');
}

fn collect_flat<'a>(
    nodes: &'a [crate::docs::PlannedNode],
    out: &mut Vec<&'a crate::docs::PlannedNode>,
) {
    for node in nodes {
        out.push(node);
        if let crate::docs::PlannedNode::Widget(widget) = node {
            collect_flat(&widget.children, out);
        }
    }
}

fn push_node(out: &mut String, node: &crate::docs::PlannedNode) {
    match node {
        crate::docs::PlannedNode::Html { path } => {
            out.push_str("HtmlFile({ path: ");
            push_roc_string(out, path);
            out.push_str(" })");
        }
        crate::docs::PlannedNode::Widget(widget) => {
            out.push_str(&widget.component);
            out.push_str("({ ");
            let spec = crate::registry::lookup(&widget.kind);
            let paint_content =
                widget.paint_content || spec.is_some_and(|kind| kind.paint_content());
            for (index, prop) in widget.props.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                match prop {
                    crate::docs::PlannedProp::Str { name, value } => {
                        out.push_str(name);
                        out.push_str(": ");
                        push_roc_string(out, value);
                    }
                    crate::docs::PlannedProp::Bool { name, value } => {
                        out.push_str(name);
                        out.push_str(": ");
                        out.push_str(if *value { "True" } else { "False" });
                    }
                }
            }
            if paint_content {
                if !widget.props.is_empty() {
                    out.push_str(", ");
                }
                out.push_str("child_count: ");
                out.push_str(&widget.children.len().to_string());
            }
            out.push_str(" })");
        }
    }
}

fn emit_nav_group(out: &mut String, group: &NavGroupView, indent: &str, with_children: bool) {
    out.push_str(indent);
    out.push_str("{ title: ");
    push_roc_string(out, &group.title);
    out.push_str(", href: ");
    push_roc_string(out, &group.href);
    out.push_str(", open: ");
    out.push_str(if group.open { "True" } else { "False" });
    out.push_str(", items: [\n");
    let item_indent = format!("{indent}    ");
    for item in &group.items {
        out.push_str(&item_indent);
        out.push_str("{ title: ");
        push_roc_string(out, &item.title);
        out.push_str(", href: ");
        push_roc_string(out, &item.href);
        out.push_str(", class_name: ");
        push_roc_string(out, &item.class_name);
        out.push_str(" },\n");
    }
    out.push_str(indent);
    if with_children {
        out.push_str("], children: [\n");
        let child_indent = format!("{indent}    ");
        for child in &group.children {
            emit_nav_group(out, child, &child_indent, false);
        }
        out.push_str(indent);
        out.push_str("] },\n");
    } else {
        out.push_str("], children: [] },\n");
    }
}

fn push_roc_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}
