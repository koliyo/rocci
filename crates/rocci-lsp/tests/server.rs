use std::path::PathBuf;

use lsp_server::Request;
use lsp_types::{
    ClientCapabilities, CompletionParams, CompletionResponse, DiagnosticSeverity,
    DidOpenTextDocumentParams, DocumentSymbolParams, DocumentSymbolResponse,
    GeneralClientCapabilities, GotoDefinitionParams, HoverParams, InitializeParams,
    PartialResultParams, Position, PositionEncodingKind, Range, SemanticTokens,
    SemanticTokensParams, TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
    Uri, WorkDoneProgressParams,
};
use rocci_lsp::{
    InspectedRegion, Language, LanguageServer, RegionContext, RegionPurpose, TOKEN_ENUM_MEMBER,
    TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_PROPERTY, TOKEN_STRING, TOKEN_TYPE, TOKEN_VARIABLE,
    extract_rocci_regions, extract_rocdown_regions, method_inspect_regions,
};
use rocci_template::{PositionEncoding, SourceFile};

const KITCHEN_SINK: &str = include_str!("../../../test/AllSyntax.rocci");
const EMBEDDED_ROCCI: &str = include_str!("../../../test/EmbeddedLanguages.rocci");
const EMBEDDED_ROCDOWN: &str = include_str!("../../../test/EmbeddedLanguages.rocdown");
const ALL_SYNTAX_ROCDOWN: &str = include_str!("../../../test/AllSyntax.rocdown");

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
fn template_tokens_and_embedded_roc_highlighting() {
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
    assert!(
        token_type_at(KITCHEN_SINK, &tokens, roc_fn).is_some(),
        "ordinary Roc functions should be highlighted by the Roc lexical backend"
    );
    let interp = KITCHEN_SINK.find("{name}").expect("name interp") + 1;
    assert_eq!(
        token_type_at(KITCHEN_SINK, &tokens, interp),
        Some(TOKEN_VARIABLE),
        "Roc interpolation holes should be highlighted by the Roc lexical backend"
    );

    let regions = server
        .inspect_regions(&test_uri())
        .expect("inspected regions");
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, KITCHEN_SINK, roc_fn,)));
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, KITCHEN_SINK, interp)));
    let params = KITCHEN_SINK
        .find("|{ name ?? \"World\" }|")
        .expect("params")
        + 3;
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, KITCHEN_SINK, params)));
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
        .inspect_regions(&test_uri())
        .expect("inspected regions");
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

