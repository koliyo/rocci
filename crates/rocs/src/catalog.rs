use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteHint {
    Derived,
    Explicit(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePage {
    pub id: String,
    pub source_path: String,
    pub route_hint: RouteHint,
    pub title: String,
    pub description: String,
    pub headings: Vec<PageHeading>,
    pub outgoing_links: Vec<String>,
    pub article_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHeading {
    pub level: u8,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateRoute {
    pub route: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    InvalidRoute {
        reason: &'static str,
        path: String,
        route: String,
    },
    DuplicateRoutes(Vec<DuplicateRoute>),
    BrokenLinks(Vec<String>),
    InvalidNavigation(String),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoute {
                reason,
                path,
                route,
            } => write!(f, "invalid route `{route}` ({reason}) in {path}"),
            Self::DuplicateRoutes(dups) => {
                let mut first = true;
                for dup in dups {
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;
                    write!(
                        f,
                        "duplicate route `{}` used by {}",
                        dup.route,
                        dup.paths.join(", ")
                    )?;
                }
                Ok(())
            }
            Self::InvalidNavigation(message) => f.write_str(message),
            Self::BrokenLinks(messages) => f.write_str(&messages.join("\n")),
        }
    }
}

impl std::error::Error for CatalogError {}

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

pub fn validate(pages: &[SourcePage]) -> Result<Vec<ResolvedPage>, CatalogError> {
    let mut resolved = Vec::with_capacity(pages.len());
    for page in pages {
        let route = page_route(page);
        let reason = if route.is_empty() {
            Some("empty")
        } else if !route.starts_with('/') {
            Some("not absolute")
        } else if route.contains("..") {
            Some("..")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(CatalogError::InvalidRoute {
                reason,
                path: page.source_path.clone(),
                route,
            });
        }
        resolved.push(ResolvedPage {
            id: page.id.clone(),
            source_path: page.source_path.clone(),
            title: page.title.clone(),
            description: page.description.clone(),
            headings: page.headings.clone(),
            outgoing_links: page.outgoing_links.clone(),
            article_html: page.article_html.clone(),
            output_path: route_output_path(&route),
            route,
        });
    }

    let mut by_route: BTreeMap<&str, Vec<&ResolvedPage>> = BTreeMap::new();
    for page in &resolved {
        by_route.entry(&page.route).or_default().push(page);
    }
    let mut duplicates = Vec::new();
    for (route, group) in by_route {
        if group.len() > 1 {
            duplicates.push(DuplicateRoute {
                route: route.to_string(),
                paths: group.iter().map(|page| page.source_path.clone()).collect(),
            });
        }
    }
    if !duplicates.is_empty() {
        return Err(CatalogError::DuplicateRoutes(duplicates));
    }

    validate_links(&resolved)?;

    resolved.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    Ok(resolved)
}

fn validate_links(pages: &[ResolvedPage]) -> Result<(), CatalogError> {
    let by_route: BTreeMap<&str, &ResolvedPage> = pages
        .iter()
        .map(|page| (page.route.as_str(), page))
        .collect();
    let mut broken = Vec::new();
    for page in pages {
        for link in &page.outgoing_links {
            if link.is_empty()
                || link.starts_with("http://")
                || link.starts_with("https://")
                || link.starts_with("mailto:")
                || link.starts_with("tel:")
                || link.starts_with("/assets/")
            {
                continue;
            }
            if let Some(fragment) = link.strip_prefix('#') {
                if !page.headings.iter().any(|heading| heading.id == fragment) {
                    broken.push(format!(
                        "broken heading link `{link}` in {}",
                        page.source_path
                    ));
                }
                continue;
            }
            if !link.starts_with('/') {
                continue;
            }
            let (path, fragment) = link
                .split_once('#')
                .map_or((link.as_str(), None), |(path, fragment)| {
                    (path, Some(fragment))
                });
            let route = with_trailing_slash(path);
            let Some(target) = by_route.get(route.as_str()) else {
                broken.push(format!(
                    "broken internal link `{link}` in {}",
                    page.source_path
                ));
                continue;
            };
            if let Some(fragment) = fragment
                && !target.headings.iter().any(|heading| heading.id == fragment)
            {
                broken.push(format!(
                    "broken heading link `{link}` in {}",
                    page.source_path
                ));
            }
        }
    }
    if broken.is_empty() {
        Ok(())
    } else {
        Err(CatalogError::BrokenLinks(broken))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavSection {
    pub label: String,
    pub items: Vec<NavItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavItem {
    pub id: String,
    pub title: String,
    pub route: String,
}

pub fn resolve_navigation(
    pages: &[ResolvedPage],
    configured: &[crate::config::NavConfig],
) -> Result<Vec<NavSection>, CatalogError> {
    let by_id: BTreeMap<&str, &ResolvedPage> =
        pages.iter().map(|page| (page.id.as_str(), page)).collect();
    let mut seen = BTreeMap::<&str, &str>::new();
    let mut navigation = Vec::new();

    for section in configured {
        let mut items = Vec::new();
        for id in &section.items {
            let Some(page) = by_id.get(id.as_str()) else {
                return Err(CatalogError::InvalidNavigation(format!(
                    "navigation section `{}` references unknown page id `{id}`",
                    section.label
                )));
            };
            if let Some(previous) = seen.insert(id, &section.label) {
                return Err(CatalogError::InvalidNavigation(format!(
                    "navigation page `{id}` appears in both `{previous}` and `{}`",
                    section.label
                )));
            }
            items.push(NavItem {
                id: page.id.clone(),
                title: page.title.clone(),
                route: page.route.clone(),
            });
        }
        navigation.push(NavSection {
            label: section.label.clone(),
            items,
        });
    }

    if navigation.is_empty() {
        navigation.push(NavSection {
            label: "Documentation".into(),
            items: pages
                .iter()
                .map(|page| NavItem {
                    id: page.id.clone(),
                    title: page.title.clone(),
                    route: page.route.clone(),
                })
                .collect(),
        });
    }

    Ok(navigation)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(id: &str, path: &str, hint: RouteHint, title: &str) -> SourcePage {
        SourcePage {
            id: id.to_string(),
            source_path: path.to_string(),
            route_hint: hint,
            title: title.to_string(),
            description: String::new(),
            headings: Vec::new(),
            outgoing_links: Vec::new(),
            article_html: String::new(),
        }
    }

    #[test]
    fn derives_index_and_named_routes() {
        let pages = validate(&[
            page("guide", "guide.rocdown", RouteHint::Derived, "Guide"),
            page("index", "index.rocdown", RouteHint::Derived, "Home"),
        ])
        .unwrap();
        assert_eq!(pages[0].route, "/guide/");
        assert_eq!(pages[0].output_path, "guide/index.html");
        assert_eq!(pages[1].route, "/");
        assert_eq!(pages[1].output_path, "index.html");
    }

    #[test]
    fn derives_nested_index_routes() {
        assert_eq!(derived_route("guides/index"), "/guides/");
        assert_eq!(derived_route("guides/build"), "/guides/build/");
    }

    #[test]
    fn explicit_route_gets_trailing_slash_and_sorts_by_output() {
        let pages = validate(&[
            page("b", "b.rocdown", RouteHint::Explicit("/zeta".into()), "Z"),
            page("a", "a.rocdown", RouteHint::Explicit("/alpha/".into()), "A"),
        ])
        .unwrap();
        assert_eq!(pages[0].output_path, "alpha/index.html");
        assert_eq!(pages[1].output_path, "zeta/index.html");
        assert_eq!(pages[1].route, "/zeta/");
    }

    #[test]
    fn duplicate_routes_name_both_sources() {
        let err = validate(&[
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
        ])
        .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("duplicate route `/same/`"), "{message}");
        assert!(message.contains("alpha.rocdown"), "{message}");
        assert!(message.contains("beta.rocdown"), "{message}");
    }

    #[test]
    fn rejects_dotdot_and_relative_routes() {
        let err = validate(&[page(
            "x",
            "x.rocdown",
            RouteHint::Explicit("/ok/../secret/".into()),
            "X",
        )])
        .unwrap_err();
        assert!(err.to_string().contains("(..)"));
        let err = validate(&[page(
            "y",
            "y.rocdown",
            RouteHint::Explicit("relative".into()),
            "Y",
        )])
        .unwrap_err();
        assert!(err.to_string().contains("not absolute"));
    }

    #[test]
    fn discovery_order_does_not_change_output_order() {
        let a = page("guide", "guide.rocdown", RouteHint::Derived, "Guide");
        let b = page("index", "index.rocdown", RouteHint::Derived, "Home");
        let forward = validate(&[a.clone(), b.clone()]).unwrap();
        let reverse = validate(&[b, a]).unwrap();
        assert_eq!(
            forward
                .iter()
                .map(|p| p.output_path.as_str())
                .collect::<Vec<_>>(),
            reverse
                .iter()
                .map(|p| p.output_path.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn resolves_configured_navigation_by_stable_id() {
        let pages = validate(&[
            page("guide", "guide.rocdown", RouteHint::Derived, "Guide"),
            page("index", "index.rocdown", RouteHint::Derived, "Home"),
        ])
        .unwrap();
        let nav = resolve_navigation(
            &pages,
            &[crate::config::NavConfig {
                label: "Start".into(),
                items: vec!["index".into(), "guide".into()],
            }],
        )
        .unwrap();
        assert_eq!(nav[0].items[0].route, "/");
        assert_eq!(nav[0].items[1].title, "Guide");
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
        let err = validate(&[home, guide]).unwrap_err().to_string();
        assert!(err.contains("/missing/"), "{err}");
        assert!(err.contains("/guide/#nope"), "{err}");
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
        assert!(validate(&[home, guide]).is_ok());
    }
}
