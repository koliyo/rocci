use std::collections::BTreeSet;

use serde::Serialize;

use crate::article::PageKind;
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
    pub suppress_unlisted_warning: bool,
    pub layout: String,
    pub published: String,
    pub updated: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub collection: String,
    pub title: String,
    pub description: String,
    pub headings: Vec<PageHeading>,
    pub outgoing_links: Vec<String>,
    pub image_urls: Vec<String>,
    pub article_html: String,
    pub island_css: String,
    pub kind: PageKind,
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
    pub kind: PageKind,
    pub title: String,
    pub description: String,
    pub layout: String,
    pub published: String,
    pub updated: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub collection: String,
    pub headings: Vec<PageHeading>,
    pub outgoing_links: Vec<String>,
    pub article_html: String,
    #[serde(skip)]
    pub island_css: String,
    #[serde(skip)]
    pub island_html: Vec<String>,
    pub route: String,
    pub output_path: String,
    pub aliases: Vec<String>,
    pub draft: bool,
    #[serde(skip)]
    pub suppress_unlisted_warning: bool,
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
    #[serde(default)]
    pub children: Vec<NavSection>,
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
