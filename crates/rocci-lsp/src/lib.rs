pub mod analysis;
pub mod analyzer;
#[cfg(not(target_arch = "wasm32"))]
pub mod embedded;
pub mod regions;
#[cfg(not(target_arch = "wasm32"))]
pub mod roc_backend;
pub mod tokens;

use std::collections::HashMap;

use lsp_server::{ErrorCode, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    PublishDiagnostics,
};
use lsp_types::request::{
    Completion, DocumentSymbolRequest, GotoDefinition, HoverRequest, Request as _,
    SemanticTokensFullRequest, SemanticTokensRangeRequest,
};
use lsp_types::{
    CompletionOptions, CompletionParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentSymbolParams, GotoDefinitionParams, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, OneOf, PositionEncodingKind,
    PublishDiagnosticsParams, SemanticTokensFullOptions, SemanticTokensOptions,
    SemanticTokensParams, SemanticTokensRangeParams, SemanticTokensServerCapabilities,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use rocci_template::PositionEncoding;

pub use analyzer::{DocumentAnalysis, DocumentAnalyzer, RocciAnalysis, RocciAnalyzer};
pub use regions::{
    InspectedRegion, Language, Region, RegionContext, RegionPurpose, RegionSpan, RegionTree,
    RegionValidationError, css_ranges, executable_roc_ranges, extract_rocci_regions,
    inspect_regions,
};
#[cfg(not(target_arch = "wasm32"))]
pub use roc_backend::{ChildRocBackend, FakeRocBackend, NullRocBackend, RocBackend};
pub use tokens::{
    MOD_DECLARATION, MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION, MOD_READONLY, TOKEN_COMMENT,
    TOKEN_DECORATOR, TOKEN_ENUM_MEMBER, TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_MACRO,
    TOKEN_NAMESPACE, TOKEN_NUMBER, TOKEN_OPERATOR, TOKEN_PARAMETER, TOKEN_PROPERTY, TOKEN_STRING,
    TOKEN_STRUCT, TOKEN_TYPE, TOKEN_VARIABLE,
};

pub fn method_inspect_regions() -> &'static str {
    "rocci/inspectRegions"
}

pub struct LanguageServer {
    encoding: PositionEncoding,
    analyzers: Vec<Box<dyn DocumentAnalyzer>>,
    documents: HashMap<String, Box<dyn DocumentAnalysis>>,
}

impl LanguageServer {
    pub fn new() -> Self {
        Self {
            encoding: PositionEncoding::Utf16,
            analyzers: vec![Box::new(RocciAnalyzer)],
            documents: HashMap::new(),
        }
    }

    pub fn with_analyzers(analyzers: Vec<Box<dyn DocumentAnalyzer>>) -> Self {
        Self {
            encoding: PositionEncoding::Utf16,
            analyzers,
            documents: HashMap::new(),
        }
    }

    pub fn encoding(&self) -> PositionEncoding {
        self.encoding
    }

