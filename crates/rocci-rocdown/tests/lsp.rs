use lsp_types::{
    ClientCapabilities, DiagnosticSeverity, DidOpenTextDocumentParams, DocumentSymbolParams,
    DocumentSymbolResponse, GeneralClientCapabilities, HoverParams, InitializeParams,
    PartialResultParams, Position, PositionEncodingKind, SemanticTokensParams,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Uri,
    WorkDoneProgressParams,
};
use rocci_lsp::LanguageServer;
use rocci_rocdown::RocdownAnalyzer;

const ALL_SYNTAX_ROCDOWN: &str = include_str!("../../../test/AllSyntax.rocdown");
const EMBEDDED_ROCDOWN: &str = include_str!("../../../test/EmbeddedLanguages.rocdown");

use std::path::PathBuf;

fn test_uri() -> Uri {
    "file:///AllSyntax.rocdown".parse().expect("test uri")
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
            assert!(syms.iter().any(|s| s.name == "@img"), "missing @img symbol");
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
    assert!(syms.iter().any(|s| s.name == "@docs note"));
    assert!(syms.iter().any(|s| s.name == "@docs tabs"));
}
