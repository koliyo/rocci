use rocci_highlight::*;

#[test]
fn golden_roc_highlight() {
    let src = "main = \\{} -> \"Hello World\"";
    let (lang, spans) = highlight_source("roc", src);
    assert_eq!(lang, LanguageId::Roc);
    assert!(!spans.is_empty());

    // Invariant assertions
    assert_invariants(src, &spans);

    let variable_span = spans
        .iter()
        .find(|s| s.kind == HighlightKind::Variable)
        .expect("variable");
    assert_eq!(&src[variable_span.start()..variable_span.end()], "main");

    let string_span = spans
        .iter()
        .find(|s| s.kind == HighlightKind::String)
        .expect("string");
    assert_eq!(
        &src[string_span.start()..string_span.end()],
        "\"Hello World\""
    );
}

#[test]
fn golden_css_highlight() {
    let src = ".btn-primary { background: #e64b2f; padding: 0.5rem 1rem; }";
    let (lang, spans) = highlight_source("css", src);
    assert_eq!(lang, LanguageId::Css);
    assert_invariants(src, &spans);

    let prop_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.kind == HighlightKind::Property)
        .collect();
    assert!(!prop_spans.is_empty());
}

#[test]
fn golden_html_highlight() {
    let src = "<div class=\"hero-section\"><h1 id=\"main-title\">Hello</h1></div>";
    let (lang, spans) = highlight_source("html", src);
    assert_eq!(lang, LanguageId::Html);
    assert_invariants(src, &spans);

    let tag_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.kind == HighlightKind::Tag)
        .collect();
    assert!(!tag_spans.is_empty());
}

#[test]
fn golden_rocci_composite_highlight() {
    let src = r#"module [Card]

@component Card = |{ title : Str, count : U64 }| {
    @css {
        .card { padding: 1rem; color: #333; }
    }
    <div class="card">
        <h3>{title}</h3>
        <p>"Count: " {Num.toStr(count)}</p>
        @if count > 0 {
            <span class="active">"Available"</span>
        }
    </div>
}
"#;
    let (lang, spans) = highlight_source("rocci", src);
    assert_eq!(lang, LanguageId::Rocci);
    assert_invariants(src, &spans);

    // Verify component keyword & declaration
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Keyword && &src[s.start()..s.end()] == "@component")
    );
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Function && &src[s.start()..s.end()] == "Card")
    );

    // Verify embedded CSS tokens
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Keyword && &src[s.start()..s.end()] == "@css")
    );

    // Verify template elements
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Tag && &src[s.start()..s.end()] == "div")
    );
}

#[test]
fn golden_rocdown_composite_highlight() {
    let src = r#"# Rocdown Guide

Here is a paragraph with [Link](/url) and `inline code`.

@roc {
    message = "Hello from executable region"
}

```roc
# Display-only code fence
add = |a, b| a + b
```
"#;
    let (lang, spans) = highlight_source("rocdown", src);
    assert_eq!(lang, LanguageId::Rocdown);
    assert_invariants(src, &spans);

    // Verify markdown header keyword
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Keyword && &src[s.start()..s.end()] == "#")
    );

    // Verify executable @roc keyword
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Keyword && &src[s.start()..s.end()] == "@roc")
    );

    // Verify fenced code tokens
    assert!(spans.iter().any(|s| s.kind == HighlightKind::Comment
        && &src[s.start()..s.end()] == "# Display-only code fence"));
}

#[test]
fn golden_unknown_language_fallback() {
    let src = "fn main() { println!(\"hello\"); }";
    let (lang, spans) = highlight_source("rust", src);
    assert_eq!(lang, LanguageId::Other("rust".to_string()));
    assert!(
        spans.is_empty(),
        "unknown language produces empty spans for safe fallback"
    );
}

fn assert_invariants(src: &str, spans: &[HighlightSpan]) {
    let mut prev_end = 0;
    for (i, span) in spans.iter().enumerate() {
        assert!(
            span.start() >= prev_end,
            "span {} ({:?}) overlaps previous end {}",
            i,
            span.span,
            prev_end
        );
        assert!(
            span.end() <= src.len(),
            "span {} ({:?}) exceeds source len {}",
            i,
            span.span,
            src.len()
        );
        assert!(
            src.is_char_boundary(span.start()),
            "span {} start {} not on char boundary",
            i,
            span.start()
        );
        assert!(
            src.is_char_boundary(span.end()),
            "span {} end {} not on char boundary",
            i,
            span.end()
        );
        assert!(!span.is_empty(), "span {} is empty", i);
        prev_end = span.end();
    }
}
