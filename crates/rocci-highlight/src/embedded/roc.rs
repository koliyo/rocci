use std::sync::OnceLock;

use crate::token::{HighlightKind, HighlightSpan, MOD_DOCUMENTATION};
use crate::tree_sitter::TreeSitterHighlighter;

unsafe extern "C" {
    fn tree_sitter_roc() -> tree_sitter::Language;
}

pub fn language() -> tree_sitter::Language {
    unsafe { tree_sitter_roc() }
}

const ROC_HIGHLIGHTS_QUERY: &str = include_str!("../../grammars/roc/queries/highlights.scm");

static HIGHLIGHTER: OnceLock<Option<TreeSitterHighlighter>> = OnceLock::new();

pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    let highlighter = HIGHLIGHTER.get_or_init(|| {
        match TreeSitterHighlighter::new(language(), ROC_HIGHLIGHTS_QUERY) {
            Ok(h) => Some(h),
            Err(e) => {
                eprintln!("Roc TreeSitter query error: {:?}", e);
                None
            }
        }
    });

    let Some(hl) = highlighter.as_ref() else {
        return Vec::new();
    };

    let raw_tokens = hl.highlight(src, map_roc_capture);
    let mut tokens = repair_roc_token_boundaries(src, raw_tokens);
    fill_roc_gap_tokens(src, &mut tokens);
    tokens
}

fn repair_roc_token_boundaries(src: &str, mut tokens: Vec<HighlightSpan>) -> Vec<HighlightSpan> {
    for tok in &mut tokens {
        let start = tok.span.start as usize;
        let end = tok.span.end as usize;
        if start > 0 && start <= src.len() && end <= src.len() {
            let bytes = src.as_bytes();
            if matches!(
                tok.kind,
                HighlightKind::Variable
                    | HighlightKind::Function
                    | HighlightKind::Property
                    | HighlightKind::Type
                    | HighlightKind::EnumMember
            ) {
                let mut new_start = start;
                while new_start > 0 {
                    let prev_byte = bytes[new_start - 1];
                    if prev_byte.is_ascii_alphanumeric() || prev_byte == b'_' {
                        new_start -= 1;
                    } else {
                        break;
                    }
                }
                if new_start < start {
                    tok.span.start = new_start as u32;
                }
            }
        }
    }
    tokens
}

fn fill_roc_gap_tokens(src: &str, tokens: &mut Vec<HighlightSpan>) {
    tokens.sort_by_key(|t| t.span.start);
    let mut gap_tokens = Vec::new();
    let mut prev_end = 0usize;

    for tok in tokens.iter() {
        let start = tok.start();
        if start > prev_end {
            scan_roc_gap(&src[prev_end..start], prev_end, &mut gap_tokens);
        }
        prev_end = prev_end.max(tok.end());
    }
    if prev_end < src.len() {
        scan_roc_gap(&src[prev_end..], prev_end, &mut gap_tokens);
    }

    tokens.extend(gap_tokens);
}

fn scan_roc_gap(gap: &str, offset: usize, out: &mut Vec<HighlightSpan>) {
    let mut cur = rocci_template::Cursor::at(gap, 0);
    while !cur.is_eof() {
        cur.skip_trivia();
        if cur.is_eof() {
            break;
        }
        let start = cur.pos;
        let ch = cur.peek().unwrap();

        if ch.is_ascii_uppercase() {
            while let Some(c) = cur.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    cur.bump();
                } else {
                    break;
                }
            }
            out.push(HighlightSpan::new(
                rocci_template::Span::new(offset + start, offset + cur.pos),
                HighlightKind::Namespace,
                0,
                20,
            ));
        } else if ch.is_ascii_lowercase() || ch == '_' {
            while let Some(c) = cur.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    cur.bump();
                } else {
                    break;
                }
            }
            if cur.peek() == Some('!') {
                cur.bump();
            }
            out.push(HighlightSpan::new(
                rocci_template::Span::new(offset + start, offset + cur.pos),
                HighlightKind::Variable,
                0,
                20,
            ));
        } else if ch == ':'
            || ch == ','
            || ch == '('
            || ch == ')'
            || ch == '{'
            || ch == '}'
            || ch == '['
            || ch == ']'
        {
            cur.bump();
            out.push(HighlightSpan::new(
                rocci_template::Span::new(offset + start, offset + cur.pos),
                HighlightKind::Punctuation,
                0,
                20,
            ));
        } else if ch == '.' || ch == '=' || ch == '!' || ch == '?' || ch == '|' {
            cur.bump();
            out.push(HighlightSpan::new(
                rocci_template::Span::new(offset + start, offset + cur.pos),
                HighlightKind::Operator,
                0,
                20,
            ));
        } else {
            cur.bump();
        }
    }
}

fn map_roc_capture(capture: &str) -> Option<(HighlightKind, u32, u32)> {
    match capture {
        "keyword"
        | "keyword.control"
        | "keyword.control.import"
        | "keyword.control.conditional"
        | "keyword.control.return"
        | "keyword.operator"
        | "keyword.storage.type"
        | "constant.builtin.boolean" => Some((HighlightKind::Keyword, 0, 50)),

        "function" | "function.builtin" | "function.method" => {
            Some((HighlightKind::Function, 0, 45))
        }

        "type"
        | "type.builtin"
        | "type.definition"
        | "type.parameter"
        | "type.enum.variant"
        | "type.roc-special.inferred" => Some((HighlightKind::Type, 0, 45)),

        "constructor" => Some((HighlightKind::EnumMember, 0, 45)),

        "variable.parameter" => Some((HighlightKind::Parameter, 0, 45)),

        "variable.other.member" | "variable.other.member.roc-special.in-typedef" => {
            Some((HighlightKind::Property, 0, 40))
        }

        "variable" => Some((HighlightKind::Variable, 0, 30)),

        "string" | "string.special.url" => Some((HighlightKind::String, 0, 40)),

        "constant.numeric.integer" | "constant.numeric.float" => {
            Some((HighlightKind::Number, 0, 40))
        }
        "constant.character" | "constant.character.escape" => Some((HighlightKind::String, 0, 40)),

        "comment.line" => Some((HighlightKind::Comment, 0, 60)),
        "comment.block.documentation" => Some((HighlightKind::Comment, MOD_DOCUMENTATION, 60)),

        "operator" => Some((HighlightKind::Operator, 0, 40)),
        "namespace" | "namespace.roc-special.builtin" => Some((HighlightKind::Namespace, 0, 45)),
        "punctuation.delimiter" | "punctuation.bracket" | "punctuation.special" => {
            Some((HighlightKind::Punctuation, 0, 35))
        }

        "special.roc-special.package"
        | "special.roc-special.provided"
        | "special.roc-special.exposed" => Some((HighlightKind::Variable, 0, 35)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roc_highlight_record() {
        let code = r#"
read_count! = |db| {
    row = Sqlite.query!(
        {
            db,
            limits: Sqlite.default_query_limits,
        },
    )?
}
"#;
        let res = highlight(code);
        assert!(!res.is_empty(), "expected tokens for roc record");
        assert!(
            res.iter().any(|s| &code[s.start()..s.end()] == "Sqlite"),
            "expected Sqlite token"
        );
        assert!(
            res.iter()
                .any(|s| &code[s.start()..s.end()] == "default_query_limits"),
            "expected default_query_limits token"
        );
    }
}
