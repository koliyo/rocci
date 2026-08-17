use std::sync::OnceLock;

use super::tree_sitter::{HighlightToken, TreeSitterHighlighter};
use crate::tokens::*;

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

pub fn highlight(src: &str) -> Vec<HighlightToken> {
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
        "property" => Some((TOKEN_PROPERTY, 0, 45)),
        "type" => Some((TOKEN_TYPE, 0, 45)),
        "string" => Some((TOKEN_STRING, 0, 40)),
        "number" => Some((TOKEN_NUMBER, 0, 40)),
        "keyword" => Some((TOKEN_KEYWORD, 0, 50)),
        "comment" => Some((TOKEN_COMMENT, 0, 60)),
        "function" => Some((TOKEN_FUNCTION, 0, 45)),
        "variable" => Some((TOKEN_VARIABLE, 0, 30)),
        "operator" => Some((TOKEN_OPERATOR, 0, 35)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_highlight() {
        let res = highlight(".card { padding: 1rem; }");
        eprintln!("CSS highlight tokens: {:?}", res);
        assert!(!res.is_empty(), "expected CSS tokens");
    }
}
