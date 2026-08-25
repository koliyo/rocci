use lsp_server::Request;
use lsp_types::{
    ClientCapabilities, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidOpenTextDocumentParams, DocumentSymbolParams, DocumentSymbolResponse,
    GeneralClientCapabilities, GotoDefinitionParams, Hover, HoverContents, HoverParams,
    InitializeParams, MarkupContent, MarkupKind, PartialResultParams, Position,
    PositionEncodingKind, Range, SemanticTokens, SemanticTokensParams, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};
use rocci_lsp::{
    FakeRocBackend, InspectedRegion, Language, LanguageServer, RegionContext, RegionPurpose,
    TOKEN_ENUM_MEMBER, TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_PROPERTY, TOKEN_TYPE, TOKEN_VARIABLE,
    extract_rocci_regions, method_inspect_regions,
};
use rocci_template::{PositionEncoding, SourceFile};

const KITCHEN_SINK: &str = include_str!("../../../test/AllSyntax.rocci");
const EMBEDDED_ROCCI: &str = include_str!("../../../test/EmbeddedLanguages.rocci");

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
    assert!(names.contains(&"GET /"));
    assert!(names.contains(&"PATCH /actions/patch"));
    assert!(names.contains(&"POST /actions/increment"));
    assert!(names.contains(&"GET /sse"));
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
    assert!(names.contains(&"GET /api/users"), "{names:?}");

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
    let result = resp
        .response_result
        .as_ref()
        .unwrap_or_else(|err| panic!("inspect_regions request failed: {err:?}"));
    let regions: Vec<InspectedRegion> =
        serde_json::from_value(result.clone()).expect("deserialized regions");
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
}

#[test]
fn malformed_and_unclosed_syntax_produces_valid_regions() {
    let broken_rocci_inputs = [
        INCOMPLETE_TAG,
        "@component Foo = |{}| { <p>{user. </p> }",
        "@css { .foo { color: ",
        "@patch:fragment(\"/api\") { let x = ",
        include_str!("../../../test/MalformedHandlers.rocci"),
        "@init {",
    ];
    for src in broken_rocci_inputs {
        let parsed = rocci_template::parse(SourceFile::new("broken.rocci", src));
        let tree = extract_rocci_regions("broken.rocci", src, &parsed.document);
        tree.validate(src.len()).unwrap_or_else(|err| {
            panic!("validation failed on broken rocci:\n{src}\nErr: {err:?}")
        });
    }
}

#[test]
fn semantic_tokens_invariants_no_overlaps_and_single_line_spans() {
    let fixtures = [
        ("AllSyntax.rocci", KITCHEN_SINK),
        ("EmbeddedLanguages.rocci", EMBEDDED_ROCCI),
    ];

    for (name, src) in fixtures {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let uri: Uri = format!("file:///{name}").parse().unwrap();
            server.did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "rocci".to_string(),
                    version: 1,
                    text: src.to_string(),
                },
            });

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
}

