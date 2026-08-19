//! Parse, compile, lower, and generate sites from `.rocdown` Markdown documents.

mod article;
mod ast;
mod build;
mod catalog;
mod config;
mod dev;
mod docs;
pub mod highlight;
mod img;
mod imports;
mod islands;
mod links;
mod lower;
pub mod lsp;
mod markdown;
mod page;
mod params;
mod parse;
mod plan;
mod pprint;
mod registry;
mod runtime;
mod scan;
mod site;
pub mod standalone;
pub mod theme;

pub use article::{PageClass, PageKind, classify_document, render_document, roc_imports_datastar};
pub use ast::{
    BlockCall, BlockContent, BraceSection, BracketList, BracketRecord, Document, EndMarker,
    EndSection, HeadingInfo, Item, LineContent, LinkInfo, MdNode, PageDecl, PageMeta, ParamField,
    ParamValue, RenderDecl, RocDecl, UseDecl,
};
pub use build::{
    BuildReport, BuildSession, build, build_configured, build_configured_with_host,
    build_with_host, discover_rocdown,
};
pub use catalog::{
    CatalogDiagnostic, Edge, EdgeKind, PageHeading, ResolveOptions, ResolveResult, ResolvedSite,
    RouteHint, Severity as CatalogSeverity, SourcePage, resolve,
};
pub use config::{BuildConfig, CONFIG_FILE, NavConfig, SiteConfig, SiteMeta, load_config};
pub use dev::{DevServer, run, run_with_host, run_with_host_at};
pub use docs::{
    ArticleNode, DocsField, ExampleRecord, ExampleTestOptions, IncludeOptions, IncludeOrigin,
    PageDocs, PlannedNode, PlannedProp, PlannedWidget, extract_lines, extract_region, field_bool,
    field_string, field_strings, include_path_error, load_page_docs, markdown_fragment,
    plan_segments, render_article, resolve_include_path, run_examples, search_text,
};
pub use highlight::{extract_rocdown_regions, highlight_rocdown, highlight_rocdown_document};
pub use img::{
    ImgFields, ImgHtmlAttr, StaticImage, collect_local_media, extract_img_fields, is_remote_asset,
    normalize_local_asset_url, resolve_local_asset,
};
pub use links::{PageRef, index_pages, index_pages_in_dir, page_ref_from_source};
pub use lsp::{RocdownAnalysis, RocdownAnalyzer};
pub use parse::{MarkdownBodyOptions, ParseOutput};
pub use plan::{BuildPlan, DEFAULT_CSP, plan};
pub use pprint::format_ast;
pub use rocci_roc_host::HostChoice;
pub use runtime::{HTML, HTML_BINDINGS, THEME, runtime_bytes, stage_into};
pub use site::{
    CheckFormat, CheckReport, InspectKind, check, find_site_root, inspect, load_site,
    resolve_loaded, site_preview_route, test_examples,
};
pub use standalone::{
    StandaloneFailedFile, StandaloneModule, StandalonePlan, StandaloneReady,
    discover_rocdown_files, linked_standalone_inputs, plan_standalone,
};
pub use theme::{ThemeArgs, compile_options as theme_compile_options};

pub use rocci_template::{
    ComponentInfo, Diagnostic, DiagnosticFrame, FixtureInfo, InitInfo, LowerOptions, MappedModule,
    OriginKind, RouteInfo, Segment, Severity, SourceFile, Span, StyleArtifact, StyleKind,
    TemplateItem, format_diagnostic, supports_ansi, type_name_from_path, wrap_type_module,
};
pub use rocci_theme::{
    ColorSchemePolicy, ResolvedTheme, ThemeOptions, ThemeOrigin, builtin_ids, discovered_ids,
    resolve as resolve_theme,
};

pub const BASIC_CLI_PLATFORM: &str = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst";
pub const STAGING_ENV: &str = "ROCDOWN_STAGING";

