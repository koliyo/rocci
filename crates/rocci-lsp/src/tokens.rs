use lsp_types::{
    Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend,
};
use rocci_template::{
    AttrValue, ComponentCall, ComponentDecl, ContextDecl, CssDecl, Document, Element, FixtureDecl,
    InitDecl, ModuleItem, OnDecl, PositionEncoding, SourceFile, Span, TemplateItem,
};

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

pub const MOD_DECLARATION: u32 = 1 << 0;
pub const MOD_DEFAULT_LIBRARY: u32 = 1 << 1;
pub const MOD_READONLY: u32 = 1 << 2;
pub const MOD_DOCUMENTATION: u32 = 1 << 3;

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
    let regions = crate::regions::extract_rocci_regions(name, text, document);
    finish_tokens(name, text, encoding, range, |collector| {
        collect_document(collector, document);
        collect_embedded_regions(collector, &regions);
    })
}

pub fn semantic_tokens_rocdown(
    name: &str,
    text: &str,
    document: &rocci_rocdown::Document,
    headings: &[rocci_rocdown::HeadingInfo],
    encoding: PositionEncoding,
    range: Option<Range>,
) -> SemanticTokens {
    let regions = crate::regions::extract_rocdown_regions(name, text, document, headings);
    finish_tokens(name, text, encoding, range, |collector| {
        collect_rocdown(collector, document, headings);
        collect_embedded_regions(collector, &regions);
    })
}