#[test]
fn semantic_tokens_malformed_and_incomplete_recovery() {
    let broken_inputs = [
        INCOMPLETE_TAG,
        "@component Broken = |{}| { <p>{person. </p> }",
        "@css { .card { color: #fff; width: ",
        "@patch:fragment(\"/api\") { let result = ",
        include_str!("../../../test/MalformedHandlers.rocci"),
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

#[test]
fn semantic_tokens_counter_rocci_qualified_precision() {
    let src = include_str!("../../../examples/rocci/standalone/counter/Counter.rocci");
    for utf8 in [true, false] {
        let mut server = initialize(utf8);
        let uri = test_uri();
        open(&mut server, src);

        let result = server
            .semantic_tokens_full(SemanticTokensParams {
                text_document: TextDocumentIdentifier { uri },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .expect("tokens response");
        let lsp_types::SemanticTokensResult::Tokens(tokens) = result else {
            panic!("expected full tokens");
        };

        let mut cur_line = 0u32;
        let mut cur_col = 0u32;
        let mut matched_sqlite_query = false;
        let mut matched_sqlite_execute = false;
        let mut matched_default_limits = false;

        for tok in &tokens.data {
            cur_line += tok.delta_line;
            if tok.delta_line == 0 {
                cur_col += tok.delta_start;
            } else {
                cur_col = tok.delta_start;
            }
            let line_text = src.lines().nth(cur_line as usize).unwrap_or("");
            if (cur_col as usize + tok.length as usize) <= line_text.len() {
                let slice = &line_text[cur_col as usize..cur_col as usize + tok.length as usize];
                if (slice == "query" || slice == "query!") && line_text.contains("Sqlite.query!") {
                    matched_sqlite_query = true;
                }
                if (slice == "execute" || slice == "execute!")
                    && line_text.contains("Sqlite.execute!")
                {
                    matched_sqlite_execute = true;
                }
                if slice == "default_query_limits"
                    && line_text.contains("Sqlite.default_query_limits")
                {
                    matched_default_limits = true;
                }
                assert_ne!(slice, "uery", "token was chopped off by one");
                assert_ne!(slice, "xecute", "token was chopped off by one");
                assert_ne!(slice, "efault_query_limits", "token was chopped off by one");
            }
        }

        assert!(matched_sqlite_query, "did not match Sqlite.query");
        assert!(matched_sqlite_execute, "did not match Sqlite.execute");
        assert!(
            matched_default_limits,
            "did not match Sqlite.default_query_limits"
        );
    }
}

#[test]
fn handlers_have_final_symbols_hover_completion_and_distinct_role_tokens() {
    let src = r#"
@get:view("/") { Html.text("v") }
@get:fragment("/search") { Html.text("s") }
@patch:fragment("/x") { Html.text("p") }
@post:command("/c") { {} }
@get:live("/events/main") { Html.text("l") }
@get:live("/events/shared") { Html.text("s") }

@component Unused = |{}| {
    <p>x</p>
}
"#;
    let mut server = initialize(true);
    open(&mut server, src);
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
    assert!(names.contains(&"GET /"), "{names:?}");
    assert!(names.contains(&"PATCH /x"), "{names:?}");
    assert!(names.contains(&"POST /c"), "{names:?}");
    assert!(names.contains(&"GET /search"), "{names:?}");
    assert!(names.contains(&"GET /events/main"), "{names:?}");
    assert!(names.contains(&"GET /events/shared"), "{names:?}");

    let view_at = src.find("@get:view").expect("@get:view");
    let (line, character) = line_col(src, view_at);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("view hover");
    let lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(
        markup.value.contains("@get:view(\"/\")"),
        "{}",
        markup.value
    );

    let at_view = src.find("@get:view").expect("@get:view") + 1;
    let (line, character) = line_col(src, at_view);
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
    assert!(items.iter().any(|item| item.label == "get"));
    assert!(items.iter().any(|item| item.label == "post"));
    assert!(items.iter().any(|item| item.label == "fragment"));
    assert!(items.iter().any(|item| item.label == "patch"));
    assert!(items.iter().any(|item| item.label == "command"));

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
    let patch_kw = src.find("@patch").expect("@patch");
    assert_eq!(token_type_at(src, &tokens, patch_kw), Some(TOKEN_KEYWORD));
    let role = src.find("@patch:fragment").expect("role") + "@patch:".len();
    assert_eq!(token_type_at(src, &tokens, role), Some(TOKEN_ENUM_MEMBER));
}

#[test]
fn hover_includes_component_doc_comments() {
    let src = r#"module [Hello]

## Greeting card.
@component Hello = |{ name }|
    <p>{name}</p>
"#;
    let mut server = initialize(true);
    open(&mut server, src);
    let hello = src.find("Hello =").expect("hello");
    let (line, character) = line_col(src, hello);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("hover");
    let lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("@component Hello ="), "{markup:?}");
    assert!(markup.value.contains("Greeting card."), "{markup:?}");
}

fn roc_type_hover(range: Option<Range>) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "```roc\nStr\n```".to_string(),
        }),
        range,
    }
}

fn projected_ident_range(src: &str, ident: &str, after: &str) -> Range {
    let compiled = rocci_template::compile(
        SourceFile::new("test.rocci", src),
        &rocci_template::LowerOptions::default(),
    );
    let type_name = rocci_template::type_name_from_path(std::path::Path::new("/test.rocci"));
    let projection =
        rocci_template::project_type_module(&compiled.roc, &compiled.segments, &type_name);
    let from = src.find(after).unwrap_or_else(|| panic!("missing {after}"));
    let rel = src[from..]
        .find(ident)
        .unwrap_or_else(|| panic!("missing {ident} after {after}"));
    let offset = (from + rel) as u32;
    let mapped =
        rocci_template::source_to_generated(src, &projection.roc, &projection.segments, offset)
            .expect("source map");
    let proj = SourceFile::new("projection.roc", &projection.roc);
    let (start_line, start_col) = proj.position(mapped.offset, PositionEncoding::Utf16);
    let (end_line, end_col) =
        proj.position(mapped.offset + ident.len() as u32, PositionEncoding::Utf16);
    Range {
        start: Position::new(start_line, start_col),
        end: Position::new(end_line, end_col),
    }
}

