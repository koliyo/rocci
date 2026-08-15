use rocci_template::{
    Cursor, Diagnostic, Span, is_ident_continue, is_ident_start, parse_declaration_from,
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
    On,
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
            "on" => Self::On,
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
            Self::On => "on",
        }
    }

    fn is_rocci(self) -> bool {
        matches!(
            self,
            Self::Component | Self::Fixture | Self::Css | Self::Context | Self::Init | Self::On
        )
    }
}

#[derive(Clone, Debug)]
pub struct ScannedDecl {
    pub kind: Reserved,
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
    if src[pos..].contains('\u{FEFF}') {
        if let Some(off) = src[pos..].find('\u{FEFF}') {
            diagnostics.push(Diagnostic::error(
                Span::new(pos + off, pos + off + '\u{FEFF}'.len_utf8()),
                "a UTF-8 BOM is only allowed at the start of a Rocdown file",
            ));
        }
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
    let kind = Reserved::from_name(&src[name_start..name_end])?;
    if !header_matches(src, name_end, kind) {
        return None;
    }

    let (end, mut extra) = if kind.is_rocci() {
        match parse_declaration_from(src, at) {
            Some(parsed) => (parsed.end, parsed.diagnostics),
            None => return None,
        }
    } else {
        skip_brace_block(src, at, kind)
    };
    if let Some(diag) = trailing_text(src, end) {
        extra.push(diag);
    }
    diagnostics.append(&mut extra);
    Some(ScannedDecl {
        kind,
        line_start,
        at,
        end,
    })
}

fn header_matches(src: &str, after_name: usize, kind: Reserved) -> bool {
    let mut cur = Cursor::at(src, after_name);
    match kind {
        Reserved::On => {
            cur.skip_trivia();
            cur.peek() == Some(':')
        }
        Reserved::Component => {
            cur.skip_trivia();
            cur.peek().is_some_and(is_ident_start)
        }
        Reserved::Fixture | Reserved::Context => {
            cur.skip_trivia();
            matches!(cur.peek(), Some('{')) || cur.peek().is_some_and(is_ident_start)
        }
        Reserved::Page | Reserved::Roc | Reserved::Render | Reserved::Css | Reserved::Init => {
            cur.skip_trivia();
            cur.peek() == Some('{')
        }
    }
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
            "text after a declaration's closing `}` must be on the next line",
        ))
    } else {
        None
    }
}

fn skip_0_3_spaces(line: &str) -> &str {
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

fn fence_open(stripped: &str) -> Option<(u8, usize)> {
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

fn is_fence_close(line: &str, ch: u8, n: usize) -> bool {
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
