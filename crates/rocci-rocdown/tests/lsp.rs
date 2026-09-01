use lsp_types::{
    ClientCapabilities, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidOpenTextDocumentParams, DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse,
    GeneralClientCapabilities, GotoDefinitionParams, Hover, HoverContents, HoverParams,
    InitializeParams, MarkupContent, MarkupKind, PartialResultParams, Position,
    PositionEncodingKind, Range, SemanticTokensParams, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};
use rocci_lsp::{FakeRocBackend, LanguageServer};
use rocci_rocdown::{CompileOptions, RocdownAnalyzer, compile};
use rocci_template::{PositionEncoding, SourceFile, project_type_module, type_name_from_path};

const ALL_SYNTAX_ROCDOWN: &str = include_str!("../../../test/AllSyntax.rocdown");
const EMBEDDED_ROCDOWN: &str = include_str!("../../../test/EmbeddedLanguages.rocdown");
const BLOCKS_ROCDOWN: &str = include_str!("../../../examples/rocdown/pages/Blocks.rocdown");

use std::path::PathBuf;

fn test_uri() -> Uri {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test/AllSyntax.rocdown")
        .canonicalize()
        .expect("AllSyntax.rocdown path");
    format!("file://{}", path.display())
        .parse()
        .expect("test uri")
}

fn embedded_uri() -> Uri {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test/EmbeddedLanguages.rocdown")
        .canonicalize()
        .expect("embedded rocdown path");
    format!("file://{}", path.display())
        .parse()
        .expect("embedded uri")
}

fn initialize_server() -> LanguageServer {
    let mut server = LanguageServer::with_analyzers(vec![Box::new(RocdownAnalyzer)]);
    server.initialize(InitializeParams {
        capabilities: ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: Some(vec![
                    PositionEncodingKind::UTF8,
                    PositionEncodingKind::UTF16,
                ]),
                ..GeneralClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        },
        ..InitializeParams::default()
    });
    server
}

#[test]
fn test_rocdown_lsp_all_syntax() {
    let mut server = initialize_server();
    let uri = test_uri();

    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: ALL_SYNTAX_ROCDOWN.to_string(),
            },
        })
        .expect("should publish diagnostics");

    assert!(
        published
            .diagnostics
            .iter()
            .all(|d| d.severity != Some(DiagnosticSeverity::ERROR)),
        "unexpected error diagnostics: {:?}",
        published.diagnostics
    );

    let symbols = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("document symbols");

    match symbols {
        DocumentSymbolResponse::Nested(syms) => {
            assert!(
                syms.iter().any(|s| s.name == "@page"),
                "missing @page symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == "All syntax"),
                "missing heading symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == "Hello"),
                "missing component Hello symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == ":figure"),
                "missing :figure symbol"
            );
            let figure = syms.iter().find(|s| s.name == ":figure").unwrap();
            let figure_children = figure.children.as_deref().unwrap_or(&[]);
            assert!(
                figure_children.iter().any(|s| s.name == ":img"),
                "missing nested :img symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == ":note"),
                "missing :note symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == ":details"),
                "missing :details symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == ":steps"),
                "missing :steps symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == ":tabs"),
                "missing :tabs symbol"
            );
            let tabs = syms.iter().find(|s| s.name == ":tabs").unwrap();
            let tabs_children = tabs.children.as_deref().unwrap_or(&[]);
            assert!(
                tabs_children.iter().any(|s| s.name == ":tab"),
                "missing nested :tab symbol"
            );
        }
        DocumentSymbolResponse::Flat(_) => panic!("expected nested symbols"),
    }

    let page_hover = server.hover(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(0, 2),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    });
    assert!(page_hover.is_some(), "hover on @page should return info");

    let tokens_res = server
        .semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("semantic tokens");

    match tokens_res {
        lsp_types::SemanticTokensResult::Tokens(tokens) => {
            assert!(!tokens.data.is_empty(), "expected semantic tokens");
        }
        _ => panic!("expected full tokens"),
    }
}

