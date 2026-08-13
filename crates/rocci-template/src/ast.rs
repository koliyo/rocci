use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub items: Vec<ModuleItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleItem {
    Roc { span: Span },
    Component(ComponentDecl),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDecl {
    pub name: Ident,
    pub params: Span,
    pub body: TemplateBlock,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident {
    pub span: Span,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateBlock {
    pub items: Vec<TemplateItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateItem {
    Element(Element),
    ComponentCall(ComponentCall),
    Fragment(Fragment),
    Text(TextNode),
    Interpolation(Interpolation),
    If(IfDirective),
    For(ForDirective),
    Match(MatchDirective),
    Let(LetDirective),
}

impl TemplateItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Element(item) => item.span,
            Self::ComponentCall(item) => item.span,
            Self::Fragment(item) => item.span,
            Self::Text(item) => item.span,
            Self::Interpolation(item) => item.span,
            Self::If(item) => item.span,
            Self::For(item) => item.span,
            Self::Match(item) => item.span,
            Self::Let(item) => item.span,
        }
    }

    pub fn is_let(&self) -> bool {
        matches!(self, Self::Let(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name: Ident,
    pub attrs: Vec<Attr>,
    pub children: Vec<TemplateItem>,
    pub self_closing: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentCall {
    pub path: ComponentPath,
    pub attrs: Vec<Attr>,
    pub children: Option<Vec<TemplateItem>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentPath {
    pub parts: Vec<Ident>,
    pub roc_name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fragment {
    pub children: Vec<TemplateItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextNode {
    pub span: Span,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interpolation {
    pub expr: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attr {
    pub name: Ident,
    pub value: AttrValue,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttrValue {
    Static { span: Span, value: String },
    Expr { expr: Span },
    Boolean,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfDirective {
    pub condition: Span,
    pub then_body: TemplateBlock,
    pub else_ifs: Vec<(Span, TemplateBlock)>,
    pub else_body: Option<TemplateBlock>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForDirective {
    pub binder: Ident,
    pub collection: Span,
    pub body: TemplateBlock,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchDirective {
    pub scrutinee: Span,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Span,
    pub value: Box<TemplateItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetDirective {
    pub binder: Ident,
    pub expr: Span,
    pub span: Span,
}

pub fn extra_param_names(src: &str, params: Span) -> Vec<String> {
    let raw = params.of(src).trim();
    let inner = raw
        .strip_prefix('|')
        .and_then(|s| s.strip_suffix('|'))
        .unwrap_or(raw);
    split_top_level(inner, ',')
        .into_iter()
        .skip(1)
        .filter_map(ident_from_param)
        .collect()
}

fn split_top_level(inner: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut part_start = 0;
    let mut chars = inner.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '"' => {
                while let Some((_, next)) = chars.next() {
                    if next == '\\' {
                        chars.next();
                    } else if next == '"' {
                        break;
                    }
                }
            }
            c if c == sep && depth == 0 => {
                parts.push(&inner[part_start..i]);
                part_start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&inner[part_start..]);
    parts
}

fn ident_from_param(part: &str) -> Option<String> {
    let trimmed = part.trim();
    let ident = trimmed
        .trim_start_matches(|ch: char| !ch.is_ascii_alphabetic() && ch != '_')
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    if ident.is_empty() { None } else { Some(ident) }
}
