pub mod analysis;
pub mod analyzer;
#[cfg(not(target_arch = "wasm32"))]
pub mod embedded;
pub mod log;
#[cfg(not(target_arch = "wasm32"))]
pub mod projection_workspace;
pub mod regions;
#[cfg(not(target_arch = "wasm32"))]
pub mod roc_backend;
pub mod tokens;

use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use lsp_server::{ErrorCode, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    PublishDiagnostics,
};
use lsp_types::request::{
    Completion, DocumentSymbolRequest, GotoDefinition, HoverRequest, References, Request as _,
    SemanticTokensFullRequest, SemanticTokensRangeRequest,
};
use lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, CompletionTextEdit,
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentSymbolParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, Location, LocationLink, OneOf,
    Position, PositionEncodingKind, PublishDiagnosticsParams, Range, ReferenceParams,
    SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensRangeParams, SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Uri,
};
use rocci_template::{
    PositionEncoding, Segment, SourceFile, Span, map_generated_span, project_type_module,
    source_to_generated, type_name_from_path,
};

pub use analyzer::{DocumentAnalysis, DocumentAnalyzer, RocciAnalysis, RocciAnalyzer};
pub use regions::{
    InspectedRegion, Language, Region, RegionContext, RegionPurpose, RegionSpan, RegionTree,
    RegionValidationError, css_ranges, executable_roc_ranges, extract_rocci_regions,
    inspect_regions,
};
#[cfg(not(target_arch = "wasm32"))]
pub use roc_backend::{
    ChildRocBackend, FakeRocBackend, NullRocBackend, PROJECTION_PLACEHOLDER_URI, RocBackend,
};
pub use tokens::{
    MOD_DECLARATION, MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION, MOD_READONLY, TOKEN_COMMENT,
    TOKEN_DECORATOR, TOKEN_ENUM_MEMBER, TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_MACRO,
    TOKEN_NAMESPACE, TOKEN_NUMBER, TOKEN_OPERATOR, TOKEN_PARAMETER, TOKEN_PROPERTY, TOKEN_STRING,
    TOKEN_STRUCT, TOKEN_TYPE, TOKEN_VARIABLE,
};

pub fn method_inspect_regions() -> &'static str {
    "rocci/inspectRegions"
}

const CHILD_ENCODING: PositionEncoding = PositionEncoding::Utf16;

struct ProjectionFile {
    path: PathBuf,
    roc: String,
    segments: Vec<Segment>,
}

struct RocState {
    backend: Box<dyn RocBackend>,
    dir: PathBuf,
    files: HashMap<String, ProjectionFile>,
}

pub struct LanguageServer {
    encoding: PositionEncoding,
    analyzers: Vec<Box<dyn DocumentAnalyzer>>,
    documents: HashMap<String, Box<dyn DocumentAnalysis>>,
    roc: Mutex<RocState>,
}

static ROC_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn new_roc_state() -> RocState {
    let seq = ROC_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
    RocState {
        backend: Box::new(NullRocBackend),
        dir: std::env::temp_dir().join(format!("rocci-lsp-roc-{}-{seq}", std::process::id())),
        files: HashMap::new(),
    }
}

impl LanguageServer {
    pub fn new() -> Self {
        Self {
            encoding: PositionEncoding::Utf16,
            analyzers: vec![Box::new(RocciAnalyzer)],
            documents: HashMap::new(),
            roc: Mutex::new(new_roc_state()),
        }
    }

    pub fn with_analyzers(analyzers: Vec<Box<dyn DocumentAnalyzer>>) -> Self {
        Self {
            encoding: PositionEncoding::Utf16,
            analyzers,
            documents: HashMap::new(),
            roc: Mutex::new(new_roc_state()),
        }
    }

    pub fn set_roc_backend(&mut self, backend: Box<dyn RocBackend>) {
        if let Ok(mut roc) = self.roc.lock() {
            roc.backend = backend;
        }
    }

