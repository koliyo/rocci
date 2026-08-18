use lsp_types::{
    ClientCapabilities, DiagnosticSeverity, DidOpenTextDocumentParams, DocumentSymbolParams,
    DocumentSymbolResponse, GeneralClientCapabilities, HoverParams, InitializeParams,
    PartialResultParams, Position, PositionEncodingKind, SemanticTokensParams,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Uri,
    WorkDoneProgressParams,
};
use rocci_lsp::LanguageServer;

const ALL_SYNTAX_ROCCI: &str = include_str!("../../../test/AllSyntax.rocci");

fn test_uri() -> Uri {
    "file:///AllSyntax.rocci".parse().expect("test uri")
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
fn test_lsp_all_syntax_rocci() {
    let mut server = initialize_server();
    let uri = test_uri();

    // 1. Open document and assert diagnostics
    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocci".to_string(),
                version: 1,
                text: ALL_SYNTAX_ROCCI.to_string(),
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
                syms.iter().any(|s| s.name == "Badge"),
                "missing Badge symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == "Hello"),
                "missing Hello symbol"
            );
            assert!(
                syms.iter().any(|s| s.name == "CounterPage"),
                "missing CounterPage symbol"
            );
        }
        DocumentSymbolResponse::Flat(_) => panic!("expected nested symbols"),
    }

    // 3. Hover on component
    let hello_pos = ALL_SYNTAX_ROCCI.find("<Hello").expect("<Hello") + 1;
    let (line, character) = {
        let mut l = 0;
        let mut col = 0;
        for (i, ch) in ALL_SYNTAX_ROCCI.char_indices() {
            if i >= hello_pos {
                break;
            }
            if ch == '\n' {
                l += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (l, col)
    };

    let hover_res = server.hover(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(line, character),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    });
    assert!(hover_res.is_some(), "hover on <Hello> should return info");

    // 4. Semantic Tokens Full
    let tokens = server.semantic_tokens_full(SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    });
    assert!(tokens.is_some(), "semantic tokens should be present");

    // 5. Inspect Regions
    let regions = server.inspect_regions(&uri);
    assert!(
        regions.is_some(),
        "inspect regions should return region tree"
    );
    let regions = regions.unwrap();
    assert!(!regions.is_empty(), "region tree should have nodes");
}
