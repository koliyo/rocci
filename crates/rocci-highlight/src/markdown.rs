use rocci_template::Span;

use crate::token::{HighlightKind, HighlightSpan, floor_char_boundary};

pub fn highlight_markdown(source: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let mut pos = 0usize;
    if let Some(end) = highlight_frontmatter(source, &mut spans) {
        pos = end;
    }
    while pos < source.len() {
        let before = pos;
        pos = floor_char_boundary(source, pos);
        if pos >= source.len() {
            break;
        }
        if is_line_start(source, pos) {
            let rest = &source[pos..];
            if rest.starts_with("```") || rest.starts_with("~~~") {
                pos = highlight_fence(source, pos, &mut spans);
            } else if source.as_bytes()[pos] == b'#' {
                pos = highlight_heading(source, pos, &mut spans);
            } else {
                pos = highlight_prose_line(source, pos, &mut spans);
            }
        } else {
            pos += source[pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
        }
        if pos <= before {
            pos = next_char(source, before);
        }
    }
    spans
}

fn highlight_frontmatter(source: &str, spans: &mut Vec<HighlightSpan>) -> Option<usize> {
    if !source.starts_with("---") {
        return None;
    }
    let after_open = skip_line(source, 0);
    if after_open == 0 {
        return None;
    }
    push_span(source, spans, 0, 3, HighlightKind::Punctuation, 80);
    let mut pos = after_open;
    while pos < source.len() {
        let before = pos;
        pos = floor_char_boundary(source, pos);
        if pos >= source.len() {
            break;
        }
        if is_line_start(source, pos) && source[pos..].starts_with("---") {
            push_span(source, spans, pos, pos + 3, HighlightKind::Punctuation, 80);
            return Some(skip_line(source, pos));
        }
        if is_line_start(source, pos) {
            pos = highlight_frontmatter_line(source, pos, spans);
        } else {
            pos = next_char(source, pos);
        }
        if pos <= before {
            pos = next_char(source, before);
        }
    }
    Some(source.len())
}

fn highlight_frontmatter_line(source: &str, start: usize, spans: &mut Vec<HighlightSpan>) -> usize {
    let line_end = line_end(source, start);
    let line = &source[start..line_end];
    if let Some(colon) = line.find(':') {
        let key_end = start + colon;
        if key_end > start {
            push_span(source, spans, start, key_end, HighlightKind::Property, 70);
        }
        push_span(
            source,
            spans,
            key_end,
            key_end + 1,
            HighlightKind::Punctuation,
            60,
        );
        let value_start = skip_spaces(source, key_end + 1, line_end);
        if value_start < line_end {
            push_span(
                source,
                spans,
                value_start,
                line_end,
                HighlightKind::String,
                50,
            );
        }
    }
    skip_line(source, start)
}

fn highlight_heading(source: &str, start: usize, spans: &mut Vec<HighlightSpan>) -> usize {
    let mut hashes = start;
    while hashes < source.len() && source.as_bytes()[hashes] == b'#' {
        hashes += 1;
    }
    if hashes == start {
        return next_char(source, start);
    }
    push_span(source, spans, start, hashes, HighlightKind::Keyword, 80);
    let line_end = line_end(source, start);
    let text_start = skip_spaces(source, hashes, line_end);
    if text_start < line_end {
        push_span(
            source,
            spans,
            text_start,
            line_end,
            HighlightKind::Function,
            70,
        );
    }
    skip_line(source, start)
}

fn highlight_prose_line(source: &str, start: usize, spans: &mut Vec<HighlightSpan>) -> usize {
    let eol = line_end(source, start);
    let mut pos = start;
    if let Some(marker) = list_marker_at(source, pos, eol) {
        push_span(
            source,
            spans,
            marker.0,
            marker.1,
            HighlightKind::Operator,
            50,
        );
        pos = marker.1;
    }
    while pos < eol {
        let before = pos;
        pos = floor_char_boundary(source, pos);
        if pos >= eol {
            break;
        }
        let rest = &source[pos..eol];
        if (rest.starts_with('[') || rest.starts_with("!["))
            && let Some((label, dest)) = inline_link_at(source, pos, eol)
        {
            push_span(source, spans, label.0, label.1, HighlightKind::Variable, 50);
            push_span(source, spans, dest.0, dest.1, HighlightKind::Keyword, 52);
            pos = dest.1;
            continue;
        }
        if rest.starts_with("**") || rest.starts_with("__") || rest.starts_with("~~") {
            let marker = &rest[..2];
            if let Some(rel) = rest[2..].find(marker) {
                push_span(
                    source,
                    spans,
                    pos,
                    pos + 2 + rel + 2,
                    HighlightKind::Operator,
                    48,
                );
                pos += 2 + rel + 2;
                continue;
            }
        }
        pos += source[pos..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        if pos <= before {
            pos = next_char(source, before);
        }
    }
    skip_line(source, start)
}

fn list_marker_at(source: &str, start: usize, eol: usize) -> Option<(usize, usize)> {
    let mut pos = skip_spaces(source, start, eol);
    if pos >= eol {
        return None;
    }
    match source.as_bytes()[pos] {
        b'*' | b'-' | b'+' => Some((pos, pos + 1)),
        b'0'..=b'9' => {
            let digits_start = pos;
            while pos < eol && source.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos < eol && matches!(source.as_bytes()[pos], b'.' | b')') {
                Some((digits_start, pos + 1))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn inline_link_at(
    source: &str,
    start: usize,
    eol: usize,
) -> Option<((usize, usize), (usize, usize))> {
    let bytes = source.as_bytes();
    let mut i = start;
    if bytes.get(i) == Some(&b'!') {
        i += 1;
    }
    if bytes.get(i) != Some(&b'[') {
        return None;
    }
    let label_start = i;
    i += 1;
    let mut depth = 1u32;
    while i < eol {
        let before = i;
        match bytes[i] {
            b'\\' if i + 1 < eol => i += 2,
            b'[' => {
                depth += 1;
                i += 1;
            }
            b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
                if depth == 0 {
                    break;
                }
            }
            _ => i += 1,
        }
        if i <= before {
            i += 1;
        }
    }
    if depth != 0 {
        return None;
    }
    let label_end = i;
    i = skip_spaces(source, i, eol);
    if bytes.get(i) != Some(&b'(') {
        return None;
    }
    let dest_start = i;
    let dest_end = source[i..eol].rfind(')')? + i + 1;
    if dest_end <= dest_start {
        return None;
    }
    Some(((label_start, label_end), (dest_start, dest_end)))
}

fn highlight_fence(source: &str, start: usize, spans: &mut Vec<HighlightSpan>) -> usize {
    let marker = if source[start..].starts_with("```") {
        "```"
    } else {
        "~~~"
    };
    let marker_end = start + marker.len();
    push_span(
        source,
        spans,
        start,
        marker_end,
        HighlightKind::Punctuation,
        80,
    );
    let eol = line_end(source, start);
    let info_start = skip_spaces(source, marker_end, eol);
    if info_start < eol {
        push_span(source, spans, info_start, eol, HighlightKind::Type, 70);
    }
    let mut pos = skip_line(source, start);
    while pos < source.len() {
        let before = pos;
        if is_line_start(source, pos) && source[pos..].starts_with(marker) {
            push_span(
                source,
                spans,
                pos,
                pos + marker.len(),
                HighlightKind::Punctuation,
                80,
            );
            return skip_line(source, pos);
        }
        let next = skip_line(source, pos);
        if next > pos {
            push_span(
                source,
                spans,
                pos,
                line_end(source, pos),
                HighlightKind::String,
                40,
            );
            pos = next;
        }
        if pos <= before {
            pos = next_char(source, before);
        }
    }
    pos
}

fn is_line_start(source: &str, pos: usize) -> bool {
    pos == 0 || source.as_bytes().get(pos - 1) == Some(&b'\n')
}

fn line_end(source: &str, pos: usize) -> usize {
    source[pos..]
        .find('\n')
        .map(|rel| {
            let end = pos + rel;
            if end > 0 && source.as_bytes()[end - 1] == b'\r' {
                end - 1
            } else {
                end
            }
        })
        .unwrap_or(source.len())
}

fn skip_line(source: &str, pos: usize) -> usize {
    match source[pos..].find('\n') {
        Some(rel) => pos + rel + 1,
        None => source.len(),
    }
}

fn skip_spaces(source: &str, mut pos: usize, end: usize) -> usize {
    while pos < end {
        match source.as_bytes()[pos] {
            b' ' | b'\t' => pos += 1,
            _ => break,
        }
    }
    pos
}

fn next_char(source: &str, pos: usize) -> usize {
    let mut next = pos + 1;
    while next < source.len() && !source.is_char_boundary(next) {
        next += 1;
    }
    next.min(source.len())
}

fn push_span(
    source: &str,
    spans: &mut Vec<HighlightSpan>,
    start: usize,
    end: usize,
    kind: HighlightKind,
    priority: u32,
) {
    if start < end
        && end <= source.len()
        && source.is_char_boundary(start)
        && source.is_char_boundary(end)
    {
        spans.push(HighlightSpan::new(Span::new(start, end), kind, 0, priority));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_frontmatter_and_heading() {
        let src = "---\ntype: Architecture\n---\n\n# Hello\n";
        let spans = highlight_markdown(src);
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Property
                && src[span.start()..span.end()].contains("type")),
            "{spans:?}"
        );
        assert!(
            spans
                .iter()
                .any(|span| span.kind == HighlightKind::Keyword
                    && &src[span.start()..span.end()] == "#"),
            "{spans:?}"
        );
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Function
                && src[span.start()..span.end()].contains("Hello")),
            "{spans:?}"
        );
    }

    #[test]
    fn highlights_fenced_code() {
        let src = "```roc\nmain = {}\n```\n";
        let spans = highlight_markdown(src);
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Type
                && src[span.start()..span.end()].contains("roc")),
            "{spans:?}"
        );
    }

    #[test]
    fn unclosed_fence_still_terminates() {
        let src = "```\nno close";
        let spans = highlight_markdown(src);
        assert!(!spans.is_empty());
    }

    #[test]
    fn highlights_link_destination_and_bold() {
        let src = "* [handlers](/docs/applications/handlers/) and **app**\n";
        let spans = highlight_markdown(src);
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Operator
                && &src[span.start()..span.end()] == "*"),
            "{spans:?}"
        );
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Variable
                && src[span.start()..span.end()].contains("[handlers]")),
            "{spans:?}"
        );
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Keyword
                && &src[span.start()..span.end()] == "(/docs/applications/handlers/)"),
            "{spans:?}"
        );
        assert!(
            spans.iter().any(|span| span.kind == HighlightKind::Operator
                && src[span.start()..span.end()].contains("**app**")),
            "{spans:?}"
        );
    }
}
