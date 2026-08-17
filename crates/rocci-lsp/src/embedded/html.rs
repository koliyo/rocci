use std::sync::OnceLock;

use super::tree_sitter::{HighlightToken, TreeSitterHighlighter};
use crate::tokens::*;

const HTML_HIGHLIGHTS_QUERY: &str = r#"
(tag_name) @tag
(erroneous_end_tag_name) @tag
(attribute_name) @attribute
(quoted_attribute_value) @string
(attribute_value) @string
(comment) @comment
(doctype) @keyword
[
  "<"
  ">"
  "</"
  "/>"
  "="
] @operator
"#;

unsafe extern "C" {
    fn tree_sitter_html() -> tree_sitter::Language;
}

pub fn language() -> tree_sitter::Language {
    unsafe { tree_sitter_html() }
}

static HIGHLIGHTER: OnceLock<Option<TreeSitterHighlighter>> = OnceLock::new();

pub fn highlight(src: &str) -> Vec<HighlightToken> {
    let highlighter = HIGHLIGHTER
        .get_or_init(|| TreeSitterHighlighter::new(language(), HTML_HIGHLIGHTS_QUERY).ok());

    let Some(hl) = highlighter.as_ref() else {
        return Vec::new();
    };

    hl.highlight(src, |capture| match capture {
        "tag" => Some((TOKEN_TYPE, MOD_DEFAULT_LIBRARY, 45)),
        "attribute" => Some((TOKEN_PROPERTY, 0, 45)),
        "string" => Some((TOKEN_STRING, 0, 40)),
        "comment" => Some((TOKEN_COMMENT, 0, 60)),
        "keyword" => Some((TOKEN_KEYWORD, 0, 50)),
        "operator" => Some((TOKEN_OPERATOR, 0, 35)),
        _ => None,
    })
}
