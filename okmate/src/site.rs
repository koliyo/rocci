use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use okf::{BuildSummary, Bundle, Profile};
use serde::Serialize;

use crate::views::{Document, NavNode, ReviewRow, review_rows, toc_from_headings};

const APP_CSS: &str = include_str!("../assets/app.css");
const DATASTAR_JS: &str = include_str!("../assets/datastar.js");

#[derive(Serialize)]
struct NavPage {
    title: String,
    route: String,
    path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    collection: String,
}

pub fn build(root: &Path, output: &Path, profile: Profile) -> Result<BuildSummary> {
    let summary = okf::build(root, output, profile)?;
    let bundle = okf::load(root, profile)?;
    write_html_pages(&bundle, output)?;
    write_pages_json(&bundle, output)?;
    write_assets(output)?;
    Ok(summary)
}

pub fn write_html_pages(bundle: &Bundle, output: &Path) -> Result<()> {
    let rows = review_rows(bundle);
    if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
        write_route(
            output,
            "/",
            document(bundle, "/", "Knowledge", toc_from_headings(&index.headings))
                .with_article(&index.article_html)
                .render_home()?,
        )?;
    } else {
        write_route(
            output,
            "/",
            document(bundle, "/", "Knowledge", Vec::new())
                .with_article("<h1>Knowledge</h1>")
                .render_home()?,
        )?;
    }

    for concept in &bundle.concepts {
        let title = okf::string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        let route = format!("/{}/", concept.id);
        write_route(
            output,
            &route,
            document(bundle, &route, title, toc_from_headings(&concept.headings))
                .with_article(&concept.article_html)
                .with_meta(concept)
                .render_page()?,
        )?;
    }

    for index in &bundle.indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        let route = format!("/{collection}/");
        let title = collection_title(index);
        write_route(
            output,
            &route,
            document(bundle, &route, &title, toc_from_headings(&index.headings))
                .with_article(&index.article_html)
                .render_page()?,
        )?;
    }

    write_route(
        output,
        "/review/",
        document(
            bundle,
            "/review/",
            "Knowledge Governance & Review Queue",
            Vec::new(),
        )
        .with_review(rows)
        .render_review()?,
    )?;

    write_route(
        output,
        "/settings/",
        settings_document(bundle).render_settings()?,
    )?;

    Ok(())
}

fn document(
    bundle: &Bundle,
    route: &str,
    title: &str,
    toc: Vec<crate::views::TocEntry>,
) -> Document {
    Document {
        title: title.to_string(),
        nav: nav_tree(bundle, route),
        toc,
        article_html: String::new(),
        concept_type: String::new(),
        status: String::new(),
        authority: String::new(),
        review_rows: Vec::new(),
        message: String::new(),
        config_path: String::new(),
        settings_roots: Vec::new(),
    }
}

pub(crate) fn settings_shell(bundle: &Bundle) -> crate::views::Document {
    settings_document(bundle)
}

fn settings_document(bundle: &Bundle) -> Document {
    let config = crate::config::load().unwrap_or_default();
    let mut document = document(bundle, "/settings/", "Knowledge roots", Vec::new());
    document.config_path = crate::config::config_path().display().to_string();
    document.settings_roots = crate::http::settings_roots(&config);
    document
}

impl Document {
    fn with_article(mut self, html: &str) -> Self {
        self.article_html = html.to_string();
        self
    }

    fn with_meta(mut self, concept: &okf::Concept) -> Self {
        self.concept_type = okf::string_field(&concept.metadata, "type")
            .unwrap_or("Concept")
            .to_string();
        self.status = okf::string_field(&concept.metadata, "status")
            .unwrap_or("draft")
            .to_string();
        self.authority = okf::string_field(&concept.metadata, "authority")
            .unwrap_or("descriptive")
            .to_string();
        self
    }

    fn with_review(mut self, rows: Vec<ReviewRow>) -> Self {
        self.review_rows = rows;
        self
    }
}

fn write_route(output: &Path, route: &str, html: String) -> Result<()> {
    let path = route_to_path(output, route);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, html).with_context(|| format!("failed to write {}", path.display()))
}

fn route_to_path(output: &Path, route: &str) -> PathBuf {
    if route == "/" {
        output.join("index.html")
    } else {
        output.join(route.trim_matches('/')).join("index.html")
    }
}

fn write_pages_json(bundle: &Bundle, output: &Path) -> Result<()> {
    fs::write(
        output.join("pages.json"),
        format!("{}\n", serde_json::to_string_pretty(&nav_pages(bundle))?),
    )
    .context("failed to write pages.json")
}

fn write_assets(output: &Path) -> Result<()> {
    let dir = output.join("__okmate");
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    fs::write(dir.join("app.css"), APP_CSS).context("failed to write app.css")?;
    fs::write(dir.join("datastar.js"), DATASTAR_JS).context("failed to write datastar.js")
}

