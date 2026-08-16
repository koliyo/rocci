use std::path::PathBuf;

use lsp_types::{
    ClientCapabilities, CompletionParams, CompletionResponse, DiagnosticSeverity,
    DidOpenTextDocumentParams, DocumentSymbolParams, DocumentSymbolResponse,
    GeneralClientCapabilities, GotoDefinitionParams, HoverParams, InitializeParams,
    PartialResultParams, Position, PositionEncodingKind, Range, SemanticTokens,
    SemanticTokensParams, TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
    Uri, WorkDoneProgressParams,
};
use rocci_lsp::{LanguageServer, TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_PROPERTY, TOKEN_TYPE};
use rocci_template::{PositionEncoding, SourceFile};

const KITCHEN_SINK: &str = include_str!("../../../test/AllSyntax.rocci");

const INCOMPLETE_TAG: &str = r#"
@component Broken = |{}| {
    <Hello name={person.name}
}

@component Ok = |{ name }| {
    <p>{name}</p>
}
"#;

fn test_uri() -> Uri {
    "file:///test.rocci".parse().expect("test uri")
}

fn initialize(utf8: bool) -> LanguageServer {
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
    server
}

fn open(server: &mut LanguageServer, text: &str) -> lsp_types::PublishDiagnosticsParams {
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: test_uri(),
                language_id: "rocci".to_string(),
                version: 1,
                text: text.to_string(),
            },
        })
        .expect("rocci documents should publish diagnostics")
}

fn identifier() -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: test_uri() }
}

fn position_params(line: u32, character: u32) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: identifier(),
        position: Position::new(line, character),
    }
}

#[test]
fn kitchen_sink_has_no_error_diagnostics_and_component_symbols() {
    let mut server = initialize(true);
    let published = open(&mut server, KITCHEN_SINK);
    assert!(
        published
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != Some(DiagnosticSeverity::ERROR)),
        "{:?}",
        published.diagnostics
    );

    let symbols = server
        .document_symbol(DocumentSymbolParams {
            text_document: identifier(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("symbols");
    let DocumentSymbolResponse::Nested(symbols) = symbols else {
        panic!("expected nested document symbols");
    };
    let names: Vec<_> = symbols.iter().map(|symbol| symbol.name.as_str()).collect();
    assert!(names.contains(&"Badge"));
    assert!(names.contains(&"Hello"));
    assert!(names.contains(&"CounterPage"));
}

#[test]
fn incomplete_tag_publishes_an_error_range() {
    let mut server = initialize(true);
    let published = open(&mut server, INCOMPLETE_TAG);
    let error = published
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::ERROR))
        .expect("error diagnostic");
    assert!(
        error.range.start != error.range.end
            || error.range.start.character > 0
            || error.range.start.line > 0,
        "expected a located range, got {:?}",
        error.range
    );
}

#[test]
fn hello_tag_jumps_to_hello_component() {
    let mut server = initialize(true);
    open(&mut server, KITCHEN_SINK);
    let hello_tag = KITCHEN_SINK.find("<Hello").expect("Hello tag") + 1;
    let (line, character) = line_col(KITCHEN_SINK, hello_tag);

    let response = server
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("definition");
    let lsp_types::GotoDefinitionResponse::Scalar(location) = response else {
        panic!("expected a single location");
    };
    let hello_decl = KITCHEN_SINK.find("Hello = |{ name").expect("hello decl");
    let (decl_line, decl_character) = line_col(KITCHEN_SINK, hello_decl);
    assert_eq!(location.range.start.line, decl_line);
    assert_eq!(location.range.start.character, decl_character);

    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("hover");
    let lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("@component Hello ="));
}

#[test]
fn utf8_and_utf16_map_non_bmp_diagnostics_differently() {
    let src = "@component View = |{}| {\n    😀@fi ready {\n        <Ready />\n    }\n}\n";
    let mut utf8 = initialize(true);
    let mut utf16 = initialize(false);
    assert_eq!(utf8.encoding(), PositionEncoding::Utf8);
    assert_eq!(utf16.encoding(), PositionEncoding::Utf16);

    let utf8_diag = open(&mut utf8, src)
        .diagnostics
        .into_iter()
        .find(|diagnostic| diagnostic.message.contains("`@if`"))
        .expect("utf-8 diagnostic");
    let utf16_diag = open(&mut utf16, src)
        .diagnostics
        .into_iter()
        .find(|diagnostic| diagnostic.message.contains("`@if`"))
        .expect("utf-16 diagnostic");

    assert_eq!(utf8_diag.range.start.line, utf16_diag.range.start.line);
    assert_ne!(
        utf8_diag.range.start.character, utf16_diag.range.start.character,
        "non-BMP characters should shift UTF-8 vs UTF-16 columns"
    );
}

