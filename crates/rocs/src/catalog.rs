use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Serialize;

use crate::config::NavConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CatalogDiagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub path: String,
    pub message: String,
}

impl CatalogDiagnostic {
    pub fn error(code: &'static str, path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn warning(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl std::fmt::Display for CatalogDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{} {kind} {}: {}", self.code, self.path, self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteHint {
    Derived,
    Explicit(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePage {
    pub id: String,
    pub id_explicit: bool,
    pub source_path: String,
    pub route_hint: RouteHint,
    pub aliases: Vec<String>,
    pub draft: bool,
    pub title: String,
    pub description: String,
    pub headings: Vec<PageHeading>,
    pub outgoing_links: Vec<String>,
    pub image_urls: Vec<String>,
    pub article_html: String,
    pub docs: crate::docs::PageDocs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageHeading {
    pub level: u8,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavLink {
    pub id: String,
    pub title: String,
    pub route: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedPage {
    pub id: String,
    pub source_path: String,
    pub title: String,
    pub description: String,
    pub headings: Vec<PageHeading>,
    pub outgoing_links: Vec<String>,
    pub article_html: String,
    pub route: String,
    pub output_path: String,
    pub aliases: Vec<String>,
    pub draft: bool,
    pub unlisted: bool,
    pub breadcrumbs: Vec<NavLink>,
    pub previous: Option<NavLink>,
    pub next: Option<NavLink>,
    #[serde(skip)]
    pub article: Vec<crate::docs::ArticleNode>,
    pub examples: Vec<crate::docs::ExampleRecord>,
    pub includes: Vec<crate::docs::IncludeOrigin>,
    pub docs_kinds: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Page,
    Heading,
    Asset,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edge {
    pub from_id: String,
    pub raw: String,
    pub target: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavSection {
    pub label: String,
    pub items: Vec<NavItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavItem {
    pub id: String,
    pub title: String,
    pub route: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedSite {
    pub pages: Vec<ResolvedPage>,
    pub navigation: Vec<NavSection>,
    pub graph: Vec<Edge>,
    pub unlisted: Vec<String>,
    pub snippet_paths: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub navigation: Vec<NavConfig>,
    pub files: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    pub site: ResolvedSite,
    pub diagnostics: Vec<CatalogDiagnostic>,
}

impl ResolveResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(CatalogDiagnostic::is_error)
    }

    pub fn error_summary(&self) -> String {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn derived_route(id: &str) -> String {
    if id == "index" {
        "/".to_string()
    } else if let Some(section) = id.strip_suffix("/index") {
        format!("/{section}/")
    } else {
        format!("/{id}/")
    }
}

pub fn with_trailing_slash(route: &str) -> String {
    if route == "/" || route.ends_with('/') {
        route.to_string()
    } else {
        format!("{route}/")
    }
}

pub fn page_route(page: &SourcePage) -> String {
    with_trailing_slash(&match &page.route_hint {
        RouteHint::Explicit(route) => route.clone(),
        RouteHint::Derived => derived_route(&page.id),
    })
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
            let alias = with_trailing_slash(alias);
            if let Some(reason) = invalid_route_reason(&alias) {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2005",
                    &page.source_path,
                    format!("invalid alias `{alias}` ({reason})"),
                ));
            } else {
                aliases.push(alias);
            }
        }
        resolved.push(ResolvedPage {
            id: page.id.clone(),
            source_path: page.source_path.clone(),
            title: page.title.clone(),
            description: page.description.clone(),
            headings: page.headings.clone(),
            outgoing_links: page.outgoing_links.clone(),
            article_html: page.article_html.clone(),
            output_path: if route.starts_with('/') && !route.contains("..") {
                route_output_path(&route)
            } else {
                String::new()
            },
            route,
            aliases,
            draft: page.draft,
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

    let graph = resolve_graph(pages, &resolved, &options.files, &mut diagnostics);
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

fn resolve_graph(
    sources: &[SourcePage],
    pages: &[ResolvedPage],
    files: &BTreeSet<String>,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> Vec<Edge> {
    let by_id: BTreeMap<&str, &ResolvedPage> =
        pages.iter().map(|page| (page.id.as_str(), page)).collect();
    let mut by_route: BTreeMap<&str, &ResolvedPage> = BTreeMap::new();
    for page in pages {
        by_route.entry(page.route.as_str()).or_insert(page);
        for alias in &page.aliases {
            by_route.entry(alias.as_str()).or_insert(page);
        }
    }
    let mut graph = Vec::new();
    for (source, page) in sources.iter().zip(pages.iter()) {
        for raw in source.outgoing_links.iter().chain(source.image_urls.iter()) {
            let is_image = source.image_urls.iter().any(|url| url == raw)
                && !source.outgoing_links.iter().any(|url| url == raw);
            match resolve_ref(raw, page, pages, &by_id, &by_route, files, is_image) {
                Ok(Some(edge)) => {
                    if edge.kind == EdgeKind::Page
                        && !page.draft
                        && let Some(target) = by_id.get(edge.target.as_str())
                        && target.draft
                    {
                        diagnostics.push(CatalogDiagnostic::error(
                            "RD2104",
                            &page.source_path,
                            format!("published page links to draft `{raw}`"),
                        ));
                    }
                    graph.push(edge);
                }
                Ok(None) => {}
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
    }
    graph.sort_by(|a, b| {
        a.from_id
            .cmp(&b.from_id)
            .then(a.raw.cmp(&b.raw))
            .then(a.target.cmp(&b.target))
    });
    graph
}

fn resolve_ref(
    raw: &str,
    page: &ResolvedPage,
    pages: &[ResolvedPage],
    by_id: &BTreeMap<&str, &ResolvedPage>,
    by_route: &BTreeMap<&str, &ResolvedPage>,
    files: &BTreeSet<String>,
    is_image: bool,
) -> Result<Option<Edge>, CatalogDiagnostic> {
    if raw.is_empty() {
        return Ok(None);
    }
    if has_scheme(raw) {
        return Ok(Some(edge(page, raw, raw, EdgeKind::External)));
    }
    let (path, fragment) = split_fragment(raw);
    if path.is_empty() {
        let Some(fragment) = fragment else {
            return Ok(None);
        };
        return heading_edge(page, raw, page, fragment);
    }
    if path.starts_with("/assets/") || (is_image && path.starts_with('/') && looks_like_asset(path))
    {
        return asset_edge(page, raw, path.strip_prefix('/').unwrap_or(path), files);
    }
    if path.starts_with('/') {
        let route = with_trailing_slash(path);
        let Some(target) = by_route.get(route.as_str()) else {
            return Err(CatalogDiagnostic::error(
                "RD2101",
                &page.source_path,
                format!("broken internal link `{raw}`"),
            ));
        };
        return page_or_heading_edge(page, raw, target, fragment);
    }
    if is_relative(path) {
        let Some(normalized) = resolve_relative(&page.source_path, path) else {
            return Err(CatalogDiagnostic::error(
                "RD2106",
                &page.source_path,
                format!("relative link `{raw}` escapes the content root"),
            ));
        };
        if let Some(target) = page_for_path(pages, &normalized) {
            return page_or_heading_edge(page, raw, target, fragment);
        }
        if files.contains(&normalized) {
            return Ok(Some(edge(
                page,
                raw,
                &format!("/{normalized}"),
                EdgeKind::Asset,
            )));
        }
        if looks_like_asset(&normalized) {
            return Err(CatalogDiagnostic::error(
                "RD2103",
                &page.source_path,
                format!("missing asset `{raw}`"),
            ));
        }
        return Err(CatalogDiagnostic::error(
            "RD2101",
            &page.source_path,
            format!("broken internal link `{raw}`"),
        ));
    }
    if is_image {
        return asset_edge(page, raw, path, files);
    }
    match wiki_target(path, pages, by_id) {
        WikiMatch::One(target) => page_or_heading_edge(page, raw, target, fragment),
        WikiMatch::None => Err(CatalogDiagnostic::error(
            "RD2101",
            &page.source_path,
            format!("broken internal link `{raw}`"),
        )),
        WikiMatch::Ambiguous(paths) => Err(CatalogDiagnostic::error(
            "RD2105",
            &page.source_path,
            format!("ambiguous wiki link `{raw}` matches {paths}"),
        )),
    }
}

enum WikiMatch<'a> {
    None,
    One(&'a ResolvedPage),
    Ambiguous(String),
}

fn wiki_target<'a>(
    name: &str,
    pages: &'a [ResolvedPage],
    by_id: &BTreeMap<&str, &'a ResolvedPage>,
) -> WikiMatch<'a> {
    let stem = name.strip_suffix(".rocdown").unwrap_or(name);
    if let Some(page) = by_id.get(stem) {
        return WikiMatch::One(page);
    }
    let mut matches = Vec::new();
    for page in pages {
        let file_stem = Path::new(&page.source_path)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let id_stem = page.id.rsplit('/').next().unwrap_or(&page.id);
        if page.id == stem
            || id_stem == stem
            || file_stem == stem
            || page.title == stem
            || page.title == name
        {
            matches.push(page);
        }
    }
    matches.sort_by(|a, b| a.id.cmp(&b.id));
    matches.dedup_by(|a, b| a.id == b.id);
    match matches.as_slice() {
        [] => WikiMatch::None,
        [page] => WikiMatch::One(page),
        many => WikiMatch::Ambiguous(
            many.iter()
                .map(|page| page.source_path.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ),
    }
}

fn page_for_path<'a>(pages: &'a [ResolvedPage], normalized: &str) -> Option<&'a ResolvedPage> {
    let id = normalized.strip_suffix(".rocdown").unwrap_or(normalized);
    pages.iter().find(|page| {
        page.source_path == normalized
            || page.source_path == format!("{id}.rocdown")
            || page.id == id
    })
}

fn page_or_heading_edge(
    from: &ResolvedPage,
    raw: &str,
    target: &ResolvedPage,
    fragment: Option<&str>,
) -> Result<Option<Edge>, CatalogDiagnostic> {
    match fragment {
        Some(fragment) => heading_edge(from, raw, target, fragment),
        None => Ok(Some(edge(from, raw, &target.id, EdgeKind::Page))),
    }
}

fn heading_edge(
    from: &ResolvedPage,
    raw: &str,
    target: &ResolvedPage,
    fragment: &str,
) -> Result<Option<Edge>, CatalogDiagnostic> {
    if target.headings.iter().any(|heading| heading.id == fragment) {
        Ok(Some(edge(
            from,
            raw,
            &format!("{}#{fragment}", target.id),
            EdgeKind::Heading,
        )))
    } else {
        Err(CatalogDiagnostic::error(
            "RD2102",
            &from.source_path,
            format!("broken heading link `{raw}`"),
        ))
    }
}

fn asset_edge(
    from: &ResolvedPage,
    raw: &str,
    path: &str,
    files: &BTreeSet<String>,
) -> Result<Option<Edge>, CatalogDiagnostic> {
    if files.contains(path) {
        Ok(Some(edge(from, raw, &format!("/{path}"), EdgeKind::Asset)))
    } else {
        Err(CatalogDiagnostic::error(
            "RD2103",
            &from.source_path,
            format!("missing asset `{raw}`"),
        ))
    }
}

fn edge(from: &ResolvedPage, raw: &str, target: &str, kind: EdgeKind) -> Edge {
    Edge {
        from_id: from.id.clone(),
        raw: raw.to_string(),
        target: target.to_string(),
        kind,
    }
}

fn is_relative(path: &str) -> bool {
    path.starts_with("./")
        || path.starts_with("../")
        || path.contains('/')
        || Path::new(path).extension().is_some()
}

fn looks_like_asset(path: &str) -> bool {
    matches!(
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "pdf" | "css" | "js")
    )
}

fn resolve_relative(source_path: &str, rel: &str) -> Option<String> {
    let base = source_path
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("");
    let mut parts = Vec::new();
    if !base.is_empty() {
        parts.extend(base.split('/').filter(|seg| !seg.is_empty()));
    }
    let rel = rel
        .strip_prefix("./")
        .or_else(|| rel.strip_prefix(".\\"))
        .unwrap_or(rel);
    for seg in rel.split(['/', '\\']) {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

fn has_scheme(path: &str) -> bool {
    let Some((scheme, _)) = path.split_once(':') else {
        return false;
    };
    !scheme.is_empty() && scheme.chars().all(|ch| ch.is_ascii_alphabetic())
}

fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some(("", fragment)) => ("", Some(fragment)),
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}

fn resolve_navigation(
    pages: &[ResolvedPage],
    configured: &[NavConfig],
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> Vec<NavSection> {
    let by_id: BTreeMap<&str, &ResolvedPage> =
        pages.iter().map(|page| (page.id.as_str(), page)).collect();
    let mut seen = BTreeMap::<&str, &str>::new();
    let mut navigation = Vec::new();

    for section in configured {
        let ids = if section.items.is_empty() {
            directory_ids(pages, section.directory.as_deref().unwrap_or_default())
        } else {
            section.items.clone()
        };
        if ids.is_empty() {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2204",
                "rocs.toml",
                format!("navigation section `{}` has no pages", section.label),
            ));
            continue;
        }
        let mut items = Vec::new();
        for id in ids {
            let Some(page) = by_id.get(id.as_str()) else {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2201",
                    "rocs.toml",
                    format!(
                        "navigation section `{}` references unknown page id `{id}`",
                        section.label
                    ),
                ));
                continue;
            };
            if let Some(previous) = seen.insert(page.id.as_str(), section.label.as_str()) {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2203",
                    &page.source_path,
                    format!(
                        "navigation page `{id}` appears in both `{previous}` and `{}`",
                        section.label
                    ),
                ));
            }
            items.push(NavItem {
                id: page.id.clone(),
                title: page.title.clone(),
                route: page.route.clone(),
            });
        }
        if !items.is_empty() {
            navigation.push(NavSection {
                label: section.label.clone(),
                items,
            });
        }
    }

    if navigation.is_empty() && configured.is_empty() {
        navigation.push(NavSection {
            label: "Documentation".into(),
            items: pages
                .iter()
                .filter(|page| !page.draft)
                .map(|page| NavItem {
                    id: page.id.clone(),
                    title: page.title.clone(),
                    route: page.route.clone(),
                })
                .collect(),
        });
    }

    navigation
}

fn directory_ids(pages: &[ResolvedPage], directory: &str) -> Vec<String> {
    let prefix = directory.trim_matches('/');
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut matched: Vec<&ResolvedPage> = pages
        .iter()
        .filter(|page| {
            !page.draft && (page.id == prefix || page.id.starts_with(&format!("{prefix}/")))
        })
        .collect();
    matched.sort_by(|a, b| {
        let a_index = a.id == prefix || a.id.ends_with("/index");
        let b_index = b.id == prefix || b.id.ends_with("/index");
        match (a_index, b_index) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.id.cmp(&b.id),
        }
    });
    matched.into_iter().map(|page| page.id.clone()).collect()
}

fn apply_journey(
    pages: &mut [ResolvedPage],
    navigation: &[NavSection],
    diagnostics: &mut Vec<CatalogDiagnostic>,
) {
    let listed: Vec<NavLink> = navigation
        .iter()
        .flat_map(|section| {
            section.items.iter().map(|item| NavLink {
                id: item.id.clone(),
                title: item.title.clone(),
                route: item.route.clone(),
            })
        })
        .collect();
    let listed_ids: BTreeSet<&str> = listed.iter().map(|item| item.id.as_str()).collect();
    let home = pages
        .iter()
        .find(|page| page.route == "/")
        .map(|page| NavLink {
            id: page.id.clone(),
            title: page.title.clone(),
            route: page.route.clone(),
        });
    let section_for: BTreeMap<&str, &str> = navigation
        .iter()
        .flat_map(|section| {
            section
                .items
                .iter()
                .map(move |item| (item.id.as_str(), section.label.as_str()))
        })
        .collect();

    for page in pages.iter_mut() {
        page.unlisted = !page.draft && page.route != "/" && !listed_ids.contains(page.id.as_str());
        if page.unlisted {
            diagnostics.push(CatalogDiagnostic::warning(
                "RD2202",
                &page.source_path,
                format!("page `{}` is unlisted", page.id),
            ));
        }
        let mut crumbs = Vec::new();
        if let Some(home) = &home {
            crumbs.push(home.clone());
        }
        if let Some(label) = section_for.get(page.id.as_str())
            && home.as_ref().is_none_or(|home| home.id != page.id)
        {
            let section_href = navigation
                .iter()
                .find(|section| section.label == *label)
                .and_then(|section| section.items.first())
                .map(|item| item.route.clone())
                .unwrap_or_default();
            crumbs.push(NavLink {
                id: String::new(),
                title: (*label).to_string(),
                route: section_href,
            });
        }
        if home.as_ref().is_none_or(|home| home.id != page.id) {
            crumbs.push(NavLink {
                id: page.id.clone(),
                title: page.title.clone(),
                route: page.route.clone(),
            });
        }
        page.breadcrumbs = crumbs;

        if let Some(index) = listed.iter().position(|item| item.id == page.id) {
            if index > 0 {
                page.previous = Some(listed[index - 1].clone());
            }
            if index + 1 < listed.len() {
                page.next = Some(listed[index + 1].clone());
            }
        }
    }
}

pub fn format_diagnostics(diagnostics: &[CatalogDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(id: &str, path: &str, hint: RouteHint, title: &str) -> SourcePage {
        SourcePage {
            id: id.to_string(),
            id_explicit: false,
            source_path: path.to_string(),
            route_hint: hint,
            aliases: Vec::new(),
            draft: false,
            title: title.to_string(),
            description: String::new(),
            headings: Vec::new(),
            outgoing_links: Vec::new(),
            image_urls: Vec::new(),
            article_html: String::new(),
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
}