#[test]
fn blocks_example_begin_end_children_are_nested() {
    let mut server = initialize_server();
    let uri: Uri = "file:///Blocks.rocdown".parse().expect("blocks uri");
    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: BLOCKS_ROCDOWN.to_string(),
            },
        })
        .expect("should publish diagnostics");

    let errors: Vec<&str> = published
        .diagnostics
        .iter()
        .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
        .map(|d| d.message.as_str())
        .collect();
    assert!(
        errors.iter().all(|msg| {
            !msg.contains("`:step` is only valid inside `:steps`")
                && !msg.contains("`:tab` is only valid inside `:tabs`")
                && !msg.contains("`:tabs` requires `group`")
                && !msg.contains("`:tabs` requires `kind`")
        }),
        "begin/end children should keep their parent: {errors:?}"
    );
    assert!(
        published
            .diagnostics
            .iter()
            .all(|d| d.source.as_deref() == Some("rocdown")),
        "rocdown diagnostics should be sourced as rocdown: {:?}",
        published.diagnostics
    );

    let symbols = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("document symbols");
    let DocumentSymbolResponse::Nested(syms) = symbols else {
        panic!("expected nested symbols");
    };
    let steps = syms.iter().find(|s| s.name == ":steps").expect(":steps");
    let steps_children = steps.children.as_deref().unwrap_or(&[]);
    assert!(
        steps_children.iter().any(|s| s.name == ":step"),
        "missing nested :step under :steps: {steps_children:?}"
    );
    let tabs = syms.iter().find(|s| s.name == ":tabs").expect(":tabs");
    let tabs_children = tabs.children.as_deref().unwrap_or(&[]);
    assert!(
        tabs_children.iter().any(|s| s.name == ":tab"),
        "missing nested :tab under :tabs: {tabs_children:?}"
    );
}

#[test]
fn test_rocdown_lsp_embedded_languages() {
    let mut server = initialize_server();
    let uri = embedded_uri();

    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: EMBEDDED_ROCDOWN.to_string(),
            },
        })
        .expect("should publish diagnostics");

    assert!(
        published
            .diagnostics
            .iter()
            .all(|d| d.severity != Some(DiagnosticSeverity::ERROR)),
        "unexpected error diagnostics: {:?}",
        published.diagnostics
    );

    let symbols = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("symbols");

    let DocumentSymbolResponse::Nested(syms) = symbols else {
        panic!("expected nested symbols");
    };
    assert!(syms.iter().any(|s| s.name == ":note"));
    assert!(syms.iter().any(|s| s.name == ":tabs"));
    let tabs = syms.iter().find(|s| s.name == ":tabs").unwrap();
    let children = tabs.children.as_deref().unwrap_or(&[]);
    assert!(
        children.iter().any(|s| s.name == ":tab"),
        "expected nested :tab under :tabs"
    );
}

const COLON_BLOCKS: &str = "\
:note Don't do this.

:note[title: \"Watch\"] {{
Nested.
}}

:tabs.begin[group: \"os\", kind: \"platform\"]
    :tab[id: \"mac\", label: \"macOS\"] Mac panel.
    :tab[id: \"linux\", label: \"Linux\"] Linux panel.
:tabs.end

:img[src: \"./x.png\", alt: \"x\"]
";

fn colon_uri() -> Uri {
    "file:///ColonBlocks.rocdown".parse().expect("colon uri")
}

fn line_col(src: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut last_nl = 0usize;
    for (idx, ch) in src[..offset].char_indices() {
        if ch == '\n' {
            line += 1;
            last_nl = idx + 1;
        }
    }
    (line, (offset - last_nl) as u32)
}

fn position_params(uri: &Uri, line: u32, character: u32) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        position: Position::new(line, character),
    }
}

fn open_colon(server: &mut LanguageServer) -> Uri {
    let uri = colon_uri();
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: COLON_BLOCKS.to_string(),
            },
        })
        .expect("open colon blocks");
    uri
}

