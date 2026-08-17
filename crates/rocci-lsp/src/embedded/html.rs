use rocci_highlight::HighlightSpan;

pub fn highlight(src: &str) -> Vec<HighlightSpan> {
    rocci_highlight::embedded::html::highlight(src)
}
