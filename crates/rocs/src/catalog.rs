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
    pub article_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPage {
    pub id: String,
    pub source_path: String,
    pub title: String,
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
        }
    }
}

impl std::error::Error for CatalogError {}

pub fn derived_route(id: &str) -> String {
    if id == "index" {
        "/".to_string()
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

    resolved.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    Ok(resolved)
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
}