fn labels(items: &[lsp_types::CompletionItem]) -> Vec<&str> {
    items.iter().map(|item| item.label.as_str()).collect()
}

fn find_symbol<'a>(syms: &'a [DocumentSymbol], name: &str) -> Option<&'a DocumentSymbol> {
    for symbol in syms {
        if symbol.name == name {
            return Some(symbol);
        }
        if let Some(children) = &symbol.children
            && let Some(found) = find_symbol(children, name)
        {
            return Some(found);
        }
    }
    None
}

#[test]
fn completes_builtin_kinds_and_registry_fields() {
    let mut server = initialize_server();
    let uri = open_colon(&mut server);

    let kind_src = ":no";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 2,
                text: kind_src.to_string(),
            },
        })
        .expect("open kind prefix");
    let (line, character) = line_col(kind_src, kind_src.len());
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("kind completion")
    else {
        panic!("expected completion array");
    };
    let kind_labels = labels(&items);
    assert!(kind_labels.contains(&"note"), "{kind_labels:?}");

    let root_src = ":";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 3,
                text: root_src.to_string(),
            },
        })
        .expect("open kind root");
    let (line, character) = line_col(root_src, root_src.len());
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("root kind completion")
    else {
        panic!("expected completion array");
    };
    let root_labels = labels(&items);
    assert!(root_labels.contains(&"img"), "{root_labels:?}");
    assert!(root_labels.contains(&"note"), "{root_labels:?}");
    assert!(
        !root_labels.contains(&"tab"),
        "root completions should not offer child-only kinds: {root_labels:?}"
    );

    let field_src = ":note[ti";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 3,
                text: field_src.to_string(),
            },
        })
        .expect("open field prefix");
    let (line, character) = line_col(field_src, field_src.len());
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("field completion")
    else {
        panic!("expected completion array");
    };
    let field_labels = labels(&items);
    assert!(field_labels.contains(&"title"), "{field_labels:?}");

    let kind_value_src = ":tabs[group: \"os\", kind: \"pla\"";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 5,
                text: kind_value_src.to_string(),
            },
        })
        .expect("open kind value");
    let (line, character) = line_col(kind_value_src, kind_value_src.len());
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("enum completion")
    else {
        panic!("expected completion array");
    };
    let value_labels = labels(&items);
    assert!(value_labels.contains(&"platform"), "{value_labels:?}");
}

#[test]
fn kind_completion_inside_tabs_prefers_accepts() {
    let mut server = initialize_server();
    let uri = open_colon(&mut server);
    let src = "\
:tabs.begin[group: \"os\", kind: \"platform\"]
    :
    :tab[id: \"mac\", label: \"macOS\"] Hello.
:tabs.end
";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 6,
                text: src.to_string(),
            },
        })
        .expect("open tabs body");
    let offset = src.find("    :\n").expect("inner colon") + 5;
    let (line, character) = line_col(src, offset);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("tabs child completion")
    else {
        panic!("expected completion array");
    };
    let labels = labels(&items);
    assert!(labels.contains(&"tab"), "{labels:?}");
    assert!(
        !labels.contains(&"note"),
        "tabs completions should prefer accepts kinds: {labels:?}"
    );
    assert!(
        !labels.contains(&"tabs"),
        "tabs completions should not offer nested tabs: {labels:?}"
    );
}

#[test]
fn hover_and_symbols_use_colon_kind_names() {
    let mut server = initialize_server();
    let uri = open_colon(&mut server);

    let symbols = server
        .document_symbol(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("symbols");
    let DocumentSymbolResponse::Nested(syms) = symbols else {
        panic!("expected nested symbols");
    };
    assert!(find_symbol(&syms, ":tabs").is_some());
    assert!(find_symbol(&syms, ":tab").is_some());
    assert!(find_symbol(&syms, ":note").is_some());
    assert!(find_symbol(&syms, ":img").is_some());

    let tab_at = COLON_BLOCKS.find(":tab[id: \"mac\"").expect("tab") + 1;
    let (line, character) = line_col(COLON_BLOCKS, tab_at);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("tab hover");
    let lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains(":tab"), "{}", markup.value);
    assert!(markup.value.contains("macOS"), "{}", markup.value);
}

