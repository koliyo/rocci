use rocci_highlight::*;

#[test]
fn highlight_roc() {
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
fn highlight_css() {
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
fn highlight_html() {
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
fn highlight_rocci_composite() {
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
fn highlight_roc_qualified_module_identifiers() {
    let src = r#"
row = Sqlite.query!(
    {
        db,
        query: "SELECT value FROM counter WHERE id = 1",
        limits: Sqlite.default_query_limits,
    },
)?
res = Sqlite.execute!(db)
count_str = Num.toStr(42)
item = Dict.get(items, "key")
"#;
    let (lang, spans) = highlight_source("roc", src);
    assert_eq!(lang, LanguageId::Roc);
    assert_invariants(src, &spans);

    // Verify Sqlite.query! spans
    let sqlite_spans: Vec<_> = spans
        .iter()
        .filter(|s| &src[s.start()..s.end()] == "Sqlite")
        .collect();
    assert_eq!(sqlite_spans.len(), 3, "expected 3 Sqlite module references");
    for s in sqlite_spans {
        assert_eq!(s.kind, HighlightKind::Namespace);
    }

    // Verify qualified function / member spans match exact text without dropped first character
    let query_span = spans
        .iter()
        .find(|s| {
            s.start() > src.find("Sqlite.query").unwrap()
                && s.start() < src.find("Sqlite.query").unwrap() + 15
                && &src[s.start()..s.end()] == "query!"
        })
        .expect("query! span");
    assert_eq!(&src[query_span.start()..query_span.end()], "query!");

    let default_limits_span = spans
        .iter()
        .find(|s| &src[s.start()..s.end()] == "default_query_limits")
        .expect("default_query_limits span");
    assert_eq!(
        &src[default_limits_span.start()..default_limits_span.end()],
        "default_query_limits"
    );

    let execute_span = spans
        .iter()
        .find(|s| {
            s.start() > src.find("Sqlite.execute").unwrap()
                && s.start() < src.find("Sqlite.execute").unwrap() + 17
                && &src[s.start()..s.end()] == "execute!"
        })
        .expect("execute! span");
    assert_eq!(&src[execute_span.start()..execute_span.end()], "execute!");

    let to_str_span = spans
        .iter()
        .find(|s| &src[s.start()..s.end()] == "toStr")
        .expect("toStr span");
    assert_eq!(&src[to_str_span.start()..to_str_span.end()], "toStr");

    let dict_get_span = spans
        .iter()
        .find(|s| {
            s.start() > src.find("Dict.get").unwrap()
                && s.start() < src.find("Dict.get").unwrap() + 10
                && &src[s.start()..s.end()] == "get"
        })
        .expect("get span");
    assert_eq!(&src[dict_get_span.start()..dict_get_span.end()], "get");
}

#[test]
fn highlight_unknown_language_fallback() {
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
