use std::collections::BTreeMap;

use super::graph::resolve_graph;
use super::nav::{apply_journey, resolve_navigation};
use super::types::*;

pub fn is_collection_id(id: &str) -> bool {
    id == "index" || id.ends_with("/index")
}

pub fn derived_route(id: &str) -> String {
    if id == "index" {
        "/".to_string()
    } else if let Some(section) = id.strip_suffix("/index") {
        format!("/{section}/")
    } else {
        format!("/{id}")
    }
}

pub fn with_trailing_slash(route: &str) -> String {
    if route == "/" || route.ends_with('/') {
        route.to_string()
    } else {
        format!("{route}/")
    }
}

pub fn without_trailing_slash(route: &str) -> String {
    if route == "/" {
        return "/".to_string();
    }
    route.strip_suffix('/').unwrap_or(route).to_string()
}

pub fn routes_match(left: &str, right: &str) -> bool {
    let left = with_trailing_slash(left);
    let right = with_trailing_slash(right);
    if left == right {
        return true;
    }
    if let Some(stripped) = left.strip_prefix("/docs") {
        return stripped == right || (left == "/docs/" && right == "/");
    }
    if let Some(stripped) = right.strip_prefix("/docs") {
        return stripped == left || (right == "/docs/" && left == "/");
    }
    false
}

pub fn canonical_route(route: &str, collection: bool) -> String {
    if !route.starts_with('/') {
        return route.to_string();
    }
    if route == "/" {
        return "/".to_string();
    }
    if collection {
        with_trailing_slash(route)
    } else {
        without_trailing_slash(route)
    }
}

pub fn page_route(page: &SourcePage) -> String {
    let raw = match &page.route_hint {
        RouteHint::Explicit(route) => route.clone(),
        RouteHint::Derived => derived_route(&page.id),
    };
    canonical_route(&raw, is_collection_id(&page.id))
}

pub fn route_output_path(route: &str) -> String {
    if route == "/" {
        "index.html".to_string()
    } else {
        let trimmed = route.strip_prefix('/').unwrap_or(route);
        let without_slash = trimmed.strip_suffix('/').unwrap_or(trimmed);
        format!("{without_slash}/index.html")
    }
}

pub fn resolve(pages: &[SourcePage], options: &ResolveOptions) -> ResolveResult {
    let mut diagnostics = Vec::new();
    report_duplicate_ids(pages, &mut diagnostics);

    let mut resolved = Vec::with_capacity(pages.len());
    for page in pages {
        let route = page_route(page);
        if let Some(reason) = invalid_route_reason(&route) {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2004",
                &page.source_path,
                format!("invalid route `{route}` ({reason})"),
            ));
        }
        let mut aliases = Vec::new();
        for alias in &page.aliases {
            let alias = canonical_route(alias, is_collection_id(&page.id));
            if alias == route {
                continue;
            }
            if let Some(reason) = invalid_route_reason(&alias) {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2005",
                    &page.source_path,
                    format!("invalid alias `{alias}` ({reason})"),
                ));
            } else if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }
        resolved.push(ResolvedPage {
            id: page.id.clone(),
            source_path: page.source_path.clone(),
            kind: page.kind,
            title: page.title.clone(),
            description: page.description.clone(),
            layout: page.layout.clone(),
            published: page.published.clone(),
            updated: page.updated.clone(),
            authors: page.authors.clone(),
            tags: page.tags.clone(),
            collection: page.collection.clone(),
            headings: page.headings.clone(),
            outgoing_links: page.outgoing_links.clone(),
            article_html: page.article_html.clone(),
            island_css: page.island_css.clone(),
            island_html: Vec::new(),
            output_path: if route.starts_with('/') && !route.contains("..") {
                route_output_path(&route)
            } else {
                String::new()
            },
            route,
            aliases,
            draft: page.draft,
            suppress_unlisted_warning: page.suppress_unlisted_warning,
            unlisted: false,
            breadcrumbs: Vec::new(),
            previous: None,
            next: None,
            article: page.docs.article.clone(),
            examples: page.docs.examples.clone(),
            includes: page.docs.includes.clone(),
            docs_kinds: crate::docs::collect_kinds(&page.docs.article),
        });
    }

    report_route_collisions(&resolved, &mut diagnostics);

    crate::docs::fill_link_cards(&mut resolved);

    let graph = resolve_graph(
        pages,
        &resolved,
        &options.peer_pages,
        &options.files,
        &mut diagnostics,
    );
    crate::docs::rewrite_resolved_links(&mut resolved, &graph);
    let navigation = resolve_navigation(&resolved, &options.navigation, &mut diagnostics);
    apply_journey(&mut resolved, &navigation, &mut diagnostics);

    let unlisted = resolved
        .iter()
        .filter(|page| page.unlisted && !page.draft)
        .map(|page| page.id.clone())
        .collect();

    let snippet_paths = pages
        .iter()
        .flat_map(|page| page.docs.snippet_paths.iter().cloned())
        .collect();

    resolved.sort_by(|a, b| a.output_path.cmp(&b.output_path).then(a.id.cmp(&b.id)));

    ResolveResult {
        site: ResolvedSite {
            pages: resolved,
            navigation,
            graph,
            unlisted,
            snippet_paths,
        },
        diagnostics,
    }
}

