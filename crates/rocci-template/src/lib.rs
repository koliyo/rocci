//! Parse `.rocci` modules and lower explicit components to ordinary Roc.
//!
//! This crate owns the bounded template grammar only. It does not invoke the
//! Roc compiler, type-check Roc, spawn HTTP, or depend on desktop runtimes.
//! `@context` / `@init` / `@on` are lowered to Roc functions plus route
//! metadata for the CLI dispatcher.

mod ast;
mod diagnostic;
mod lexer;
mod lower;
mod parser;
mod pprint;
mod resolve;
mod source_map;
mod span;
mod validate;

pub use ast::{
    Attr, AttrValue, ComponentCall, ComponentDecl, ComponentPath, ContextDecl, CssDecl, Document,
    Element, FixtureDecl, ForDirective, Fragment, Ident, IfDirective, InitDecl, Interpolation,
    LetDirective, MatchArm, MatchDirective, ModuleItem, OnDecl, ParsedParams, TemplateBlock,
    TemplateItem, TextNode, parse_component_params, strip_param_defaults,
};
pub use diagnostic::{Diagnostic, DiagnosticFrame, Severity, supports_ansi};
pub use lexer::{Cursor, is_ident_continue, is_ident_start, trim_span};
pub use lower::{
    ComponentInfo, FixtureInfo, InitInfo, LowerOptions, LoweredModule, LoweredTemplate, RouteInfo,
    StyleArtifact, StyleKind, TemplateValueCtx, file_scope_id, lower_template_items, route_fn_name,
    template_items_have_action,
};
pub use parser::{
    ParseDeclOutput, ParseOutput, ParseTemplateOutput, parse_declaration_from,
    parse_template_item_from,
};
pub use pprint::format_ast;
pub use resolve::{camel_to_pascal, component_matches, component_roc_name, pascal_to_camel};
pub use source_map::{OriginKind, Segment};
pub use span::{PositionEncoding, SourceFile, Span};
pub use validate::{validate, validate_template_items};

use crate::parser::parse as parse_impl;

pub fn parse(source: SourceFile<'_>) -> ParseOutput {
    parse_impl(source)
}

pub fn lower(source: SourceFile<'_>, document: &Document, options: &LowerOptions) -> LoweredModule {
    crate::lower::lower(source, document, options)
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
}

impl CompileOutput {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }
}

pub fn compile(source: SourceFile<'_>, options: &LowerOptions) -> CompileOutput {
    let parsed = parse(source);
    let mut diagnostics = parsed.diagnostics;
    validate(source.src, &parsed.document, &mut diagnostics);
    let lowered = lower(source, &parsed.document, options);
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
        diagnostics,
    }
}

pub fn format_diagnostic(source: SourceFile<'_>, diagnostic: &Diagnostic) -> String {
    DiagnosticFrame::from_source(source, diagnostic).render_for_stderr()
}
