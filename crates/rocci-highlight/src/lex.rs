use rocci_template::{Cursor, Span};

use crate::token::{HighlightKind, HighlightSpan, MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION};

const ROC_KEYWORDS: &[&str] = &[
    "and",
    "app",
    "as",
    "crash",
    "dbg",
    "else",
    "expect",
    "exposes",
    "generates",
    "if",
    "implements",
    "import",
    "imports",
    "interface",
    "is",
    "match",
    "module",
    "not",
    "or",
    "package",
    "platform",
    "then",
    "when",
    "where",
];

pub fn highlight_roc(src: &str) -> Vec<HighlightSpan> {
    let mut out = Vec::new();
    let mut cur = Cursor::new(src);
    while !cur.is_eof() {
        let before = cur.pos;
        scan_roc_token(src, &mut cur, &mut out);
        if cur.pos <= before {
            cur.bump();
        }
    }
    out
}

pub fn highlight_css(src: &str) -> Vec<HighlightSpan> {
    let mut out = Vec::new();
    let mut cur = Cursor::new(src);
    while !cur.is_eof() {
        let before = cur.pos;
        scan_css_token(src, &mut cur, &mut out);
        if cur.pos <= before {
            cur.bump();
        }
    }
    out
}

pub fn highlight_html(src: &str) -> Vec<HighlightSpan> {
    let mut out = Vec::new();
    let mut cur = Cursor::new(src);
    while !cur.is_eof() {
        let before = cur.pos;
        scan_html_token(src, &mut cur, &mut out);
        if cur.pos <= before {
            cur.bump();
        }
    }
    out
}

fn push(
    out: &mut Vec<HighlightSpan>,
    start: usize,
    end: usize,
    kind: HighlightKind,
    priority: u32,
) {
    if start < end {
        out.push(HighlightSpan::new(Span::new(start, end), kind, 0, priority));
    }
}

fn scan_roc_token(src: &str, cur: &mut Cursor<'_>, out: &mut Vec<HighlightSpan>) {
    cur.skip_whitespace();
    if cur.is_eof() {
        return;
    }
    let start = cur.pos;
    match cur.peek() {
        Some('#') => {
            let docs = cur.starts_with("##");
            while let Some(ch) = cur.peek() {
                if ch == '\n' {
                    break;
                }
                cur.bump();
            }
            out.push(HighlightSpan::new(
                Span::new(start, cur.pos),
                HighlightKind::Comment,
                if docs { MOD_DOCUMENTATION } else { 0 },
                60,
            ));
        }
        Some('"') => {
            cur.bump();
            while let Some(ch) = cur.peek() {
                if ch == '"' {
                    cur.bump();
                    break;
                }
                if ch == '\\' {
                    cur.bump();
                    cur.bump();
                    continue;
                }
                if ch == '\n' {
                    break;
                }
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::String, 55);
        }
        Some(ch) if ch.is_ascii_digit() => {
            while matches!(cur.peek(), Some(c) if c.is_ascii_digit() || c == '_') {
                cur.bump();
            }
            if cur.peek() == Some('.') {
                let next = cur.rest().chars().nth(1);
                if next.is_some_and(|c| c.is_ascii_digit()) {
                    cur.bump();
                    while matches!(cur.peek(), Some(c) if c.is_ascii_digit() || c == '_') {
                        cur.bump();
                    }
                }
            }
            push(out, start, cur.pos, HighlightKind::Number, 40);
        }
        Some(ch) if ch.is_ascii_uppercase() => {
            while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
                cur.bump();
            }
            let word = &src[start..cur.pos];
            let kind = if word == "Ok" || word == "Err" || word == "True" || word == "False" {
                HighlightKind::EnumMember
            } else {
                HighlightKind::Type
            };
            push(out, start, cur.pos, kind, 45);
        }
        Some(ch) if ch.is_ascii_lowercase() || ch == '_' => {
            while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
                cur.bump();
            }
            if cur.peek() == Some('!') {
                cur.bump();
                push(out, start, cur.pos, HighlightKind::Function, 45);
                return;
            }
            let word = &src[start..cur.pos];
            let kind = if ROC_KEYWORDS.contains(&word) {
                HighlightKind::Keyword
            } else {
                HighlightKind::Variable
            };
            push(
                out,
                start,
                cur.pos,
                kind,
                if kind == HighlightKind::Keyword {
                    50
                } else {
                    30
                },
            );
        }
        Some('.' | '=' | '!' | '?' | '|' | '+' | '-' | '*' | '/' | '<' | '>' | '&') => {
            cur.bump();
            push(out, start, cur.pos, HighlightKind::Operator, 35);
        }
        Some(':' | ',' | '(' | ')' | '{' | '}' | '[' | ']' | '\\') => {
            cur.bump();
            push(out, start, cur.pos, HighlightKind::Punctuation, 35);
        }
        Some(_) => {
            cur.bump();
        }
        None => {}
    }
}