use crate::parse::parse as parse_impl;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub lower: LowerOptions,
    pub raw_html: bool,
    pub theme: ThemeOptions,
    pub pages: Vec<PageRef>,
    pub resolve_links: bool,
    pub resolve_includes: bool,
    pub check_assets: bool,
    pub default_route: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            lower: LowerOptions::default(),
            raw_html: false,
            theme: ThemeOptions::default(),
            pages: Vec::new(),
            resolve_links: true,
            resolve_includes: true,
            check_assets: false,
            default_route: None,
        }
    }
}

pub struct CompileOutput {
    pub roc: String,
    pub segments: Vec<Segment>,
    pub diagnostics: Vec<Diagnostic>,
    pub components: Vec<ComponentInfo>,
    pub fixtures: Vec<FixtureInfo>,
    pub styles: Vec<StyleArtifact>,
    pub state_type: Option<String>,
    pub init: Option<InitInfo>,
    pub routes: Vec<RouteInfo>,
    pub document: Document,
    pub page_meta: PageMeta,
    pub headings: Vec<HeadingInfo>,
    pub links: Vec<LinkInfo>,
    pub theme: Option<ResolvedTheme>,
    pub timings: CompileTimings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompileTimings {
    pub parse_ms: u128,
    pub lower_ms: u128,
}

impl CompileOutput {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }
}

pub fn parse(source: SourceFile<'_>, raw_html: bool) -> ParseOutput {
    parse_impl(source, raw_html)
}

/// Parse a Markdown-only region while retaining byte spans in the original source.
///
/// Unlike [`parse`], this does not scan Rocdown declarations and does not enable
/// Rocdown wikilinks. It is intended for inert Markdown collections such as OKF.
pub fn parse_markdown_body(
    source: SourceFile<'_>,
    body: Span,
    options: MarkdownBodyOptions,
) -> ParseOutput {
    parse::parse_markdown_body(source, body, options)
}

pub fn parse_fragment(source: SourceFile<'_>, body: Span, raw_html: bool) -> ParseOutput {
    parse::parse_fragment(source, body, raw_html)
}

pub fn compile(source: SourceFile<'_>, options: &CompileOptions) -> CompileOutput {
    let parse_started = Instant::now();
    let mut parsed = parse(source, options.raw_html);
    let parse_ms = parse_started.elapsed().as_millis();
    if options.resolve_links {
        links::resolve_document(source, &mut parsed, options);
    }
    if options.check_assets {
        img::check_document_assets(source, &parsed.document, options, &mut parsed.diagnostics);
    }
    let mut diagnostics = parsed.diagnostics;
    let lower_started = Instant::now();
    let lowered = lower::lower(
        source,
        &parsed.document,
        &parsed.headings,
        options,
        &mut diagnostics,
    );
    let lower_ms = lower_started.elapsed().as_millis();
    CompileOutput {
        roc: lowered.roc,
        segments: lowered.segments,
        components: lowered.components,
        fixtures: lowered.fixtures,
        styles: lowered.styles,
        state_type: lowered.state_type,
        init: lowered.init,
        routes: lowered.routes,
        document: parsed.document,
        page_meta: lowered.page_meta,
        headings: parsed.headings,
        links: parsed.links,
        theme: lowered.theme,
        diagnostics,
        timings: CompileTimings { parse_ms, lower_ms },
    }
}

pub fn compile_islands(source: SourceFile<'_>, options: &CompileOptions) -> CompileOutput {
    let parse_started = Instant::now();
    let parsed = parse(source, options.raw_html);
    let parse_ms = parse_started.elapsed().as_millis();
    let mut diagnostics = parsed.diagnostics;
    let lower_started = Instant::now();
    let lowered = lower::lower_islands(source, &parsed.document, options, &mut diagnostics);
    let lower_ms = lower_started.elapsed().as_millis();
    CompileOutput {
        roc: lowered.roc,
        segments: lowered.segments,
        components: lowered.components,
        fixtures: lowered.fixtures,
        styles: lowered.styles,
        state_type: lowered.state_type,
        init: lowered.init,
        routes: lowered.routes,
        document: parsed.document,
        page_meta: lowered.page_meta,
        headings: parsed.headings,
        links: parsed.links,
        theme: lowered.theme,
        diagnostics,
        timings: CompileTimings { parse_ms, lower_ms },
    }
}