#[test]
fn completes_local_component_and_directive() {
    let mut server = initialize(true);
    open(&mut server, KITCHEN_SINK);

    let hello_tag = KITCHEN_SINK.find("<Hello").expect("Hello tag") + 1;
    let (line, character) = line_col(KITCHEN_SINK, hello_tag);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("completion")
    else {
        panic!("expected completion array");
    };
    assert!(items.iter().any(|item| item.label == "Hello"));

    let at_if = KITCHEN_SINK.find("@if").expect("@if") + 1;
    let (line, character) = line_col(KITCHEN_SINK, at_if);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("directive completion")
    else {
        panic!("expected completion array");
    };
    assert!(items.iter().any(|item| item.label == "if"));
}

#[test]
fn template_tokens_leave_roc_regions_for_nested_highlighting() {
    let mut server = initialize(true);
    open(&mut server, KITCHEN_SINK);
    let result = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: identifier(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("semantic tokens");
    let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
        panic!("expected full semantic tokens");
    };

    let component_kw = KITCHEN_SINK.find("@component").expect("component");
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, component_kw),
        Some(TOKEN_KEYWORD)
    );
    let hello_tag = KITCHEN_SINK.find("<Hello").expect("Hello") + 1;
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, hello_tag),
        Some(TOKEN_FUNCTION)
    );
    let span_tag = KITCHEN_SINK.find("<span").expect("span") + 1;
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, span_tag),
        Some(TOKEN_TYPE)
    );
    let class_attr = KITCHEN_SINK.find("class=").expect("class");
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, class_attr),
        Some(TOKEN_PROPERTY)
    );
    let at_if = KITCHEN_SINK.find("@if").expect("@if");
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, at_if),
        Some(TOKEN_KEYWORD)
    );

    let roc_fn = KITCHEN_SINK.find("badgeClass =").expect("badgeClass");
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, roc_fn),
        None,
        "ordinary Roc should not be tokenized by the template server"
    );
    let interp = KITCHEN_SINK.find("{name}").expect("name interp") + 1;
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, interp),
        None,
        "Roc interpolation holes should be left for nested Roc highlighting"
    );

    let regions = server
        .embedded_ranges(&test_uri())
        .expect("embedded ranges");
    assert!(regions.iter().any(
        |region| region.language == "roc" && range_covers(&region.range, KITCHEN_SINK, roc_fn,)
    ));
    assert!(regions.iter().any(
        |region| region.language == "roc" && range_covers(&region.range, KITCHEN_SINK, interp)
    ));
    let params = KITCHEN_SINK
        .find("|{ name ?? \"World\" }|")
        .expect("params")
        + 3;
    assert!(regions.iter().any(
        |region| region.language == "roc" && range_covers(&region.range, KITCHEN_SINK, params)
    ));
}

#[test]
fn css_blocks_are_keywords_with_embedded_css_ranges() {
    const SRC: &str = r#"
@css {
    .card { padding: 1rem; }
}

@component Hello = |{ name }| {
    @css {
        .greeting { color: navy; }
    }
    <p class="greeting">{name}</p>
}
"#;
    let mut server = initialize(true);
    open(&mut server, SRC);
    let result = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: identifier(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("semantic tokens");
    let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
        panic!("expected full semantic tokens");
    };

    let file_css = SRC.find("@css").expect("file @css");
    assert_eq!(token_type_at(SRC, &tokens, file_css), Some(TOKEN_KEYWORD));
    let component_css = SRC.rfind("@css").expect("component @css");
    assert_eq!(
        token_type_at(SRC, &tokens, component_css),
        Some(TOKEN_KEYWORD)
    );

    let regions = server
        .embedded_ranges(&test_uri())
        .expect("embedded ranges");
    let card = SRC.find(".card").expect("card");
    let greeting = SRC.find(".greeting").expect("greeting");
    assert!(
        regions
            .iter()
            .any(|region| region.language == "css" && range_covers(&region.range, SRC, card))
    );
    assert!(
        regions
            .iter()
            .any(|region| region.language == "css" && range_covers(&region.range, SRC, greeting))
    );
}

const GUIDE: &str = include_str!("../../../examples/rocdown/Guide.rocdown");

fn rocdown_uri() -> Uri {
    "file:///test.rocdown".parse().expect("test uri")
}

fn guide_uri() -> Uri {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/rocdown/Guide.rocdown")
        .canonicalize()
        .expect("guide path");
    format!("file://{}", path.display())
        .parse()
        .expect("guide uri")
}

fn open_rocdown_at(
    server: &mut LanguageServer,
    uri: Uri,
    text: &str,
) -> lsp_types::PublishDiagnosticsParams {
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "rocdown".to_string(),
                version: 1,
                text: text.to_string(),
            },
        })
        .expect("rocdown documents should publish diagnostics")
}

fn open_rocdown(server: &mut LanguageServer, text: &str) -> lsp_types::PublishDiagnosticsParams {
    open_rocdown_at(server, rocdown_uri(), text)
}

fn rocdown_identifier() -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: rocdown_uri() }
}

fn rocdown_position_params(line: u32, character: u32) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: rocdown_identifier(),
        position: Position::new(line, character),
    }
}