fn embedded_rocdown_uri() -> Uri {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test/EmbeddedLanguages.rocdown")
        .canonicalize()
        .expect("embedded rocdown path");
    format!("file://{}", path.display())
        .parse()
        .expect("embedded rocdown uri")
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
    assert!(
        token_type_at(GUIDE, &tokens, published).is_some(),
        "executable @roc body should be highlighted by Roc lexical backend"
    );
    let fence = GUIDE.find("answer = 42").expect("fenced roc");
    assert!(
        token_type_at(GUIDE, &tokens, fence).is_some(),
        "display fences should be highlighted by lexical backend"
    );

    let regions = server
        .inspect_regions(&guide_uri())
        .expect("inspected regions");
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, GUIDE, published)));
    assert!(
        !regions.iter().any(|region| region.language == "roc"
            && region.purpose == "executable"
            && range_covers(&region.range, GUIDE, fence)),
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

#[test]
fn embedded_languages_rocci_symbols_tokens_and_regions() {
    let mut server = initialize(true);
    let published = open(&mut server, EMBEDDED_ROCCI);
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
    assert!(names.contains(&"UserCard"), "{names:?}");
    assert!(names.contains(&"Badge"), "{names:?}");
    assert!(names.contains(&"StatusView"), "{names:?}");
    assert!(names.contains(&"UserDirectory"), "{names:?}");
    assert!(names.contains(&"sampleUser"), "{names:?}");
    assert!(names.contains(&"State"), "{names:?}");
    assert!(names.contains(&"init!"), "{names:?}");

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

    let user_card_kw = EMBEDDED_ROCCI
        .find("@component\nUserCard")
        .expect("@component");
    assert_eq!(
        token_type_at(EMBEDDED_ROCCI, &tokens, user_card_kw),
        Some(TOKEN_KEYWORD)
    );
    let section_tag = EMBEDDED_ROCCI.find("<section").expect("<section") + 1;
    assert_eq!(
        token_type_at(EMBEDDED_ROCCI, &tokens, section_tag),
        Some(TOKEN_TYPE)
    );
    let id_attr = EMBEDDED_ROCCI.find("id=\"user-profile\"").expect("id attr");
    assert_eq!(
        token_type_at(EMBEDDED_ROCCI, &tokens, id_attr),
        Some(TOKEN_PROPERTY)
    );

    let action_kw = EMBEDDED_ROCCI.find("@post").expect("@post");
    assert_eq!(
        token_type_at(EMBEDDED_ROCCI, &tokens, action_kw),
        Some(TOKEN_KEYWORD)
    );

    let ordinary_roc = EMBEDDED_ROCCI.find("formatUser =").expect("formatUser");
    assert!(
        token_type_at(EMBEDDED_ROCCI, &tokens, ordinary_roc).is_some(),
        "ordinary Roc should be tokenized by the Roc lexical backend"
    );

    let regions = server
        .inspect_regions(&test_uri())
        .expect("inspected regions");
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, EMBEDDED_ROCCI, ordinary_roc)));
    let css_pos = EMBEDDED_ROCCI
        .find(".card--selected")
        .expect("card--selected");
    assert!(
        regions.iter().any(|region| region.language == "css"
            && range_covers(&region.range, EMBEDDED_ROCCI, css_pos))
    );
}

#[test]
fn embedded_languages_rocdown_symbols_tokens_and_regions() {
    let mut server = initialize(true);
    let uri = embedded_rocdown_uri();
    let published = open_rocdown_at(&mut server, uri.clone(), EMBEDDED_ROCDOWN);
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
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("symbols");
    let DocumentSymbolResponse::Nested(symbols) = symbols else {
        panic!("expected nested document symbols");
    };
    let names: Vec<_> = symbols.iter().map(|symbol| symbol.name.as_str()).collect();
    assert!(names.contains(&"@page"), "{names:?}");
    assert!(names.contains(&"@roc"), "{names:?}");
    assert!(names.contains(&"Card"), "{names:?}");
    assert!(names.contains(&"cardSample"), "{names:?}");
    assert!(names.contains(&"@docs note"), "{names:?}");
    assert!(names.contains(&"@docs tabs"), "{names:?}");
    assert!(names.contains(&"@docs include"), "{names:?}");
    assert!(names.contains(&"@docs example"), "{names:?}");
    assert!(names.contains(&"@docs link-card"), "{names:?}");
    assert!(names.contains(&"@docs details"), "{names:?}");
    assert!(
        names.contains(&"Embedded Languages Test Suite 🌐"),
        "{names:?}"
    );

    let tabs_sym = symbols
        .iter()
        .find(|sym| sym.name == "@docs tabs")
        .expect("@docs tabs symbol");
    let tab_children = tabs_sym.children.as_ref().expect("tabs children");
    let child_names: Vec<_> = tab_children.iter().map(|s| s.name.as_str()).collect();
    assert!(child_names.contains(&"@docs tab"), "{child_names:?}");

    let result = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("semantic tokens");
    let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
        panic!("expected full semantic tokens");
    };

    let docs_kw = EMBEDDED_ROCDOWN.find("@docs note").expect("@docs note");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &tokens, docs_kw),
        Some(TOKEN_KEYWORD)
    );
    let note_kind = EMBEDDED_ROCDOWN.find("note {").expect("note {");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &tokens, note_kind),
        Some(TOKEN_TYPE)
    );
    let title_field = EMBEDDED_ROCDOWN
        .find("title: \"Important")
        .expect("title field");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &tokens, title_field),
        Some(TOKEN_PROPERTY)
    );

    let regions = server.inspect_regions(&uri).expect("inspected regions");

    let roc_decl = EMBEDDED_ROCDOWN
        .find("Status : [Active(U64)")
        .expect("roc decl");
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, EMBEDDED_ROCDOWN, roc_decl)));
    let render_expr = EMBEDDED_ROCDOWN
        .find("Card({ title: \"Rendered")
        .expect("render expr");
    assert!(regions.iter().any(|region| region.language == "roc"
        && region.purpose == "executable"
        && range_covers(&region.range, EMBEDDED_ROCDOWN, render_expr)));
    let css_pos = EMBEDDED_ROCDOWN.find(".docs-banner").expect("docs-banner");
    assert!(
        regions.iter().any(|region| region.language == "css"
            && range_covers(&region.range, EMBEDDED_ROCDOWN, css_pos))
    );

    let display_roc = EMBEDDED_ROCDOWN
        .find("Hello from display Roc")
        .expect("display roc");
    assert!(
        !regions.iter().any(|region| region.language == "roc"
            && region.purpose == "executable"
            && range_covers(&region.range, EMBEDDED_ROCDOWN, display_roc)),
        "display fences must NOT be reported as executable roc regions"
    );
    assert!(
        regions.iter().any(|region| region.language == "roc"
            && region.purpose == "displayOnly"
            && range_covers(&region.range, EMBEDDED_ROCDOWN, display_roc)),
        "display fences MUST be reported as display-only fences"
    );
    let display_html = EMBEDDED_ROCDOWN
        .find("display-container")
        .expect("display-container");
    assert!(
        regions.iter().any(|region| region.language == "html"
            && region.purpose == "displayOnly"
            && range_covers(&region.range, EMBEDDED_ROCDOWN, display_html)),
        "display html fences MUST be reported as display-only html fences"
    );
    let escaped_docs = EMBEDDED_ROCDOWN.find(r"\@docs note").expect(r"\@docs note");
    assert!(
        !regions.iter().any(|region| region.purpose == "executable"
            && range_covers(&region.range, EMBEDDED_ROCDOWN, escaped_docs)),
        "escaped directives must NOT be reported as executable regions"
    );
}

