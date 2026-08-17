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

const MOD_DECLARATION: u32 = 1 << 0;
const MOD_DEFAULT_LIBRARY: u32 = 1 << 1;

pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::TYPE,
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::STRING,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::OPERATOR,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ],
    }
}

struct RawToken {
    span: Span,
    kind: u32,
    modifiers: u32,
}

struct Collector<'a> {
    src: &'a str,
    tokens: Vec<RawToken>,
}

impl<'a> Collector<'a> {
    fn token(&mut self, span: Span, kind: u32, modifiers: u32) {
        if !span.is_empty() {
            self.tokens.push(RawToken {
                span,
                kind,
                modifiers,
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
    finish_tokens(name, text, encoding, range, |collector| {
        collect_document(collector, document);
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
    finish_tokens(name, text, encoding, range, |collector| {
        collect_rocdown(collector, document, headings);
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
            collector.token(span, TOKEN_KEYWORD, 0);
        }
    }
    for item in &document.items {
        match item {
            rocci_rocdown::Item::Markdown(_) => {}
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
        }
    }
}

fn collect_docs(collector: &mut Collector<'_>, docs: &rocci_rocdown::DocsDecl) {
    collect_keyword(collector, docs.span, docs.kind_span.start, "@docs");
    collector.token(docs.kind_span, TOKEN_TYPE, 0);
    let (fields, content) = rocci_rocdown::split_docs_body(collector.src, docs.body);
    for field in fields {
        collector.token(field.name_span, TOKEN_PROPERTY, 0);
        let val_str = field.value.of(collector.src).trim();
        if val_str.starts_with('"') {
            collector.token(field.value, TOKEN_STRING, 0);
        } else if val_str == "true"
            || val_str == "false"
            || val_str == "Bool.true"
            || val_str == "Bool.false"
        {
            collector.token(field.value, TOKEN_KEYWORD, 0);
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
        collector.token(keyword, TOKEN_KEYWORD, 0);
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
        collector.token(span, TOKEN_KEYWORD, 0);
    }
}

fn collect_context(collector: &mut Collector<'_>, context: &ContextDecl) {
    if let Some(span) = ident_between(
        collector.src,
        context.span.start,
        context.ty.start,
        "@context",
    ) {
        collector.token(span, TOKEN_KEYWORD, 0);
    }
}

fn collect_init(collector: &mut Collector<'_>, init: &InitDecl) {
    if let Some(span) = ident_between(collector.src, init.span.start, init.body.start, "@init") {
        collector.token(span, TOKEN_KEYWORD, 0);
    }
}

fn collect_on(collector: &mut Collector<'_>, on: &OnDecl) {
    if let Some(span) = ident_between(collector.src, on.span.start, on.method.span.start, "@on") {
        collector.token(span, TOKEN_KEYWORD, 0);
    }
    collector.token(on.method.span, TOKEN_KEYWORD, 0);
    collector.token(on.path_span, TOKEN_STRING, 0);
}

fn collect_component(collector: &mut Collector<'_>, component: &ComponentDecl) {
    collector.token(component.name.span, TOKEN_FUNCTION, MOD_DECLARATION);
    if let Some(span) = ident_between(
        collector.src,
        component.span.start,
        component.name.span.start,
        "@component",
    ) {
        collector.token(span, TOKEN_KEYWORD, 0);
    }
    collect_items(collector, &component.body.items);
}

fn collect_fixture(collector: &mut Collector<'_>, fixture: &FixtureDecl) {
    collector.token(fixture.name.span, TOKEN_FUNCTION, MOD_DECLARATION);
    if let Some(span) = ident_between(
        collector.src,
        fixture.span.start,
        fixture.name.span.start,
        "@fixture",
    ) {
        collector.token(span, TOKEN_KEYWORD, 0);
    }
    if let Some(span) = ident_between(
        collector.src,
        fixture.span.start,
        fixture.name.span.start,
        "target",
    ) {
        collector.token(span, TOKEN_PROPERTY, 0);
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
                collector.token(directive_keyword(dir.span, "if"), TOKEN_KEYWORD, 0);
                collect_items(collector, &dir.then_body.items);
                for (cond, body) in &dir.else_ifs {
                    if let Some(span) = else_if_keyword(collector.src, cond.start) {
                        collector.token(span, TOKEN_KEYWORD, 0);
                    }
                    collect_items(collector, &body.items);
                }
                if let Some(body) = &dir.else_body {
                    if let Some(span) = keyword_before(collector.src, body.span.start, "@else") {
                        collector.token(span, TOKEN_KEYWORD, 0);
                    }
                    collect_items(collector, &body.items);
                }
            }
            TemplateItem::For(dir) => {
                collector.token(directive_keyword(dir.span, "for"), TOKEN_KEYWORD, 0);
                collector.token(dir.binder.span, TOKEN_PARAMETER, MOD_DECLARATION);
                if let Some(span) = ident_between(
                    collector.src,
                    dir.binder.span.end,
                    dir.collection.start,
                    "in",
                ) {
                    collector.token(span, TOKEN_KEYWORD, 0);
                }
                collect_items(collector, &dir.body.items);
            }
            TemplateItem::Match(dir) => {
                collector.token(directive_keyword(dir.span, "match"), TOKEN_KEYWORD, 0);
                for arm in &dir.arms {
                    if let Some(span) =
                        ident_between(collector.src, arm.pattern.end, arm.value.span().start, "=>")
                    {
                        collector.token(span, TOKEN_OPERATOR, 0);
                    }
                    collect_items(collector, std::slice::from_ref(&*arm.value));
                }
            }
            TemplateItem::Let(dir) => {
                collector.token(directive_keyword(dir.span, "let"), TOKEN_KEYWORD, 0);
                collector.token(dir.binder.span, TOKEN_PARAMETER, MOD_DECLARATION);
            }
            TemplateItem::Css(css) => collect_css(collector, css),
            TemplateItem::Text(_) => {}
        }
    }
}

fn collect_element(collector: &mut Collector<'_>, el: &Element) {
    collector.token(el.name.span, TOKEN_TYPE, MOD_DEFAULT_LIBRARY);
    collect_attrs(collector, &el.attrs);
    collect_items(collector, &el.children);
    if !el.self_closing
        && let Some(span) = closing_name(collector.src, el.span, &el.name.name)
    {
        collector.token(span, TOKEN_TYPE, MOD_DEFAULT_LIBRARY);
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
        collector.token(part.span, kind, 0);
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
        collector.token(Span::new(start, start + part.name.len()), kind, 0);
        start += part.name.len() + 1;
    }
}

fn collect_attrs(collector: &mut Collector<'_>, attrs: &[rocci_template::Attr]) {
    for attr in attrs {
        collector.token(attr.name.span, TOKEN_PROPERTY, 0);
        match &attr.value {
            AttrValue::Static { span, .. } => collector.token(*span, TOKEN_STRING, 0),
            AttrValue::Expr { .. } => {}
            AttrValue::Action { name, .. } => {
                let at_start = (name.span.start as usize).saturating_sub(1);
                collector.token(
                    Span::new(at_start, name.span.end as usize),
                    TOKEN_KEYWORD,
                    0,
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
            });
        });
    }
    line_tokens.sort_by_key(|token| (token.span.start, token.span.end, token.kind));
    let mut data = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;
    let mut prev_end = 0u32;
    for token in line_tokens {
        if token.span.start < prev_end {
            continue;
        }
        let (line, col) = source.position(token.span.start, encoding);
        let (_, end_col) = source.position(token.span.end, encoding);
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
    let start = span.start as usize;
    let end = (span.end as usize).min(src.len());
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
