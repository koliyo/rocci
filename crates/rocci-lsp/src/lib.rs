mod analysis;
mod tokens;

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
use rocci_template::{CompileOutput, PositionEncoding, SourceFile};

use crate::analysis::{compile_text, offset_at};

pub use tokens::{
    EmbeddedRange, TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_NAMESPACE, TOKEN_OPERATOR, TOKEN_PARAMETER,
    TOKEN_PROPERTY, TOKEN_STRING, TOKEN_TYPE,
};

pub struct LanguageServer {
    encoding: PositionEncoding,
    documents: HashMap<String, OpenDocument>,
}

struct OpenDocument {
    text: String,
    compiled: CompileOutput,
}

impl LanguageServer {
    pub fn new() -> Self {
        Self {
            encoding: PositionEncoding::Utf16,
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
                experimental: Some(serde_json::json!({ "embeddedRanges": true })),
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
        if !is_rocci(
            &params.text_document.uri,
            Some(&params.text_document.language_id),
        ) {
            return None;
        }
        Some(self.set_document(params.text_document.uri, params.text_document.text))
    }

    pub fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
    ) -> Option<PublishDiagnosticsParams> {
        let key = uri_key(&params.text_document.uri);
        if !self.documents.contains_key(&key) {
            return None;
        }
        let text = params.content_changes.into_iter().next()?.text;
        Some(self.set_document(params.text_document.uri, text))
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
        let (name, doc) = self.document(&params.text_document.uri)?;
        Some(analysis::document_symbols(
            name,
            &doc.text,
            &doc.compiled,
            self.encoding,
        ))
    }

    pub fn hover(&self, params: HoverParams) -> Option<lsp_types::Hover> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let (name, doc) = self.document(uri)?;
        let source = SourceFile::new(name, &doc.text);
        let offset = offset_at(source, position, self.encoding);
        analysis::hover(name, &doc.text, &doc.compiled, offset, self.encoding)
    }

    pub fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Option<lsp_types::GotoDefinitionResponse> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let position = params.text_document_position_params.position;
        let (name, doc) = self.document(&uri)?;
        let source = SourceFile::new(name, &doc.text);
        let offset = offset_at(source, position, self.encoding);
        analysis::goto_definition(name, &doc.text, &doc.compiled, offset, self.encoding, uri)
    }

    pub fn completion(&self, params: CompletionParams) -> Option<lsp_types::CompletionResponse> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let (name, doc) = self.document(uri)?;
        let source = SourceFile::new(name, &doc.text);
        let offset = offset_at(source, position, self.encoding);
        Some(analysis::completion(&doc.text, &doc.compiled, offset))
    }

    pub fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Option<lsp_types::SemanticTokensResult> {
        let (name, doc) = self.document(&params.text_document.uri)?;
        Some(lsp_types::SemanticTokensResult::Tokens(
            tokens::semantic_tokens(name, &doc.text, &doc.compiled.document, self.encoding, None),
        ))
    }

    pub fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Option<lsp_types::SemanticTokensRangeResult> {
        let (name, doc) = self.document(&params.text_document.uri)?;
        Some(lsp_types::SemanticTokensRangeResult::Tokens(
            tokens::semantic_tokens(
                name,
                &doc.text,
                &doc.compiled.document,
                self.encoding,
                Some(params.range),
            ),
        ))
    }

    pub fn embedded_ranges(&self, uri: &Uri) -> Option<Vec<tokens::EmbeddedRange>> {
        let (name, doc) = self.document(uri)?;
        Some(tokens::embedded_ranges(
            name,
            &doc.text,
            &doc.compiled.document,
            self.encoding,
        ))
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
            method if method == tokens::method_embedded_ranges() => {
                match serde_json::from_value::<SemanticTokensParams>(req.params) {
                    Ok(params) => {
                        Response::new_ok(id, self.embedded_ranges(&params.text_document.uri))
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

    fn set_document(&mut self, uri: Uri, text: String) -> PublishDiagnosticsParams {
        let name = uri_key(&uri);
        let compiled = compile_text(&name, &text);
        let diagnostics = analysis::diagnostics(&name, &text, &compiled, self.encoding);
        self.documents.insert(name, OpenDocument { text, compiled });
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        }
    }

    fn document(&self, uri: &Uri) -> Option<(&str, &OpenDocument)> {
        self.documents
            .get_key_value(&uri_key(uri))
            .map(|(key, doc)| (key.as_str(), doc))
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

fn is_rocci(uri: &Uri, language_id: Option<&str>) -> bool {
    language_id == Some("rocci") || uri_key(uri).ends_with(".rocci")
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
