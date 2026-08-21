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
    "published",
    "updated",
    "authors",
    "tags",
    "collection",
    "summary",
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
            "layout" => match string_literal(src, value) {
                Some(layout) => meta.layout = Some(layout),
                None => match value_path(src, value) {
                    Some(path) => meta.layout = Some(path),
                    None => diagnostics.push(Diagnostic::error(
                        value,
                        "`layout` must be a compile-time string literal or statically resolvable Roc value path",
                    )),
                },
            },
            "draft" => match bool_literal(src, value) {
                Some(draft) => meta.draft = draft,
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`draft` must be `True` or `False`",
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
            "published" => match string_literal(src, value) {
                Some(date) => meta.published = Some(date),
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`published` must be a compile-time string literal",
                )),
            },
            "updated" => match string_literal(src, value) {
                Some(date) => meta.updated = Some(date),
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`updated` must be a compile-time string literal",
                )),
            },
            "authors" => match string_list(src, value) {
                Some(authors) => meta.authors = authors,
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`authors` must be a list of compile-time string literals",
                )),
            },
            "tags" => match string_list(src, value) {
                Some(tags) => meta.tags = tags,
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`tags` must be a list of compile-time string literals",
                )),
            },
            "collection" => match string_literal(src, value) {
                Some(collection) => meta.collection = Some(collection),
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`collection` must be a compile-time string literal",
                )),
            },
            "summary" => match string_literal(src, value) {
                Some(summary) => meta.summary = Some(summary),
                None => diagnostics.push(Diagnostic::error(
                    value,
                    "`summary` must be a compile-time string literal",
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
                let before = cur.pos;
                cur.skip_roc_token();
                if cur.pos == before {
                    cur.bump();
                }
            }
        }
        Some('(') => {
            cur.bump();
            cur.paren += 1;
            while cur.pos < end && !cur.is_eof() && cur.paren > 0 {
                let before = cur.pos;
                cur.skip_roc_token();
                if cur.pos == before {
                    cur.bump();
                }
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
        "True" => Some(true),
        "False" => Some(false),
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
        let ident = cur.scan_ident()?;
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
    let stmt_indent = indent_at(cur.src, cur.pos);
    let mut started = false;
    loop {
        if cur.pos >= end {
            return;
        }
        if started && cur.is_top_level() {
            match cur.peek() {
                None => return,
                Some('\n' | '\r') => {
                    if continues_indented(cur, end, stmt_indent) {
                        continue;
                    }
                    return;
                }
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

fn indent_at(src: &str, pos: usize) -> usize {
    let line_start = src[..pos].rfind(['\n', '\r']).map(|i| i + 1).unwrap_or(0);
    src[line_start..pos]
        .chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count()
}

fn continues_indented(cur: &mut Cursor<'_>, end: usize, stmt_indent: usize) -> bool {
    let saved = cur.pos;
    loop {
        if cur.pos >= end || cur.peek().is_none() {
            cur.pos = saved;
            return false;
        }
        if matches!(cur.peek(), Some('\n' | '\r')) {
            let before = cur.pos;
            cur.bump();
            if cur.pos == before {
                cur.pos = saved;
                return false;
            }
            continue;
        }
        let indent_start = cur.pos;
        cur.skip_spaces_tabs();
        if matches!(cur.peek(), Some('\n' | '\r')) {
            if cur.pos == indent_start {
                cur.pos = saved;
                return false;
            }
            continue;
        }
        let indent = cur.src[indent_start..cur.pos].chars().count();
        if indent > stmt_indent && cur.peek().is_some() {
            return true;
        }
        cur.pos = saved;
        return false;
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
            let mut name = cur.ident_text(ident).to_string();
            let mut look = Cursor::at(src, cur.pos);
            look.skip_trivia();
            if look.peek() == Some('!') {
                name.push('!');
                look.bump();
                look.skip_trivia();
            }
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

pub fn roc_rest_name(src: &str, stmt: Span) -> Option<String> {
    let mut cur = Cursor::at(src, stmt.start as usize);
    let end = stmt.end as usize;
    cur.skip_trivia();
    if cur.pos >= end {
        return None;
    }
    let ident = cur.scan_ident()?;
    let mut name = cur.ident_text(ident).to_string();
    cur.skip_trivia();
    if cur.peek() == Some('!') {
        name.push('!');
        cur.bump();
        cur.skip_trivia();
    }
    match cur.peek() {
        Some('=' | ':') => Some(name),
        _ => None,
    }
}

pub fn import_local_name(src: &str, span: Span) -> Option<String> {
    let text = span.of(src);
    if text.contains("exposing") {
        return None;
    }
    let mut cur = Cursor::at(src, span.start as usize);
    cur.skip_trivia();
    if !is_import(&cur) {
        return None;
    }
    cur.eat_str("import");
    cur.skip_trivia();
    let mut last = None;
    while let Some(ident) = cur.scan_ident() {
        last = Some(cur.ident_text(ident).to_string());
        cur.skip_trivia();
        if cur.peek() == Some('.') {
            cur.bump();
            cur.skip_trivia();
            continue;
        }
        break;
    }
    cur.skip_trivia();
    if eat_keyword(&mut cur, "as") {
        cur.skip_trivia();
        let ident = cur.scan_ident()?;
        return Some(cur.ident_text(ident).to_string());
    }
    last
}

pub fn roc_name_appears(name: &str, text: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut from = 0;
    while from <= text.len() {
        let Some(rel) = text[from..].find(name) else {
            return false;
        };
        let start = from + rel;
        let end = start + name.len();
        let before_ok = start == 0
            || text[..start]
                .chars()
                .next_back()
                .is_none_or(|ch| !is_ident_continue(ch));
        let after_ok = end >= text.len()
            || text[end..]
                .chars()
                .next()
                .is_none_or(|ch| !is_ident_continue(ch) && ch != '!');
        if before_ok && after_ok {
            return true;
        }
        from = start.saturating_add(1);
        if from <= start {
            return false;
        }
    }
    false
}

fn eat_keyword(cur: &mut Cursor<'_>, kw: &str) -> bool {
    if !cur.starts_with(kw) {
        return false;
    }
    let after = cur.pos + kw.len();
    let next = cur.src.get(after..).and_then(|s| s.chars().next());
    if next.is_some_and(is_ident_continue) {
        return false;
    }
    cur.pos = after;
    true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roc_binding_names_handles_multiline_and_nested_brackets() {
        let src = r#"
Status : [Active(U64), Idle]

items = [
    { id: 1, name: "Item 1 🚀" },
    { id: 2, name: "Item 2 ✨" },
]
status = Active(42)
isLoaded = True
"#;
        let names = roc_binding_names(src, Span::new(0, src.len()));
        let binding_names: Vec<_> = names.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(binding_names, vec!["items", "status", "isLoaded"]);
    }

    #[test]
    fn roc_binding_names_handles_unclosed_delimiters_without_hanging() {
        let src = "broken = [\n  { id: 1, \n";
        let names = roc_binding_names(src, Span::new(0, src.len()));
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].0, "broken");
    }

    #[test]
    fn roc_binding_names_includes_effectful_bindings() {
        let src = "read_count! = |db| db\nincrement_count! = |db| db\n";
        let names = roc_binding_names(src, Span::new(0, src.len()));
        let binding_names: Vec<_> = names.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(binding_names, vec!["read_count!", "increment_count!"]);
    }

    #[test]
    fn roc_rest_name_classifies_bindings_aliases_and_unknown() {
        assert_eq!(
            roc_rest_name("read_count! = |db| db", Span::new(0, 21)).as_deref(),
            Some("read_count!")
        );
        assert_eq!(
            roc_rest_name("Status : [Active, Idle]", Span::new(0, 23)).as_deref(),
            Some("Status")
        );
        assert_eq!(roc_rest_name("expect x == 1", Span::new(0, 13)), None);
    }

    #[test]
    fn import_local_name_reads_module_and_alias() {
        assert_eq!(
            import_local_name("import pf.Sqlite", Span::new(0, 16)).as_deref(),
            Some("Sqlite")
        );
        assert_eq!(
            import_local_name("import pf.Sqlite as Sql", Span::new(0, 23)).as_deref(),
            Some("Sql")
        );
        assert_eq!(
            import_local_name("import pf.Stdout exposing [line!]", Span::new(0, 33)),
            None
        );
    }

    #[test]
    fn bool_literal_is_roc_true_false() {
        assert_eq!(bool_literal("True", Span::new(0, 4)), Some(true));
        assert_eq!(bool_literal("False", Span::new(0, 5)), Some(false));
        assert_eq!(bool_literal("Bool.true", Span::new(0, 9)), None);
        assert_eq!(bool_literal("true", Span::new(0, 4)), None);
    }

    #[test]
    fn roc_name_appears_uses_ident_boundaries() {
        assert!(roc_name_appears(
            "feature_count",
            "{ count: feature_count }"
        ));
        assert!(roc_name_appears("count", "{ count: feature_count }"));
        assert!(!roc_name_appears("count", "feature_count = 3"));
        assert!(roc_name_appears("read_count!", "count = read_count!(db)"));
        assert!(!roc_name_appears("read_count", "count = read_count!(db)"));
    }

    #[test]
    fn split_roc_body_keeps_indented_lambda_bodies() {
        let src = "\
import pf.Sqlite

read_count! = |db|
    Sqlite.query!(
        {
            db,
            query: \"SELECT 1\",
            params: {},
        },
    )
";
        let (imports, rest) = split_roc_body(src, Span::new(0, src.len()));
        assert_eq!(imports.len(), 1);
        assert_eq!(rest.len(), 1);
        assert!(rest[0].of(src).contains("read_count!"));
        assert!(rest[0].of(src).contains("Sqlite.query!"));
    }

    #[test]
    fn skip_value_terminates_on_malformed_input() {
        let src = "[ 1, 2, 3, \n\n";
        let mut cur = Cursor::at(src, 0);
        skip_value(&mut cur, src.len());
        assert!(cur.pos >= src.len() || cur.is_eof());
    }
}
