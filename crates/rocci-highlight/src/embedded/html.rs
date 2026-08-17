use std::sync::OnceLock;

use crate::token::{HighlightKind, HighlightSpan, MOD_DEFAULT_LIBRARY};
use crate::tree_sitter::TreeSitterHighlighter;

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

pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    let highlighter = HIGHLIGHTER
        .get_or_init(|| TreeSitterHighlighter::new(language(), HTML_HIGHLIGHTS_QUERY).ok());

    let Some(hl) = highlighter.as_ref() else {
        return Vec::new();
    };

    hl.highlight(src, |capture| match capture {
        "tag" => Some((HighlightKind::Tag, MOD_DEFAULT_LIBRARY, 45)),
        "attribute" => Some((HighlightKind::Property, 0, 45)),
        "string" => Some((HighlightKind::String, 0, 40)),
        "comment" => Some((HighlightKind::Comment, 0, 60)),
        "keyword" => Some((HighlightKind::Keyword, 0, 50)),
        "operator" => Some((HighlightKind::Operator, 0, 35)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_highlight() {
        let res = highlight("<div class=\"container\"><p>Hello</p></div>");
        assert!(!res.is_empty(), "expected HTML tokens");
    }
}
