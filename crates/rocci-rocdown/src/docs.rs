use std::path::{Component, Path, PathBuf};

use rocci_template::{Cursor, Span, is_ident_start};

use crate::page::{bool_literal, skip_value, string_list, string_literal};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsField {
    pub name: String,
    pub name_span: Span,
    pub value: Span,
}

pub fn split_docs_body(src: &str, body: Span) -> (Vec<DocsField>, Span) {
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;
    let mut fields = Vec::new();
    loop {
        cur.skip_trivia();
        if cur.pos >= end {
            return (fields, Span::new(end, end));
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let checkpoint = cur.pos;
        let Some(name_span) = cur.scan_ident() else {
            return (fields, remainder(src, checkpoint, end));
        };
        if !name_span.of(src).chars().next().is_some_and(is_ident_start) {
            return (fields, remainder(src, checkpoint, end));
        }
        cur.skip_trivia();
        if !cur.eat(':') {
            return (fields, remainder(src, checkpoint, end));
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        skip_value(&mut cur, end);
        if cur.pos == value_start {
            return (fields, remainder(src, checkpoint, end));
        }
        let name = name_span.of(src).to_string();
        fields.push(DocsField {
            name,
            name_span,
            value: Span::new(value_start, cur.pos.min(end)),
        });
        cur.skip_trivia();
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
}

fn remainder(src: &str, start: usize, end: usize) -> Span {
    rocci_template::trim_span(src, Span::new(start, end))
}

pub fn field_string(src: &str, field: &DocsField) -> Option<String> {
    string_literal(src, field.value)
}

pub fn field_bool(src: &str, field: &DocsField) -> Option<bool> {
    bool_literal(src, field.value)
}

pub fn field_strings(src: &str, field: &DocsField) -> Option<Vec<String>> {
    string_list(src, field.value)
}

pub fn include_path_error(path: &str) -> Option<String> {
    if path.is_empty() {
        return Some("include path must not be empty".into());
    }
    if path.contains('\0') {
        return Some("include path must not contain NUL bytes".into());
    }
    if Path::new(path).is_absolute() {
        return Some("include path must be relative".into());
    }
    if path.contains("..") {
        return Some("include path must not contain `..`".into());
    }
    None
}

pub fn extract_region(text: &str, name: &str) -> Result<(String, usize, usize), String> {
    let start_marker = format!("docs-region: {name}");
    let end_marker = format!("docs-region-end: {name}");
    let mut start_line = None;
    let mut end_line = None;
    for (index, line) in text.lines().enumerate() {
        if line.contains(&start_marker) {
            if start_line.is_some() {
                return Err(format!("duplicate region `{name}`"));
            }
            start_line = Some(index);
        }
        if line.contains(&end_marker) {
            if end_line.is_some() {
                return Err(format!("duplicate region end `{name}`"));
            }
            end_line = Some(index);
        }
    }
    let Some(start) = start_line else {
        return Err(format!("missing region `{name}`"));
    };
    let Some(end) = end_line else {
        return Err(format!("unclosed region `{name}`"));
    };
    if end <= start {
        return Err(format!("region `{name}` ends before it starts"));
    }
    let excerpt = text
        .lines()
        .skip(start + 1)
        .take(end.saturating_sub(start + 1))
        .collect::<Vec<_>>()
        .join("\n");
    Ok((excerpt, start + 2, end))
}

pub fn extract_lines(text: &str, start: u32, end: u32) -> Result<(String, usize, usize), String> {
    if start == 0 || end == 0 || end < start {
        return Err("line range must be 1-based with end >= start".into());
    }
    let lines: Vec<&str> = text.lines().collect();
    let from = start as usize;
    let to = end as usize;
    if to > lines.len() {
        return Err(format!(
            "line range {start}-{end} is past the end of the file"
        ));
    }
    Ok((lines[from - 1..to].join("\n"), from, to))
}

pub fn resolve_include_path(from_file: &str, path: &str) -> Result<PathBuf, String> {
    if let Some(err) = include_path_error(path) {
        return Err(err);
    }
    let from_file = from_file.strip_prefix("file://").unwrap_or(from_file);
    let base = Path::new(from_file)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let joined = base.join(path);
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                return Err("include path must not contain `..`".into());
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Ok(out)
}