fn finish_tokens(
    name: &str,
    text: &str,
    encoding: PositionEncoding,
    range: Option<Range>,
    collect: impl FnOnce(&mut Collector<'_>),
) -> SemanticTokens {
    let source = SourceFile::new(name, text);
    let mut collector = Collector {
        src: text,
        tokens: Vec::new(),
    };
    collect(&mut collector);
    let range_span = range.map(|range| {
        Span::new(
            source.offset_at(range.start.line, range.start.character, encoding) as usize,
            source.offset_at(range.end.line, range.end.character, encoding) as usize,
        )
    });
    SemanticTokens {
        result_id: None,
        data: encode_tokens(source, &mut collector.tokens, encoding, range_span),
    }
}

fn collect_embedded_regions(collector: &mut Collector<'_>, regions: &crate::regions::RegionTree) {
    for region in &regions.regions {
        let region_start = floor_char_boundary(collector.src, region.span.start as usize);
        let region_end = floor_char_boundary(
            collector.src,
            (region.span.end as usize).min(collector.src.len()),
        );
        if region_start >= region_end {
            continue;
        }
        let slice = &collector.src[region_start..region_end];

        match region.language {
            crate::regions::Language::Roc => {
                let hl_tokens = crate::embedded::roc::highlight(slice);
                for tok in hl_tokens {
                    let tok_start =
                        floor_char_boundary(collector.src, region_start + tok.span.start as usize);
                    let tok_end = floor_char_boundary(
                        collector.src,
                        (region_start + tok.span.end as usize).min(region_end),
                    );
                    if tok_start < tok_end {
                        collector.token_with_priority(
                            Span::new(tok_start, tok_end),
                            tok.kind,
                            tok.modifiers,
                            tok.priority,
                        );
                    }
                }
            }
            crate::regions::Language::Css => {
                let hl_tokens = crate::embedded::css::highlight(slice);
                for tok in hl_tokens {
                    let tok_start =
                        floor_char_boundary(collector.src, region_start + tok.span.start as usize);
                    let tok_end = floor_char_boundary(
                        collector.src,
                        (region_start + tok.span.end as usize).min(region_end),
                    );
                    if tok_start < tok_end {
                        collector.token_with_priority(
                            Span::new(tok_start, tok_end),
                            tok.kind,
                            tok.modifiers,
                            tok.priority,
                        );
                    }
                }
            }
            crate::regions::Language::Html => {
                if region.purpose == crate::regions::RegionPurpose::DisplayOnly {
                    let hl_tokens = crate::embedded::html::highlight(slice);
                    for tok in hl_tokens {
                        let tok_start = floor_char_boundary(
                            collector.src,
                            region_start + tok.span.start as usize,
                        );
                        let tok_end = floor_char_boundary(
                            collector.src,
                            (region_start + tok.span.end as usize).min(region_end),
                        );
                        if tok_start < tok_end {
                            collector.token_with_priority(
                                Span::new(tok_start, tok_end),
                                tok.kind,
                                tok.modifiers,
                                tok.priority,
                            );
                        }
                    }
                }
            }
            crate::regions::Language::Markdown
            | crate::regions::Language::RocciTemplate
            | crate::regions::Language::Other(_) => {}
        }
    }
}

fn collect_document(collector: &mut Collector<'_>, document: &Document) {
    for item in &document.items {
        match item {
            ModuleItem::Roc { .. } => {}
            ModuleItem::Component(component) => collect_component(collector, component),
            ModuleItem::Fixture(fixture) => collect_fixture(collector, fixture),
            ModuleItem::Css(css) => collect_css(collector, css),
            ModuleItem::Context(context) => collect_context(collector, context),
            ModuleItem::Init(init) => collect_init(collector, init),
            ModuleItem::On(on) => collect_on(collector, on),
        }
    }
}

fn collect_rocdown(
    collector: &mut Collector<'_>,
    document: &rocci_rocdown::Document,
    headings: &[rocci_rocdown::HeadingInfo],
) {
    for heading in headings {
        if let Some(span) = heading_marker(collector.src, heading.span, heading.level) {
            collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
        }
    }
    for item in &document.items {
        match item {
            rocci_rocdown::Item::Markdown(md_node) => {
                let mut md_tokens = Vec::new();
                crate::embedded::markdown::collect_markdown_node(
                    collector.src,
                    md_node,
                    &mut md_tokens,
                );
                for tok in md_tokens {
                    collector.token_with_priority(tok.span, tok.kind, tok.modifiers, tok.priority);
                }
            }
            rocci_rocdown::Item::Page(page) => {
                collect_keyword(collector, page.span, page.body.start, "@page");
            }
            rocci_rocdown::Item::Roc(roc) => {
                collect_keyword(collector, roc.span, roc.body.start, "@roc");
            }
            rocci_rocdown::Item::Render(render) => {
                collect_keyword(collector, render.span, render.expr.start, "@render");
            }
            rocci_rocdown::Item::Component(component) => collect_component(collector, component),
            rocci_rocdown::Item::Fixture(fixture) => collect_fixture(collector, fixture),
            rocci_rocdown::Item::Css(css) => collect_css(collector, css),
            rocci_rocdown::Item::Context(context) => collect_context(collector, context),
            rocci_rocdown::Item::Init(init) => collect_init(collector, init),
            rocci_rocdown::Item::On(on) => collect_on(collector, on),
            rocci_rocdown::Item::Template(item) => {
                collect_items(collector, std::slice::from_ref(item))
            }
            rocci_rocdown::Item::Docs(docs) => {
                collect_docs(collector, docs);
            }
            rocci_rocdown::Item::Img(img) => {
                collect_img(collector, img);
            }
        }
    }
}

fn collect_img(collector: &mut Collector<'_>, img: &rocci_rocdown::ImgDecl) {
    collect_keyword(collector, img.span, img.body.start, "@img");
    let mut cur = rocci_template::Cursor::at(collector.src, img.body.start as usize);
    let end = img.body.end as usize;
    while cur.pos < end && !cur.is_eof() {
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let Some(name_span) = cur.scan_ident() else {
            break;
        };
        collector.token_with_priority(name_span, TOKEN_PROPERTY, 0, 50);
        cur.skip_trivia();
        if !cur.eat(':') {
            break;
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        if cur.peek() == Some('"') {
            cur.skip_string();
            let value_span = Span::new(value_start, cur.pos.min(end));
            collector.token_with_priority(value_span, TOKEN_STRING, 0, 50);
        }
    }
}

fn collect_docs(collector: &mut Collector<'_>, docs: &rocci_rocdown::DocsDecl) {
    collect_keyword(collector, docs.span, docs.kind_span.start, "@docs");
    collector.token_with_priority(docs.kind_span, TOKEN_TYPE, 0, 55);
    let (fields, content) = rocci_rocdown::split_docs_body(collector.src, docs.body);
    for field in fields {
        collector.token_with_priority(field.name_span, TOKEN_PROPERTY, 0, 50);
        let val_str = field.value.of(collector.src).trim();
        if val_str.starts_with('"') {
            collector.token_with_priority(field.value, TOKEN_STRING, 0, 50);
        } else if val_str == "true"
            || val_str == "false"
            || val_str == "Bool.true"
            || val_str == "Bool.false"
        {
            collector.token_with_priority(field.value, TOKEN_KEYWORD, 0, 50);
        }
    }
    if !content.is_empty() && (content.start as usize) < collector.src.len() {
        let source = SourceFile::new("docs", collector.src);
        let parsed = rocci_rocdown::parse_fragment(source, content, false);
        collect_rocdown(collector, &parsed.document, &parsed.headings);
    }
}

fn collect_keyword(collector: &mut Collector<'_>, span: Span, before: u32, word: &str) {
    if let Some(keyword) = ident_between(collector.src, span.start, before, word) {
        collector.token_with_priority(keyword, TOKEN_KEYWORD, 0, 55);
    }
}

fn heading_marker(src: &str, span: Span, level: u8) -> Option<Span> {
    let text = span.of(src);
    let indent = text.len() - text.trim_start_matches([' ', '\t']).len();
    let trimmed = &text[indent..];
    let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
    if hashes == 0 {
        return None;
    }
    let start = span.start as usize + indent;
    Some(Span::new(start, start + hashes.min(level as usize)))
}

fn collect_css(collector: &mut Collector<'_>, css: &CssDecl) {
    if let Some(span) = ident_between(collector.src, css.span.start, css.body.start, "@css") {
        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
    }
}

fn collect_context(collector: &mut Collector<'_>, context: &ContextDecl) {
    if let Some(span) = ident_between(
        collector.src,
        context.span.start,
        context.ty.start,
        "@context",
    ) {
        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
    }
}

fn collect_init(collector: &mut Collector<'_>, init: &InitDecl) {
    if let Some(span) = ident_between(collector.src, init.span.start, init.body.start, "@init") {
        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
    }
}

fn collect_on(collector: &mut Collector<'_>, on: &OnDecl) {
    if let Some(span) = ident_between(collector.src, on.span.start, on.method.span.start, "@on") {
        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
    }
    collector.token_with_priority(on.method.span, TOKEN_KEYWORD, 0, 50);
    collector.token_with_priority(on.path_span, TOKEN_STRING, 0, 50);
}

fn collect_component(collector: &mut Collector<'_>, component: &ComponentDecl) {
    collector.token_with_priority(component.name.span, TOKEN_FUNCTION, MOD_DECLARATION, 55);
    if let Some(span) = ident_between(
        collector.src,
        component.span.start,
        component.name.span.start,
        "@component",
    ) {
        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
    }
    collect_items(collector, &component.body.items);
}

fn collect_fixture(collector: &mut Collector<'_>, fixture: &FixtureDecl) {
    collector.token_with_priority(fixture.name.span, TOKEN_FUNCTION, MOD_DECLARATION, 55);
    if let Some(span) = ident_between(
        collector.src,
        fixture.span.start,
        fixture.name.span.start,
        "@fixture",
    ) {
        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
    }
    if let Some(span) = ident_between(
        collector.src,
        fixture.span.start,
        fixture.name.span.start,
        "target",
    ) {
        collector.token_with_priority(span, TOKEN_PROPERTY, 0, 50);
    }
    collect_path(collector, &fixture.target.parts);
}

fn collect_items(collector: &mut Collector<'_>, items: &[TemplateItem]) {
    for item in items {
        match item {
            TemplateItem::Element(el) => collect_element(collector, el),
            TemplateItem::ComponentCall(call) => collect_call(collector, call),
            TemplateItem::Fragment(frag) => collect_items(collector, &frag.children),
            TemplateItem::Interpolation(_) => {}
            TemplateItem::If(dir) => {
                collector.token_with_priority(
                    directive_keyword(dir.span, "if"),
                    TOKEN_KEYWORD,
                    0,
                    55,
                );
                collect_items(collector, &dir.then_body.items);
                for (cond, body) in &dir.else_ifs {
                    if let Some(span) = else_if_keyword(collector.src, cond.start) {
                        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
                    }
                    collect_items(collector, &body.items);
                }
                if let Some(body) = &dir.else_body {
                    if let Some(span) = keyword_before(collector.src, body.span.start, "@else") {
                        collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
                    }
                    collect_items(collector, &body.items);
                }
            }
            TemplateItem::For(dir) => {
                collector.token_with_priority(
                    directive_keyword(dir.span, "for"),
                    TOKEN_KEYWORD,
                    0,
                    55,
                );
                collector.token_with_priority(
                    dir.binder.span,
                    TOKEN_PARAMETER,
                    MOD_DECLARATION,
                    50,
                );
                if let Some(span) = ident_between(
                    collector.src,
                    dir.binder.span.end,
                    dir.collection.start,
                    "in",
                ) {
                    collector.token_with_priority(span, TOKEN_KEYWORD, 0, 55);
                }
                collect_items(collector, &dir.body.items);
            }
            TemplateItem::Match(dir) => {
                collector.token_with_priority(
                    directive_keyword(dir.span, "match"),
                    TOKEN_KEYWORD,
                    0,
                    55,
                );
                for arm in &dir.arms {
                    if let Some(span) =
                        ident_between(collector.src, arm.pattern.end, arm.value.span().start, "=>")
                    {
                        collector.token_with_priority(span, TOKEN_OPERATOR, 0, 50);
                    }
                    collect_items(collector, std::slice::from_ref(&*arm.value));
                }
            }
            TemplateItem::Let(dir) => {
                collector.token_with_priority(
                    directive_keyword(dir.span, "let"),
                    TOKEN_KEYWORD,
                    0,
                    55,
                );
                collector.token_with_priority(
                    dir.binder.span,
                    TOKEN_PARAMETER,
                    MOD_DECLARATION,
                    50,
                );
            }
            TemplateItem::Css(css) => collect_css(collector, css),
            TemplateItem::Text(_) => {}
        }
    }
}

fn collect_element(collector: &mut Collector<'_>, el: &Element) {
    collector.token_with_priority(el.name.span, TOKEN_TYPE, MOD_DEFAULT_LIBRARY, 55);
    collect_attrs(collector, &el.attrs);
    collect_items(collector, &el.children);
    if !el.self_closing
        && let Some(span) = closing_name(collector.src, el.span, &el.name.name)
    {
        collector.token_with_priority(span, TOKEN_TYPE, MOD_DEFAULT_LIBRARY, 55);
    }
}

fn collect_call(collector: &mut Collector<'_>, call: &ComponentCall) {
    collect_path(collector, &call.path.parts);
    collect_attrs(collector, &call.attrs);
    if let Some(children) = &call.children {
        collect_items(collector, children);
        let path = path_source(&call.path.parts);
        if let Some(span) = closing_name(collector.src, call.span, &path) {
            collect_path_at(collector, span, &call.path.parts);
        }
    }
}

fn collect_path(collector: &mut Collector<'_>, parts: &[rocci_template::Ident]) {
    for (i, part) in parts.iter().enumerate() {
        let kind = if i + 1 == parts.len() {
            TOKEN_FUNCTION
        } else {
            TOKEN_NAMESPACE
        };
        collector.token_with_priority(part.span, kind, 0, 50);
    }
}

fn collect_path_at(collector: &mut Collector<'_>, span: Span, parts: &[rocci_template::Ident]) {
    let mut start = span.start as usize;
    for (i, part) in parts.iter().enumerate() {
        let kind = if i + 1 == parts.len() {
            TOKEN_FUNCTION
        } else {
            TOKEN_NAMESPACE
        };
        collector.token_with_priority(Span::new(start, start + part.name.len()), kind, 0, 50);
        start += part.name.len() + 1;
    }
}

fn collect_attrs(collector: &mut Collector<'_>, attrs: &[rocci_template::Attr]) {
    for attr in attrs {
        collector.token_with_priority(attr.name.span, TOKEN_PROPERTY, 0, 50);
        match &attr.value {
            AttrValue::Static { span, .. } => {
                collector.token_with_priority(*span, TOKEN_STRING, 0, 50)
            }
            AttrValue::Expr { .. } => {}
            AttrValue::Action { name, .. } => {
                let at_start = (name.span.start as usize).saturating_sub(1);
                collector.token_with_priority(
                    Span::new(at_start, name.span.end as usize),
                    TOKEN_KEYWORD,
                    0,
                    55,
                );
            }
            AttrValue::Boolean => {}
        }
    }
}

fn path_source(parts: &[rocci_template::Ident]) -> String {
    parts
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn directive_keyword(span: Span, name: &str) -> Span {
    Span::new(span.start as usize, span.start as usize + 1 + name.len())
}

fn closing_name(src: &str, span: Span, name: &str) -> Option<Span> {
    let text = span.of(src);
    let needle = format!("</{name}");
    let idx = text.rfind(&needle)?;
    let start = span.start as usize + idx + 2;
    Some(Span::new(start, start + name.len()))
}

fn ident_between(src: &str, start: u32, end: u32, word: &str) -> Option<Span> {
    let from = start as usize;
    let to = (end as usize).min(src.len());
    if from >= to {
        return None;
    }
    let hay = &src[from..to];
    let mut search = 0;
    while let Some(rel) = hay[search..].find(word) {
        let i = search + rel;
        let before = if i == 0 {
            true
        } else {
            !is_ident_char(hay[..i].chars().next_back().unwrap())
        };
        let after_i = i + word.len();
        let after = hay[after_i..]
            .chars()
            .next()
            .is_none_or(|ch| !is_ident_char(ch));
        if before && after {
            return Some(Span::new(from + i, from + after_i));
        }
        search = i + 1;
    }
    None
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn else_if_keyword(src: &str, before: u32) -> Option<Span> {
    let mut i = skip_ws_back(src, before as usize);
    i = match_back(src, i, "if")?;
    i = skip_ws_back(src, i);
    i = match_back(src, i, "else")?;
    i = match_back(src, i, "@")?;
    Some(Span::new(i, before as usize))
}

fn keyword_before(src: &str, before: u32, keyword: &str) -> Option<Span> {
    let i = skip_ws_back(src, before as usize);
    let start = match_back(src, i, keyword)?;
    Some(Span::new(start, i))
}

fn skip_ws_back(src: &str, mut i: usize) -> usize {
    while i > 0 {
        let ch = src[..i].chars().next_back().unwrap();
        if !ch.is_whitespace() {
            break;
        }
        i -= ch.len_utf8();
    }
    i
}

fn match_back(src: &str, end: usize, word: &str) -> Option<usize> {
    let start = end.checked_sub(word.len())?;
    if src.get(start..end) == Some(word) {
        Some(start)
    } else {
        None
    }
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

fn count_units(text: &str, encoding: PositionEncoding) -> u32 {
    match encoding {
        PositionEncoding::Utf8 => text.len() as u32,
        PositionEncoding::Utf16 => text.chars().map(utf16_len).sum(),
    }
}

fn utf16_len(ch: char) -> u32 {
    if (ch as u32) > 0xFFFF { 2 } else { 1 }
}

fn encode_tokens(
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
