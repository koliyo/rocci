use rocci_highlight::HighlightSpan;

pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    rocci_highlight::embedded::css::highlight(src)
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
