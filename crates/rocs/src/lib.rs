//! Rocs is a static documentation generator built on Rocci.
//!
//! Rust owns discovery, the page catalog, routes, static Markdown rendering, and
//! `@docs` validation. Rocci owns the site shell and documentation components,
//! compiled once per build. Article prose stays in fragment files so Roc does
//! not parse it. Dynamic `@render` / Rocci islands stay on the Roc path and are
//! not part of the static catalog yet.

mod article;
mod build;
mod catalog;
mod config;
mod dev;
mod docs;
pub mod okf;
mod plan;
mod runtime;
mod site;

pub use article::render_document;
pub use build::{BuildReport, BuildSession, build, build_configured, discover_rocdown};
pub use catalog::{
    CatalogDiagnostic, Edge, EdgeKind, PageHeading, ResolveOptions, ResolveResult, ResolvedSite,
    RouteHint, Severity, SourcePage, resolve,
};
pub use config::{BuildConfig, NavConfig, SiteConfig, SiteMeta, load_config};
pub use dev::{DevServer, run, run_knowledge};
pub use docs::{
    ArticleNode, ExampleRecord, ExampleTestOptions, IncludeOptions, IncludeOrigin, PageDocs,
    load_page_docs, markdown_fragment, render_article, run_examples, search_text,
};
pub use plan::{BuildPlan, DEFAULT_CSP, plan};
pub use runtime::{HTML, HTML_BINDINGS, THEME, runtime_bytes, stage_into};
pub use site::{
    CheckFormat, CheckReport, InspectKind, check, inspect, load_site, resolve_loaded, test_examples,
};

pub const BASIC_CLI_PLATFORM: &str = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst";
pub const STAGING_ENV: &str = "ROCS_STAGING";
