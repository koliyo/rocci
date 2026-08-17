use lsp_types::{
    Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend,
};
use rocci_template::{Document, PositionEncoding, SourceFile, Span};

pub use rocci_highlight::{MOD_DECLARATION, MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION, MOD_READONLY};

pub const TOKEN_KEYWORD: u32 = 0;
pub const TOKEN_FUNCTION: u32 = 1;
pub const TOKEN_TYPE: u32 = 2;
pub const TOKEN_NAMESPACE: u32 = 3;
pub const TOKEN_PROPERTY: u32 = 4;
pub const TOKEN_STRING: u32 = 5;
pub const TOKEN_PARAMETER: u32 = 6;
pub const TOKEN_OPERATOR: u32 = 7;
pub const TOKEN_VARIABLE: u32 = 8;
pub const TOKEN_NUMBER: u32 = 9;
pub const TOKEN_COMMENT: u32 = 10;
pub const TOKEN_ENUM_MEMBER: u32 = 11;
pub const TOKEN_STRUCT: u32 = 12;
pub const TOKEN_MACRO: u32 = 13;
pub const TOKEN_DECORATOR: u32 = 14;

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,     // 0
            SemanticTokenType::FUNCTION,    // 1
            SemanticTokenType::TYPE,        // 2
            SemanticTokenType::NAMESPACE,   // 3
            SemanticTokenType::PROPERTY,    // 4
            SemanticTokenType::STRING,      // 5
            SemanticTokenType::PARAMETER,   // 6
            SemanticTokenType::OPERATOR,    // 7
            SemanticTokenType::VARIABLE,    // 8
            SemanticTokenType::NUMBER,      // 9
            SemanticTokenType::COMMENT,     // 10
            SemanticTokenType::ENUM_MEMBER, // 11
            SemanticTokenType::STRUCT,      // 12
            SemanticTokenType::MACRO,       // 13
            SemanticTokenType::DECORATOR,   // 14
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,     // 1 << 0
            SemanticTokenModifier::DEFAULT_LIBRARY, // 1 << 1
            SemanticTokenModifier::READONLY,        // 1 << 2
            SemanticTokenModifier::DOCUMENTATION,   // 1 << 3
        ],
    }
}

#[derive(Clone, Debug)]
pub struct RawToken {
    pub span: Span,
    pub kind: u32,
    pub modifiers: u32,
    pub priority: u32,
}

pub struct Collector<'a> {
    pub src: &'a str,
    pub tokens: Vec<RawToken>,
}

impl<'a> Collector<'a> {
    pub fn token(&mut self, span: Span, kind: u32, modifiers: u32) {
        self.token_with_priority(span, kind, modifiers, 50);
    }

    pub fn token_with_priority(&mut self, span: Span, kind: u32, modifiers: u32, priority: u32) {
        if !span.is_empty() && (span.end as usize) <= self.src.len() {
            self.tokens.push(RawToken {
                span,
                kind,
                modifiers,
                priority,
            });
        }
    }
}

pub fn semantic_tokens(
    name: &str,
    text: &str,
    document: &Document,
    encoding: PositionEncoding,
    range: Option<Range>,
) -> SemanticTokens {
    let source = SourceFile::new(name, text);
    let spans = rocci_highlight::highlight_rocci_document(text, document);
    let mut raw_tokens: Vec<RawToken> = spans
        .into_iter()
        .map(|s| RawToken {
            span: s.span,
            kind: s.kind.to_lsp_index(),
            modifiers: s.modifiers,
            priority: s.priority,
        })
        .collect();
    let range_span = range.map(|range| {
        Span::new(
            source.offset_at(range.start.line, range.start.character, encoding) as usize,
            source.offset_at(range.end.line, range.end.character, encoding) as usize,
        )
    });
    SemanticTokens {
        result_id: None,
        data: encode_tokens(source, &mut raw_tokens, encoding, range_span),
    }
}

pub fn semantic_tokens_rocdown(
    name: &str,
    text: &str,
    document: &rocci_rocdown::Document,
    headings: &[rocci_rocdown::HeadingInfo],
    encoding: PositionEncoding,
    range: Option<Range>,
) -> SemanticTokens {
    let source = SourceFile::new(name, text);
    let spans = rocci_highlight::highlight_rocdown_document(text, document, headings);
    let mut raw_tokens: Vec<RawToken> = spans
        .into_iter()
        .map(|s| RawToken {
            span: s.span,
            kind: s.kind.to_lsp_index(),
            modifiers: s.modifiers,
            priority: s.priority,
        })
        .collect();
    let range_span = range.map(|range| {
        Span::new(
            source.offset_at(range.start.line, range.start.character, encoding) as usize,
            source.offset_at(range.end.line, range.end.character, encoding) as usize,
        )
    });
    SemanticTokens {
        result_id: None,
        data: encode_tokens(source, &mut raw_tokens, encoding, range_span),
    }
}

pub fn encode_tokens(
    source: SourceFile<'_>,
    tokens: &mut [RawToken],
    encoding: PositionEncoding,
    range: Option<Span>,
) -> Vec<SemanticToken> {
    let mut line_tokens = Vec::new();
    for token in tokens.iter() {
        if let Some(range) = range
            && (token.span.end <= range.start || token.span.start >= range.end)
        {
            continue;
        }
        for_each_line_span(source.src, token.span, |span| {
            line_tokens.push(RawToken {
                span,
                kind: token.kind,
                modifiers: token.modifiers,
                priority: token.priority,
            });
        });
    }

    line_tokens.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then_with(|| b.priority.cmp(&a.priority))
            .then_with(|| (b.span.end - b.span.start).cmp(&(a.span.end - a.span.start)))
            .then_with(|| a.kind.cmp(&b.kind))
    });

    let line_index = LineIndex::new(source.src);
    let mut data = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;
    let mut prev_end = 0u32;

    for token in line_tokens {
        if token.span.start < prev_end {
            continue;
        }
        let (line, col) = line_index.position(token.span.start, encoding);
        let (_, end_col) = line_index.position(token.span.end, encoding);
        let length = end_col.saturating_sub(col);
        if length == 0 {
            continue;
        }
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 { col - prev_col } else { col };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: token.kind,
            token_modifiers_bitset: token.modifiers,
        });
        prev_line = line;
        prev_col = col;
        prev_end = token.span.end;
    }
    data
}

fn count_units(text: &str, encoding: PositionEncoding) -> u32 {
    match encoding {
        PositionEncoding::Utf8 => text.len() as u32,
        PositionEncoding::Utf16 => text.chars().map(utf16_len).sum(),
    }
}

fn utf16_len(ch: char) -> u32 {
    if (ch as u32) > 0xFFFF { 2 } else { 1 }
}

struct LineIndex<'a> {
    src: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    fn new(src: &'a str) -> Self {
        let mut line_starts = vec![0];
        for (i, &b) in src.as_bytes().iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { src, line_starts }
    }

    #[inline]
    fn position(&self, offset: u32, encoding: PositionEncoding) -> (u32, u32) {
        let offset = floor_char_boundary(self.src, (offset as usize).min(self.src.len()));
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx];
        let col = count_units(&self.src[line_start..offset], encoding);
        (line_idx as u32, col)
    }
}

fn floor_char_boundary(src: &str, mut index: usize) -> usize {
    index = index.min(src.len());
    while index > 0 && !src.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn for_each_line_span(src: &str, span: Span, mut f: impl FnMut(Span)) {
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
