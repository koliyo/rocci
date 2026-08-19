use rocci_template::{Cursor, Diagnostic, Span};

use crate::ast::{BracketList, BracketRecord, ParamField, ParamValue};
use crate::page::string_literal;

pub fn parse_article_params(
    src: &str,
    start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Option<BracketRecord>, usize) {
    let mut cur = Cursor::at(src, start);
    cur.skip_spaces_tabs();
    if cur.peek() != Some('[') {
        return (None, start);
    }
    let open = cur.pos;
    cur.bump();
    skip_param_ws(&mut cur);
    if cur.peek() == Some(']') {
        cur.bump();
        return (
            Some(BracketRecord {
                fields: Vec::new(),
                span: Span::new(open, cur.pos),
            }),
            cur.pos,
        );
    }
    if !looks_like_record(src, cur.pos) {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            "article params must be a record of `name: value` fields",
        ));
        let (end, mut extra) = skip_to_close_bracket(src, open);
        diagnostics.append(&mut extra);
        return (None, end);
    }
    let fields = parse_record_fields(src, &mut cur, diagnostics);
    skip_param_ws(&mut cur);
    if !cur.eat(']') {
        diagnostics.push(Diagnostic::error(
            Span::new(open, cur.pos),
            "unterminated `[` params; expected `]`",
        ));
        if cur.pos <= open {
            cur.bump();
        }
    }
    (
        Some(BracketRecord {
            fields,
            span: Span::new(open, cur.pos),
        }),
        cur.pos,
    )
}

fn parse_record_fields(
    src: &str,
    cur: &mut Cursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ParamField> {
    let mut fields = Vec::new();
    loop {
        skip_param_ws(cur);
        if cur.peek() == Some(']') || cur.is_eof() {
            break;
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let Some(name_span) = cur.scan_ident() else {
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected a field name in `[` params",
            ));
            recover_param(cur);
            continue;
        };
        let name = name_span.of(src).to_string();
        skip_param_ws(cur);
        if !cur.eat(':') {
            diagnostics.push(Diagnostic::error(
                name_span,
                format!("expected `:` after param `{name}`"),
            ));
            recover_param(cur);
            continue;
        }
        skip_param_ws(cur);
        let Some(value) = parse_param_value(src, cur, diagnostics) else {
            recover_param(cur);
            continue;
        };
        fields.push(ParamField {
            name,
            name_span,
            value,
        });
        skip_param_ws(cur);
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
    fields
}

fn parse_param_value(
    src: &str,
    cur: &mut Cursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ParamValue> {
    skip_param_ws(cur);
    match cur.peek() {
        Some('"') => {
            let start = cur.pos;
            cur.skip_string();
            let span = Span::new(start, cur.pos);
            let value = string_literal(src, span).unwrap_or_default();
            Some(ParamValue::StringLit { value, span })
        }
        Some('[') => parse_nested_bracket(src, cur, diagnostics),
        Some(ch) if ch == '-' || ch.is_ascii_digit() => Some(parse_number(src, cur)),
        Some(_) => parse_ident_or_bool(src, cur, diagnostics),
        None => {
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected a param value",
            ));
            None
        }
    }
}

fn parse_nested_bracket(
    src: &str,
    cur: &mut Cursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ParamValue> {
    let open = cur.pos;
    cur.bump();
    skip_param_ws(cur);
    if cur.peek() == Some(']') {
        cur.bump();
        return Some(ParamValue::List(BracketList {
            items: Vec::new(),
            span: Span::new(open, cur.pos),
        }));
    }
    if looks_like_record(src, cur.pos) {
        let fields = parse_record_fields(src, cur, diagnostics);
        skip_param_ws(cur);
        if !cur.eat(']') {
            diagnostics.push(Diagnostic::error(
                Span::new(open, cur.pos),
                "unterminated nested `[` record",
            ));
            if cur.pos <= open {
                cur.bump();
            }
        }
        return Some(ParamValue::Record(BracketRecord {
            fields,
            span: Span::new(open, cur.pos),
        }));
    }
    let mut items = Vec::new();
    loop {
        skip_param_ws(cur);
        if cur.peek() == Some(']') || cur.is_eof() {
            break;
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let Some(value) = parse_param_value(src, cur, diagnostics) else {
            recover_param(cur);
            continue;
        };
        items.push(value);
        skip_param_ws(cur);
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
    skip_param_ws(cur);
    if !cur.eat(']') {
        diagnostics.push(Diagnostic::error(
            Span::new(open, cur.pos),
            "unterminated `[` list; expected `]`",
        ));
        if cur.pos <= open {
            cur.bump();
        }
    }
    Some(ParamValue::List(BracketList {
        items,
        span: Span::new(open, cur.pos),
    }))
}

fn parse_number(src: &str, cur: &mut Cursor<'_>) -> ParamValue {
    let start = cur.pos;
    if cur.peek() == Some('-') {
        cur.bump();
    }
    let mut saw_dot = false;
    while let Some(ch) = cur.peek() {
        if ch.is_ascii_digit() {
            cur.bump();
        } else if ch == '.' && !saw_dot {
            saw_dot = true;
            cur.bump();
        } else {
            break;
        }
    }
    if cur.pos <= start {
        cur.bump();
    }
    let span = Span::new(start, cur.pos);
    ParamValue::NumberLit {
        value: span.of(src).to_string(),
        span,
    }
}

fn parse_ident_or_bool(
    src: &str,
    cur: &mut Cursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ParamValue> {
    let start = cur.pos;
    let Some(ident) = cur.scan_ident() else {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            "expected a param value",
        ));
        return None;
    };
    if ident.of(src) == "Bool" && cur.eat('.') {
        let Some(flag) = cur.scan_ident() else {
            diagnostics.push(Diagnostic::error(
                Span::new(start, cur.pos),
                "expected `Bool.true` or `Bool.false`",
            ));
            return None;
        };
        let span = Span::new(start, flag.end as usize);
        return match flag.of(src) {
            "true" => Some(ParamValue::BoolLit { value: true, span }),
            "false" => Some(ParamValue::BoolLit { value: false, span }),
            other => {
                diagnostics.push(Diagnostic::error(
                    span,
                    format!("expected `Bool.true` or `Bool.false`, found `Bool.{other}`"),
                ));
                None
            }
        };
    }
    Some(ParamValue::Ident {
        name: ident.of(src).to_string(),
        span: ident,
    })
}

fn looks_like_record(src: &str, pos: usize) -> bool {
    let mut cur = Cursor::at(src, pos);
    skip_param_ws(&mut cur);
    if cur.scan_ident().is_none() {
        return false;
    }
    skip_param_ws(&mut cur);
    cur.peek() == Some(':')
}

fn skip_param_ws(cur: &mut Cursor<'_>) {
    cur.skip_whitespace();
}

fn recover_param(cur: &mut Cursor<'_>) {
    let before = cur.pos;
    while !cur.is_eof() {
        match cur.peek() {
            Some(',' | ']') => return,
            Some('"') => cur.skip_string(),
            Some(_) => {
                cur.bump();
            }
            None => break,
        }
        if cur.pos <= before {
            cur.bump();
            return;
        }
    }
}

fn skip_to_close_bracket(src: &str, open: usize) -> (usize, Vec<Diagnostic>) {
    let mut cur = Cursor::at(src, open);
    let mut diagnostics = Vec::new();
    if !cur.eat('[') {
        return (open, diagnostics);
    }
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
    }
    (cur.pos, diagnostics)
}
