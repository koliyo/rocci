use rocci_highlight::HighlightSpan;

pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    rocci_highlight::embedded::roc::highlight(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roc_highlight() {
        let res = highlight("badgeClass = |s| s");
        assert!(!res.is_empty(), "expected tokens, but got none");
    }
}
