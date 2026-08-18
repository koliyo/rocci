use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::ast::{Concept, Edge, Index};
use crate::diagnostic::Diagnostic;
use crate::validate::external_url;

pub fn resolve_graph(
    concepts: &[Concept],
    indexes: &[Index],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Edge> {
    let concept_paths = concepts
        .iter()
        .map(|concept| (concept.path.as_str(), concept))
        .collect::<BTreeMap<_, _>>();
    let index_paths = indexes
        .iter()
        .map(|index| index.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for concept in concepts {
        for link in &concept.links {
            if external_url(&link.url) || link.url.starts_with('#') {
                continue;
            }
            let (path, fragment) = split_fragment(&link.url);
            let resolved = resolve_bundle_path(&concept.path, path);
            let Some(resolved) = resolved else {
                diagnostics.push(Diagnostic::error(
                    "OKF3001",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("link `{}` escapes the knowledge bundle", link.url),
                ));
                continue;
            };
            let directory_index = if resolved.ends_with('/') {
                format!("{resolved}index.md")
            } else {
                String::new()
            };
            let target = concept_paths.get(resolved.as_str()).copied();
            let valid_index = index_paths.contains(resolved.as_str())
                || (!directory_index.is_empty() && index_paths.contains(directory_index.as_str()));
            let broken = target.is_none() && !valid_index;
            if broken {
                diagnostics.push(Diagnostic::warning(
                    "OKF3002",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("broken concept link `{}`", link.url),
                ));
            } else if let (Some(target), Some(fragment)) = (target, fragment)
                && !target.headings.iter().any(|heading| heading.id == fragment)
            {
                diagnostics.push(Diagnostic::warning(
                    "OKF3004",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("unknown heading `{fragment}` in `{resolved}`"),
                ));
            }
            edges.push(Edge {
                from: concept.id.clone(),
                to: resolved
                    .strip_suffix(".md")
                    .unwrap_or(&resolved)
                    .to_string(),
                raw: link.url.clone(),
                broken,
            });
        }
    }
    edges.sort_by(|a, b| (&a.from, &a.to, &a.raw).cmp(&(&b.from, &b.to, &b.raw)));
    edges
}

pub fn resolve_bundle_path(source_path: &str, raw: &str) -> Option<String> {
    let raw = raw.replace('\\', "/");
    let directory = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let joined = if let Some(absolute) = raw.strip_prefix('/') {
        PathBuf::from(absolute)
    } else {
        directory.join(&raw)
    };
    let trailing_slash = raw.ends_with('/');
    let mut parts = Vec::new();
    for component in joined.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    let mut path = parts.join("/");
    if trailing_slash && !path.is_empty() {
        path.push('/');
    }
    Some(path)
}

pub fn published_href(source_path: &str, raw: &str) -> Option<String> {
    if external_url(raw) || raw.starts_with('#') {
        return None;
    }
    let (path, fragment) = split_fragment(raw);
    if path.is_empty() {
        return None;
    }
    let resolved = resolve_bundle_path(source_path, path)?;
    let route = published_route(&resolved)?;
    Some(match fragment {
        Some(fragment) if !fragment.is_empty() => format!("{route}#{fragment}"),
        _ => route,
    })
}

fn published_route(resolved: &str) -> Option<String> {
    if resolved.is_empty() {
        return Some("/".into());
    }
    if resolved.ends_with('/') {
        return Some(format!("/{resolved}"));
    }
    let stem = resolved.strip_suffix(".md")?;
    if stem.is_empty() || stem == "index" {
        return Some("/".into());
    }
    if let Some(collection) = stem.strip_suffix("/index") {
        return Some(format!("/{collection}/"));
    }
    Some(format!("/{stem}/"))
}

pub fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_href_maps_bundle_markdown_to_site_routes() {
        assert_eq!(
            published_href("plans/example.md", "/decisions/foo.md"),
            Some("/decisions/foo/".into())
        );
        assert_eq!(
            published_href("plans/example.md", "../decisions/foo.md"),
            Some("/decisions/foo/".into())
        );
        assert_eq!(
            published_href("index.md", "architecture/"),
            Some("/architecture/".into())
        );
        assert_eq!(
            published_href("index.md", "/architecture/index.md"),
            Some("/architecture/".into())
        );
        assert_eq!(
            published_href("plans/example.md", "/index.md"),
            Some("/".into())
        );
        assert_eq!(
            published_href("plans/example.md", "/decisions/foo.md#context"),
            Some("/decisions/foo/#context".into())
        );
    }

    #[test]
    fn published_href_leaves_non_markdown_and_escapes_unchanged() {
        assert_eq!(published_href("index.md", "https://example.com/docs"), None);
        assert_eq!(published_href("plans/example.md", "#heading"), None);
        assert_eq!(published_href("plans/example.md", "../../README.md"), None);
        assert_eq!(published_href("index.md", "migration-matrix.tsv"), None);
        assert_eq!(published_href("index.md", "/assets/diagram.png"), None);
    }
}