fn projected_needle_range(src: &str, needle: &str) -> Range {
    let compiled = rocci_template::compile(
        SourceFile::new("test.rocci", src),
        &rocci_template::LowerOptions::default(),
    );
    let type_name = rocci_template::type_name_from_path(std::path::Path::new("/test.rocci"));
    let projection =
        rocci_template::project_type_module(&compiled.roc, &compiled.segments, &type_name);
    let offset = projection.roc.find(needle).expect("needle") as u32;
    let proj = SourceFile::new("projection.roc", &projection.roc);
    let (start_line, start_col) = proj.position(offset, PositionEncoding::Utf16);
    let (end_line, end_col) = proj.position(offset + needle.len() as u32, PositionEncoding::Utf16);
    Range {
        start: Position::new(start_line, start_col),
        end: Position::new(end_line, end_col),
    }
}

fn roc_error(range: Range, message: &str) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("roc-experimental-lsp".to_string()),
        message: message.to_string(),
        ..Diagnostic::default()
    }
}

#[test]
fn interpolation_hover_forwards_mapped_roc_backend() {
    let src = r#"
@component Hello = |{ title }| {
    <p>{title}</p>
}
"#;
    let range = projected_ident_range(src, "title", "{title}");
    let mut fake = FakeRocBackend::default();
    fake.set_any_hover(roc_type_hover(Some(range)));
    let mut server = initialize(true);
    server.set_roc_backend(Box::new(fake));
    open(&mut server, src);

    let title = src.find("{title}").expect("interp") + 1;
    let (line, character) = line_col(src, title);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("roc hover");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("Str"), "{markup:?}");
    assert!(!markup.value.contains("@component"), "{markup:?}");
    let mapped = hover.range.expect("mapped hover range");
    let (start_line, start_character) = line_col(src, title);
    let (end_line, end_character) = line_col(src, title + "title".len());
    assert_eq!(mapped.start.line, start_line);
    assert_eq!(mapped.start.character, start_character);
    assert_eq!(mapped.end.line, end_line);
    assert_eq!(mapped.end.character, end_character);
}

#[test]
fn roc_block_ident_hover_forwards_mapped_roc_backend() {
    let src = r#"
greet = |name| name

@component Hello = |{ name }| {
    <p>{greet(name)}</p>
}
"#;
    let mut fake = FakeRocBackend::default();
    fake.set_any_hover(roc_type_hover(None));
    let mut server = initialize(true);
    server.set_roc_backend(Box::new(fake));
    open(&mut server, src);

    let greet = src.find("greet =").expect("greet");
    let (line, character) = line_col(src, greet);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("roc hover");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("Str"), "{markup:?}");
}

#[test]
fn component_hover_stays_host_when_roc_backend_answers() {
    let src = r#"
## Greeting card.
@component Hello = |{ name }|
    <p>{name}</p>
"#;
    let mut fake = FakeRocBackend::default();
    fake.set_any_hover(roc_type_hover(None));
    let mut server = initialize(true);
    server.set_roc_backend(Box::new(fake));
    open(&mut server, src);

    let hello = src.find("Hello =").expect("hello");
    let (line, character) = line_col(src, hello);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("host hover");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("@component Hello"), "{markup:?}");
    assert!(markup.value.contains("Greeting card."), "{markup:?}");
    assert!(!markup.value.contains("```roc\nStr"), "{markup:?}");
}

#[test]
fn interpolation_type_error_maps_to_expr_span() {
    let src = r#"
@component Hello = |{ title }| {
    <p>{title + 1}</p>
}
"#;
    let expr = "title + 1";
    let range = projected_ident_range(src, expr, "{title + 1}");
    let mut fake = FakeRocBackend::default();
    fake.set_diagnostics(vec![roc_error(range, "TYPE MISMATCH")]);
    let mut server = initialize(true);
    server.set_roc_backend(Box::new(fake));
    let published = open(&mut server, src);
    let diag = published
        .diagnostics
        .iter()
        .find(|d| d.message == "TYPE MISMATCH")
        .expect("mapped roc diagnostic");
    assert_eq!(diag.source.as_deref(), Some("roc"));
    let start = src.find(expr).expect("expr");
    let (start_line, start_character) = line_col(src, start);
    let (end_line, end_character) = line_col(src, start + expr.len());
    assert_eq!(diag.range.start.line, start_line);
    assert_eq!(diag.range.start.character, start_character);
    assert_eq!(diag.range.end.line, end_line);
    assert_eq!(diag.range.end.character, end_character);
}

#[test]
fn scaffolding_roc_diagnostics_are_dropped() {
    let src = r#"
@component Hello = |{ title }| {
    <p>{title + 1}</p>
}
"#;
    let range = projected_needle_range(src, "Html.text");
    let mut fake = FakeRocBackend::default();
    fake.set_diagnostics(vec![roc_error(range, "scaffolding")]);
    let mut server = initialize(true);
    server.set_roc_backend(Box::new(fake));
    let published = open(&mut server, src);
    assert!(
        published
            .diagnostics
            .iter()
            .all(|d| d.message != "scaffolding"),
        "{:?}",
        published.diagnostics
    );
}
