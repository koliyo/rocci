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

pub fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}
