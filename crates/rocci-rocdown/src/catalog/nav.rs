use std::collections::{BTreeMap, BTreeSet};

use crate::config::NavConfig;

use super::types::*;

pub(crate) fn resolve_navigation(
    pages: &[ResolvedPage],
    configured: &[NavConfig],
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> Vec<NavSection> {
    let by_id: BTreeMap<&str, &ResolvedPage> =
        pages.iter().map(|page| (page.id.as_str(), page)).collect();
    let mut seen = BTreeMap::<String, String>::new();
    let mut navigation = Vec::new();

    for section in configured {
        if let Some(resolved) = resolve_nav_section(section, pages, &by_id, &mut seen, diagnostics)
        {
            navigation.push(resolved);
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
            children: Vec::new(),
        });
    }

    navigation
}

fn resolve_nav_section(
    section: &NavConfig,
    pages: &[ResolvedPage],
    by_id: &BTreeMap<&str, &ResolvedPage>,
    seen: &mut BTreeMap<String, String>,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> Option<NavSection> {
    let ids = if section.items.is_empty() && section.groups.is_empty() {
        directory_ids(pages, section.directory.as_deref().unwrap_or_default())
    } else {
        section.items.clone()
    };
    let mut items = Vec::new();
    for id in ids {
        let Some(page) = by_id.get(id.as_str()) else {
            if section.directory.as_deref() == Some("examples") {
                continue;
            }
            diagnostics.push(CatalogDiagnostic::error(
                "RD2201",
                "rocdown.toml",
                format!(
                    "navigation section `{}` references unknown page id `{id}`",
                    section.label
                ),
            ));
            continue;
        };
        if let Some(previous) = seen.insert(page.id.clone(), section.label.clone()) {
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
    let mut children = Vec::new();
    for group in &section.groups {
        if let Some(child) = resolve_nav_section(group, pages, by_id, seen, diagnostics) {
            children.push(child);
        }
    }
    if items.is_empty() && children.is_empty() {
        diagnostics.push(CatalogDiagnostic::error(
            "RD2204",
            "rocdown.toml",
            format!("navigation section `{}` has no pages", section.label),
        ));
        return None;
    }
    Some(NavSection {
        label: section.label.clone(),
        items,
        children,
    })
}

fn nav_section_links(section: &NavSection) -> Vec<NavLink> {
    let mut links: Vec<NavLink> = section
        .items
        .iter()
        .map(|item| NavLink {
            id: item.id.clone(),
            title: item.title.clone(),
            route: item.route.clone(),
        })
        .collect();
    for child in &section.children {
        links.extend(nav_section_links(child));
    }
    links
}

fn collect_section_crumbs<'a>(
    section: &'a NavSection,
    map: &mut BTreeMap<&'a str, (&'a str, String)>,
) {
    for child in &section.children {
        collect_section_crumbs(child, map);
    }
    let href = first_nav_item(section)
        .map(|item| item.route.clone())
        .unwrap_or_default();
    for item in &section.items {
        map.entry(item.id.as_str())
            .or_insert((section.label.as_str(), href.clone()));
    }
}

pub(crate) fn first_nav_item(section: &NavSection) -> Option<&NavItem> {
    section
        .items
        .first()
        .or_else(|| section.children.iter().find_map(first_nav_item))
}

pub(crate) fn section_contains(section: &NavSection, id: &str) -> bool {
    section.items.iter().any(|item| item.id == id)
        || section
            .children
            .iter()
            .any(|child| section_contains(child, id))
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

fn warn_indexless_clusters(
    pages: &[ResolvedPage],
    listed: &[NavLink],
    diagnostics: &mut Vec<CatalogDiagnostic>,
) {
    let draft: BTreeSet<&str> = pages
        .iter()
        .filter(|page| page.draft)
        .map(|page| page.id.as_str())
        .collect();
    let mut buckets: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for link in listed {
        if draft.contains(link.id.as_str()) {
            continue;
        }
        let Some((dir, _)) = link.id.rsplit_once('/') else {
            continue;
        };
        buckets.entry(dir).or_default().push(link.id.as_str());
    }
    for (dir, ids) in buckets {
        let index_id = format!("{dir}/index");
        if ids.contains(&index_id.as_str()) || ids.len() < 2 {
            continue;
        }
        diagnostics.push(CatalogDiagnostic::warning(
            "RD2205",
            "rocdown.toml",
            format!(
                "navigation directory `{dir}` lists {} without a listed `{index_id}`; add an index to create a section",
                ids.join(", ")
            ),
        ));
    }
}

pub(crate) fn apply_journey(
    pages: &mut [ResolvedPage],
    navigation: &[NavSection],
    diagnostics: &mut Vec<CatalogDiagnostic>,
) {
    let listed: Vec<NavLink> = navigation.iter().flat_map(nav_section_links).collect();
    warn_indexless_clusters(pages, &listed, diagnostics);
    let listed_ids: BTreeSet<&str> = listed.iter().map(|item| item.id.as_str()).collect();
    let home = pages
        .iter()
        .find(|page| page.route == "/")
        .map(|page| NavLink {
            id: page.id.clone(),
            title: page.title.clone(),
            route: page.route.clone(),
        });
    let mut section_for = BTreeMap::<&str, (&str, String)>::new();
    for section in navigation {
        collect_section_crumbs(section, &mut section_for);
    }

    for page in pages.iter_mut() {
        page.unlisted = !page.draft && page.route != "/" && !listed_ids.contains(page.id.as_str());
        if page.unlisted && !page.suppress_unlisted_warning {
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
        if let Some((label, section_href)) = section_for.get(page.id.as_str())
            && home.as_ref().is_none_or(|home| home.id != page.id)
        {
            crumbs.push(NavLink {
                id: String::new(),
                title: (*label).to_string(),
                route: section_href.clone(),
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