    #[doc(hidden)]
    pub fn projection_path(&self, uri: &Uri) -> Option<std::path::PathBuf> {
        self.roc
            .lock()
            .ok()?
            .files
            .get(&uri_key(uri))
            .map(|file| file.path.clone())
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
                references_provider: Some(OneOf::Left(true)),
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
        self.sync_projection(&name, &uri, analysis.as_ref());
        let mut diagnostics = analysis.diagnostics();
        diagnostics.extend(self.mapped_roc_diagnostics(&name, analysis.as_ref()));
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
        self.sync_projection(&name, &uri, analysis.as_ref());
        let mut diagnostics = analysis.diagnostics();
        diagnostics.extend(self.mapped_roc_diagnostics(&name, analysis.as_ref()));
        self.documents.insert(name, analysis);
        Some(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        })
    }

    pub fn did_close(&mut self, params: DidCloseTextDocumentParams) -> PublishDiagnosticsParams {
        let name = uri_key(&params.text_document.uri);
        self.documents.remove(&name);
        if let Ok(mut roc) = self.roc.lock() {
            roc.files.remove(&name);
        }
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
        let uri = &params.text_document_position_params.text_document.uri;
        let (executable, offset) =
            self.cursor_at(uri, params.text_document_position_params.position)?;
        if executable {
            match self.mapped_roc_hover(uri, offset) {
                Some(hover) => return Some(hover),
                None => crate::log::verbose(format!(
                    "no mapped roc hover at {offset} in {}",
                    uri_key(uri)
                )),
            }
        }
        self.document(uri)?.hover(&params)
    }

    pub fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Option<lsp_types::GotoDefinitionResponse> {
        let uri = &params.text_document_position_params.text_document.uri;
        let (executable, offset) =
            self.cursor_at(uri, params.text_document_position_params.position)?;
        if executable && let Some(response) = self.mapped_roc_definition(uri, offset) {
            return Some(response);
        }
        self.document(uri)?.goto_definition(&params)
    }

    pub fn completion(&self, params: CompletionParams) -> Option<lsp_types::CompletionResponse> {
        let uri = &params.text_document_position.text_document.uri;
        let (executable, offset) = self.cursor_at(uri, params.text_document_position.position)?;
        if executable && let Some(response) = self.mapped_roc_completion(uri, offset) {
            return Some(response);
        }
        self.document(uri)?.completion(&params)
    }

    pub fn references(&self, params: ReferenceParams) -> Option<Vec<Location>> {
        let uri = &params.text_document_position.text_document.uri;
        let (executable, offset) = self.cursor_at(uri, params.text_document_position.position)?;
        if executable {
            return self.mapped_roc_references(uri, offset, params.context.include_declaration);
        }
        None
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
            References::METHOD => match serde_json::from_value::<ReferenceParams>(req.params) {
                Ok(params) => Response::new_ok(id, self.references(params)),
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

    fn cursor_at(&self, uri: &Uri, position: Position) -> Option<(bool, u32)> {
        let doc = self.document(uri)?;
        let source = SourceFile::new(doc.source_name(), doc.source_text());
        let offset = analysis::offset_at(source, position, self.encoding);
        Some((doc.executable_roc_at(offset), offset))
    }

    fn sync_projection(&self, name: &str, uri: &Uri, analysis: &dyn DocumentAnalysis) {
        let Some((roc, segments)) = analysis.generated_roc() else {
            return;
        };
        let path_for_type = uri.path().as_str();
        let type_name = if path_for_type.is_empty() {
            type_name_from_path(std::path::Path::new(analysis.source_name()))
        } else {
            type_name_from_path(std::path::Path::new(path_for_type))
        };
        let projection = project_type_module(roc, segments, &type_name);
        let Ok(mut roc_state) = self.roc.lock() else {
            return;
        };
        let path = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let dir = crate::projection_workspace::workspace_dir(&roc_state.dir, name);
                if let Err(err) = crate::projection_workspace::stage_package(
                    &dir,
                    &type_name,
                    crate::projection_workspace::source_dir(uri, analysis.source_name()).as_deref(),
                ) {
                    crate::log::always(format!("projection workspace failed for {name}: {err}"));
                }
                crate::projection_workspace::projection_path(&roc_state.dir, name, &type_name)
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = uri;
                roc_state.dir.join(projection_file_name(name, &type_name))
            }
        };
        if let Err(err) = roc_state.backend.sync_projection(&path, &projection.roc) {
            crate::log::always(format!(
                "projection sync failed for {name} -> {}: {err}",
                path.display()
            ));
        } else {
            crate::log::verbose(format!(
                "synced projection {} ({} bytes, {} segments, workspace {})",
                path.display(),
                projection.roc.len(),
                projection.segments.len(),
                path.parent()
                    .map(|dir| dir.display().to_string())
                    .unwrap_or_default()
            ));
        }
        roc_state.files.insert(
            name.to_string(),
            ProjectionFile {
                path,
                roc: projection.roc,
                segments: projection.segments,
            },
        );
    }

    fn mapped_roc_hover(&self, uri: &Uri, offset: u32) -> Option<Hover> {
        let (source_name, source_text) = {
            let doc = self.document(uri)?;
            (doc.source_name().to_string(), doc.source_text().to_string())
        };
        let name = uri_key(uri);
        let Ok(mut roc_state) = self.roc.lock() else {
            return None;
        };
        let (path, roc, segments) = {
            let file = roc_state.files.get(&name)?;
            (file.path.clone(), file.roc.clone(), file.segments.clone())
        };
        let mapped = source_to_generated(&source_text, &roc, &segments, offset)?;
        let proj = SourceFile::new("projection.roc", &roc);
        let (line, character) = proj.position(mapped.offset, CHILD_ENCODING);
        crate::log::verbose(format!(
            "hover {name} source@{offset} -> {} {}:{} ({:?})",
            path.display(),
            line,
            character,
            mapped.origin
        ));
        let mut hover = roc_state
            .backend
            .hover(&path, Position::new(line, character))?;
        hover.range = hover_range_for_cursor(
            &source_name,
            &source_text,
            &roc,
            &segments,
            self.encoding,
            offset,
            hover.range,
        );
        crate::log::verbose(format!("hover {name} result range={:?}", hover.range));
        Some(hover)
    }

    fn mapped_roc_diagnostics(
        &self,
        name: &str,
        analysis: &dyn DocumentAnalysis,
    ) -> Vec<Diagnostic> {
        let Ok(mut roc_state) = self.roc.lock() else {
            return Vec::new();
        };
        let (path, roc, segments) = {
            let Some(file) = roc_state.files.get(name) else {
                return Vec::new();
            };
            (file.path.clone(), file.roc.clone(), file.segments.clone())
        };
        let raw = roc_state.backend.diagnostics(&path);
        let source_name = analysis.source_name();
        let source_text = analysis.source_text();
        let encoding = self.encoding;
        raw.into_iter()
            .filter_map(|diagnostic| {
                remap_roc_diagnostic(
                    source_name,
                    source_text,
                    &roc,
                    &segments,
                    encoding,
                    diagnostic,
                )
            })
            .collect()
    }

    fn mapped_roc_definition(&self, uri: &Uri, offset: u32) -> Option<GotoDefinitionResponse> {
        let (source_name, source_text) = {
            let doc = self.document(uri)?;
            (doc.source_name().to_string(), doc.source_text().to_string())
        };
        let name = uri_key(uri);
        let Ok(mut roc_state) = self.roc.lock() else {
            return None;
        };
        let (path, roc, segments) = {
            let file = roc_state.files.get(&name)?;
            (file.path.clone(), file.roc.clone(), file.segments.clone())
        };
        let mapped = source_to_generated(&source_text, &roc, &segments, offset)?;
        let proj = SourceFile::new("projection.roc", &roc);
        let (line, character) = proj.position(mapped.offset, CHILD_ENCODING);
        let response = roc_state
            .backend
            .definition(&path, Position::new(line, character))?;
        map_definition_response(
            &ProjectionMap {
                doc_uri: uri,
                path: &path,
                source_name: &source_name,
                source_text: &source_text,
                projection: &roc,
                segments: &segments,
                encoding: self.encoding,
            },
            response,
        )
    }

    fn mapped_roc_completion(&self, uri: &Uri, offset: u32) -> Option<CompletionResponse> {
        let (source_name, source_text) = {
            let doc = self.document(uri)?;
            (doc.source_name().to_string(), doc.source_text().to_string())
        };
        let name = uri_key(uri);
        let Ok(mut roc_state) = self.roc.lock() else {
            return None;
        };
        let (path, roc, segments) = {
            let file = roc_state.files.get(&name)?;
            (file.path.clone(), file.roc.clone(), file.segments.clone())
        };
        let mapped = source_to_generated(&source_text, &roc, &segments, offset)?;
        let proj = SourceFile::new("projection.roc", &roc);
        let (line, character) = proj.position(mapped.offset, CHILD_ENCODING);
        let response = roc_state
            .backend
            .completion(&path, Position::new(line, character))?;
        Some(map_completion_response(
            &source_name,
            &source_text,
            &roc,
            &segments,
            self.encoding,
            response,
        ))
    }

    fn mapped_roc_references(
        &self,
        uri: &Uri,
        offset: u32,
        include_declaration: bool,
    ) -> Option<Vec<Location>> {
        let (source_name, source_text) = {
            let doc = self.document(uri)?;
            (doc.source_name().to_string(), doc.source_text().to_string())
        };
        let name = uri_key(uri);
        let Ok(mut roc_state) = self.roc.lock() else {
            return None;
        };
        let (path, roc, segments) = {
            let file = roc_state.files.get(&name)?;
            (file.path.clone(), file.roc.clone(), file.segments.clone())
        };
        let mapped = source_to_generated(&source_text, &roc, &segments, offset)?;
        let proj = SourceFile::new("projection.roc", &roc);
        let (line, character) = proj.position(mapped.offset, CHILD_ENCODING);
        let locations = roc_state.backend.references(
            &path,
            Position::new(line, character),
            include_declaration,
        )?;
        let mapped: Vec<_> = locations
            .into_iter()
            .filter_map(|location| {
                map_location(
                    &ProjectionMap {
                        doc_uri: uri,
                        path: &path,
                        source_name: &source_name,
                        source_text: &source_text,
                        projection: &roc,
                        segments: &segments,
                        encoding: self.encoding,
                    },
                    location,
                )
            })
            .collect();
        if mapped.is_empty() {
            None
        } else {
            Some(mapped)
        }
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

#[cfg(target_arch = "wasm32")]
fn projection_file_name(uri_key: &str, type_name: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    uri_key.hash(&mut hasher);
    format!("{type_name}_{:x}.roc", hasher.finish())
}

fn source_span_covering(segments: &[Segment], offset: u32) -> Option<Span> {
    segments
        .iter()
        .filter(|segment| segment.origin.maps_roc_semantics() && segment.source.contains(offset))
        .min_by_key(|segment| segment.source.len())
        .map(|segment| segment.source)
}

fn range_covers_offset(
    source: SourceFile<'_>,
    range: Range,
    offset: u32,
    encoding: PositionEncoding,
) -> bool {
    let start = analysis::offset_at(source, range.start, encoding);
    let end = analysis::offset_at(source, range.end, encoding);
    if end > start {
        offset >= start && offset < end
    } else {
        offset == start
    }
}

fn hover_range_for_cursor(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    offset: u32,
    child_range: Option<Range>,
) -> Option<Range> {
    let source = SourceFile::new(source_name, source_text);
    if let Some(range) = child_range {
        if let Some(mapped) = map_generated_range(
            source_name,
            source_text,
            projection,
            segments,
            encoding,
            range,
        ) {
            if range_covers_offset(source, mapped, offset, encoding) {
                return Some(mapped);
            }
            crate::log::verbose(format!(
                "dropping remapped hover range {mapped:?} that does not cover source@{offset}"
            ));
        } else {
            crate::log::verbose(format!(
                "child hover range {range:?} did not map back to source@{offset}"
            ));
        }
    }
    let span = source_span_covering(segments, offset)?;
    Some(analysis::lsp_range(source, span, encoding))
}

fn remap_roc_diagnostic(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    mut diagnostic: Diagnostic,
) -> Option<Diagnostic> {
    diagnostic.range = map_generated_range(
        source_name,
        source_text,
        projection,
        segments,
        encoding,
        diagnostic.range,
    )?;
    diagnostic.source = Some("roc".to_string());
    diagnostic.related_information = None;
    Some(diagnostic)
}

fn map_generated_range(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    range: Range,
) -> Option<Range> {
    let proj = SourceFile::new("projection.roc", projection);
    let start = analysis::offset_at(proj, range.start, CHILD_ENCODING);
    let end = analysis::offset_at(proj, range.end, CHILD_ENCODING);
    let span = map_generated_span(
        source_text,
        projection,
        segments,
        Span::new(start as usize, end as usize),
    )?;
    Some(analysis::lsp_range(
        SourceFile::new(source_name, source_text),
        span,
        encoding,
    ))
}

fn is_projection_uri(uri: &Uri, path: &std::path::Path) -> bool {
    crate::roc_backend::projection_uri(path)
        .ok()
        .is_some_and(|projection| projection == *uri)
}

struct ProjectionMap<'a> {
    doc_uri: &'a Uri,
    path: &'a std::path::Path,
    source_name: &'a str,
    source_text: &'a str,
    projection: &'a str,
    segments: &'a [Segment],
    encoding: PositionEncoding,
}