#[test]
fn docs_hover_and_completion_work() {
    let mut server = initialize(true);
    let uri = embedded_rocdown_uri();
    open_rocdown_at(&mut server, uri.clone(), EMBEDDED_ROCDOWN);

    let docs_note = EMBEDDED_ROCDOWN.find("@docs note").expect("@docs note") + 2;
    let (line, character) = line_col(EMBEDDED_ROCDOWN, docs_note);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position::new(line, character),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("docs hover");
    let lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("@docs note"), "{}", markup.value);
    assert!(markup.value.contains("title"), "{}", markup.value);

    let docs_include_body = EMBEDDED_ROCDOWN
        .find("path: \"AllSyntax")
        .expect("include path")
        + 2;
    let (line, character) = line_col(EMBEDDED_ROCDOWN, docs_include_body);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position::new(line, character),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("include completion")
    else {
        panic!("expected completion array");
    };
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
    assert_eq!(labels, vec!["path"]);

    const EMPTY_DOCS_SRC: &str = "@page {}\n\n@docs include {\n    \n}\n";
    let mut empty_server = initialize(true);
    open_rocdown(&mut empty_server, EMPTY_DOCS_SRC);
    let at_empty = EMPTY_DOCS_SRC.find("    \n").expect("empty line") + 4;
    let (line, character) = line_col(EMPTY_DOCS_SRC, at_empty);
    let CompletionResponse::Array(items) = empty_server
        .completion(CompletionParams {
            text_document_position: rocdown_position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("empty include completion")
    else {
        panic!("expected completion array");
    };
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"path"), "{labels:?}");
    assert!(labels.contains(&"region"), "{labels:?}");
    assert!(labels.contains(&"language"), "{labels:?}");

    const ROOT_DOCS_SRC: &str = "@page {}\n\n@d\n";
    let mut root_server = initialize(true);
    open_rocdown(&mut root_server, ROOT_DOCS_SRC);
    let at_d = ROOT_DOCS_SRC.find("@d").expect("@d") + 2;
    let (line, character) = line_col(ROOT_DOCS_SRC, at_d);
    let CompletionResponse::Array(items) = root_server
        .completion(CompletionParams {
            text_document_position: rocdown_position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("root completion")
    else {
        panic!("expected completion array");
    };
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"docs"), "{labels:?}");
}

