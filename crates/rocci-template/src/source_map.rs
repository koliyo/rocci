use crate::span::Span;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginKind {
    OrdinaryRoc,
    ComponentSignature,
    Directive,
    ComponentTag,
    TextExpression,
    AttributeExpression,
    StaticMarkup,
    Scaffolding,
    Css,
    MarkdownStructure,
    MarkdownText,
    MarkdownBoilerplate,
    PageRoc,
    RocBlock,
    RenderRoc,
}

impl OriginKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OrdinaryRoc => "ordinary_roc",
            Self::ComponentSignature => "component_signature",
            Self::Directive => "directive",
            Self::ComponentTag => "component_tag",
            Self::TextExpression => "text_expression",
            Self::AttributeExpression => "attribute_expression",
            Self::StaticMarkup => "static_markup",
            Self::Scaffolding => "scaffolding",
            Self::Css => "css",
            Self::MarkdownStructure => "markdown_structure",
            Self::MarkdownText => "markdown_text",
            Self::MarkdownBoilerplate => "markdown_boilerplate",
            Self::PageRoc => "page_roc",
            Self::RocBlock => "roc_block",
            Self::RenderRoc => "render_roc",
        }
    }
}

impl fmt::Display for OriginKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub generated: Span,
    pub source: Span,
    pub origin: OriginKind,
}

impl Segment {
    pub fn new(generated: Span, source: Span, origin: OriginKind) -> Self {
        Self {
            generated,
            source,
            origin,
        }
    }
}
