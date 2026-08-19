use rocci_template::{
    ComponentDecl, ContextDecl, CssDecl, FixtureDecl, InitDecl, OnDecl, Span, TemplateItem,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Markdown(MdNode),
    Page(PageDecl),
    Roc(RocDecl),
    Render(RenderDecl),
    Component(ComponentDecl),
    Fixture(FixtureDecl),
    Css(CssDecl),
    Context(ContextDecl),
    Init(InitDecl),
    On(OnDecl),
    Use(UseDecl),
    Template(TemplateItem),
    Block(BlockCall),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Self::Markdown(node) => node.span(),
            Self::Page(item) => item.span,
            Self::Roc(item) => item.span,
            Self::Render(item) => item.span,
            Self::Component(item) => item.span,
            Self::Fixture(item) => item.span,
            Self::Css(item) => item.span,
            Self::Context(item) => item.span,
            Self::Init(item) => item.span,
            Self::On(item) => item.span,
            Self::Use(item) => item.span,
            Self::Template(item) => item.span(),
            Self::Block(item) => item.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageDecl {
    pub body: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RocDecl {
    pub body: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderDecl {
    pub expr: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseDecl {
    pub path: String,
    pub path_span: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockCall {
    pub name: String,
    pub name_span: Span,
    pub params: Option<BracketRecord>,
    pub content: Option<BlockContent>,
    pub span: Span,
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BracketRecord {
    pub fields: Vec<ParamField>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BracketList {
    pub items: Vec<ParamValue>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamField {
    pub name: String,
    pub name_span: Span,
    pub value: ParamValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamValue {
    StringLit { value: String, span: Span },
    BoolLit { value: bool, span: Span },
    NumberLit { value: String, span: Span },
    Ident { name: String, span: Span },
    Record(BracketRecord),
    List(BracketList),
}

impl ParamValue {
    pub fn span(&self) -> Span {
        match self {
            Self::StringLit { span, .. }
            | Self::BoolLit { span, .. }
            | Self::NumberLit { span, .. }
            | Self::Ident { span, .. } => *span,
            Self::Record(record) => record.span,
            Self::List(list) => list.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockContent {
    Line(LineContent),
    Brace(BraceSection),
    End(EndSection),
}

impl BlockContent {
    pub fn span(&self) -> Span {
        match self {
            Self::Line(content) => content.span,
            Self::Brace(section) => section.span,
            Self::End(section) => section.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineContent {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BraceSection {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndSection {
    pub span: Span,
    pub marker: EndMarker,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndMarker {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MdNode {
    Heading {
        level: u8,
        id: String,
        children: Vec<MdNode>,
        span: Span,
    },
    Paragraph {
        children: Vec<MdNode>,
        span: Span,
    },
    BlockQuote {
        children: Vec<MdNode>,
        span: Span,
    },
    List {
        ordered: bool,
        start: u64,
        children: Vec<MdNode>,
        span: Span,
    },
    Item {
        children: Vec<MdNode>,
        span: Span,
    },
    TaskItem {
        checked: bool,
        children: Vec<MdNode>,
        span: Span,
    },
    CodeBlock {
        info: String,
        literal: String,
        span: Span,
    },
    ThematicBreak {
        span: Span,
    },
    Table {
        children: Vec<MdNode>,
        span: Span,
    },
    TableRow {
        header: bool,
        children: Vec<MdNode>,
        span: Span,
    },
    TableCell {
        children: Vec<MdNode>,
        span: Span,
    },
    Text {
        value: String,
        span: Span,
    },
    SoftBreak {
        span: Span,
    },
    LineBreak {
        span: Span,
    },
    Code {
        value: String,
        span: Span,
    },
    Emph {
        children: Vec<MdNode>,
        span: Span,
    },
    Strong {
        children: Vec<MdNode>,
        span: Span,
    },
    Strikethrough {
        children: Vec<MdNode>,
        span: Span,
    },
    FootnoteDefinition {
        name: String,
        total_references: u32,
        children: Vec<MdNode>,
        span: Span,
    },
    FootnoteReference {
        name: String,
        reference_number: u32,
        index: u32,
        span: Span,
    },
    Link {
        url: String,
        title: String,
        children: Vec<MdNode>,
        span: Span,
    },
    Image {
        url: String,
        title: String,
        alt: String,
        span: Span,
    },
    RawHtml {
        html: String,
        span: Span,
    },
}

impl MdNode {
    pub fn span(&self) -> Span {
        match self {
            Self::Heading { span, .. }
            | Self::Paragraph { span, .. }
            | Self::BlockQuote { span, .. }
            | Self::List { span, .. }
            | Self::Item { span, .. }
            | Self::TaskItem { span, .. }
            | Self::CodeBlock { span, .. }
            | Self::ThematicBreak { span }
            | Self::Table { span, .. }
            | Self::TableRow { span, .. }
            | Self::TableCell { span, .. }
            | Self::Text { span, .. }
            | Self::SoftBreak { span }
            | Self::LineBreak { span }
            | Self::Code { span, .. }
            | Self::Emph { span, .. }
            | Self::Strong { span, .. }
            | Self::Strikethrough { span, .. }
            | Self::FootnoteDefinition { span, .. }
            | Self::FootnoteReference { span, .. }
            | Self::Link { span, .. }
            | Self::Image { span, .. }
            | Self::RawHtml { span, .. } => *span,
        }
    }

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