fn map_location(map: &ProjectionMap<'_>, mut location: Location) -> Option<Location> {
    if !is_projection_uri(&location.uri, map.path) {
        return Some(location);
    }
    location.range = map_generated_range(
        map.source_name,
        map.source_text,
        map.projection,
        map.segments,
        map.encoding,
        location.range,
    )?;
    location.uri = map.doc_uri.clone();
    Some(location)
}

fn map_location_link(map: &ProjectionMap<'_>, mut link: LocationLink) -> Option<LocationLink> {
    if !is_projection_uri(&link.target_uri, map.path) {
        return Some(link);
    }
    link.target_range = map_generated_range(
        map.source_name,
        map.source_text,
        map.projection,
        map.segments,
        map.encoding,
        link.target_range,
    )?;
    link.target_selection_range = map_generated_range(
        map.source_name,
        map.source_text,
        map.projection,
        map.segments,
        map.encoding,
        link.target_selection_range,
    )?;
    if let Some(origin) = link.origin_selection_range {
        link.origin_selection_range = map_generated_range(
            map.source_name,
            map.source_text,
            map.projection,
            map.segments,
            map.encoding,
            origin,
        );
    }
    link.target_uri = map.doc_uri.clone();
    Some(link)
}

fn map_definition_response(
    map: &ProjectionMap<'_>,
    response: GotoDefinitionResponse,
) -> Option<GotoDefinitionResponse> {
    match response {
        GotoDefinitionResponse::Scalar(location) => {
            let location = map_location(map, location)?;
            Some(GotoDefinitionResponse::Scalar(location))
        }
        GotoDefinitionResponse::Array(locations) => {
            let mapped: Vec<_> = locations
                .into_iter()
                .filter_map(|location| map_location(map, location))
                .collect();
            if mapped.is_empty() {
                None
            } else {
                Some(GotoDefinitionResponse::Array(mapped))
            }
        }
        GotoDefinitionResponse::Link(links) => {
            let mapped: Vec<_> = links
                .into_iter()
                .filter_map(|link| map_location_link(map, link))
                .collect();
            if mapped.is_empty() {
                None
            } else {
                Some(GotoDefinitionResponse::Link(mapped))
            }
        }
    }
}