fn scan_css_token(src: &str, cur: &mut Cursor<'_>, out: &mut Vec<HighlightSpan>) {
    cur.skip_whitespace();
    if cur.is_eof() {
        return;
    }
    let start = cur.pos;
    if cur.starts_with("/*") {
        cur.pos += 2;
        while !cur.is_eof() && !cur.starts_with("*/") {
            cur.bump();
        }
        if cur.starts_with("*/") {
            cur.pos += 2;
        }
        push(out, start, cur.pos, HighlightKind::Comment, 60);
        return;
    }
    match cur.peek() {
        Some('"' | '\'') => {
            let quote = cur.bump().unwrap();
            while let Some(ch) = cur.peek() {
                if ch == quote {
                    cur.bump();
                    break;
                }
                if ch == '\\' {
                    cur.bump();
                    cur.bump();
                    continue;
                }
                if ch == '\n' {
                    break;
                }
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::String, 40);
        }
        Some('#') | Some('.') => {
            cur.bump();
            while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::Property, 45);
        }
        Some('@') => {
            cur.bump();
            while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '-') {
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::Keyword, 50);
        }
        Some(ch) if ch.is_ascii_digit() => {
            while matches!(cur.peek(), Some(c) if c.is_ascii_digit() || c == '.') {
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::Number, 40);
        }
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' || ch == '-' => {
            while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                cur.bump();
            }
            let word = &src[start..cur.pos];
            let kind = if matches!(
                word,
                "px" | "rem" | "em" | "vh" | "vw" | "ms" | "s" | "deg" | "important"
            ) {
                HighlightKind::Keyword
            } else if cur.peek() == Some('(') {
                HighlightKind::Function
            } else {
                HighlightKind::Property
            };
            push(out, start, cur.pos, kind, 45);
        }
        Some('{' | '}' | '(' | ')' | '[' | ']' | ':' | ';' | ',' | '>' | '+' | '~') => {
            cur.bump();
            push(out, start, cur.pos, HighlightKind::Operator, 35);
        }
        Some(_) => {
            cur.bump();
        }
        None => {}
    }
}

fn scan_html_token(_src: &str, cur: &mut Cursor<'_>, out: &mut Vec<HighlightSpan>) {
    cur.skip_whitespace();
    if cur.is_eof() {
        return;
    }
    let start = cur.pos;
    if cur.starts_with("<!--") {
        cur.pos += 4;
        while !cur.is_eof() && !cur.starts_with("-->") {
            cur.bump();
        }
        if cur.starts_with("-->") {
            cur.pos += 3;
        }
        push(out, start, cur.pos, HighlightKind::Comment, 60);
        return;
    }
    if cur.peek() == Some('<') {
        cur.bump();
        let _ = cur.eat('/');
        let name_start = cur.pos;
        while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '-') {
            cur.bump();
        }
        if cur.pos > name_start {
            out.push(HighlightSpan::new(
                Span::new(name_start, cur.pos),
                HighlightKind::Tag,
                MOD_DEFAULT_LIBRARY,
                45,
            ));
        }
        return;
    }
    match cur.peek() {
        Some('"' | '\'') => {
            let quote = cur.bump().unwrap();
            while let Some(ch) = cur.peek() {
                if ch == quote {
                    cur.bump();
                    break;
                }
                if ch == '\\' {
                    cur.bump();
                    cur.bump();
                    continue;
                }
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::String, 40);
        }
        Some(ch) if ch.is_ascii_alphabetic() => {
            while matches!(cur.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == ':')
            {
                cur.bump();
            }
            push(out, start, cur.pos, HighlightKind::Property, 45);
        }
        Some('>' | '/' | '=') => {
            cur.bump();
            push(out, start, cur.pos, HighlightKind::Operator, 35);
        }
        Some(_) => {
            cur.bump();
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roc_lex_paints_keywords_strings_and_comments() {
        let src = "import pf.Sqlite\n# note\nmatch Env.var!(\"DB_PATH\") {\n    Ok(path) => path\n    Err(_) => \"x\"\n}\n";
        let spans = highlight_roc(src);
        let texts: Vec<_> = spans
            .iter()
            .map(|s| (s.kind, &src[s.start()..s.end()]))
            .collect();
        assert!(
            texts
                .iter()
                .any(|(k, t)| *k == HighlightKind::Keyword && *t == "import")
        );
        assert!(
            texts
                .iter()
                .any(|(k, t)| *k == HighlightKind::Keyword && *t == "match")
        );
        assert!(
            texts
                .iter()
                .any(|(k, t)| *k == HighlightKind::String && *t == "\"DB_PATH\"")
        );
        assert!(
            texts
                .iter()
                .any(|(k, t)| *k == HighlightKind::Comment && *t == "# note")
        );
        assert!(
            texts
                .iter()
                .any(|(k, t)| *k == HighlightKind::EnumMember && *t == "Ok")
        );
        assert!(
            texts
                .iter()
                .any(|(k, t)| *k == HighlightKind::Function && *t == "var!")
        );
    }

    #[test]
    fn css_lex_paints_properties_and_strings() {
        let src = ".card { color: \"#fff\"; }";
        let spans = highlight_css(src);
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Property));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::String));
    }

    #[test]
    fn html_lex_paints_tags() {
        let src = "<div class=\"hero\"></div>";
        let spans = highlight_html(src);
        assert!(
            spans
                .iter()
                .any(|s| s.kind == HighlightKind::Tag && &src[s.start()..s.end()] == "div")
        );
        assert!(spans.iter().any(|s| s.kind == HighlightKind::String));
    }
}
