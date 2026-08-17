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

    hl.highlight(src, map_roc_capture)
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
    fn test_roc_highlight() {
        let res = highlight("main = \\{} -> \"Hello\"");
        assert!(!res.is_empty(), "expected tokens for roc, but got none");
    }
}
