pub mod composite;
pub mod embedded;
pub mod language;
pub mod regions;
pub mod token;
pub mod tree_sitter;

pub use composite::{highlight, highlight_rocci, highlight_rocci_document};
pub use language::LanguageId;
pub use regions::{
    Region, RegionBuilder, RegionContext, RegionPurpose, RegionTree, RegionValidationError,
    css_ranges, executable_roc_ranges, extract_rocci_regions,
};
pub use token::{
    HighlightKind, HighlightSpan, MOD_DECLARATION, MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION,
    MOD_READONLY, floor_char_boundary, for_each_line_span, modifier_css_classes,
    resolve_and_sort_spans,
};

pub fn highlight_source(info: &str, source: &str) -> (LanguageId, Vec<HighlightSpan>) {
    let lang = LanguageId::parse(info);
    let spans = highlight(lang.clone(), source);
    (lang, spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_roc_snippet() {
        let (lang, spans) = highlight_source("roc", "main = \\{} -> \"Hello\"");
        assert_eq!(lang, LanguageId::Roc);
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlight_css_snippet() {
        let (lang, spans) = highlight_source("css", ".card { padding: 1rem; }");
        assert_eq!(lang, LanguageId::Css);
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlight_html_snippet() {
        let (lang, spans) = highlight_source("html", "<div class=\"hero\"><h1>Hi</h1></div>");
        assert_eq!(lang, LanguageId::Html);
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlight_rocci_snippet() {
        let (lang, spans) = highlight_source(
            "rocci",
            "@component Card = |{ title }| { <div class=\"card\">{title}</div> }",
        );
        assert_eq!(lang, LanguageId::Rocci);
        assert!(!spans.is_empty());
    }
}