#[test]
fn guide_has_no_error_diagnostics_and_expected_symbols() {
    let mut server = initialize(true);
    let published = open_rocdown_at(&mut server, guide_uri(), GUIDE);
    assert!(
        published
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != Some(DiagnosticSeverity::ERROR)),
        "{:?}",
        published.diagnostics
    );

    let symbols = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: guide_uri() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("symbols");
    let DocumentSymbolResponse::Nested(symbols) = symbols else {
        panic!("expected nested document symbols");
    };
    let names: Vec<_> = symbols.iter().map(|symbol| symbol.name.as_str()).collect();
    assert!(names.contains(&"@page"), "{names:?}");
    assert!(names.contains(&"FeatureCount"), "{names:?}");
    assert!(names.contains(&"Rocdown"), "{names:?}");
}

#[test]
fn unknown_page_field_publishes_a_located_error() {
    const SRC: &str = "@page { extra: 1 }\n";
    let mut server = initialize(true);
    let published = open_rocdown(&mut server, SRC);
    let error = published
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::ERROR))
        .expect("error diagnostic");
    assert!(
        error.message.contains("unknown `@page` field"),
        "{}",
        error.message
    );
    assert!(
        error.range.start != error.range.end
            || error.range.start.character > 0
            || error.range.start.line > 0,
        "expected a located range, got {:?}",
        error.range
    );
}

#[test]
fn rocdown_tokens_and_embedded_roc_ranges() {
    let mut server = initialize(true);
    open_rocdown_at(&mut server, guide_uri(), GUIDE);
    let result = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: guide_uri() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("semantic tokens");
    let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
        panic!("expected full semantic tokens");
    };

    let component_kw = GUIDE.find("@component").expect("component");
    assert_eq!(
        token_type_at(GUIDE, &tokens, component_kw),
        Some(TOKEN_KEYWORD)
    );
    let roc_kw = GUIDE.find("@roc").expect("@roc");
    assert_eq!(token_type_at(GUIDE, &tokens, roc_kw), Some(TOKEN_KEYWORD));
    let published = GUIDE.find("published =").expect("published");
    assert_eq!(
        token_type_at(GUIDE, &tokens, published),
        None,
        "executable @roc body should be left for nested Roc highlighting"
    );
    let fence = GUIDE.find("answer = 42").expect("fenced roc");
    assert_eq!(
        token_type_at(GUIDE, &tokens, fence),
        None,
        "display fences should not be tokenized as executable Roc"
    );

    let regions = server
        .embedded_ranges(&guide_uri())
        .expect("embedded ranges");
    assert!(regions.iter().any(
        |region| region.language == "roc" && range_covers(&region.range, GUIDE, published)
    ));
    assert!(
        !regions
            .iter()
            .any(|region| region.language == "roc" && range_covers(&region.range, GUIDE, fence)),
        "display fences must not be executable roc ranges"
    );
    let css_body = GUIDE.find("box-sizing").expect("css");
    assert!(
        regions
            .iter()
            .any(|region| region.language == "css" && range_covers(&region.range, GUIDE, css_body))
    );
}

#[test]
fn rocdown_root_at_completes_declarations_not_html() {
    const SRC: &str = "\
@page {
    route: \"/\",
}

@r
";
    let mut server = initialize(true);
    open_rocdown(&mut server, SRC);
    let at_r = SRC.find("@r").expect("@r") + 2;
    let (line, character) = line_col(SRC, at_r);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: rocdown_position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("completion")
    else {
        panic!("expected completion array");
    };
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"roc"), "{labels:?}");
    assert!(labels.contains(&"render"), "{labels:?}");
    assert!(!labels.contains(&"div"), "{labels:?}");
    assert!(!labels.contains(&"p"), "{labels:?}");
}

#[test]
fn rocdown_page_completes_theme_ids() {
    const SRC: &str = "@page {\n    theme: \"roc\"\n}\n";
    let mut server = initialize(true);
    open_rocdown(&mut server, SRC);
    let at = SRC.find("roc").expect("roc") + 3;
    let (line, character) = line_col(SRC, at);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: rocdown_position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("completion")
    else {
        panic!("expected completion array");
    };
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(
        labels.iter().any(|label| label.contains("rocci")),
        "{labels:?}"
    );
}

fn token_type_at(src: &str, tokens: &SemanticTokens, offset: usize) -> Option<u32> {
    let (line, character) =
        SourceFile::new("t.rocci", src).position(offset as u32, PositionEncoding::Utf8);
    let mut cur_line = 0u32;
    let mut cur_col = 0u32;
    for token in &tokens.data {
        cur_line += token.delta_line;
        cur_col = if token.delta_line == 0 {
            cur_col + token.delta_start
        } else {
            token.delta_start
        };
        if cur_line == line && cur_col <= character && character < cur_col + token.length {
            return Some(token.token_type);
        }
    }
    None
}

fn range_covers(range: &Range, src: &str, offset: usize) -> bool {
    let (line, character) =
        SourceFile::new("t.rocci", src).position(offset as u32, PositionEncoding::Utf8);
    let start = (range.start.line, range.start.character);
    let end = (range.end.line, range.end.character);
    let pos = (line, character);
    start <= pos && pos < end
}

fn line_col(src: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut start = 0usize;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            start = i + 1;
        }
    }
    (line, (offset - start) as u32)
}
