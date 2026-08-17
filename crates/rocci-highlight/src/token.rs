use rocci_template::Span;
use serde::{Deserialize, Serialize};

pub const MOD_DECLARATION: u32 = 1 << 0;
pub const MOD_DEFAULT_LIBRARY: u32 = 1 << 1;
pub const MOD_READONLY: u32 = 1 << 2;
pub const MOD_DOCUMENTATION: u32 = 1 << 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HighlightKind {
    Keyword,
    Function,
    Type,
    Namespace,
    Property,
    String,
    Parameter,
    Operator,
    Variable,
    Number,
    Comment,
    EnumMember,
    Struct,
    Macro,
    Decorator,
    Tag,
    Punctuation,
}

impl HighlightKind {
    pub fn to_lsp_index(self) -> u32 {
        match self {
            Self::Keyword => 0,
            Self::Function => 1,
            Self::Type => 2,
            Self::Namespace => 3,
            Self::Property => 4,
            Self::String => 5,
            Self::Parameter => 6,
            Self::Operator => 7,
            Self::Variable => 8,
            Self::Number => 9,
            Self::Comment => 10,
            Self::EnumMember => 11,
            Self::Struct => 12,
            Self::Macro => 13,
            Self::Decorator => 14,
            Self::Tag => 2,         // Maps to Type with defaultLibrary in LSP
            Self::Punctuation => 7, // Maps to Operator in LSP
        }
    }

    pub fn from_lsp_index(idx: u32) -> Option<Self> {
        match idx {
            0 => Some(Self::Keyword),
            1 => Some(Self::Function),
            2 => Some(Self::Type),
            3 => Some(Self::Namespace),
            4 => Some(Self::Property),
            5 => Some(Self::String),
            6 => Some(Self::Parameter),
            7 => Some(Self::Operator),
            8 => Some(Self::Variable),
            9 => Some(Self::Number),
            10 => Some(Self::Comment),
            11 => Some(Self::EnumMember),
            12 => Some(Self::Struct),
            13 => Some(Self::Macro),
            14 => Some(Self::Decorator),
            _ => None,
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Keyword => "tok-keyword",
            Self::Function => "tok-function",
            Self::Type => "tok-type",
            Self::Namespace => "tok-namespace",
            Self::Property => "tok-property",
            Self::String => "tok-string",
            Self::Parameter => "tok-parameter",
            Self::Operator => "tok-operator",
            Self::Variable => "tok-variable",
            Self::Number => "tok-number",
            Self::Comment => "tok-comment",
            Self::EnumMember => "tok-enum-member",
            Self::Struct => "tok-struct",
            Self::Macro => "tok-macro",
            Self::Decorator => "tok-decorator",
            Self::Tag => "tok-tag",
            Self::Punctuation => "tok-punctuation",
        }
    }
}

pub fn modifier_css_classes(modifiers: u32) -> Vec<&'static str> {
    let mut classes = Vec::new();
    if (modifiers & MOD_DECLARATION) != 0 {
        classes.push("tok-definition");
    }
    if (modifiers & MOD_DEFAULT_LIBRARY) != 0 {
        classes.push("tok-default-library");
    }
    if (modifiers & MOD_READONLY) != 0 {
        classes.push("tok-readonly");
    }
    if (modifiers & MOD_DOCUMENTATION) != 0 {
        classes.push("tok-documentation");
    }
    classes
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HighlightSpan {
    pub span: Span,
    pub kind: HighlightKind,
    pub modifiers: u32,
    pub priority: u32,
}

impl HighlightSpan {
    pub fn new(span: Span, kind: HighlightKind, modifiers: u32, priority: u32) -> Self {
        Self {
            span,
            kind,
            modifiers,
            priority,
        }
    }

    pub fn start(&self) -> usize {
        self.span.start as usize
    }

    pub fn end(&self) -> usize {
        self.span.end as usize
    }

    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }
}

pub fn floor_char_boundary(src: &str, mut index: usize) -> usize {
    index = index.min(src.len());
    while index > 0 && !src.is_char_boundary(index) {
        index -= 1;
    }
    index
}

pub fn for_each_line_span(src: &str, span: Span, mut f: impl FnMut(Span)) {
    let start = floor_char_boundary(src, (span.start as usize).min(src.len()));
    let end = floor_char_boundary(src, (span.end as usize).min(src.len()));
    if start >= end {
        return;
    }
    let mut line_start = start;
    for (i, ch) in src[start..end].char_indices() {
        if ch == '\n' {
            let mut line_end = start + i;
            if line_end > line_start && src.as_bytes()[line_end - 1] == b'\r' {
                line_end -= 1;
            }
            if line_end > line_start {
                f(Span::new(line_start, line_end));
            }
            line_start = start + i + 1;
        }
    }
    if line_start < end {
        f(Span::new(line_start, end));
    }
}

pub fn resolve_and_sort_spans(src: &str, raw_spans: &[HighlightSpan]) -> Vec<HighlightSpan> {
    let mut line_spans = Vec::new();
    for token in raw_spans {
        if token.span.is_empty() || (token.span.start as usize) >= src.len() {
            continue;
        }
        for_each_line_span(src, token.span, |span| {
            line_spans.push(HighlightSpan {
                span,
                kind: token.kind,
                modifiers: token.modifiers,
                priority: token.priority,
            });
        });
    }

    line_spans.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| (b.span.end - b.span.start).cmp(&(a.span.end - a.span.start)))
            .then_with(|| a.kind.cmp(&b.kind))
    });

    let mut result = Vec::new();
    let mut prev_end = 0usize;

    for token in line_spans {
        let start = token.start();
        let end = token.end();
        if start < prev_end {
            continue;
        }
        if start >= end || end > src.len() {
            continue;
        }
        result.push(token);
        prev_end = end;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_overlaps() {
        let src = "hello world";
        let spans = vec![
            HighlightSpan::new(Span::new(0, 5), HighlightKind::Keyword, 0, 50),
            HighlightSpan::new(Span::new(0, 11), HighlightKind::String, 0, 20), // lower priority overlap
            HighlightSpan::new(Span::new(6, 11), HighlightKind::Variable, 0, 40),
        ];
        let resolved = resolve_and_sort_spans(src, &spans);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].span, Span::new(0, 5));
        assert_eq!(resolved[0].kind, HighlightKind::Keyword);
        assert_eq!(resolved[1].span, Span::new(6, 11));
        assert_eq!(resolved[1].kind, HighlightKind::Variable);
    }
}
