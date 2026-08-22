//! Parse `.rocci` modules and lower explicit components to ordinary Roc.
//!
//! This crate owns the bounded template grammar only. It does not invoke the
//! Roc compiler, type-check Roc, spawn HTTP, or depend on desktop runtimes.
//! `@context` / `@init` / `@view` / `@patch` / `@command` are lowered to Roc functions plus route
//! metadata for the CLI dispatcher.

mod ast;
mod diagnostic;
mod lexer;
mod lower;
mod parser;
mod pprint;
mod remap;
mod resolve;
mod roc;
mod source_map;
mod span;
mod validate;

pub use ast::{
    Attr, AttrValue, CommandDecl, ComponentCall, ComponentDecl, ComponentPath, ContextDecl,
    CssDecl, Document, Element, FixtureDecl, ForDirective, Fragment, FragmentDecl, Ident,
    IfDirective, InitDecl, Interpolation, LeadingComments, LetDirective, LiveDecl, MatchArm,
    MatchDirective, ModuleItem, ParsedParams, RouteDecl, TemplateBlock, TemplateItem, TextNode,
    ViewDecl, parse_component_params, strip_param_defaults,
};
pub use diagnostic::{Diagnostic, DiagnosticFrame, Severity, supports_ansi};
pub use lexer::{Cursor, is_ident_continue, is_ident_start, leading_comments_before, trim_span};
pub use lower::{
    ComponentInfo, FixtureInfo, InitInfo, LiveInfo, LowerOptions, LoweredModule, LoweredTemplate,
    RespondKind, RouteInfo, StyleArtifact, StyleKind, TemplateValueCtx, command_json_fn_name,
    file_scope_id, lower_template_items, route_fn_name, template_items_have_action,
};
pub use parser::{
    ParseDeclOutput, ParseOutput, ParseTemplateOutput, parse_declaration_from,
    parse_template_item_from,
};
pub use pprint::{HandlerInspect, format_ast, inspect_handlers};
pub use remap::{MappedModule, remap_roc_output};
pub use resolve::{
    camel_to_pascal, component_matches, component_roc_name, is_ambiguous_pascal, pascal_to_camel,
};
pub use roc::{type_name_from_path, wrap_type_module};
pub use source_map::{OriginKind, Segment};
pub use span::{PositionEncoding, SourceFile, Span};
pub use validate::{validate, validate_template_items};

use crate::parser::parse as parse_impl;
use std::time::Instant;

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
    pub lives: Vec<LiveInfo>,
    pub routes: Vec<RouteInfo>,
    pub document: Document,
    pub timings: CompileTimings,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompileTimings {
    pub parse_ms: u128,
    pub validate_ms: u128,
    pub lower_ms: u128,
}

impl CompileOutput {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }
}

pub fn compile(source: SourceFile<'_>, options: &LowerOptions) -> CompileOutput {
    let parse_started = Instant::now();
    let parsed = parse(source);
    let parse_ms = parse_started.elapsed().as_millis();
    let mut diagnostics = parsed.diagnostics;
    let validate_started = Instant::now();
    validate(source.src, &parsed.document, &mut diagnostics);
    let validate_ms = validate_started.elapsed().as_millis();
    let lower_started = Instant::now();
    let lowered = lower(source, &parsed.document, options);
    let lower_ms = lower_started.elapsed().as_millis();
    CompileOutput {
        roc: lowered.roc,
        segments: lowered.segments,
        components: lowered.components,
        fixtures: lowered.fixtures,
        styles: lowered.styles,
        state_type: lowered.state_type,
        init: lowered.init,
        lives: lowered.lives,
        routes: lowered.routes,
        document: parsed.document,
        diagnostics,
        timings: CompileTimings {
            parse_ms,
            validate_ms,
            lower_ms,
        },
    }
}

pub fn format_diagnostic(source: SourceFile<'_>, diagnostic: &Diagnostic) -> String {
    DiagnosticFrame::from_source(source, diagnostic).render_for_stderr()
}