fn map_text_edit(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    mut edit: TextEdit,
) -> Option<TextEdit> {
    edit.range = map_generated_range(
        source_name,
        source_text,
        projection,
        segments,
        encoding,
        edit.range,
    )?;
    Some(edit)
}

fn map_completion_text_edit(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    edit: CompletionTextEdit,
) -> Option<CompletionTextEdit> {
    match edit {
        CompletionTextEdit::Edit(edit) => map_text_edit(
            source_name,
            source_text,
            projection,
            segments,
            encoding,
            edit,
        )
        .map(CompletionTextEdit::Edit),
        CompletionTextEdit::InsertAndReplace(mut edit) => {
            edit.insert = map_generated_range(
                source_name,
                source_text,
                projection,
                segments,
                encoding,
                edit.insert,
            )?;
            edit.replace = map_generated_range(
                source_name,
                source_text,
                projection,
                segments,
                encoding,
                edit.replace,
            )?;
            Some(CompletionTextEdit::InsertAndReplace(edit))
        }
    }
}

fn map_completion_item(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    mut item: CompletionItem,
) -> CompletionItem {
    item.text_edit = item.text_edit.and_then(|edit| {
        map_completion_text_edit(
            source_name,
            source_text,
            projection,
            segments,
            encoding,
            edit,
        )
    });
    if let Some(edits) = item.additional_text_edits.take() {
        let mapped: Vec<_> = edits
            .into_iter()
            .filter_map(|edit| {
                map_text_edit(
                    source_name,
                    source_text,
                    projection,
                    segments,
                    encoding,
                    edit,
                )
            })
            .collect();
        if !mapped.is_empty() {
            item.additional_text_edits = Some(mapped);
        }
    }
    item
}

fn map_completion_response(
    source_name: &str,
    source_text: &str,
    projection: &str,
    segments: &[Segment],
    encoding: PositionEncoding,
    response: CompletionResponse,
) -> CompletionResponse {
    match response {
        CompletionResponse::Array(items) => CompletionResponse::Array(
            items
                .into_iter()
                .map(|item| {
                    map_completion_item(
                        source_name,
                        source_text,
                        projection,
                        segments,
                        encoding,
                        item,
                    )
                })
                .collect(),
        ),
        CompletionResponse::List(mut list) => {
            list.items = list
                .items
                .into_iter()
                .map(|item| {
                    map_completion_item(
                        source_name,
                        source_text,
                        projection,
                        segments,
                        encoding,
                        item,
                    )
                })
                .collect();
            CompletionResponse::List(list)
        }
    }
}

fn publish_diagnostics(params: PublishDiagnosticsParams) -> Notification {
    Notification::new(PublishDiagnostics::METHOD.to_string(), params)
}

fn invalid_params(id: lsp_server::RequestId, err: impl ToString) -> Response {
    Response::new_err(id, ErrorCode::InvalidParams as i32, err.to_string())
}
