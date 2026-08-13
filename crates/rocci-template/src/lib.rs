//! Parse `.rocci` modules and lower explicit components to ordinary Roc.
//!
//! This crate owns the bounded template grammar only. It does not invoke the
//! Roc compiler, define routes, watch files, or depend on HTTP/desktop runtimes.

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
    Attr, AttrValue, ComponentCall, ComponentDecl, ComponentPath, Document, Element, ForDirective,
    Fragment, Ident, IfDirective, Interpolation, LetDirective, MatchArm, MatchDirective,
    ModuleItem, ParsedParams, TemplateBlock, TemplateItem, TextNode, parse_component_params,
    strip_param_defaults,
};
pub use diagnostic::{Diagnostic, Severity};
pub use lower::{ComponentInfo, LowerOptions, LoweredModule};
pub use parser::ParseOutput;
pub use pprint::format_ast;
pub use resolve::{camel_to_pascal, component_roc_name, pascal_to_camel};
pub use source_map::{OriginKind, Segment};
pub use span::{PositionEncoding, SourceFile, Span};

use crate::parser::parse as parse_impl;
use crate::validate::validate;

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
    validate(&parsed.document, &mut diagnostics);
    let lowered = lower(source, &parsed.document, options);
    CompileOutput {
        roc: lowered.roc,
        segments: lowered.segments,
        components: lowered.components,
        document: parsed.document,
        diagnostics,
    }
}

pub fn format_diagnostic(source: SourceFile<'_>, diagnostic: &Diagnostic) -> String {
    let (line, col) = source.line_col(diagnostic.span.start);
    format!(
        "{}:{}:{}: {}: {}",
        source.name,
        line,
        col,
        match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        },
        diagnostic.message
    )
}
