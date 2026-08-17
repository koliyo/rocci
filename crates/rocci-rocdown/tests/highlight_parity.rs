use rocci_highlight::LanguageId;
use rocci_rocdown::{CompileOptions, SourceFile, compile, render_document};
use rocci_template::PositionEncoding;

#[test]
fn golden_parity_roc_snippet_drives_lsp_and_rocdown_html() {
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
    assert!(html.contains("<span class=\"tok-variable\">main</span>"));
    assert!(html.contains("<span class=\"tok-operator\">=</span>"));
    assert!(html.contains("<span class=\"tok-string\">&quot;Hello World&quot;</span>"));
}

#[test]
fn golden_parity_composite_rocci_snippet() {
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
fn golden_parity_composite_rocdown_snippet() {
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
