use crate::token::{HighlightSpan, modifier_css_classes};

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render_spans(source: &str, spans: &[HighlightSpan]) -> String {
    if spans.is_empty() {
        return escape_html(source);
    }
    let mut html = String::with_capacity(source.len() * 2);
    let mut prev_end = 0usize;
    for span in spans {
        let start = span.start().min(source.len());
        let end = span.end().min(source.len());
        if start > prev_end {
            html.push_str(&escape_html(&source[prev_end..start]));
        }
        if start < end {
            let kind_class = span.kind.css_class();
            let mod_classes = modifier_css_classes(span.modifiers);
            let class_str = if mod_classes.is_empty() {
                kind_class.to_string()
            } else {
                format!("{kind_class} {}", mod_classes.join(" "))
            };
            html.push_str("<span class=\"");
            html.push_str(&class_str);
            html.push_str("\">");
            html.push_str(&escape_html(&source[start..end]));
            html.push_str("</span>");
        }
        prev_end = end;
    }
    if prev_end < source.len() {
        html.push_str(&escape_html(&source[prev_end..]));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{HighlightKind, HighlightSpan};
    use rocci_template::Span;

    #[test]
    fn empty_spans_escape_the_source() {
        assert_eq!(render_spans("<hi>", &[]), "&lt;hi&gt;");
    }

    #[test]
    fn wraps_token_spans() {
        let spans = [HighlightSpan::new(
            Span::new(0, 2),
            HighlightKind::Keyword,
            0,
            50,
        )];
        assert_eq!(
            render_spans("fn x", &spans),
            "<span class=\"tok-keyword\">fn</span> x"
        );
    }
}
