use lsp_types::{
    ClientCapabilities, CompletionParams, DiagnosticSeverity, DidOpenTextDocumentParams,
    DocumentSymbolParams, DocumentSymbolResponse, GeneralClientCapabilities, HoverParams,
    InitializeParams, PartialResultParams, Position, PositionEncodingKind, SemanticTokensParams,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Uri,
    WorkDoneProgressParams,
};
use rocci_lsp::LanguageServer;

const ALL_SYNTAX_ROCDOWN: &str = include_str!("../../../test/AllSyntax.rocdown");

fn test_uri() -> Uri {
    "file:///AllSyntax.rocdown".parse().expect("test uri")
}

fn initialize_server() -> LanguageServer {
    let mut server = LanguageServer::new();
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
fn golden_lsp_all_syntax_rocdown() {
    let mut server = initialize_server();
    let uri = test_uri();

    // 1. Open document and assert diagnostics
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

    // 2. Document Symbols
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
            assert!(syms.iter().any(|s| s.name == "@img"), "missing @img symbol");
        }
        DocumentSymbolResponse::Flat(_) => panic!("expected nested symbols"),
    }

    // 3. Hover on @page (line 0, char 2)
    let page_hover = server.hover(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(0, 2),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    });
    assert!(page_hover.is_some(), "hover on @page should return info");

    // 4. Hover on heading "All syntax" (line 28, char 5)
    let heading_hover = server.hover(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(28, 5),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    });
    assert!(
        heading_hover.is_some(),
        "hover on heading should return info"
    );

    // 5. Completion for @page fields (line 1, char 4)
    let page_completion = server.completion(CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(1, 4),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    });
    assert!(
        page_completion.is_some(),
        "completion inside @page should return fields"
    );

    // 6. Semantic Tokens Full
    let tokens = server.semantic_tokens_full(SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    });
    assert!(tokens.is_some(), "semantic tokens should be present");

    // 7. Inspect Regions
    let regions = server.inspect_regions(&uri);
    assert!(
        regions.is_some(),
        "inspect regions should return region tree"
    );
    let regions = regions.unwrap();
    assert!(!regions.is_empty(), "region tree should have nodes");
}
