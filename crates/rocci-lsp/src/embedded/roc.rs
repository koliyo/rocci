use std::sync::OnceLock;

use super::tree_sitter::{HighlightToken, TreeSitterHighlighter};
use crate::tokens::*;

unsafe extern "C" {
    fn tree_sitter_roc() -> tree_sitter::Language;
}

pub fn language() -> tree_sitter::Language {
    unsafe { tree_sitter_roc() }
}

const ROC_HIGHLIGHTS_QUERY: &str = include_str!("../../grammars/roc/queries/highlights.scm");

static HIGHLIGHTER: OnceLock<Option<TreeSitterHighlighter>> = OnceLock::new();

pub fn highlight(src: &str) -> Vec<HighlightToken> {
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

    hl.highlight(src, map_roc_capture)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roc_highlight() {
        let res = highlight("badgeClass = |s| s");
        assert!(!res.is_empty(), "expected tokens, but got none");
    }

    #[test]
    fn test_roc_highlight_rocdown() {
        let fixture = include_str!("../../../../test/EmbeddedLanguages.rocdown");
        let parsed = rocci_rocdown::parse(
            rocci_template::SourceFile::new("EmbeddedLanguages.rocdown", fixture),
            false,
        );
        for item in &parsed.document.items {
            if let rocci_rocdown::Item::Roc(roc) = item {
                let slice = &fixture[roc.body.start as usize..roc.body.end as usize];
                eprintln!(
                    "Rocdown @roc body ({} bytes):\n---\n{}\n---",
                    slice.len(),
                    slice
                );
                let tokens = highlight(slice);
                eprintln!("Rocdown @roc tokens: {}", tokens.len());
                for t in &tokens {
                    eprintln!(
                        "  token: span={:?} text={:?} kind={}",
                        t.span,
                        &slice[t.span.start as usize..t.span.end as usize],
                        t.kind
                    );
                }
            }
        }
    }
}

fn map_roc_capture(capture: &str) -> Option<(u32, u32, u32)> {
    match capture {
        "keyword"
        | "keyword.control"
        | "keyword.control.import"
        | "keyword.control.conditional"
        | "keyword.control.return"
        | "keyword.operator"
        | "keyword.storage.type"
        | "constant.builtin.boolean" => Some((TOKEN_KEYWORD, 0, 50)),

        "function" | "function.builtin" | "function.method" => Some((TOKEN_FUNCTION, 0, 45)),

        "type"
        | "type.builtin"
        | "type.definition"
        | "type.parameter"
        | "type.enum.variant"
        | "type.roc-special.inferred" => Some((TOKEN_TYPE, 0, 45)),

        "constructor" => Some((TOKEN_ENUM_MEMBER, 0, 45)),

        "variable.parameter" => Some((TOKEN_PARAMETER, 0, 45)),

        "variable.other.member" | "variable.other.member.roc-special.in-typedef" => {
            Some((TOKEN_PROPERTY, 0, 40))
        }

        "variable" => Some((TOKEN_VARIABLE, 0, 30)),

        "string" | "string.special.url" => Some((TOKEN_STRING, 0, 40)),

        "constant.numeric.integer" | "constant.numeric.float" => Some((TOKEN_NUMBER, 0, 40)),
        "constant.character" | "constant.character.escape" => Some((TOKEN_STRING, 0, 40)),

        "comment.line" => Some((TOKEN_COMMENT, 0, 60)),
        "comment.block.documentation" => Some((TOKEN_COMMENT, MOD_DOCUMENTATION, 60)),

        "operator" => Some((TOKEN_OPERATOR, 0, 40)),
        "namespace" | "namespace.roc-special.builtin" => Some((TOKEN_NAMESPACE, 0, 45)),
        "punctuation.delimiter" | "punctuation.bracket" | "punctuation.special" => {
            Some((TOKEN_OPERATOR, 0, 35))
        }

        "special.roc-special.package"
        | "special.roc-special.provided"
        | "special.roc-special.exposed" => Some((TOKEN_VARIABLE, 0, 35)),

        _ => None,
    }
}
