use rocci_highlight::{HighlightKind, LanguageId};
use rocci_rocdown::{
    CompileOptions, PageRef, SourceFile, compile, highlight_rocdown, highlight_rocdown_document,
    render_document,
};
use rocci_template::PositionEncoding;
use std::path::PathBuf;

#[test]
fn parity_roc_snippet_drives_lsp_and_rocdown_html() {
    let snippet = "main = \\{} -> \"Hello World\"";
    let (lang, spans) = rocci_highlight::highlight_source("roc", snippet);
    assert_eq!(lang, LanguageId::Roc);

    // 1. Check LSP encoding from the spans
    let source = SourceFile::new("snippet.roc", snippet);
    let mut raw_tokens: Vec<rocci_lsp::tokens::RawToken> = spans
        .iter()
        .map(|s| rocci_lsp::tokens::RawToken {
            span: s.span,
            kind: s.kind.to_lsp_index(),
            modifiers: s.modifiers,
            priority: s.priority,
        })
        .collect();
    let lsp_tokens =
        rocci_lsp::tokens::encode_tokens(source, &mut raw_tokens, PositionEncoding::Utf8, None);
    assert!(!lsp_tokens.is_empty(), "LSP semantic tokens produced");

    // 2. Check Rocdown HTML rendering from the same snippet
    let rocdown_doc = format!("```roc\n{snippet}\n```\n");
    let compiled = compile(
        SourceFile::new("test.rocdown", &rocdown_doc),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!compiled.has_errors());
    let html = render_document(&compiled.document);

    assert!(html.contains("<pre class=\"rd-code-block\" data-language=\"roc\">"));
    assert!(html.contains("<code class=\"rd-code language-roc\">"));
    assert!(html.contains("<span class=\"tok-parameter\">main</span>"));
    assert!(html.contains("<span class=\"tok-operator\">=</span>"));
    assert!(html.contains("<span class=\"tok-string\">&quot;Hello World&quot;</span>"));
}

#[test]
fn parity_composite_rocci_snippet() {
    let snippet = r#"@component Card = |{ title }| {
    @css { .card { padding: 1rem; } }
    <div class="card">{title}</div>
}"#;
    let spans = rocci_highlight::highlight_rocci(snippet);
    assert!(!spans.is_empty());

    // HTML rendering check
    let rocdown_doc = format!("```rocci\n{snippet}\n```\n");
    let compiled = compile(
        SourceFile::new("test.rocdown", &rocdown_doc),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!compiled.has_errors());
    let html = render_document(&compiled.document);

    assert!(html.contains("<pre class=\"rd-code-block\" data-language=\"rocci\">"));
    assert!(html.contains("<span class=\"tok-keyword\">@component</span>"));
    assert!(html.contains("<span class=\"tok-function tok-definition\">Card</span>"));
    assert!(html.contains("<span class=\"tok-keyword\">@css</span>"));
    assert!(html.contains("<span class=\"tok-tag tok-default-library\">div</span>"));
}

#[test]
fn page_record_highlights_fields_and_string_values() {
    let src = include_str!("../../../test/AllSyntax.rocdown");
    let spans = rocci_rocdown::highlight_rocdown(src);
    let end = src.find("\n@roc").unwrap_or(src.len());
    let page = &src[..end];
    let texts: Vec<_> = spans
        .iter()
        .filter(|s| s.end() <= end)
        .map(|s| (s.kind, &src[s.start()..s.end()]))
        .collect();
    assert!(
        texts.iter().any(|(kind, text)| {
            *kind == rocci_highlight::HighlightKind::Keyword && *text == "@page"
        }),
        "{texts:?}"
    );
    for field in [
        "route",
        "draft",
        "theme",
        "color_scheme",
        "meta",
        "title",
        "description",
    ] {
        assert!(
            texts.iter().any(|(kind, text)| {
                *kind == rocci_highlight::HighlightKind::Property && *text == field
            }),
            "expected property {field}: {texts:?}"
        );
    }
    for value in [
        "\"/all-syntax/\"",
        "\"paper\"",
        "\"auto\"",
        "\"All syntax\"",
        "\"Rocdown kitchen sink\"",
    ] {
        assert!(
            texts.iter().any(|(kind, text)| {
                *kind == rocci_highlight::HighlightKind::String && *text == value
            }),
            "expected string {value}: {texts:?}"
        );
    }
    assert!(
        texts.iter().any(|(kind, text)| {
            matches!(
                *kind,
                rocci_highlight::HighlightKind::Keyword
                    | rocci_highlight::HighlightKind::EnumMember
            ) && *text == "False"
        }),
        "{texts:?}"
    );
    assert!(
        !texts
            .iter()
            .any(|(_, text)| page.contains("\"/all-syntax/\"")
                && (*text == "all" || *text == "syntax")),
        "path-like @page string was split: {texts:?}"
    );
}

