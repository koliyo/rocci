use rocci_template::{
    Cursor, Diagnostic, Span, TemplateItem, is_ident_continue, is_ident_start,
    parse_declaration_from, parse_template_item_from,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reserved {
    Page,
    Roc,
    Render,
    Component,
    Fixture,
    Css,
    Context,
    Init,
    Live,
    View,
    Patch,
    Command,
    On,
    Use,
    If,
    For,
    Match,
    Let,
}

impl Reserved {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "page" => Self::Page,
            "roc" => Self::Roc,
            "render" => Self::Render,
            "component" => Self::Component,
            "fixture" => Self::Fixture,
            "css" => Self::Css,
            "context" => Self::Context,
            "init" => Self::Init,
            "live" => Self::Live,
            "view" => Self::View,
            "patch" => Self::Patch,
            "command" => Self::Command,
            "on" => Self::On,
            "use" => Self::Use,
            "if" => Self::If,
            "for" => Self::For,
            "match" => Self::Match,
            "let" => Self::Let,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Roc => "roc",
            Self::Render => "render",
            Self::Component => "component",
            Self::Fixture => "fixture",
            Self::Css => "css",
            Self::Context => "context",
            Self::Init => "init",
            Self::Live => "live",
            Self::View => "view",
            Self::Patch => "patch",
            Self::Command => "command",
            Self::On => "on",
            Self::Use => "use",
            Self::If => "if",
            Self::For => "for",
            Self::Match => "match",
            Self::Let => "let",
        }
    }

    fn is_rocci(self) -> bool {
        matches!(
            self,
            Self::Component
                | Self::Fixture
                | Self::Css
                | Self::Context
                | Self::Init
                | Self::Live
                | Self::View
                | Self::Patch
                | Self::Command
                | Self::On
        )
    }

    fn is_template(self) -> bool {
        matches!(self, Self::If | Self::For | Self::Match | Self::Let)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScannedKind {
    At(Reserved),
    Html,
    Colon,
    ColonEnd,
    RemovedAt,
}

#[derive(Clone, Debug)]
pub struct ScannedDecl {
    pub kind: ScannedKind,
    pub line_start: usize,
    pub at: usize,
    pub end: usize,
}

pub fn bom_len(src: &str) -> usize {
    if src.starts_with('\u{FEFF}') {
        '\u{FEFF}'.len_utf8()
    } else {
        0
    }
}

pub fn scan(src: &str, diagnostics: &mut Vec<Diagnostic>) -> Vec<ScannedDecl> {
    let mut decls = Vec::new();
    let mut pos = bom_len(src);
    if src[pos..].contains('\u{FEFF}')
        && let Some(off) = src[pos..].find('\u{FEFF}')
    {
        diagnostics.push(Diagnostic::error(
            Span::new(pos + off, pos + off + '\u{FEFF}'.len_utf8()),
            "a UTF-8 BOM is only allowed at the start of a Rocdown file",
        ));
    }

    let mut fence: Option<(u8, usize)> = None;
    let mut list_tight = false;
    let mut quote_tight = false;

    while pos < src.len() {
        let line_start = pos;
        let nl = src[pos..].find('\n').map(|i| pos + i);
        let line_end = nl.unwrap_or(src.len());
        let line = &src[line_start..line_end];
        let next = nl.map(|i| i + 1).unwrap_or(src.len());

        if let Some((ch, n)) = fence {
            if is_fence_close(line, ch, n) {
                fence = None;
            }
            pos = next;
            continue;
        }

        if line.trim().is_empty() {
            list_tight = false;
            quote_tight = false;
            pos = next;
            continue;
        }

        let stripped = skip_0_3_spaces(line);
        if stripped.starts_with('>') {
            quote_tight = true;
            pos = next;
            continue;
        }
        if looks_like_list_marker(stripped) {
            list_tight = true;
            pos = next;
            continue;
        }
        if quote_tight || list_tight {
            pos = next;
            continue;
        }

        if let Some(decl) = try_scan_decl(src, line_start, diagnostics) {
            pos = decl.end.max(next);
            decls.push(decl);
            list_tight = false;
            quote_tight = false;
            continue;
        }
        if let Some(decl) = try_scan_colon(src, line_start, diagnostics) {
            pos = decl.end.max(next);
            decls.push(decl);
            list_tight = false;
            quote_tight = false;
            continue;
        }
        if let Some(decl) = try_scan_html(src, line_start, diagnostics) {
            pos = decl.end.max(next);
            decls.push(decl);
            list_tight = false;
            quote_tight = false;
            continue;
        }

        if let Some(open) = fence_open(stripped) {
            fence = Some(open);
        }
        pos = next;
    }

    decls
}

fn try_scan_decl(
    src: &str,
    line_start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ScannedDecl> {
    let mut at = line_start;
    while at < src.len() && matches!(src.as_bytes().get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    if src.as_bytes().get(at) == Some(&b'\\') && src.as_bytes().get(at + 1) == Some(&b'@') {
        return None;
    }
    if src.as_bytes().get(at) != Some(&b'@') {
        return None;
    }
    let name_start = at + 1;
    let mut name_end = name_start;
    let bytes = src.as_bytes();
    if name_end >= src.len() || !is_ident_start(bytes[name_end] as char) {
        return None;
    }
    name_end += 1;
    while name_end < src.len() && is_ident_continue(bytes[name_end] as char) {
        name_end += 1;
    }
    let name = &src[name_start..name_end];
    if name == "docs" || name == "img" {
        let line_end = src[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(src.len());
        diagnostics.push(Diagnostic::error(
            Span::new(at, name_end),
            if name == "docs" {
                "`@docs` was removed; write `:note` (or another builtin kind) instead of `@docs <kind> { ... }`"
                    .to_string()
            } else {
                "`@img` was removed; write `:img[src: \"...\", alt: \"...\"]` instead of `@img { ... }`"
                    .to_string()
            },
        ));
        return Some(ScannedDecl {
            kind: ScannedKind::RemovedAt,
            line_start,
            at,
            end: line_end.max(name_end),
        });
    }
    let kind = Reserved::from_name(name)?;
    if !header_matches(src, name_end, kind) {
        return None;
    }

    let (end, mut extra) = if kind.is_rocci() {
        let parsed = parse_declaration_from(src, at)?;
        (parsed.end, parsed.diagnostics)
    } else if kind.is_template() {
        let parsed = parse_template_item_from(src, at)?;
        (parsed.end, parsed.diagnostics)
    } else if kind == Reserved::Use {
        skip_use_path(src, at)
    } else if kind == Reserved::Render {
        skip_render(src, at)
    } else {
        skip_brace_block(src, at, kind)
    };
    if !matches!(kind, Reserved::Let)
        && let Some(diag) = trailing_text(src, end)
    {
        extra.push(diag);
    }
    diagnostics.append(&mut extra);
    Some(ScannedDecl {
        kind: ScannedKind::At(kind),
        line_start,
        at,
        end,
    })
}

fn try_scan_colon(
    src: &str,
    line_start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ScannedDecl> {
    let mut at = line_start;
    while at < src.len() && matches!(src.as_bytes().get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    if src.as_bytes().get(at) == Some(&b'\\') && src.as_bytes().get(at + 1) == Some(&b':') {
        return None;
    }
    let header = colon_header_at(src, at)?;
    if let Some(kind) = header.removed_end_kind {
        diagnostics.push(Diagnostic::error(
            header.name_span,
            format!("`:end.{kind}` was removed; write `:{kind}.end`"),
        ));
        return Some(ScannedDecl {
            kind: ScannedKind::ColonEnd,
            line_start,
            at: header.at,
            end: line_end(src, line_start),
        });
    }
    if header.name == "end" {
        diagnostics.push(Diagnostic::error(
            header.name_span,
            "`:end` was removed; write `:kind.end`",
        ));
        return Some(ScannedDecl {
            kind: ScannedKind::ColonEnd,
            line_start,
            at: header.at,
            end: line_end(src, line_start),
        });
    }
    if let Some(suffix) = header.unknown_suffix {
        diagnostics.push(Diagnostic::error(
            header.name_span,
            format!(
                "unknown `:{}.{suffix}`; use `:{}.begin` or `:{}.end`",
                header.name, header.name, header.name
            ),
        ));
    }
    if header.suffix == Some(ColonSuffix::End) {
        diagnostics.push(Diagnostic::error(
            header.name_span,
            format!("unmatched `:{}.end`", header.name),
        ));
        return Some(ScannedDecl {
            kind: ScannedKind::ColonEnd,
            line_start,
            at: header.at,
            end: line_end(src, line_start),
        });
    }

    let mut pos = header.after_name;
    let (after_params, mut extra) = skip_optional_params(src, pos);
    diagnostics.append(&mut extra);
    pos = after_params;

    let mut cur = Cursor::at(src, pos);
    cur.skip_spaces_tabs();
    let began = header.suffix == Some(ColonSuffix::Begin);
    let end = if cur.starts_with("{{") {
        if began {
            diagnostics.push(Diagnostic::error(
                header.name_span,
                format!(
                    "`:{0}.begin` cannot mix with a double-brace body; write `:{0}.begin` ... `:{0}.end` or use double braces without `.begin`",
                    header.name
                ),
            ));
        }
        skip_article_section(src, cur.pos, diagnostics)
    } else if line_has_content(src, cur.pos) {
        if began {
            diagnostics.push(Diagnostic::error(
                header.name_span,
                format!(
                    "`:{0}.begin` cannot take line-scope content; write `:{0}.begin` ... `:{0}.end` or drop `.begin` for a one-line body",
                    header.name
                ),
            ));
        }
        line_end(src, cur.pos)
    } else if began {
        if let Some(close) = find_end_closer(src, next_line(src, header.at), header.name) {
            close
        } else {
            diagnostics.push(Diagnostic::error(
                header.name_span,
                format!(
                    "unclosed `:{}.begin`; expected `:{}.end`",
                    header.name, header.name
                ),
            ));
            cur.pos.max(header.after_name)
        }
    } else {
        cur.pos.max(header.after_name)
    };

    Some(ScannedDecl {
        kind: ScannedKind::Colon,
        line_start,
        at: header.at,
        end,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColonSuffix {
    Begin,
    End,
}

struct ColonHeader<'a> {
    at: usize,
    name: &'a str,
    name_span: Span,
    after_name: usize,
    suffix: Option<ColonSuffix>,
    unknown_suffix: Option<&'a str>,
    removed_end_kind: Option<&'a str>,
}

fn colon_header_at(src: &str, at: usize) -> Option<ColonHeader<'_>> {
    if src.as_bytes().get(at) != Some(&b':') {
        return None;
    }
    let mut cur = Cursor::at(src, at + 1);
    if cur
        .peek()
        .is_some_and(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r')
    {
        return None;
    }
    let name_span = cur.scan_tag_name()?;
    let name = name_span.of(src);
    let mut suffix = None;
    let mut unknown_suffix = None;
    let mut removed_end_kind = None;
    let mut after_name = cur.pos;
    if cur.peek() == Some('.') {
        cur.bump();
        if let Some(suffix_span) = cur.scan_tag_name() {
            after_name = cur.pos;
            match suffix_span.of(src) {
                "begin" => suffix = Some(ColonSuffix::Begin),
                "end" => suffix = Some(ColonSuffix::End),
                other if name == "end" => removed_end_kind = Some(other),
                other => unknown_suffix = Some(other),
            }
        }
    }
    Some(ColonHeader {
        at,
        name,
        name_span,
        after_name,
        suffix,
        unknown_suffix,
        removed_end_kind,
    })
}

fn skip_optional_params(src: &str, start: usize) -> (usize, Vec<Diagnostic>) {
    let mut cur = Cursor::at(src, start);
    cur.skip_spaces_tabs();
    if cur.peek() != Some('[') {
        return (start, Vec::new());
    }
    skip_bracket_params(src, cur.pos)
}

fn skip_bracket_params(src: &str, start: usize) -> (usize, Vec<Diagnostic>) {
    let mut cur = Cursor::at(src, start);
    let mut diagnostics = Vec::new();
    if !cur.eat('[') {
        return (start, diagnostics);
    }
    let open = start;
    let mut depth = 1;
    while !cur.is_eof() && depth > 0 {
        let before = cur.pos;
        match cur.peek() {
            Some('"') => cur.skip_string(),
            Some('[') => {
                cur.bump();
                depth += 1;
            }
            Some(']') => {
                cur.bump();
                depth -= 1;
            }
            Some(_) => {
                cur.bump();
            }
            None => break,
        }
        if cur.pos <= before {
            cur.bump();
        }
    }
    if depth > 0 {
        diagnostics.push(Diagnostic::error(
            Span::new(open, cur.pos),
            "unterminated `[` params; expected `]`",
        ));
        if cur.pos <= open {
            cur.bump();
        }
    }
    (cur.pos, diagnostics)
}

pub(crate) fn skip_article_section(
    src: &str,
    start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> usize {
    if !src.get(start..).is_some_and(|rest| rest.starts_with("{{")) {
        return start;
    }
    let open = start;
    let mut pos = start + 2;
    let mut depth = 1;
    let mut fence: Option<(u8, usize)> = None;
    while pos < src.len() && depth > 0 {
        let before = pos;
        let nl = src[pos..].find('\n').map(|i| pos + i);
        let line_end = nl.unwrap_or(src.len());
        let line = &src[pos..line_end];
        let next = nl.map(|i| i + 1).unwrap_or(src.len());

        if let Some((ch, n)) = fence {
            if is_fence_close(line, ch, n) {
                fence = None;
            }
            pos = next.max(before + 1);
            continue;
        }

        let stripped = skip_0_3_spaces(line);
        if let Some(open_fence) = fence_open(stripped) {
            fence = Some(open_fence);
            pos = next.max(before + 1);
            continue;
        }

        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() && depth > 0 {
            if bytes[i] == b'{' && bytes.get(i + 1) == Some(&b'{') {
                depth += 1;
                i += 2;
                continue;
            }
            if bytes[i] == b'}' && bytes.get(i + 1) == Some(&b'}') {
                depth -= 1;
                i += 2;
                continue;
            }
            i += 1;
        }
        if depth == 0 {
            pos += i;
            break;
        }
        pos = next.max(before + 1);
    }
    if depth > 0 {
        diagnostics.push(Diagnostic::error(
            Span::new(open, pos),
            "unterminated `{{` section; expected `}}`",
        ));
        if pos <= open {
            pos = (open + 1).min(src.len());
        }
    }
    pos
}

fn find_end_closer(src: &str, start: usize, kind: &str) -> Option<usize> {
    let mut pos = start;
    let mut depth = 1;
    let mut fence: Option<(u8, usize)> = None;
    while pos < src.len() && depth > 0 {
        let before = pos;
        let nl = src[pos..].find('\n').map(|i| pos + i);
        let line_end_at = nl.unwrap_or(src.len());
        let line = &src[pos..line_end_at];
        let next = nl.map(|i| i + 1).unwrap_or(src.len());

        if let Some((ch, n)) = fence {
            if is_fence_close(line, ch, n) {
                fence = None;
            }
            pos = next.max(before + 1);
            continue;
        }

        let stripped = skip_0_3_spaces(line);
        if let Some(open_fence) = fence_open(stripped) {
            fence = Some(open_fence);
            pos = next.max(before + 1);
            continue;
        }

        if let Some(header) = line_colon_header(src, pos) {
            if header.suffix == Some(ColonSuffix::End) && header.name == kind {
                depth -= 1;
                if depth == 0 {
                    return Some(line_end_at);
                }
            } else if header.suffix == Some(ColonSuffix::Begin) && header.name == kind {
                depth += 1;
            }
        }
        pos = next.max(before + 1);
    }
    None
}

fn line_colon_header(src: &str, line_start: usize) -> Option<ColonHeader<'_>> {
    let mut at = line_start;
    while at < src.len() && matches!(src.as_bytes().get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    colon_header_at(src, at)
}

fn line_has_content(src: &str, pos: usize) -> bool {
    let rest = src.get(pos..).unwrap_or("");
    let line = rest.split('\n').next().unwrap_or(rest);
    line.chars().any(|ch| !ch.is_whitespace())
}

fn line_end(src: &str, pos: usize) -> usize {
    match src[pos.min(src.len())..].find('\n') {
        Some(i) => pos + i,
        None => src.len(),
    }
}

fn next_line(src: &str, pos: usize) -> usize {
    match src[pos.min(src.len())..].find('\n') {
        Some(i) => pos + i + 1,
        None => src.len(),
    }
}

fn try_scan_html(
    src: &str,
    line_start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ScannedDecl> {
    let mut at = line_start;
    while at < src.len() && matches!(src.as_bytes().get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    if src.as_bytes().get(at) != Some(&b'<') {
        return None;
    }
    let after = at + 1;
    let next = src[after..].chars().next()?;
    if next == '/' || next == '!' || next == '?' {
        return None;
    }
    if next != '>' {
        if !is_ident_start(next) {
            return None;
        }
        let mut name_end = after + next.len_utf8();
        while name_end < src.len() {
            let Some(ch) = src[name_end..].chars().next() else {
                break;
            };
            if !is_ident_continue(ch) {
                break;
            }
            name_end += ch.len_utf8();
        }
        if matches!(src[name_end..].chars().next(), Some(':' | '@')) {
            return None;
        }
    }
    let parsed = parse_template_item_from(src, at)?;
    if !matches!(
        parsed.item,
        TemplateItem::Element(_) | TemplateItem::ComponentCall(_) | TemplateItem::Fragment(_)
    ) {
        return None;
    }
    diagnostics.extend(parsed.diagnostics);
    Some(ScannedDecl {
        kind: ScannedKind::Html,
        line_start,
        at,
        end: parsed.end,
    })
}

fn header_matches(src: &str, after_name: usize, kind: Reserved) -> bool {
    let mut cur = Cursor::at(src, after_name);
    match kind {
        Reserved::On => {
            cur.skip_trivia();
            cur.peek() == Some(':')
        }
        Reserved::View => {
            cur.skip_trivia();
            cur.peek() == Some('(')
        }
        Reserved::Patch | Reserved::Command => {
            cur.skip_trivia();
            matches!(cur.peek(), Some('(') | Some(':'))
        }
        Reserved::Component => {
            cur.skip_trivia();
            cur.peek().is_some_and(is_ident_start)
        }
        Reserved::Fixture | Reserved::Context => {
            cur.skip_trivia();
            matches!(cur.peek(), Some('{')) || cur.peek().is_some_and(is_ident_start)
        }
        Reserved::Live => {
            cur.skip_trivia();
            matches!(cur.peek(), Some('=') | Some('{'))
        }
        Reserved::Page | Reserved::Roc | Reserved::Css | Reserved::Init => {
            cur.skip_trivia();
            cur.peek() == Some('{')
        }
        Reserved::Render => header_matches_render(src, after_name),
        Reserved::Use => {
            cur.skip_trivia();
            cur.peek() == Some('"')
        }
        Reserved::If | Reserved::Match => header_has_body_brace(src, after_name),
        Reserved::For => header_matches_for(src, after_name),
        Reserved::Let => header_matches_let(src, after_name),
    }
}

fn header_has_body_brace(src: &str, after_name: usize) -> bool {
    let mut cur = Cursor::at(src, after_name);
    cur.skip_spaces_tabs();
    let mut paren = 0usize;
    let mut bracket = 0usize;
    while !cur.is_eof() {
        match cur.peek() {
            Some('"') => cur.skip_string(),
            Some('#') => return false,
            Some('(') => {
                cur.bump();
                paren += 1;
            }
            Some(')') => {
                cur.bump();
                paren = paren.saturating_sub(1);
            }
            Some('[') => {
                cur.bump();
                bracket += 1;
            }
            Some(']') => {
                cur.bump();
                bracket = bracket.saturating_sub(1);
            }
            Some('{') if paren == 0 && bracket == 0 => return true,
            Some('{') => cur.skip_balanced_braces(),
            Some('\n' | '\r') if paren == 0 && bracket == 0 => return false,
            Some(_) => {
                cur.bump();
            }
            None => return false,
        }
    }
    false
}

fn header_matches_for(src: &str, after_name: usize) -> bool {
    let mut cur = Cursor::at(src, after_name);
    cur.skip_spaces_tabs();
    if cur.scan_ident().is_none() {
        return false;
    }
    cur.skip_spaces_tabs();
    if !cur.eat_str("in") {
        return false;
    }
    let after_in = cur.pos;
    if cur.peek().is_some_and(is_ident_continue) {
        return false;
    }
    header_has_body_brace(src, after_in)
}

fn header_matches_let(src: &str, after_name: usize) -> bool {
    let mut cur = Cursor::at(src, after_name);
    cur.skip_spaces_tabs();
    if cur.scan_ident().is_none() {
        return false;
    }
    cur.skip_spaces_tabs();
    cur.peek() == Some('=')
}

fn header_matches_render(src: &str, after_name: usize) -> bool {
    let mut cur = Cursor::at(src, after_name);
    cur.skip_trivia();
    matches!(cur.peek(), Some('{') | Some('<')) || cur.peek().is_some_and(is_ident_start)
}

fn skip_render(src: &str, at: usize) -> (usize, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut cur = Cursor::at(src, at);
    cur.eat('@');
    cur.scan_ident();
    cur.skip_trivia();
    if cur.peek() == Some('{') {
        let brace_start = cur.pos;
        cur.skip_balanced_braces();
        if cur.pos <= brace_start || src.as_bytes().get(cur.pos.saturating_sub(1)) != Some(&b'}') {
            diagnostics.push(Diagnostic::error(
                Span::new(brace_start, cur.pos),
                "unterminated `@render` block; expected `}`",
            ));
        }
        return (cur.pos, diagnostics);
    }
    if cur.peek() == Some('<') {
        while !cur.is_eof() && !matches!(cur.peek(), Some('\n')) {
            let before = cur.pos;
            cur.bump();
            if cur.pos == before {
                break;
            }
        }
        return (cur.pos, diagnostics);
    }
    if cur.scan_ident().is_none() {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            "expected a PascalCase component after `@render`",
        ));
        if !cur.is_eof() {
            cur.bump();
        }
        return (cur.pos, diagnostics);
    }
    while cur.eat('.') {
        if cur.scan_ident().is_none() {
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected identifier after `.`",
            ));
            break;
        }
    }
    cur.skip_trivia();
    if cur.peek() != Some('(') {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            "expected `(` after `@render` target; write `@render MyComponent({ ... })`",
        ));
        return (cur.pos, diagnostics);
    }
    let paren_start = cur.pos;
    cur.skip_balanced_parens();
    if cur.pos <= paren_start || src.as_bytes().get(cur.pos.saturating_sub(1)) != Some(&b')') {
        diagnostics.push(Diagnostic::error(
            Span::new(paren_start, cur.pos),
            "unterminated `@render` call; expected `)`",
        ));
    }
    (cur.pos, diagnostics)
}

fn skip_use_path(src: &str, at: usize) -> (usize, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut cur = Cursor::at(src, at);
    cur.eat('@');
    cur.scan_ident();
    cur.skip_trivia();
    if cur.peek() != Some('"') {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            "expected a string path after `@use`",
        ));
        if !cur.is_eof() {
            cur.bump();
        }
        return (cur.pos, diagnostics);
    }
    let start = cur.pos;
    cur.skip_string();
    if cur.pos <= start || src.as_bytes().get(cur.pos.saturating_sub(1)) != Some(&b'"') {
        diagnostics.push(Diagnostic::error(
            Span::new(start, cur.pos),
            "unterminated `@use` path; expected `\"`",
        ));
        if cur.pos == start && !cur.is_eof() {
            cur.bump();
        }
    }
    (cur.pos, diagnostics)
}

fn skip_brace_block(src: &str, at: usize, kind: Reserved) -> (usize, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut cur = Cursor::at(src, at);
    cur.eat('@');
    cur.scan_ident();
    cur.skip_trivia();
    if cur.peek() != Some('{') {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            format!("expected `{{` to open `@{}`", kind.as_str()),
        ));
        return (cur.pos, diagnostics);
    }
    let brace_start = cur.pos;
    cur.skip_balanced_braces();
    if cur.pos <= brace_start || src.as_bytes().get(cur.pos - 1) != Some(&b'}') {
        diagnostics.push(Diagnostic::error(
            Span::new(brace_start, cur.pos),
            format!("unterminated `@{}` block; expected `}}`", kind.as_str()),
        ));
    }
    (cur.pos, diagnostics)
}

fn trailing_text(src: &str, end: usize) -> Option<Diagnostic> {
    let rest = src.get(end..)?;
    let line = rest.split('\n').next()?;
    if line.chars().any(|ch| !ch.is_whitespace()) {
        Some(Diagnostic::error(
            Span::new(end, end + line.len()),
            "text after a declaration must be on the next line",
        ))
    } else {
        None
    }
}

pub(crate) fn skip_0_3_spaces(line: &str) -> &str {
    let mut n = 0;
    let mut idx = 0;
    for ch in line.chars() {
        if ch == ' ' && n < 3 {
            n += 1;
            idx += 1;
        } else {
            break;
        }
    }
    &line[idx..]
}

pub(crate) fn fence_open(stripped: &str) -> Option<(u8, usize)> {
    let bytes = stripped.as_bytes();
    let ch = *bytes.first()?;
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let n = bytes.iter().take_while(|b| **b == ch).count();
    if n < 3 {
        return None;
    }
    if ch == b'`' && stripped[n..].contains('`') {
        return None;
    }
    Some((ch, n))
}

pub(crate) fn is_fence_close(line: &str, ch: u8, n: usize) -> bool {
    let stripped = skip_0_3_spaces(line);
    let bytes = stripped.as_bytes();
    let m = bytes.iter().take_while(|b| **b == ch).count();
    m >= n && stripped[m..].trim().is_empty()
}

fn looks_like_list_marker(stripped: &str) -> bool {
    let bytes = stripped.as_bytes();
    match bytes.first() {
        Some(b'-' | b'+' | b'*') => {
            matches!(bytes.get(1), Some(b' ' | b'\t') | None)
                && bytes.get(1) != Some(&b'-')
                && !(stripped.starts_with("---")
                    || stripped.starts_with("***")
                    || stripped.starts_with("+++"))
        }
        Some(b'0'..=b'9') => {
            let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
            if digits == 0 || digits > 9 {
                return false;
            }
            matches!(bytes.get(digits), Some(b'.' | b')'))
                && matches!(bytes.get(digits + 1), Some(b' ' | b'\t') | None)
        }
        _ => false,
    }
}

pub fn inner_span(src: &str, at: usize) -> Span {
    brace_inner_span(src, at)
}

fn brace_inner_span(src: &str, at: usize) -> Span {
    let mut cur = Cursor::at(src, at);
    cur.eat('@');
    cur.scan_ident();
    cur.skip_trivia();
    if cur.peek() != Some('{') {
        return Span::point(cur.pos);
    }
    let start = cur.pos;
    cur.skip_balanced_braces();
    if cur.pos <= start + 1 {
        return Span::point(start.saturating_add(1));
    }
    rocci_template::trim_span(src, Span::new(start + 1, cur.pos.saturating_sub(1)))
}

pub fn scan_range(
    src: &str,
    start: usize,
    end: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ScannedDecl> {
    let mut decls = Vec::new();
    let mut pos = start.min(end);
    let end = end.min(src.len());
    let mut fence: Option<(u8, usize)> = None;
    let mut list_tight = false;
    let mut quote_tight = false;

    while pos < end {
        let line_start = pos;
        let nl = src[pos..end].find('\n').map(|i| pos + i);
        let line_end = nl.unwrap_or(end);
        let line = &src[line_start..line_end];
        let next = nl.map(|i| i + 1).unwrap_or(end);

        if let Some((ch, n)) = fence {
            if is_fence_close(line, ch, n) {
                fence = None;
            }
            pos = next;
            continue;
        }

        if line.trim().is_empty() {
            list_tight = false;
            quote_tight = false;
            pos = next;
            continue;
        }

        let stripped = skip_0_3_spaces(line);
        if stripped.starts_with('>') {
            quote_tight = true;
            pos = next;
            continue;
        }
        if looks_like_list_marker(stripped) {
            list_tight = true;
            pos = next;
            continue;
        }
        if quote_tight || list_tight {
            pos = next;
            continue;
        }

        if let Some(decl) = try_scan_decl(src, line_start, diagnostics) {
            pos = decl.end.max(next).min(end);
            if decl.at < end {
                decls.push(decl);
            }
            list_tight = false;
            quote_tight = false;
            continue;
        }
        if let Some(decl) = try_scan_colon(src, line_start, diagnostics) {
            pos = decl.end.max(next).min(end);
            if decl.at < end {
                decls.push(decl);
            }
            list_tight = false;
            quote_tight = false;
            continue;
        }
        if let Some(decl) = try_scan_html(src, line_start, diagnostics) {
            pos = decl.end.max(next).min(end);
            if decl.at < end {
                decls.push(decl);
            }
            list_tight = false;
            quote_tight = false;
            continue;
        }

        if let Some(open) = fence_open(stripped) {
            fence = Some(open);
        }
        pos = next;
    }

    decls
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_template::Diagnostic;

    #[test]
    fn skip_article_section_ignores_braces_inside_fences() {
        let src = "{{ \n```roc\npair = { a: 1, b: 2 }\n```\n}}\n";
        let mut diagnostics = Vec::new();
        let end = skip_article_section(src, 0, &mut diagnostics);
        assert!(
            diagnostics.iter().all(|d| !Diagnostic::is_error(d)),
            "{diagnostics:?}"
        );
        assert!(src[..end].ends_with("}}"));
    }

    #[test]
    fn skip_article_section_advances_on_unclosed_input() {
        let src = "{{ still open";
        let mut diagnostics = Vec::new();
        let end = skip_article_section(src, 0, &mut diagnostics);
        assert!(end > 0);
        assert_eq!(end, src.len());
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("unterminated `{{`")),
            "{diagnostics:?}"
        );
    }
}
