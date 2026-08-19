use rocci_template::Span;

#[path = "md.generated.rs"]
mod md_generated;
pub use md_generated::MdNode;

#[path = "ast.generated.rs"]
mod ast_generated;
pub use ast_generated::*;

#[path = "node_kind.generated.rs"]
#[allow(dead_code)]
mod node_kind;
#[cfg(test)]
pub(crate) use node_kind::NodeKind;

impl BlockCall {
    pub fn content_span(&self) -> Option<Span> {
        self.content.as_ref().map(BlockContent::span)
    }

    pub fn is_colon(&self, src: &str) -> bool {
        self.span
            .of(src)
            .trim_start_matches([' ', '\t'])
            .starts_with(':')
    }

    pub fn payload_span(&self) -> Span {
        let start = self
            .params
            .as_ref()
            .map(|params| params.span.start)
            .or_else(|| self.content.as_ref().map(|content| content.span().start))
            .unwrap_or(self.span.end);
        let end = self
            .content
            .as_ref()
            .map(|content| content.span().end)
            .or_else(|| self.params.as_ref().map(|params| params.span.end))
            .unwrap_or(self.span.end);
        if start > end {
            Span::point(self.span.end as usize)
        } else {
            Span::new(start as usize, end as usize)
        }
    }
}

impl BlockContent {
    pub fn scope_name(&self) -> &'static str {
        match self {
            Self::Line(_) => "line",
            Self::Brace(_) => "section",
            Self::End(_) => "end",
        }
    }
}

impl MdNode {
    pub fn children(&self) -> &[MdNode] {
        match self {
            Self::Heading { children, .. }
            | Self::Paragraph { children, .. }
            | Self::BlockQuote { children, .. }
            | Self::List { children, .. }
            | Self::Item { children, .. }
            | Self::TaskItem { children, .. }
            | Self::Table { children, .. }
            | Self::TableRow { children, .. }
            | Self::TableCell { children, .. }
            | Self::Emph { children, .. }
            | Self::Strong { children, .. }
            | Self::Strikethrough { children, .. }
            | Self::FootnoteDefinition { children, .. }
            | Self::Link { children, .. } => children,
            Self::CodeBlock { .. }
            | Self::ThematicBreak { .. }
            | Self::Text { .. }
            | Self::SoftBreak { .. }
            | Self::LineBreak { .. }
            | Self::Code { .. }
            | Self::FootnoteReference { .. }
            | Self::Image { .. }
            | Self::RawHtml { .. } => &[],
        }
    }

    pub fn walk<'a>(&'a self, visit: &mut impl FnMut(&'a MdNode)) {
        visit(self);
        for child in self.children() {
            child.walk(visit);
        }
    }

    pub fn text_content(&self) -> String {
        match self {
            Self::Text { value, .. } | Self::Code { value, .. } => value.clone(),
            Self::SoftBreak { .. } | Self::LineBreak { .. } => " ".to_string(),
            Self::Image { alt, .. } => alt.clone(),
            Self::Heading { children, .. }
            | Self::Paragraph { children, .. }
            | Self::BlockQuote { children, .. }
            | Self::List { children, .. }
            | Self::Item { children, .. }
            | Self::TaskItem { children, .. }
            | Self::Table { children, .. }
            | Self::TableRow { children, .. }
            | Self::TableCell { children, .. }
            | Self::Emph { children, .. }
            | Self::Strong { children, .. }
            | Self::Strikethrough { children, .. }
            | Self::FootnoteDefinition { children, .. }
            | Self::Link { children, .. } => children.iter().map(Self::text_content).collect(),
            Self::CodeBlock { .. }
            | Self::ThematicBreak { .. }
            | Self::FootnoteReference { .. }
            | Self::RawHtml { .. } => String::new(),
        }
    }

    pub fn children_mut(&mut self) -> &mut [MdNode] {
        match self {
            Self::Heading { children, .. }
            | Self::Paragraph { children, .. }
            | Self::BlockQuote { children, .. }
            | Self::List { children, .. }
            | Self::Item { children, .. }
            | Self::TaskItem { children, .. }
            | Self::Table { children, .. }
            | Self::TableRow { children, .. }
            | Self::TableCell { children, .. }
            | Self::Emph { children, .. }
            | Self::Strong { children, .. }
            | Self::Strikethrough { children, .. }
            | Self::FootnoteDefinition { children, .. }
            | Self::Link { children, .. } => children,
            Self::CodeBlock { .. }
            | Self::ThematicBreak { .. }
            | Self::Text { .. }
            | Self::SoftBreak { .. }
            | Self::LineBreak { .. }
            | Self::Code { .. }
            | Self::FootnoteReference { .. }
            | Self::Image { .. }
            | Self::RawHtml { .. } => &mut [],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadingInfo {
    pub level: u8,
    pub id: String,
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkInfo {
    pub url: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PageMeta {
    pub id: Option<String>,
    pub route: Option<String>,
    pub aliases: Vec<String>,
    pub draft: bool,
    pub layout: Option<String>,
    pub meta: Option<Span>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub theme: Option<String>,
    pub color_scheme: Option<String>,
    pub published: Option<String>,
    pub updated: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub collection: Option<String>,
    pub summary: Option<String>,
}

#[cfg(test)]
mod node_kind_highlight {
    use super::NodeKind;

    fn host_paints(kind: NodeKind) -> bool {
        match kind {
            NodeKind::Document
            | NodeKind::Item
            | NodeKind::PageDecl
            | NodeKind::RocDecl
            | NodeKind::RenderDecl
            | NodeKind::UseDecl
            | NodeKind::BlockCall
            | NodeKind::BlockContent
            | NodeKind::EndMarker => true,
            NodeKind::ParamField
            | NodeKind::ParamValue
            | NodeKind::BracketRecord
            | NodeKind::BracketList
            | NodeKind::LineContent
            | NodeKind::BraceSection
            | NodeKind::EndSection => false,
        }
    }

    #[test]
    fn every_kind_is_painted_or_omitted() {
        for &kind in NodeKind::ALL {
            assert_eq!(
                host_paints(kind),
                !kind.highlight_omitted(),
                "{kind:?} must be painted by the host collector or listed in [highlight.omit]"
            );
        }
    }
}
