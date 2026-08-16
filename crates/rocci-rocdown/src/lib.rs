//! Parse `.rocdown` documents and lower Markdown plus `@` declarations to Roc.

mod ast;
mod docs;
mod links;
mod lower;
mod markdown;
mod page;
mod parse;
mod pprint;
mod scan;

pub use ast::{
    DocsDecl, Document, HeadingInfo, Item, LinkInfo, MdNode, PageDecl, PageMeta, RenderDecl,
    RocDecl,
};
pub use docs::{
    DocsField, extract_lines, extract_region, field_bool, field_string, field_strings,
    include_path_error, resolve_include_path, split_docs_body,
};
pub use links::{PageRef, index_pages, index_pages_in_dir, page_ref_from_source};
pub use parse::{MarkdownBodyOptions, ParseOutput};
pub use pprint::format_ast;
pub use rocci_template::{
    ComponentInfo, Diagnostic, DiagnosticFrame, FixtureInfo, InitInfo, LowerOptions, OriginKind,
    RouteInfo, Segment, Severity, SourceFile, Span, StyleArtifact, StyleKind, TemplateItem,
    format_diagnostic, supports_ansi,
};
pub use rocci_theme::{
    ColorSchemePolicy, ResolvedTheme, ThemeOptions, ThemeOrigin, builtin_ids, discovered_ids,
    resolve as resolve_theme,
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
    let mut diagnostics = parsed.diagnostics;
    let lowered = lower::lower(source, &parsed.document, options, &mut diagnostics);
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
