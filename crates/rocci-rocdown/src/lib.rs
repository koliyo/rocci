//! Parse `.rocdown` documents and lower Markdown plus `@` declarations to Roc.

mod ast;
mod docs;
pub mod highlight;
mod img;
mod links;
mod lower;
pub mod lsp;
mod markdown;
mod page;
mod parse;
mod pprint;
mod scan;
pub mod standalone;
pub mod theme;

pub use ast::{
    DocsDecl, Document, HeadingInfo, ImgDecl, Item, LinkInfo, MdNode, PageDecl, PageMeta,
    RenderDecl, RocDecl,
};
pub use docs::{
    DocsField, extract_lines, extract_region, field_bool, field_string, field_strings,
    include_path_error, resolve_include_path, split_docs_body,
};
pub use highlight::{extract_rocdown_regions, highlight_rocdown, highlight_rocdown_document};
pub use img::{
    ImgFields, ImgHtmlAttr, StaticImage, collect_local_media, extract_img_fields, is_remote_asset,
    normalize_local_asset_url, resolve_local_asset,
};
pub use links::{PageRef, index_pages, index_pages_in_dir, page_ref_from_source};
pub use lsp::{RocdownAnalysis, RocdownAnalyzer};
pub use parse::{MarkdownBodyOptions, ParseOutput};
pub use pprint::format_ast;
pub use rocci_template::{
    ComponentInfo, Diagnostic, DiagnosticFrame, FixtureInfo, InitInfo, LowerOptions, MappedModule,
    OriginKind, RouteInfo, Segment, Severity, SourceFile, Span, StyleArtifact, StyleKind,
    TemplateItem, format_diagnostic, supports_ansi, type_name_from_path, wrap_type_module,
};
pub use rocci_theme::{
    ColorSchemePolicy, ResolvedTheme, ThemeOptions, ThemeOrigin, builtin_ids, discovered_ids,
    resolve as resolve_theme,
};
pub use theme::{ThemeArgs, compile_options as theme_compile_options};

// Standalone interactive document planning
pub use standalone::{
    StandaloneFailedFile, StandaloneModule, StandalonePlan, StandaloneReady,
    discover_rocdown_files, linked_standalone_inputs, plan_standalone,
};

use crate::parse::parse as parse_impl;

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub lower: LowerOptions,
    pub raw_html: bool,
    pub theme: ThemeOptions,
    pub pages: Vec<PageRef>,
    pub resolve_links: bool,
    pub resolve_includes: bool,
    pub check_assets: bool,
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
    let mut parsed = parse(source, options.raw_html);
    if options.resolve_links {
        links::resolve_document(source, &mut parsed, options);
    }
    if options.check_assets {
        img::check_document_assets(source, &parsed.document, options, &mut parsed.diagnostics);
    }
    let mut diagnostics = parsed.diagnostics;
    let lowered = lower::lower(
        source,
        &parsed.document,
        &parsed.headings,
        options,
        &mut diagnostics,
    );
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
    }
}