#[test]
fn parity_composite_rocdown_snippet() {
    let snippet = "# Header\n\n```roc\nx = 1\n```\n";
    let spans = rocci_rocdown::highlight_rocdown(snippet);
    assert!(!spans.is_empty());

    let rocdown_doc = format!("```rocdown\n{snippet}\n```\n");
    let compiled = compile(
        SourceFile::new("test.rocdown", &rocdown_doc),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!compiled.has_errors());
    let html = render_document(&compiled.document);

    assert!(html.contains("<pre class=\"rd-code-block\" data-language=\"rocdown\">"));
    assert!(html.contains("<span class=\"tok-keyword\">#</span>"));
}

#[test]
fn test_hostile_html_and_unclosed_constructs() {
    let hostile = "<script>alert('xss') & \"quotes\"</script>";
    let rocdown_doc = format!("```html\n{hostile}\n```\n\n```unknown\n{hostile}\n```\n");
    let compiled = compile(
        SourceFile::new("test.rocdown", &rocdown_doc),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!compiled.has_errors());
    let html = render_document(&compiled.document);

    // Ensure raw `<script>` is never in the HTML
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&amp;"));
    assert!(html.contains("&quot;quotes&quot;"));
}

#[test]
fn markdown_prose_matches_md_shape() {
    let src = "Shared streams are [handlers](/docs/applications/handlers/).\n\nA standalone **app** is a directory.\n";
    let spans = highlight_rocdown(src);
    let painted: Vec<_> = spans
        .iter()
        .map(|s| (&src[s.start()..s.end()], s.kind))
        .collect();
    assert!(
        painted
            .iter()
            .any(|(text, kind)| { *kind == HighlightKind::Variable && *text == "[handlers]" }),
        "{painted:?}"
    );
    assert!(
        painted.iter().any(|(text, kind)| {
            *kind == HighlightKind::Keyword && *text == "(/docs/applications/handlers/)"
        }),
        "{painted:?}"
    );
    assert!(
        !painted
            .iter()
            .any(|(text, _)| *text == "applications/handlers/" || *text == "/docs/"),
        "destination must not be split: {painted:?}"
    );
    assert!(
        painted
            .iter()
            .any(|(text, kind)| *kind == HighlightKind::Operator && *text == "**app**"),
        "{painted:?}"
    );
}

#[test]
fn link_destination_uses_source_text_after_resolve() {
    let src = "See [handlers](/docs/applications/handlers/).\n";
    let compiled = compile(
        SourceFile::new("standalone.rocdown", src),
        &CompileOptions {
            resolve_links: true,
            pages: vec![PageRef {
                stem: "handlers".into(),
                file_name: "handlers.rocdown".into(),
                path: PathBuf::from("applications/handlers.rocdown"),
                route: "/applications/handlers/".into(),
                explicit_route: false,
                heading_ids: Vec::new(),
            }],
            ..CompileOptions::default()
        },
    );
    let spans = highlight_rocdown_document(src, &compiled.document, &compiled.headings);
    let painted: Vec<_> = spans
        .iter()
        .map(|s| (&src[s.start()..s.end()], s.kind))
        .collect();
    assert!(
        painted.iter().any(|(text, kind)| {
            *kind == HighlightKind::Keyword && *text == "(/docs/applications/handlers/)"
        }),
        "resolved url must not steal a path suffix: {painted:?}"
    );
}

#[test]
fn rocci_fence_highlights_host_keywords() {
    let src = "```rocci\n@context { db : Sqlite.Db }\n\n@init {\n  db = 1\n}\n```\n";
    let spans = highlight_rocdown(src);
    let painted: Vec<_> = spans
        .iter()
        .map(|s| (&src[s.start()..s.end()], s.kind))
        .collect();
    assert!(
        painted
            .iter()
            .any(|(text, kind)| *kind == HighlightKind::Keyword && *text == "@context"),
        "{painted:?}"
    );
    assert!(
        painted
            .iter()
            .any(|(text, kind)| *kind == HighlightKind::Keyword && *text == "@init"),
        "{painted:?}"
    );
    assert!(
        painted
            .iter()
            .any(|(text, kind)| *kind == HighlightKind::Punctuation && *text == "```"),
        "{painted:?}"
    );
}

#[test]
fn test_unknown_language_fallback() {
    let doc = "```foo_bar_lang\nsome arbitrary code\n```\n";
    let compiled = compile(
        SourceFile::new("test.rocdown", doc),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!compiled.has_errors());
    let html = render_document(&compiled.document);

    assert!(html.contains("<pre class=\"rd-code-block\"><code class=\"rd-code language-foo_bar_lang\">some arbitrary code\n</code></pre>"));
    assert!(!html.contains("data-language="));
}