fn nav_pages(bundle: &Bundle) -> Vec<NavPage> {
    let mut pages = vec![
        NavPage {
            title: "Dashboard".into(),
            route: "/".into(),
            path: "index.md".into(),
            description: String::new(),
            collection: String::new(),
        },
        NavPage {
            title: "Review queue".into(),
            route: "/review/".into(),
            path: "review".into(),
            description: String::new(),
            collection: String::new(),
        },
        NavPage {
            title: "Settings".into(),
            route: "/settings/".into(),
            path: "settings".into(),
            description: String::new(),
            collection: String::new(),
        },
    ];
    for concept in &bundle.concepts {
        let id = concept.id.trim_matches('/');
        pages.push(NavPage {
            title: okf::string_field(&concept.metadata, "title")
                .unwrap_or(&concept.id)
                .to_string(),
            route: format!("/{id}/"),
            path: concept.path.clone(),
            description: okf::string_field(&concept.metadata, "description")
                .unwrap_or("")
                .to_string(),
            collection: collection_label(&concept.path),
        });
    }
    for index in &bundle.indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        pages.push(NavPage {
            title: collection_title(index),
            route: format!("/{collection}/"),
            path: index.path.clone(),
            description: String::new(),
            collection: collection.to_string(),
        });
    }
    pages.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then(left.path.cmp(&right.path))
    });
    pages
}

fn collection_label(path: &str) -> String {
    path.split('/')
        .next()
        .filter(|segment| !segment.is_empty() && *segment != path && *segment != "review")
        .unwrap_or("")
        .to_string()
}

fn collection_title(index: &okf::Index) -> String {
    if let Some(heading) = index.headings.iter().find(|heading| heading.level == 1) {
        return heading.text.clone();
    }
    index
        .path
        .strip_suffix("/index.md")
        .and_then(|collection| collection.rsplit('/').next())
        .unwrap_or(index.path.as_str())
        .to_string()
}

fn nav_tree(bundle: &Bundle, current: &str) -> Vec<NavNode> {
    let current = normalize_route(current);
    let mut items = vec![
        leaf("/", "Dashboard", &current),
        leaf("/review/", "Review queue", &current),
        leaf("/settings/", "Settings", &current),
    ];
    items.extend(nav_forest(bundle, &current));
    items
}

fn leaf(href: &str, title: &str, current: &str) -> NavNode {
    NavNode {
        href: href.into(),
        title: title.into(),
        current: href == current,
        open: false,
        children: Vec::new(),
    }
}

fn nav_forest(bundle: &Bundle, current: &str) -> Vec<NavNode> {
    let mut by_path: BTreeMap<String, NavNode> = BTreeMap::new();
    for index in &bundle.indexes {
        let Some(path) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        let href = format!("/{path}/");
        by_path.insert(
            path.to_string(),
            NavNode {
                href: href.clone(),
                title: collection_title(index),
                current: href == current,
                open: current.starts_with(&href),
                children: Vec::new(),
            },
        );
    }
    let paths: Vec<String> = by_path.keys().cloned().collect();
    for concept in &bundle.concepts {
        if by_path.contains_key(&concept.id) {
            continue;
        }
        let Some(owner) = paths
            .iter()
            .filter(|name| concept.id == **name || concept.id.starts_with(&format!("{name}/")))
            .max_by_key(|name| name.len())
            .cloned()
        else {
            continue;
        };
        let title = okf::string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        if let Some(node) = by_path.get_mut(&owner) {
            let href = format!("/{}/", concept.id);
            node.children.push(NavNode {
                href: href.clone(),
                title: title.to_string(),
                current: href == current,
                open: false,
                children: Vec::new(),
            });
        }
    }
    for node in by_path.values_mut() {
        node.children.sort_by(|left, right| {
            left.title
                .cmp(&right.title)
                .then(left.href.cmp(&right.href))
        });
    }

    let mut children_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut roots = Vec::new();
    for path in &paths {
        let parent = paths
            .iter()
            .filter(|candidate| path.starts_with(&format!("{candidate}/")))
            .max_by_key(|candidate| candidate.len());
        if let Some(parent) = parent {
            children_of
                .entry(parent.clone())
                .or_default()
                .push(path.clone());
        } else {
            roots.push(path.clone());
        }
    }

    fn take_node(
        path: &str,
        by_path: &mut BTreeMap<String, NavNode>,
        children_of: &BTreeMap<String, Vec<String>>,
    ) -> NavNode {
        let mut node = by_path.remove(path).expect("nav node");
        if let Some(child_paths) = children_of.get(path) {
            for child in child_paths {
                node.children.push(take_node(child, by_path, children_of));
            }
        }
        node.children.sort_by(|left, right| {
            left.title
                .cmp(&right.title)
                .then(left.href.cmp(&right.href))
        });
        node
    }

    roots.sort();
    roots
        .into_iter()
        .map(|path| take_node(&path, &mut by_path, &children_of))
        .collect()
}

fn normalize_route(route: &str) -> String {
    let path = route.split(['?', '#']).next().unwrap_or(route);
    if path == "/" {
        return "/".into();
    }
    format!("/{}/", path.trim_matches('/'))
}
