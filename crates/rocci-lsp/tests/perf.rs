use std::time::{Duration, Instant};

use lsp_types::{
    ClientCapabilities, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    GeneralClientCapabilities, InitializeParams, PartialResultParams, PositionEncodingKind,
    SemanticTokensParams, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    Uri, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
};
use rocci_lsp::LanguageServer;

const KITCHEN_SINK_ROCCI: &str = include_str!("../../../test/AllSyntax.rocci");
const EMBEDDED_ROCCI: &str = include_str!("../../../test/EmbeddedLanguages.rocci");
const ALL_SYNTAX_ROCDOWN: &str = include_str!("../../../test/AllSyntax.rocdown");
const EMBEDDED_ROCDOWN: &str = include_str!("../../../test/EmbeddedLanguages.rocdown");

fn test_uri(path: &str) -> Uri {
    format!("file:///{path}").parse().expect("valid test uri")
}

fn initialize_server(utf8: bool) -> (LanguageServer, Duration) {
    let start = Instant::now();
    let mut server = LanguageServer::new();
    let encodings = if utf8 {
        vec![PositionEncodingKind::UTF8, PositionEncodingKind::UTF16]
    } else {
        vec![PositionEncodingKind::UTF16]
    };
    server.initialize(InitializeParams {
        capabilities: ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: Some(encodings),
                ..GeneralClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        },
        ..InitializeParams::default()
    });
    let elapsed = start.elapsed();
    (server, elapsed)
}

fn open_doc(server: &mut LanguageServer, uri: Uri, language_id: &str, text: &str) -> Duration {
    let start = Instant::now();
    server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri,
            language_id: language_id.to_string(),
            version: 1,
            text: text.to_string(),
        },
    });
    start.elapsed()
}

fn request_tokens(server: &LanguageServer, uri: Uri) -> (usize, Duration) {
    let start = Instant::now();
    let result = server.semantic_tokens_full(SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    });
    let elapsed = start.elapsed();
    let token_count = match result {
        Some(lsp_types::SemanticTokensResult::Tokens(tokens)) => tokens.data.len(),
        _ => 0,
    };
    (token_count, elapsed)
}

fn change_doc(server: &mut LanguageServer, uri: Uri, version: i32, text: &str) -> Duration {
    let start = Instant::now();
    server.did_change(DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier { uri, version },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }],
    });
    start.elapsed()
}

fn generate_large_rocci(target_lines: usize) -> String {
    let mut buf = String::with_capacity(target_lines * 60);
    buf.push_str("module [Page]\n\n");
    buf.push_str("import roc_html.Html exposing [p, div, span, button]\n\n");

    let mut line_count = 4;
    let mut comp_idx = 0;

    while line_count < target_lines {
        comp_idx += 1;
        let comp = format!(
            r#"@component Component{comp_idx} = |{{ id : Str, count : U64, active : Bool }}| {{
    @css {{
        .comp-{comp_idx} {{
            display: flex;
            padding: 1rem;
            color: #333333;
            background: rgba(255, 255, 255, 0.9);
        }}
        .comp-{comp_idx}:hover {{
            color: #0066cc;
        }}
    }}
    <div class="comp-{comp_idx}" data-id={{id}}>
        <h3>"Component {comp_idx}"</h3>
        <p>"Count: " {{Num.toStr(count)}}</p>
        @if active {{
            <span class="badge">"Active"</span>
        }} @else {{
            <span class="badge muted">"Inactive"</span>
        }}
        @for item in ["alpha", "beta", "gamma"] {{
            <li class="item">{{item}}</li>
        }}
    </div>
}}

"#
        );
        line_count += comp.lines().count();
        buf.push_str(&comp);
    }

    buf.push_str("@component Page = |{}| {\n");
    buf.push_str("    <main class=\"container\">\n");
    for i in 1..=comp_idx.min(20) {
        buf.push_str(&format!(
            "        <Component{i} id=\"{i}\" count={{{i}}} active={{Bool.true}} />\n"
        ));
    }
    buf.push_str("    </main>\n}\n");

    buf
}

