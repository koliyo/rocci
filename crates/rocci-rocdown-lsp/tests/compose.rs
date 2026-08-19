use lsp_types::{
    ClientCapabilities, DiagnosticSeverity, DidOpenTextDocumentParams, GeneralClientCapabilities,
    InitializeParams, PositionEncodingKind, TextDocumentItem, Uri,
};
use rocci_lsp::LanguageServer;
use rocci_rocdown_lsp::composed_server;

const ROCCI: &str = include_str!("../../../test/AllSyntax.rocci");
const ROCDOWN: &str = include_str!("../../../test/AllSyntax.rocdown");

fn initialize_server() -> LanguageServer {
    let mut server = composed_server();
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

fn open(server: &mut LanguageServer, uri: &str, language_id: &str, text: &str) {
    let uri: Uri = uri.parse().expect("test uri");
    let published = server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: language_id.to_string(),
                version: 1,
                text: text.to_string(),
            },
        })
        .unwrap_or_else(|| panic!("{language_id} documents should be analyzed"));
    assert!(
        published
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != Some(DiagnosticSeverity::ERROR)),
        "unexpected error diagnostics for {language_id}: {:?}",
        published.diagnostics
    );
}

#[test]
fn composition_server_analyzes_rocci_and_rocdown() {
    let mut server = initialize_server();
    open(&mut server, "file:///x.rocci", "rocci", ROCCI);
    open(&mut server, "file:///x.rocdown", "rocdown", ROCDOWN);
}

#[test]
fn rocdown_uri_uses_rocdown_analyzer_even_with_rocci_language_id() {
    let mut server = initialize_server();
    open(
        &mut server,
        "file:///Nested.rocdown",
        "rocci",
        ":steps.begin\n    :step[title: \"One\"] First.\n:steps.end\n",
    );
}