#[test]
fn goto_end_marker_jumps_to_opener() {
    let mut server = initialize_server();
    let uri = open_colon(&mut server);
    let end_at = COLON_BLOCKS.find(":tabs.end").expect("end marker") + 1;
    let (line, character) = line_col(COLON_BLOCKS, end_at);
    let response = server
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("goto end marker");
    let lsp_types::GotoDefinitionResponse::Scalar(location) = response else {
        panic!("expected scalar location");
    };
    let tabs_at = COLON_BLOCKS.find(":tabs.begin[").expect("tabs opener") + 1;
    let (want_line, want_character) = line_col(COLON_BLOCKS, tabs_at);
    assert_eq!(location.range.start.line, want_line);
    assert_eq!(location.range.start.character, want_character);
}

#[test]
fn interpolation_hover_goto_and_diagnostics_use_hole_span() {
    let mut server = initialize_server();
    let uri: Uri = "file:///Interp.rocdown".parse().expect("interp uri");
    let src = "@roc {\npublished = \"2026-08-23\"\n}\n\nPublished @{published}.\n";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open interp");

    let hole = src.find("@{published}").expect("hole");
    let (line, character) = line_col(src, hole + 2);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("interp hover");
    let range = hover.range.expect("hover range");
    let (start_line, start_character) = line_col(src, hole);
    let (end_line, end_character) = line_col(src, hole + "@{published}".len());
    assert_eq!(range.start.line, start_line);
    assert_eq!(range.start.character, start_character);
    assert_eq!(range.end.line, end_line);
    assert_eq!(range.end.character, end_character);
    let lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("published"), "{}", markup.value);

    let response = server
        .goto_definition(GotoDefinitionParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
        .expect("goto interp");
    let lsp_types::GotoDefinitionResponse::Scalar(location) = response else {
        panic!("expected scalar location");
    };
    let bind = src.find("published =").expect("binding");
    let (bind_line, bind_character) = line_col(src, bind);
    assert_eq!(location.range.start.line, bind_line);
    assert_eq!(location.range.start.character, bind_character);

    let heading = "# Hello @{ver}\n";
    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 2,
                text: heading.to_string(),
            },
        })
        .expect("open heading hole");
    let at = heading.find("@{").expect("heading hole");
    let (diag_line, diag_character) = line_col(heading, at);
    let diag = published
        .diagnostics
        .iter()
        .find(|d| d.message.contains("not allowed in headings"))
        .expect("heading interpolation diagnostic");
    assert_eq!(diag.range.start.line, diag_line);
    assert_eq!(diag.range.start.character, diag_character);
    let close = heading.find('}').expect("close") + 1;
    let (end_line, end_character) = line_col(heading, close);
    assert_eq!(diag.range.end.line, end_line);
    assert_eq!(diag.range.end.character, end_character);
}

#[test]
fn interpolation_hover_yields_to_roc_backend() {
    let mut fake = FakeRocBackend::default();
    fake.set_any_hover(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "```roc\nStr\n```".to_string(),
        }),
        range: None,
    });
    let mut server = initialize_server();
    server.set_roc_backend(Box::new(fake));
    let uri: Uri = "file:///Interp.rocdown".parse().expect("interp uri");
    let src = "@roc {\npublished = \"2026-08-23\"\n}\n\nPublished @{published}.\n";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open interp");

    let hole = src.find("@{published}").expect("hole") + 2;
    let (line, character) = line_col(src, hole);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("roc interp hover");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("Str"), "{}", markup.value);
    assert!(
        !markup.value.contains("Markdown interpolation"),
        "{}",
        markup.value
    );
}

