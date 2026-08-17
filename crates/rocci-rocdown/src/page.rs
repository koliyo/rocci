use rocci_template::{Cursor, Diagnostic, Span, is_ident_continue, is_ident_start};

use crate::ast::PageMeta;

const CONTROL_FIELDS: &[&str] = &[
    "id",
    "route",
    "aliases",
    "layout",
    "draft",
    "meta",
    "theme",
    "color_scheme",
];

pub fn extract_page(src: &str, body: Span, diagnostics: &mut Vec<Diagnostic>) -> PageMeta {
    let mut meta = PageMeta::default();
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;
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
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected a field name in `@page`",
            ));
            break;
        };
        let name = cur.ident_text(name_span).to_string();
        cur.skip_trivia();
        if !cur.eat(':') {
            diagnostics.push(Diagnostic::error(
                name_span,
                format!("expected `:` after `@page` field `{name}`"),
            ));
            break;
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        skip_value(&mut cur, end);
        let value = Span::new(value_start, cur.pos.min(end));
        match name.as_str() {
            "id" => match string_literal(src, value) {
                Some(id) => {
                    if let Some(err) = validate_id(&id) {
                        diagnostics.push(Diagnostic::error(value, err));
                    } else {
                        meta.id = Some(id);
                    }
                }
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`id` must be a compile-time string literal",
                )),
            },
            "route" => match string_literal(src, value) {
                Some(route) => {
                    if let Some(err) = validate_route(&route) {
                        diagnostics.push(Diagnostic::error(value, err));
                    } else {
                        meta.route = Some(route);
                    }
                }
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`route` must be a compile-time string literal",
                )),
            },
            "aliases" => match string_list(src, value) {
                Some(aliases) => {
                    let mut valid = Vec::new();
                    for alias in aliases {
                        if let Some(err) = validate_route(&alias) {
                            diagnostics
                                .push(Diagnostic::error(value, format!("alias `{alias}`: {err}")));
                        } else {
                            valid.push(alias);
                        }
                    }
                    meta.aliases = valid;
                }
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`aliases` must be a list of compile-time string literals",
                )),
            },
            "layout" => match value_path(src, value) {
                Some(path) => meta.layout = Some(path),
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`layout` must be a statically resolvable Roc value path",
                )),
            },
            "draft" => match bool_literal(src, value) {
                Some(draft) => meta.draft = draft,
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`draft` must be `Bool.true` or `Bool.false`",
                )),
            },
            "theme" => match string_literal(src, value) {
                Some(theme) => {
                    if theme.trim().is_empty() {
                        diagnostics.push(Diagnostic::error(value, "`theme` must not be empty"));
                    } else {
                        meta.theme = Some(theme);
                    }
                }
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`theme` must be a compile-time string literal",
                )),
            },
            "color_scheme" => match string_literal(src, value) {
                Some(scheme) => match scheme.parse::<rocci_theme::ColorSchemePolicy>() {
                    Ok(_) => meta.color_scheme = Some(scheme),
                    Err(err) => diagnostics.push(Diagnostic::error(value, err)),
                },
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`color_scheme` must be a compile-time string literal",
                )),
            },
            "meta" => {
                meta.meta = Some(value);
                meta.title = record_string_field(src, value, "title");
                meta.description = record_string_field(src, value, "description");
            }
            other => diagnostics.push(Diagnostic::error(
                name_span,
                format!(
                    "unknown `@page` field `{other}`; expected {}",
                    CONTROL_FIELDS.join(", ")
                ),
            )),
        }
        cur.skip_trivia();
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
    meta
}

