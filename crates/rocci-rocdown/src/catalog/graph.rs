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
                raw,
                page,
                &RefIndexes {
                    pages,
                    peer_pages,
                    by_route: &by_route,
                    files,
                },
                is_image,
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

struct RefIndexes<'a> {
    pages: &'a [ResolvedPage],
    peer_pages: &'a [PageRef],
    by_route: &'a BTreeMap<&'a str, &'a ResolvedPage>,
    files: &'a BTreeSet<String>,
}

fn resolve_ref(
    raw: &str,
    page: &ResolvedPage,
    indexes: &RefIndexes<'_>,
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
        return asset_edge(
            page,
            raw,
            path.strip_prefix('/').unwrap_or(path),
            indexes.files,
        );
    }
    if super::resolve::is_site_service_href(path) {
        return Ok(Some(edge(page, raw, raw, EdgeKind::Asset)));
    }
    if path.starts_with('/') {
        if let Some(target) = page_for_abs_route(indexes.by_route, path) {
            return page_or_heading_edge(page, raw, target, fragment);
        }
        if let Some(peer) = indexes
            .peer_pages
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
        if let Some(target) = page_for_path(indexes.pages, &normalized) {
            return page_or_heading_edge(page, raw, target, fragment);
        }
        if indexes.files.contains(&normalized) {
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
        return asset_edge(page, raw, path, indexes.files);
    }
    match crate::links::match_wiki(path, indexes.pages, resolved_wiki_identity) {
        crate::links::WikiMatch::One(target) => page_or_heading_edge(page, raw, target, fragment),
        crate::links::WikiMatch::None => Err(CatalogDiagnostic::error(
            "RD2101",
            &page.source_path,
            format!("broken internal link `{raw}`"),
        )),
        crate::links::WikiMatch::Ambiguous(matches) => Err(CatalogDiagnostic::error(
            "RD2105",
            &page.source_path,
            format!(
                "ambiguous wiki link `{raw}` matches {}",
                matches
                    .iter()
                    .map(|page| page.source_path.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )),
    }
}

fn resolved_wiki_identity(page: &ResolvedPage) -> crate::links::WikiIdentity<'_> {
    crate::links::WikiIdentity {
        id: page.id.as_str(),
        file_stem: Path::new(&page.source_path)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default(),
        title: page.title.as_str(),
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