#[test]
fn roc_block_ident_hover_yields_to_roc_backend() {
    let mut fake = FakeRocBackend::default();
    fake.set_any_hover(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "```roc\nStr\n```".to_string(),
        }),
        range: None,
    });
    let mut server = initialize_server();
    server.set_roc_backend(Box::new(fake));
    let uri: Uri = "file:///RocBlock.rocdown".parse().expect("roc uri");
    let src = "@roc {\npublished = \"2026-08-23\"\n}\n\nPublished @{published}.\n";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open roc block");

    let ident = src.find("published =").expect("binding");
    let (line, character) = line_col(src, ident);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("roc block hover");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(markup.value.contains("Str"), "{}", markup.value);
}

#[test]
fn interpolation_type_error_maps_to_expr_span() {
    let uri: Uri = "file:///Interp.rocdown".parse().expect("interp uri");
    let src = "@roc {\npublished = \"2026-08-23\"\n}\n\nPublished @{published}.\n";
    let compiled = compile(
        SourceFile::new("Interp.rocdown", src),
        &CompileOptions::default(),
    );
    let type_name = type_name_from_path(std::path::Path::new("/Interp.rocdown"));
    let projection = project_type_module(&compiled.roc, &compiled.segments, &type_name);
    let expr = "published";
    let from = src.find("@{published}").expect("hole") + 2;
    let mapped = rocci_template::source_to_generated(
        src,
        &projection.roc,
        &projection.segments,
        from as u32,
    )
    .expect("map published");
    let proj = SourceFile::new("projection.roc", &projection.roc);
    let (start_line, start_col) = proj.position(mapped.offset, PositionEncoding::Utf16);
    let (end_line, end_col) =
        proj.position(mapped.offset + expr.len() as u32, PositionEncoding::Utf16);
    let mut fake = FakeRocBackend::default();
    fake.set_diagnostics(vec![Diagnostic {
        range: Range {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "TYPE MISMATCH".to_string(),
        ..Diagnostic::default()
    }]);
    let mut server = initialize_server();
    server.set_roc_backend(Box::new(fake));
    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "rocdown".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open interp");
    let diag = published
        .diagnostics
        .iter()
        .find(|d| d.message == "TYPE MISMATCH")
        .expect("mapped roc diagnostic");
    assert_eq!(diag.source.as_deref(), Some("roc"));
    let (want_line, want_character) = line_col(src, from);
    let (end_line, end_character) = line_col(src, from + expr.len());
    assert_eq!(diag.range.start.line, want_line);
    assert_eq!(diag.range.start.character, want_character);
    assert_eq!(diag.range.end.line, end_line);
    assert_eq!(diag.range.end.character, end_character);
}

#[test]
fn interpolation_completion_uses_roc_backend() {
    let uri: Uri = "file:///InterpComplete.rocdown"
        .parse()
        .expect("interp uri");
    let src = "@roc {\npublished = \"2026-08-23\"\n}\n\nPublished @{published}.\n";
    let mut fake = FakeRocBackend::default();
    fake.set_completion(CompletionResponse::Array(vec![lsp_types::CompletionItem {
        label: "toUtf8".to_string(),
        ..lsp_types::CompletionItem::default()
    }]));
    let mut server = initialize_server();
    server.set_roc_backend(Box::new(fake));
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open interp");
    let hole = src.find("@{published}").expect("hole") + 2;
    let (line, character) = line_col(src, hole);
    let CompletionResponse::Array(items) = server
        .completion(CompletionParams {
            text_document_position: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
        .expect("roc completion")
    else {
        panic!("expected completion array");
    };
    assert!(items.iter().any(|item| item.label == "toUtf8"), "{items:?}");
}

#[test]
fn interpolation_goto_without_binding_has_no_location() {
    let mut server = initialize_server();
    let uri: Uri = "file:///UnboundInterp.rocdown"
        .parse()
        .expect("unbound uri");
    let src = "Published @{missing}.\n";
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocdown".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open unbound interp");

    let hole = src.find("@{missing}").expect("hole");
    let (line, character) = line_col(src, hole + 2);
    let hover = server
        .hover(HoverParams {
            text_document_position_params: position_params(&uri, line, character),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("unbound hover");
    let range = hover.range.expect("hover range");
    let (start_line, start_character) = line_col(src, hole);
    let (end_line, end_character) = line_col(src, hole + "@{missing}".len());
    assert_eq!(range.start.line, start_line);
    assert_eq!(range.start.character, start_character);
    assert_eq!(range.end.line, end_line);
    assert_eq!(range.end.character, end_character);

    let response = server.goto_definition(GotoDefinitionParams {
        text_document_position_params: position_params(&uri, line, character),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    });
    assert!(
        response.is_none(),
        "unbound hole must not be a definition target: {response:?}"
    );
}

#[test]
fn compile_text_resolves_peer_example_link() {
    use rocci_rocdown::lsp::compile_text;
    use std::{env, fs};

    let root = env::temp_dir().join(format!(
        "rocdown-lsp-workspace-{}-{}",
        std::process::id(),
        "peer-link"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs/applications")).unwrap();
    fs::create_dir_all(root.join("examples/counter/source")).unwrap();
    fs::write(
        root.join("docs/rocdown.toml"),
        "[site]\ntitle = \"Docs\"\n\n[[peer]]\nsource = \"../examples\"\nprefix = \"examples\"\n",
    )
    .unwrap();
    fs::write(
        root.join("examples/counter/source/Counter-rocci.rocdown"),
        "# Counter.rocci\n",
    )
    .unwrap();
    let standalone = root.join("docs/applications/standalone.rocdown");
    let src = "See [Counter.rocci](/examples/counter/source/Counter-rocci/).\n";
    fs::write(&standalone, src).unwrap();
    let compiled = compile_text(&format!("file://{}", standalone.display()), src);
    assert!(
        !compiled.has_errors(),
        "{:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
    );

    let bad = "See [typo](/examples/counter/source/Cosunter-rocci/).\n";
    fs::write(&standalone, bad).unwrap();
    let compiled = compile_text(&format!("file://{}", standalone.display()), bad);
    assert!(
        compiled
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unknown Rocdown route")
                && d.message.contains("Cosunter-rocci")),
        "{:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn compile_text_docs_tree_does_not_see_undeclared_site() {
    use rocci_rocdown::lsp::compile_text;
    use std::{env, fs};

    let root = env::temp_dir().join(format!(
        "rocdown-lsp-workspace-{}-{}",
        std::process::id(),
        "no-site-hop"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::create_dir_all(root.join("site/project")).unwrap();
    fs::write(root.join("docs/rocdown.toml"), "[site]\ntitle = \"Docs\"\n").unwrap();
    fs::write(root.join("docs/index.rocdown"), "# Docs\n").unwrap();
    fs::write(root.join("site/rocdown.toml"), "[site]\ntitle = \"Site\"\n").unwrap();
    fs::write(root.join("site/project/status.rocdown"), "# Status\n").unwrap();
    let src = "See [status](/project/status/).\n";
    let page = root.join("docs/index.rocdown");
    fs::write(&page, src).unwrap();
    let compiled = compile_text(&format!("file://{}", page.display()), src);
    assert!(
        compiled
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unknown Rocdown route")
                && d.message.contains("/project/status")),
        "{:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn compile_text_resolves_docs_prefixed_link_from_docs_tree() {
    use rocci_rocdown::lsp::compile_text;
    use std::{env, fs};

    let root = env::temp_dir().join(format!(
        "rocdown-lsp-workspace-{}-{}",
        std::process::id(),
        "docs-tree-link"
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("applications")).unwrap();
    fs::write(root.join("rocdown.toml"), "[site]\ntitle = \"Docs\"\n").unwrap();
    fs::write(root.join("applications/handlers.rocdown"), "# Handlers\n").unwrap();
    let standalone = root.join("applications/standalone.rocdown");
    let src = "Shared streams are [handlers](/docs/applications/handlers/).\n";
    fs::write(&standalone, src).unwrap();
    let compiled = compile_text(&format!("file://{}", standalone.display()), src);
    assert!(
        !compiled.has_errors(),
        "{:?}",
        compiled
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
    );
    let _ = fs::remove_dir_all(root);
}