pub(crate) fn skip_value(cur: &mut Cursor<'_>, end: usize) {
    if cur.pos >= end {
        return;
    }
    match cur.peek() {
        Some('{') => cur.skip_balanced_braces(),
        Some('"') => cur.skip_string(),
        Some('[') => {
            cur.bump();
            cur.bracket += 1;
            while cur.pos < end && !cur.is_eof() && cur.bracket > 0 {
                cur.skip_roc_token();
            }
        }
        Some('(') => {
            cur.bump();
            cur.paren += 1;
            while cur.pos < end && !cur.is_eof() && cur.paren > 0 {
                cur.skip_roc_token();
            }
        }
        _ => {
            while cur.pos < end && !cur.is_eof() {
                match cur.peek() {
                    Some(',') | Some('\n') if cur.is_top_level() => return,
                    Some('}') if cur.is_top_level() => return,
                    _ => {
                        let before = cur.pos;
                        cur.skip_roc_token();
                        if cur.pos == before {
                            cur.bump();
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn string_literal(src: &str, span: Span) -> Option<String> {
    let text = span.of(src).trim();
    if text.len() >= 2
        && text.starts_with('"')
        && text.ends_with('"')
        && !text.starts_with("\"\"\"")
    {
        Some(unescape_roc_string(&text[1..text.len() - 1]))
    } else {
        None
    }
}

fn unescape_roc_string(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(other @ ('"' | '\\')) => out.push(other),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub(crate) fn bool_literal(src: &str, span: Span) -> Option<bool> {
    match span.of(src).trim() {
        "Bool.true" => Some(true),
        "Bool.false" => Some(false),
        _ => None,
    }
}

fn value_path(src: &str, span: Span) -> Option<String> {
    let mut cur = Cursor::at(src, span.start as usize);
    let end = span.end as usize;
    cur.skip_trivia();
    let mut parts = Vec::new();
    loop {
        if cur.pos >= end {
            break;
        }
        let Some(ident) = cur.scan_ident() else {
            return None;
        };
        if !ident.of(src).chars().next().is_some_and(is_ident_start) {
            return None;
        }
        parts.push(cur.ident_text(ident).to_string());
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        if cur.eat('.') {
            cur.skip_trivia();
            continue;
        }
        if cur.pos < end
            && !span.of(src)[cur.pos - span.start as usize..]
                .trim()
                .is_empty()
        {
            return None;
        }
        break;
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    }
}

fn record_string_field(src: &str, record: Span, field: &str) -> Option<String> {
    let text = record.of(src).trim();
    if !text.starts_with('{') || !text.ends_with('}') {
        return None;
    }
    let inner = Span::new(record.start as usize + 1, record.end as usize - 1);
    let mut cur = Cursor::at(src, inner.start as usize);
    let end = inner.end as usize;
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
        let name = cur.ident_text(name_span).to_string();
        cur.skip_trivia();
        if !cur.eat(':') {
            break;
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        skip_value(&mut cur, end);
        let value = Span::new(value_start, cur.pos.min(end));
        if name == field {
            return string_literal(src, value);
        }
        cur.skip_trivia();
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
    None
}

pub fn validate_id(id: &str) -> Option<String> {
    if id.trim().is_empty() {
        return Some("`id` must not be empty".into());
    }
    if id.contains('\0') {
        return Some("`id` must not contain NUL bytes".into());
    }
    if id.contains("..") {
        return Some("`id` must not contain `..`".into());
    }
    if id.starts_with('/') {
        return Some("`id` must not start with `/`; page identity is distinct from route".into());
    }
    if id.contains('?') || id.contains('#') {
        return Some("`id` must not contain a query string or fragment".into());
    }
    None
}

pub(crate) fn string_list(src: &str, span: Span) -> Option<Vec<String>> {
    let text = span.of(src).trim();
    if !text.starts_with('[') || !text.ends_with(']') {
        return None;
    }
    let mut cur = Cursor::at(src, span.start as usize);
    let end = span.end as usize;
    cur.skip_trivia();
    if !cur.eat('[') {
        return None;
    }
    let mut items = Vec::new();
    loop {
        cur.skip_trivia();
        if cur.pos >= end {
            return None;
        }
        if cur.peek() == Some(']') {
            cur.bump();
            break;
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        if cur.peek() != Some('"') {
            return None;
        }
        let value_start = cur.pos;
        cur.skip_string();
        let value = Span::new(value_start, cur.pos.min(end));
        items.push(string_literal(src, value)?);
        cur.skip_trivia();
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
    Some(items)
}

pub fn validate_route(route: &str) -> Option<String> {
    if !route.starts_with('/') {
        return Some("`route` must be an absolute URL path starting with `/`".into());
    }
    if route.contains('\0') {
        return Some("`route` must not contain NUL bytes".into());
    }
    if route.contains("..") {
        return Some("`route` must not contain `..`".into());
    }
    if route.contains('?') || route.contains('#') {
        return Some("`route` must not contain a query string or fragment".into());
    }
    if route.to_ascii_lowercase().contains("%2f") || route.contains("%5c") || route.contains("%5C")
    {
        return Some("`route` must not contain encoded path separators".into());
    }
    None
}

pub fn split_roc_body(src: &str, body: Span) -> (Vec<Span>, Vec<Span>) {
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;
    let mut imports = Vec::new();
    let mut rest = Vec::new();
    while cur.pos < end && !cur.is_eof() {
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        let start = cur.pos;
        let import = is_import(&cur);
        skip_statement(&mut cur, end);
        let span = Span::new(start, cur.pos.min(end));
        if span.is_empty() {
            break;
        }
        if import {
            imports.push(span);
        } else {
            rest.push(span);
        }
    }
    (imports, rest)
}

fn is_import(cur: &Cursor<'_>) -> bool {
    if !cur.starts_with("import") {
        return false;
    }
    match cur
        .rest()
        .get("import".len()..)
        .and_then(|rest| rest.chars().next())
    {
        None => true,
        Some(ch) => !is_ident_continue(ch),
    }
}

fn skip_statement(cur: &mut Cursor<'_>, end: usize) {
    if cur.pos >= end {
        return;
    }
    let mut started = false;
    loop {
        if cur.pos >= end {
            return;
        }
        if started && cur.is_top_level() {
            match cur.peek() {
                None => return,
                Some('\n' | '\r') => return,
                Some('#') => {
                    cur.skip_comment();
                    return;
                }
                Some(' ' | '\t') => {
                    cur.bump();
                    continue;
                }
                _ => {}
            }
        }
        let before = cur.pos;
        skip_token_keep_newline(cur);
        if cur.pos == before {
            if matches!(cur.peek(), Some('\n' | '\r')) {
                cur.bump();
            } else if cur.pos >= end || cur.peek().is_none() {
                return;
            } else {
                cur.bump();
            }
        }
        started = true;
        if cur.pos == end {
            return;
        }
    }
}

fn skip_token_keep_newline(cur: &mut Cursor<'_>) {
    cur.skip_spaces_tabs();
    match cur.peek() {
        None | Some('\n') | Some('\r') => {}
        Some('"') => cur.skip_string(),
        Some('#') => cur.skip_comment(),
        Some(ch) if rocci_template::is_ident_start(ch) => {
            cur.scan_ident();
        }
        Some(ch) if ch.is_ascii_digit() => cur.skip_number(),
        Some('(') => {
            cur.bump();
            cur.paren += 1;
        }
        Some(')') => {
            cur.bump();
            cur.paren = cur.paren.saturating_sub(1);
        }
        Some('[') => {
            cur.bump();
            cur.bracket += 1;
        }
        Some(']') => {
            cur.bump();
            cur.bracket = cur.bracket.saturating_sub(1);
        }
        Some('{') => {
            cur.bump();
            cur.brace += 1;
        }
        Some('}') => {
            cur.bump();
            cur.brace = cur.brace.saturating_sub(1);
        }
        Some(_) => {
            cur.bump();
        }
    }
}

pub fn roc_binding_names(src: &str, body: Span) -> Vec<(String, Span)> {
    let mut names = Vec::new();
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;
    while cur.pos < end && !cur.is_eof() {
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        let start = cur.pos;
        if is_import(&cur) {
            skip_statement(&mut cur, end);
            continue;
        }
        if let Some(ident) = cur.scan_ident() {
            let name = cur.ident_text(ident).to_string();
            let after = Cursor::at(src, cur.pos);
            let mut look = after;
            look.skip_trivia();
            if look.peek() == Some('=') {
                names.push((name, ident));
            }
            cur.pos = start;
        }
        skip_statement(&mut cur, end);
        if cur.pos == start {
            cur.bump();
        }
    }
    names
}

pub fn imports_html(src: &str, imports: &[Span]) -> bool {
    imports.iter().any(|span| {
        let text = span.of(src).trim();
        let rest = text.strip_prefix("import").unwrap_or(text).trim_start();
        rest == "Html"
            || rest.starts_with("Html ")
            || rest.starts_with("Html\n")
            || rest.starts_with("Html\t")
    })
}