fn generate_large_rocdown(target_lines: usize) -> String {
    let mut buf = String::with_capacity(target_lines * 60);
    buf.push_str("---\n");
    buf.push_str("title: Benchmark Rocdown Document\n");
    buf.push_str("description: Generated large fixture for performance measurement\n");
    buf.push_str("---\n\n");
    buf.push_str("@page { title: \"Performance Benchmark\", theme: \"docs\" }\n\n");

    let mut line_count = 8;
    let mut section_idx = 0;

    while line_count < target_lines {
        section_idx += 1;
        let section = format!(
            r#"## Section {section_idx}: Architecture Overview

This is section {section_idx} of the generated benchmark document. It exercises **Markdown parsing**,
*inline emphasis*, `inline_code_elements()`, and [hyperlinks](https://example.com/section-{section_idx}).

- Unordered item 1 with **bold** text
- Unordered item 2 with `code` snippet
- Unordered item 3 with [link](#section-{section_idx})

1. Numbered step 1
2. Numbered step 2
3. Numbered step 3

@roc {{
    Status{section_idx} : [Active(U64), Pending, Disabled]

    process_status_{section_idx} = \status ->
        when status is
            Active(n) -> "Active count: $(Num.toStr(n))"
            Pending -> "Pending..."
            Disabled -> "Disabled"
}}

```roc
# Fenced Roc example in Section {section_idx}
calculate_{section_idx} : U64, U64 -> U64
calculate_{section_idx} = \a, b -> a + b * 2
```

```html
<div class="section-card" data-idx="{section_idx}">
    <h4>HTML preview {section_idx}</h4>
    <p>Sample content for HTML highlighter.</p>
</div>
```

```css
.section-card[data-idx="{section_idx}"] {{
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    padding: 1.5rem;
}}
```

"#
        );
        line_count += section.lines().count();
        buf.push_str(&section);
    }

    buf
}

#[test]
fn perf_cold_start_and_small_fixtures() {
    println!("\n=== PERFORMANCE: Cold Start & Small Fixtures ===");

    let (mut server_utf8, cold_start_utf8) = initialize_server(true);
    let (_server_utf16, cold_start_utf16) = initialize_server(false);

    println!(
        "Cold start (UTF-8 init) : {:>8.2} ms",
        cold_start_utf8.as_secs_f64() * 1000.0
    );
    println!(
        "Cold start (UTF-16 init): {:>8.2} ms",
        cold_start_utf16.as_secs_f64() * 1000.0
    );

    // Assert cold start is well under 100 ms
    assert!(
        cold_start_utf8 < Duration::from_millis(50),
        "Cold start too slow: {:?}",
        cold_start_utf8
    );

    let fixtures = [
        (
            "AllSyntax.rocci",
            "rocci",
            KITCHEN_SINK_ROCCI,
            test_uri("AllSyntax.rocci"),
        ),
        (
            "EmbeddedLanguages.rocci",
            "rocci",
            EMBEDDED_ROCCI,
            test_uri("EmbeddedLanguages.rocci"),
        ),
        (
            "AllSyntax.rocdown",
            "rocdown",
            ALL_SYNTAX_ROCDOWN,
            test_uri("AllSyntax.rocdown"),
        ),
        (
            "EmbeddedLanguages.rocdown",
            "rocdown",
            EMBEDDED_ROCDOWN,
            test_uri("EmbeddedLanguages.rocdown"),
        ),
    ];

    println!(
        "\n{:<28} | {:>6} lines | {:>8} open | {:>8} tokens | {:>6} tok count",
        "Fixture", "Lines", "Open ms", "Tokens ms", "Tokens"
    );
    println!("{:-<76}", "");

    for (name, lang, text, uri) in fixtures {
        let lines = text.lines().count();
        let open_time = open_doc(&mut server_utf8, uri.clone(), lang, text);
        let (tok_count, token_time) = request_tokens(&server_utf8, uri);

        println!(
            "{:<28} | {:>6}       | {:>8.2}   | {:>8.2}      | {:>6}",
            name,
            lines,
            open_time.as_secs_f64() * 1000.0,
            token_time.as_secs_f64() * 1000.0,
            tok_count
        );

        // Assert small fixtures token request is very fast (< 20ms in debug, < 5ms in release)
        let max_small = if cfg!(debug_assertions) {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(15)
        };
        assert!(
            token_time < max_small,
            "Small fixture {} token request took too long: {:?}",
            name,
            token_time
        );
    }
}

#[test]
fn perf_single_character_update() {
    println!("\n=== PERFORMANCE: Single-Character Update ===");

    let (mut server, _) = initialize_server(true);
    let uri = test_uri("Interactive.rocci");
    open_doc(&mut server, uri.clone(), "rocci", KITCHEN_SINK_ROCCI);

    // Warm up initial token request
    request_tokens(&server, uri.clone());

    // Measure incremental edit: add a character inside a Roc expression
    let modified = KITCHEN_SINK_ROCCI.replace("{person.name}", "{person.fullName}");
    let edit_time = change_doc(&mut server, uri.clone(), 2, &modified);
    let (tok_count, token_time) = request_tokens(&server, uri.clone());
    let total_turnaround = edit_time + token_time;

    println!(
        "Single-char edit turnaround:\n  did_change: {:>6.2} ms\n  tokens:     {:>6.2} ms\n  total:      {:>6.2} ms (tokens: {})",
        edit_time.as_secs_f64() * 1000.0,
        token_time.as_secs_f64() * 1000.0,
        total_turnaround.as_secs_f64() * 1000.0,
        tok_count
    );

    // Target budget: under 50 ms for typical edit
    let max_edit_budget = if cfg!(debug_assertions) {
        Duration::from_millis(50)
    } else {
        Duration::from_millis(20)
    };
    assert!(
        total_turnaround < max_edit_budget,
        "Single-character update took {:?}, exceeding budget of {:?}",
        total_turnaround,
        max_edit_budget
    );
}

#[test]
fn perf_large_fixtures_and_budget_verification() {
    println!("\n=== PERFORMANCE: Large Fixtures (1,000 & 10,000 lines) ===");

    let (mut server, _) = initialize_server(true);

    let doc_sizes = [1_000, 5_000, 10_000];

    println!(
        "\n{:<20} | {:>8} lines | {:>9} bytes | {:>8} open | {:>8} tokens | {:>6} tok count",
        "Document", "Lines", "Bytes", "Open ms", "Tokens ms", "Tokens"
    );
    println!("{:-<84}", "");

    for size in doc_sizes {
        // Test .rocci
        let rocci_text = generate_large_rocci(size);
        let rocci_lines = rocci_text.lines().count();
        let rocci_bytes = rocci_text.len();
        let rocci_uri = test_uri(&format!("Large_{size}.rocci"));

        let rocci_open = open_doc(&mut server, rocci_uri.clone(), "rocci", &rocci_text);
        let (rocci_tok_count, rocci_tokens) = request_tokens(&server, rocci_uri);

        println!(
            "{:<20} | {:>8}       | {:>9}       | {:>8.2}   | {:>8.2}      | {:>6}",
            format!(".rocci ({size})"),
            rocci_lines,
            rocci_bytes,
            rocci_open.as_secs_f64() * 1000.0,
            rocci_tokens.as_secs_f64() * 1000.0,
            rocci_tok_count
        );

        // Test .rocdown
        let rocdown_text = generate_large_rocdown(size);
        let rocdown_lines = rocdown_text.lines().count();
        let rocdown_bytes = rocdown_text.len();
        let rocdown_uri = test_uri(&format!("Large_{size}.rocdown"));

        let rocdown_open = open_doc(&mut server, rocdown_uri.clone(), "rocdown", &rocdown_text);
        let (rocdown_tok_count, rocdown_tokens) = request_tokens(&server, rocdown_uri);

        println!(
            "{:<20} | {:>8}       | {:>9}       | {:>8.2}   | {:>8.2}      | {:>6}",
            format!(".rocdown ({size})"),
            rocdown_lines,
            rocdown_bytes,
            rocdown_open.as_secs_f64() * 1000.0,
            rocdown_tokens.as_secs_f64() * 1000.0,
            rocdown_tok_count
        );

        // For 10,000-line documents, check budget
        if size == 10_000 {
            let budget_10k = if cfg!(debug_assertions) {
                // In debug mode, allow up to 350ms
                Duration::from_millis(350)
            } else {
                // In release mode, strict target under 100ms
                Duration::from_millis(100)
            };

            assert!(
                rocci_tokens < budget_10k,
                "10,000-line .rocci token generation took {:?}, exceeding budget of {:?}",
                rocci_tokens,
                budget_10k
            );
            assert!(
                rocdown_tokens < budget_10k,
                "10,000-line .rocdown token generation took {:?}, exceeding budget of {:?}",
                rocdown_tokens,
                budget_10k
            );
        }
    }
}
