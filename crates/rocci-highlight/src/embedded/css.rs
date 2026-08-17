use std::sync::OnceLock;

use crate::token::{HighlightKind, HighlightSpan};
use crate::tree_sitter::TreeSitterHighlighter;

const CSS_HIGHLIGHTS_QUERY: &str = r#"
(property_name) @property
(class_name) @property
(id_name) @property
(tag_name) @type
(string_value) @string
(color_value) @number
(integer_value) @number
(float_value) @number
(unit) @keyword
(comment) @comment
(at_keyword) @keyword
(important) @keyword
(function_name) @function
(plain_value) @variable
[
  "{"
  "}"
  "("
  ")"
  "["
  "]"
  ":"
  ";"
  ","
  ">"
  "+"
  "~"
] @operator
"#;

unsafe extern "C" {
    fn tree_sitter_css() -> tree_sitter::Language;
}

pub fn language() -> tree_sitter::Language {
    unsafe { tree_sitter_css() }
}

static HIGHLIGHTER: OnceLock<Option<TreeSitterHighlighter>> = OnceLock::new();

pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    let highlighter = HIGHLIGHTER.get_or_init(|| {
        match TreeSitterHighlighter::new(language(), CSS_HIGHLIGHTS_QUERY) {
            Ok(h) => Some(h),
            Err(e) => {
                eprintln!("CSS TreeSitter query error: {:?}", e);
                None
            }
        }
    });

    let Some(hl) = highlighter.as_ref() else {
        return Vec::new();
    };

    hl.highlight(src, |capture| match capture {
        "property" => Some((HighlightKind::Property, 0, 45)),
        "type" => Some((HighlightKind::Type, 0, 45)),
        "string" => Some((HighlightKind::String, 0, 40)),
        "number" => Some((HighlightKind::Number, 0, 40)),
        "keyword" => Some((HighlightKind::Keyword, 0, 50)),
        "comment" => Some((HighlightKind::Comment, 0, 60)),
        "function" => Some((HighlightKind::Function, 0, 45)),
        "variable" => Some((HighlightKind::Variable, 0, 30)),
        "operator" => Some((HighlightKind::Operator, 0, 35)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_highlight() {
        let res = highlight(".card { padding: 1rem; }");
        assert!(!res.is_empty(), "expected CSS tokens");
    }
}