fn invalid_route_reason(route: &str) -> Option<&'static str> {
    if route.is_empty() {
        Some("empty")
    } else if !route.starts_with('/') {
        Some("not absolute")
    } else if route.contains("..") {
        Some("..")
    } else {
        None
    }
}

fn report_duplicate_ids(pages: &[SourcePage], diagnostics: &mut Vec<CatalogDiagnostic>) {
    let mut by_id: BTreeMap<&str, Vec<&SourcePage>> = BTreeMap::new();
    for page in pages {
        by_id.entry(&page.id).or_default().push(page);
    }
    for (id, group) in by_id {
        if group.len() > 1 {
            let paths = group
                .iter()
                .map(|page| page.source_path.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            for page in group {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2001",
                    &page.source_path,
                    format!("duplicate page id `{id}` used by {paths}"),
                ));
            }
        }
    }
}

fn report_route_collisions(pages: &[ResolvedPage], diagnostics: &mut Vec<CatalogDiagnostic>) {
    let mut owners: BTreeMap<String, Vec<(String, &'static str)>> = BTreeMap::new();
    for page in pages {
        if invalid_route_reason(&page.route).is_none() {
            owners
                .entry(page.route.clone())
                .or_default()
                .push((page.source_path.clone(), "route"));
        }
        for alias in &page.aliases {
            if invalid_route_reason(alias).is_none() {
                owners
                    .entry(alias.clone())
                    .or_default()
                    .push((page.source_path.clone(), "alias"));
            }
        }
    }

    let mut seen_case: BTreeMap<String, String> = BTreeMap::new();
    for (route, group) in &owners {
        let key = route.to_ascii_lowercase();
        if let Some(previous) = seen_case.get(&key)
            && previous != route
        {
            for (path, _) in group {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2006",
                    path,
                    format!("route `{route}` collides with `{previous}` under case-insensitive comparison"),
                ));
            }
        } else {
            seen_case.insert(key, route.clone());
        }
        if group.len() < 2 {
            continue;
        }
        let paths = group
            .iter()
            .map(|(path, role)| format!("{path} ({role})"))
            .collect::<Vec<_>>()
            .join(", ");
        let has_alias = group.iter().any(|(_, role)| *role == "alias");
        let code = if has_alias { "RD2003" } else { "RD2002" };
        let kind = if group.iter().all(|(_, role)| *role == "alias") {
            "alias"
        } else if has_alias {
            "route/alias"
        } else {
            "route"
        };
        for (path, _) in group {
            diagnostics.push(CatalogDiagnostic::error(
                code,
                path,
                format!("duplicate {kind} `{route}` used by {paths}"),
            ));
        }
    }
}