    pub fn initialize(&mut self, params: InitializeParams) -> InitializeResult {
        self.encoding = negotiate_encoding(&params);
        InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(position_encoding_kind(self.encoding)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["<".to_string(), "@".to_string()]),
                    ..CompletionOptions::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: tokens::legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(true),
                            ..SemanticTokensOptions::default()
                        },
                    ),
                ),
                experimental: Some(serde_json::json!({
                    "inspectRegions": true,
                })),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "rocci-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        }
    }

    pub fn did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
    ) -> Option<PublishDiagnosticsParams> {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let language_id = Some(params.text_document.language_id.as_str());
        let analyzer = self
            .analyzers
            .iter()
            .find(|a| a.can_analyze(&uri, language_id))?;
        let name = uri_key(&uri);
        let analysis = analyzer.analyze(&name, &uri, &text, self.encoding);
        let diagnostics = analysis.diagnostics();
        self.documents.insert(name, analysis);
        Some(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        })
    }

    pub fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> Option<PublishDiagnosticsParams> {
        let uri = params.text_document.uri;
        let name = uri_key(&uri);
        let text = params.content_changes.into_iter().next()?.text;
        let analyzer = self.analyzers.iter().find(|a| a.can_analyze(&uri, None))?;
        let analysis = analyzer.analyze(&name, &uri, &text, self.encoding);
        let diagnostics = analysis.diagnostics();
        self.documents.insert(name, analysis);
        Some(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        })
    }

    pub fn did_close(&mut self, params: DidCloseTextDocumentParams) -> PublishDiagnosticsParams {
        self.documents.remove(&uri_key(&params.text_document.uri));
        PublishDiagnosticsParams {
            uri: params.text_document.uri,
            diagnostics: Vec::new(),
            version: None,
        }
    }

    pub fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Option<lsp_types::DocumentSymbolResponse> {
        let doc = self.document(&params.text_document.uri)?;
        doc.document_symbols(&params)
    }

    pub fn hover(&self, params: HoverParams) -> Option<lsp_types::Hover> {
        let doc = self.document(&params.text_document_position_params.text_document.uri)?;
        doc.hover(&params)
    }

    pub fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Option<lsp_types::GotoDefinitionResponse> {
        let doc = self.document(&params.text_document_position_params.text_document.uri)?;
        doc.goto_definition(&params)
    }

    pub fn completion(&self, params: CompletionParams) -> Option<lsp_types::CompletionResponse> {
        let doc = self.document(&params.text_document_position.text_document.uri)?;
        doc.completion(&params)
    }

    pub fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Option<lsp_types::SemanticTokensResult> {
        let doc = self.document(&params.text_document.uri)?;
        doc.semantic_tokens_full(&params)
    }

    pub fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Option<lsp_types::SemanticTokensRangeResult> {
        let doc = self.document(&params.text_document.uri)?;
        doc.semantic_tokens_range(&params)
    }

    pub fn inspect_regions(&self, uri: &Uri) -> Option<Vec<regions::InspectedRegion>> {
        let doc = self.document(uri)?;
        doc.inspect_regions()
    }

    pub fn handle_request(&self, req: Request) -> Response {
        let id = req.id.clone();
        match req.method.as_str() {
            DocumentSymbolRequest::METHOD => {
                match serde_json::from_value::<DocumentSymbolParams>(req.params) {
                    Ok(params) => Response::new_ok(id, self.document_symbol(params)),
                    Err(err) => invalid_params(id, err),
                }
            }
            HoverRequest::METHOD => match serde_json::from_value::<HoverParams>(req.params) {
                Ok(params) => Response::new_ok(id, self.hover(params)),
                Err(err) => invalid_params(id, err),
            },
            GotoDefinition::METHOD => {
                match serde_json::from_value::<GotoDefinitionParams>(req.params) {
                    Ok(params) => Response::new_ok(id, self.goto_definition(params)),
                    Err(err) => invalid_params(id, err),
                }
            }
            Completion::METHOD => match serde_json::from_value::<CompletionParams>(req.params) {
                Ok(params) => Response::new_ok(id, self.completion(params)),
                Err(err) => invalid_params(id, err),
            },
            SemanticTokensFullRequest::METHOD => {
                match serde_json::from_value::<SemanticTokensParams>(req.params) {
                    Ok(params) => Response::new_ok(id, self.semantic_tokens_full(params)),
                    Err(err) => invalid_params(id, err),
                }
            }
            SemanticTokensRangeRequest::METHOD => {
                match serde_json::from_value::<SemanticTokensRangeParams>(req.params) {
                    Ok(params) => Response::new_ok(id, self.semantic_tokens_range(params)),
                    Err(err) => invalid_params(id, err),
                }
            }

            method if method == method_inspect_regions() => {
                match serde_json::from_value::<SemanticTokensParams>(req.params) {
                    Ok(params) => {
                        Response::new_ok(id, self.inspect_regions(&params.text_document.uri))
                    }
                    Err(err) => invalid_params(id, err),
                }
            }
            other => Response::new_err(
                id,
                ErrorCode::MethodNotFound as i32,
                format!("unknown method {other}"),
            ),
        }
    }

    pub fn handle_notification(&mut self, not: Notification) -> Option<Notification> {
        match not.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params =
                    serde_json::from_value::<DidOpenTextDocumentParams>(not.params).ok()?;
                self.did_open(params).map(publish_diagnostics)
            }
            DidChangeTextDocument::METHOD => {
                let params =
                    serde_json::from_value::<DidChangeTextDocumentParams>(not.params).ok()?;
                self.did_change(params).map(publish_diagnostics)
            }
            DidCloseTextDocument::METHOD => {
                let params =
                    serde_json::from_value::<DidCloseTextDocumentParams>(not.params).ok()?;
                Some(publish_diagnostics(self.did_close(params)))
            }
            _ => None,
        }
    }

    fn document(&self, uri: &Uri) -> Option<&(dyn DocumentAnalysis + 'static)> {
        self.documents.get(&uri_key(uri)).map(|b| &**b)
    }
}

impl Default for LanguageServer {
    fn default() -> Self {
        Self::new()
    }
}

fn negotiate_encoding(params: &InitializeParams) -> PositionEncoding {
    let Some(encodings) = params
        .capabilities
        .general
        .as_ref()
        .and_then(|general| general.position_encodings.as_ref())
    else {
        return PositionEncoding::Utf16;
    };
    if encodings.contains(&PositionEncodingKind::UTF8) {
        PositionEncoding::Utf8
    } else {
        PositionEncoding::Utf16
    }
}

fn position_encoding_kind(encoding: PositionEncoding) -> PositionEncodingKind {
    match encoding {
        PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
        PositionEncoding::Utf16 => PositionEncodingKind::UTF16,
    }
}

fn uri_key(uri: &Uri) -> String {
    uri.to_string()
}

fn publish_diagnostics(params: PublishDiagnosticsParams) -> Notification {
    Notification::new(PublishDiagnostics::METHOD.to_string(), params)
}

fn invalid_params(id: lsp_server::RequestId, err: impl ToString) -> Response {
    Response::new_err(id, ErrorCode::InvalidParams as i32, err.to_string())
}
