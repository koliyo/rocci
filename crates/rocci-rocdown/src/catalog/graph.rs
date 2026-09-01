use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::resolve::{routes_match, with_trailing_slash, without_trailing_slash};
use super::types::*;
use crate::PageRef;

pub(crate) fn resolve_graph(
    sources: &[SourcePage],
    pages: &[ResolvedPage],
    peer_pages: &[PageRef],
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
            match resolve_ref(
                raw, page, pages, peer_pages, &by_id, &by_route, files, is_image,
            ) {
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
    peer_pages: &[PageRef],
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
    if path == "/sitemap.xml"
        || path == "/robots.txt"
        || path == "/llms.txt"
        || path == "/pages.json"
        || path == "/islands.json"
        || path == "/404.html"
        || path.ends_with("/feed.xml")
        || path == "/feed.xml"
    {
        return Ok(Some(edge(page, raw, raw, EdgeKind::Asset)));
    }
    if path.starts_with('/') {
        if let Some(target) = page_for_abs_route(by_route, path) {
            return page_or_heading_edge(page, raw, target, fragment);
        }
        if let Some(peer) = peer_pages
            .iter()
            .find(|peer| routes_match(&peer.route, path))
        {
            return peer_page_or_heading_edge(page, raw, peer, fragment);
        }
        return Err(CatalogDiagnostic::error(
            "RD2101",
            &page.source_path,
            format!("broken internal link `{raw}`"),
        ));
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

fn page_for_abs_route<'a>(
    by_route: &BTreeMap<&str, &'a ResolvedPage>,
    path: &str,
) -> Option<&'a ResolvedPage> {
    lookup_route(by_route, path).or_else(|| {
        let slashed = with_trailing_slash(path);
        if let Some(stripped) = slashed.strip_prefix("/docs/") {
            lookup_route(by_route, &format!("/{stripped}"))
        } else if slashed == "/docs/" {
            by_route.get("/").copied()
        } else {
            lookup_route(by_route, &format!("/docs{slashed}")).or_else(|| {
                lookup_route(by_route, &format!("/docs{}", without_trailing_slash(path)))
            })
        }
    })
}

fn lookup_route<'a>(
    by_route: &BTreeMap<&str, &'a ResolvedPage>,
    path: &str,
) -> Option<&'a ResolvedPage> {
    let slashed = with_trailing_slash(path);
    let bare = without_trailing_slash(path);
    by_route
        .get(path)
        .or_else(|| by_route.get(slashed.as_str()))
        .or_else(|| by_route.get(bare.as_str()))
        .copied()
}

fn page_for_path<'a>(pages: &'a [ResolvedPage], normalized: &str) -> Option<&'a ResolvedPage> {
    let id = normalized.strip_suffix(".rocdown").unwrap_or(normalized);
    pages.iter().find(|page| {
        if (id.is_empty() || id == ".") && (page.id == "index" || page.route == "/") {
            return true;
        }
        page.source_path == normalized
            || page.source_path == format!("{id}.rocdown")
            || page.id == id
            || page.id.strip_prefix("docs/").is_some_and(|p| p == id)
            || page
                .source_path
                .strip_prefix("docs/")
                .is_some_and(|p| p == normalized)
    })
}

fn peer_page_or_heading_edge(
    from: &ResolvedPage,
    raw: &str,
    peer: &PageRef,
    fragment: Option<&str>,
) -> Result<Option<Edge>, CatalogDiagnostic> {
    match fragment {
        Some(fragment) => {
            if peer.heading_ids.iter().any(|id| id == fragment) {
                Ok(Some(edge(
                    from,
                    raw,
                    &format!("{}#{fragment}", peer.route),
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
        None => Ok(Some(edge(from, raw, &peer.route, EdgeKind::Page))),
    }
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
    if target.headings.iter().any(|heading| heading.id == fragment)
        || has_source_line_anchor(target, fragment)
    {
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

fn has_source_line_anchor(target: &ResolvedPage, fragment: &str) -> bool {
    is_source_line_anchor_id(fragment)
        && target.article_html.contains(&format!("id=\"{fragment}\""))
}

fn is_source_line_anchor_id(fragment: &str) -> bool {
    crate::links::is_source_line_anchor_id(fragment)
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