#[test]
fn non_bmp_position_encoding_in_fixtures() {
    let mut utf8_rocci = initialize(true);
    let mut utf16_rocci = initialize(false);
    open(&mut utf8_rocci, EMBEDDED_ROCCI);
    open(&mut utf16_rocci, EMBEDDED_ROCCI);

    let offset_after_emoji = EMBEDDED_ROCCI.find("🦄").expect("unicorn emoji") + 4;
    let (u8_line, u8_col) = SourceFile::new("t.rocci", EMBEDDED_ROCCI)
        .position(offset_after_emoji as u32, PositionEncoding::Utf8);
    let (u16_line, u16_col) = SourceFile::new("t.rocci", EMBEDDED_ROCCI)
        .position(offset_after_emoji as u32, PositionEncoding::Utf16);
    assert_eq!(u8_line, u16_line);
    assert_ne!(
        u8_col, u16_col,
        "UTF-8 byte length (4) vs UTF-16 code units (2) must differ"
    );

    let offset_gothic = EMBEDDED_ROCDOWN.find("𐍈").expect("gothic letter") + 4;
    let (u8_rd_line, u8_rd_col) = SourceFile::new("t.rocdown", EMBEDDED_ROCDOWN)
        .position(offset_gothic as u32, PositionEncoding::Utf8);
    let (u16_rd_line, u16_rd_col) = SourceFile::new("t.rocdown", EMBEDDED_ROCDOWN)
        .position(offset_gothic as u32, PositionEncoding::Utf16);
    assert_eq!(u8_rd_line, u16_rd_line);
    assert_ne!(
        u8_rd_col, u16_rd_col,
        "Gothic character UTF-8 (4) vs UTF-16 (2) must differ"
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

#[test]
fn rocci_region_tree_validation_and_invariants() {
    let rocci_fixtures = [
        ("AllSyntax.rocci", KITCHEN_SINK),
        ("EmbeddedLanguages.rocci", EMBEDDED_ROCCI),
    ];
    for (name, src) in rocci_fixtures {
        let parsed = rocci_template::parse(SourceFile::new(name, src));
        let tree = extract_rocci_regions(name, src, &parsed.document);
        tree.validate(src.len())
            .unwrap_or_else(|err| panic!("validation failed on {name}: {err:?}"));
        assert!(
            tree.regions.len() > 5,
            "expected multiple regions in {name}"
        );
        for region in &tree.regions {
            assert!(region.span.start <= region.span.end);
            assert!(region.span.end as usize <= src.len());
            if let Some(parent) = region.parent {
                let p = &tree.regions[parent];
                assert!(
                    region.span.start >= p.span.start && region.span.end <= p.span.end,
                    "region {region:?} not contained in parent {p:?}"
                );
            }
        }
    }

    let rocdown_fixtures = [
        ("AllSyntax.rocdown", ALL_SYNTAX_ROCDOWN),
        ("EmbeddedLanguages.rocdown", EMBEDDED_ROCDOWN),
        ("Guide.rocdown", GUIDE),
    ];
    for (name, src) in rocdown_fixtures {
        let parsed = rocci_rocdown::parse(SourceFile::new(name, src), false);
        let tree = extract_rocdown_regions(name, src, &parsed.document, &parsed.headings);
        tree.validate(src.len())
            .unwrap_or_else(|err| panic!("validation failed on {name}: {err:?}"));
        assert!(
            tree.regions.len() > 5,
            "expected multiple regions in {name}"
        );
        for region in &tree.regions {
            assert!(region.span.start <= region.span.end);
            assert!(region.span.end as usize <= src.len());
            if let Some(parent) = region.parent {
                let p = &tree.regions[parent];
                assert!(
                    region.span.start >= p.span.start && region.span.end <= p.span.end,
                    "region {region:?} not contained in parent {p:?}"
                );
            }
            if region.context == RegionContext::Fence {
                assert_eq!(
                    region.purpose,
                    RegionPurpose::DisplayOnly,
                    "fences must be display-only"
                );
            }
        }
    }
}

#[test]
fn inspect_regions_lsp_request_and_payload() {
    let mut server = initialize(true);
    open(&mut server, EMBEDDED_ROCCI);

    let req = Request::new(
        1.into(),
        method_inspect_regions().to_string(),
        serde_json::to_value(&SemanticTokensParams {
            text_document: identifier(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .unwrap(),
    );
    let resp = server.handle_request(req);
    assert!(
        resp.error.is_none(),
        "inspect_regions request failed: {:?}",
        resp.error
    );
    let regions: Vec<InspectedRegion> =
        serde_json::from_value(resp.result.unwrap()).expect("deserialized regions");
    assert!(!regions.is_empty(), "expected inspected regions");

    let root = &regions[0];
    assert_eq!(root.id, 0);
    assert_eq!(root.language, "rocci");
    assert_eq!(root.context, "document");
    assert_eq!(root.purpose, "hostStructure");
    assert_eq!(root.parent, None);

    for r in &regions {
        assert!(r.span.start <= r.span.end);
        if let Some(parent_id) = r.parent {
            let parent = &regions[parent_id];
            assert!(r.span.start >= parent.span.start && r.span.end <= parent.span.end);
        }
    }

    let uri = embedded_rocdown_uri();
    open_rocdown_at(&mut server, uri.clone(), EMBEDDED_ROCDOWN);
    let rd_req = Request::new(
        2.into(),
        method_inspect_regions().to_string(),
        serde_json::to_value(&SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .unwrap(),
    );
    let rd_resp = server.handle_request(rd_req);
    assert!(
        rd_resp.error.is_none(),
        "rd inspect_regions failed: {:?}",
        rd_resp.error
    );
    let rd_regions: Vec<InspectedRegion> =
        serde_json::from_value(rd_resp.result.unwrap()).expect("deserialized rd regions");
    assert!(!rd_regions.is_empty(), "expected rd inspected regions");
    assert_eq!(rd_regions[0].language, "markdown");
    assert_eq!(rd_regions[0].context, "document");

    let fence_regions: Vec<_> = rd_regions.iter().filter(|r| r.context == "fence").collect();
    assert!(
        !fence_regions.is_empty(),
        "expected fence regions in rocdown"
    );
    for fence in fence_regions {
        assert_eq!(fence.purpose, "displayOnly");
    }
}

#[test]
fn region_boundaries_independent_of_tokens() {
    let parsed = rocci_template::parse(SourceFile::new("EmbeddedLanguages.rocci", EMBEDDED_ROCCI));
    let tree = extract_rocci_regions("EmbeddedLanguages.rocci", EMBEDDED_ROCCI, &parsed.document);

    let format_user_offset = EMBEDDED_ROCCI.find("formatUser =").expect("formatUser");
    let leaf = tree
        .find_at(format_user_offset)
        .expect("leaf for formatUser");
    assert_eq!(leaf.language, Language::Roc);
    assert_eq!(leaf.context, RegionContext::Module);
    assert_eq!(leaf.purpose, RegionPurpose::Executable);

    let css_offset = EMBEDDED_ROCCI
        .find(".card--selected")
        .expect(".card--selected");
    let css_leaf = tree.find_at(css_offset).expect("leaf for css");
    assert_eq!(css_leaf.language, Language::Css);
    assert_eq!(css_leaf.context, RegionContext::Stylesheet);

    let interp_offset = EMBEDDED_ROCCI.find("{formatUser(user)}").expect("interp") + 1;
    let interp_leaf = tree.find_at(interp_offset).expect("leaf for interp");
    assert_eq!(interp_leaf.language, Language::Roc);
    assert_eq!(interp_leaf.context, RegionContext::Expression);
    assert_eq!(interp_leaf.purpose, RegionPurpose::Executable);

    let parsed_rd = rocci_rocdown::parse(
        SourceFile::new("EmbeddedLanguages.rocdown", EMBEDDED_ROCDOWN),
        false,
    );
    let rd_tree = extract_rocdown_regions(
        "EmbeddedLanguages.rocdown",
        EMBEDDED_ROCDOWN,
        &parsed_rd.document,
        &parsed_rd.headings,
    );

    let page_offset = EMBEDDED_ROCDOWN
        .find("route: \"/embedded-languages/\"")
        .expect("page route");
    let page_leaf = rd_tree.find_at(page_offset).expect("leaf for page");
    assert_eq!(page_leaf.language, Language::Roc);
    assert_eq!(page_leaf.purpose, RegionPurpose::Metadata);

    let fence_offset = EMBEDDED_ROCDOWN
        .find("main = \\{} -> \"Hello from display Roc!\"")
        .expect("display fence");
    let fence_leaf = rd_tree.find_at(fence_offset).expect("leaf for fence");
    assert_eq!(fence_leaf.language, Language::Roc);
    assert_eq!(fence_leaf.context, RegionContext::Fence);
    assert_eq!(fence_leaf.purpose, RegionPurpose::DisplayOnly);
}

#[test]
fn malformed_and_unclosed_syntax_produces_valid_regions() {
    let broken_rocci_inputs = [
        INCOMPLETE_TAG,
        "@component Foo = |{}| { <p>{user. </p> }",
        "@css { .foo { color: ",
        "@on:get(\"/api\") { let x = ",
        "@init {",
    ];
    for src in broken_rocci_inputs {
        let parsed = rocci_template::parse(SourceFile::new("broken.rocci", src));
        let tree = extract_rocci_regions("broken.rocci", src, &parsed.document);
        tree.validate(src.len()).unwrap_or_else(|err| {
            panic!("validation failed on broken rocci:\n{src}\nErr: {err:?}")
        });
    }

    let broken_rocdown_inputs = [
        "@page {\n  draft: Bool.true,\n\n# Unclosed page",
        "@docs note {\n  title: \"unclosed\"\n",
        "```roc\nlet x = 1\n",
        "@if isTrue {\n  <p>unclosed if\n",
    ];
    for src in broken_rocdown_inputs {
        let parsed = rocci_rocdown::parse(SourceFile::new("broken.rocdown", src), false);
        let tree =
            extract_rocdown_regions("broken.rocdown", src, &parsed.document, &parsed.headings);
        tree.validate(src.len()).unwrap_or_else(|err| {
            panic!("validation failed on broken rocdown:\n{src}\nErr: {err:?}")
        });
    }
}

#[test]
fn semantic_tokens_invariants_no_overlaps_and_single_line_spans() {
    let fixtures = [
        ("AllSyntax.rocci", KITCHEN_SINK, true),
        ("EmbeddedLanguages.rocci", EMBEDDED_ROCCI, true),
        ("AllSyntax.rocdown", ALL_SYNTAX_ROCDOWN, false),
        ("EmbeddedLanguages.rocdown", EMBEDDED_ROCDOWN, false),
        ("Guide.rocdown", GUIDE, false),
    ];

    for (name, src, is_rocci) in fixtures {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let uri: Uri = format!("file:///{name}").parse().unwrap();
            if is_rocci {
                server.did_open(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: uri.clone(),
                        language_id: "rocci".to_string(),
                        version: 1,
                        text: src.to_string(),
                    },
                });
            } else {
                open_rocdown_at(&mut server, uri.clone(), src);
            }

            let result = server
                .semantic_tokens_full(SemanticTokensParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                })
                .expect("tokens response");
            let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
                panic!("expected full tokens for {name}");
            };

            assert!(!tokens.data.is_empty(), "expected tokens for {name}");

            let mut cur_line = 0u32;
            let mut cur_col = 0u32;
            let mut prev_token_end = 0u32;

            for (idx, tok) in tokens.data.iter().enumerate() {
                assert!(tok.length > 0, "token {idx} in {name} has length 0");

                cur_line += tok.delta_line;
                if tok.delta_line == 0 {
                    cur_col += tok.delta_start;
                    assert!(
                        cur_col >= prev_token_end,
                        "token {idx} at line {cur_line} col {cur_col} overlaps prev end {prev_token_end} in {name} (utf8={utf8})"
                    );
                } else {
                    cur_col = tok.delta_start;
                }
                prev_token_end = cur_col + tok.length;
            }
        }
    }
}

#[test]
fn semantic_tokens_embedded_languages_coverage() {
    let mut server = initialize(true);
    open(&mut server, EMBEDDED_ROCCI);

    let result = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: identifier(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("tokens response");
    let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
        panic!("expected full tokens");
    };

    // Roc module functions and types in .rocci
    let fn_pos = EMBEDDED_ROCCI.find("formatUser =").expect("formatUser");
    assert!(token_type_at(EMBEDDED_ROCCI, &tokens, fn_pos).is_some());

    let roc_type_pos = EMBEDDED_ROCCI.find("User : {").expect("User :");
    assert_eq!(
        token_type_at(EMBEDDED_ROCCI, &tokens, roc_type_pos),
        Some(TOKEN_TYPE)
    );

    // CSS rules in .rocci
    let css_prop = EMBEDDED_ROCCI.find("padding: 1rem").expect("padding: 1rem");
    assert_eq!(
        token_type_at(EMBEDDED_ROCCI, &tokens, css_prop),
        Some(TOKEN_PROPERTY)
    );

    // Now test .rocdown embedded languages
    let uri = embedded_rocdown_uri();
    open_rocdown_at(&mut server, uri.clone(), EMBEDDED_ROCDOWN);
    let rd_result = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("rd tokens");
    let lsp_types::SemanticTokensResult::Tokens(rd_tokens) = rd_result else {
        panic!("expected rd tokens");
    };

    // Executable Roc in @roc
    let status_decl = EMBEDDED_ROCDOWN.find("Status : [").expect("Status : [");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, status_decl),
        Some(TOKEN_TYPE)
    );
    let u64_type = EMBEDDED_ROCDOWN.find("U64)").expect("U64)");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, u64_type),
        Some(TOKEN_TYPE)
    );
    let active_val = EMBEDDED_ROCDOWN
        .find("status = Active(42)")
        .expect("status = Active(42)")
        + 9;
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, active_val),
        Some(TOKEN_ENUM_MEMBER)
    );

    // Display-only Roc fence
    let display_roc_str = EMBEDDED_ROCDOWN
        .find("\"Hello from display Roc!\"")
        .expect("display roc string");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, display_roc_str),
        Some(TOKEN_STRING)
    );

    // Display-only HTML fence
    let display_html_tag = EMBEDDED_ROCDOWN
        .find("<div class=\"display-container\"")
        .expect("<div");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, display_html_tag + 1),
        Some(TOKEN_TYPE)
    );
    let display_html_attr = EMBEDDED_ROCDOWN
        .find("class=\"display-container\"")
        .expect("class=");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, display_html_attr),
        Some(TOKEN_PROPERTY)
    );

    // Display-only CSS fence
    let display_css_prop = EMBEDDED_ROCDOWN.find("font-size").expect("font-size");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, display_css_prop),
        Some(TOKEN_PROPERTY)
    );

    // Markdown inline code
    let md_code = EMBEDDED_ROCDOWN
        .find("`Num.toStr(42)`")
        .expect("`Num.toStr(42)`");
    assert_eq!(
        token_type_at(EMBEDDED_ROCDOWN, &rd_tokens, md_code),
        Some(TOKEN_STRING)
    );
}

#[test]
fn semantic_tokens_malformed_and_incomplete_recovery() {
    let broken_inputs = [
        INCOMPLETE_TAG,
        "@component Broken = |{}| { <p>{person. </p> }",
        "@css { .card { color: #fff; width: ",
        "@on:get(\"/api\") { let result = ",
        "```roc\nlet broken = \"unclosed string\n```",
        "```html\n<div class=\"unclosed\n```",
        "```css\n.foo { color:\n```",
    ];

    for src in broken_inputs {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let uri = test_uri();
            open(&mut server, src);

            let result = server.semantic_tokens_full(SemanticTokensParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            });
            assert!(
                result.is_some(),
                "expected token result on broken input:\n{src}"
            );
        }
    }
}
