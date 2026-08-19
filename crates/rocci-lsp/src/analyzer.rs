use lsp_types::{
    CompletionParams, CompletionResponse, Diagnostic, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, SemanticTokensParams,
    SemanticTokensRangeParams, SemanticTokensRangeResult, SemanticTokensResult, Uri,
};
use rocci_template::{PositionEncoding, SourceFile};

use crate::analysis::offset_at;
use crate::regions::InspectedRegion;
use crate::{analysis, regions, tokens};

pub trait DocumentAnalysis: Send + Sync {
    fn diagnostics(&self) -> Vec<Diagnostic>;
    fn document_symbols(&self, params: &DocumentSymbolParams) -> Option<DocumentSymbolResponse>;
    fn hover(&self, params: &HoverParams) -> Option<Hover>;
    fn goto_definition(&self, params: &GotoDefinitionParams) -> Option<GotoDefinitionResponse>;
    fn completion(&self, params: &CompletionParams) -> Option<CompletionResponse>;
    fn semantic_tokens_full(&self, params: &SemanticTokensParams) -> Option<SemanticTokensResult>;
    fn semantic_tokens_range(
        &self,
        params: &SemanticTokensRangeParams,
    ) -> Option<SemanticTokensRangeResult>;
    fn inspect_regions(&self) -> Option<Vec<InspectedRegion>>;
}

pub trait DocumentAnalyzer: Send + Sync {
    fn can_analyze(&self, uri: &Uri, language_id: Option<&str>) -> bool;
    fn analyze(
        &self,
        name: &str,
        uri: &Uri,
        text: &str,
        encoding: PositionEncoding,
    ) -> Box<dyn DocumentAnalysis>;
}

pub struct RocciAnalysis {
    pub name: String,
    pub uri: Uri,
    pub text: String,
    pub compiled: rocci_template::CompileOutput,
    pub encoding: PositionEncoding,
}

impl DocumentAnalysis for RocciAnalysis {
    fn diagnostics(&self) -> Vec<Diagnostic> {
        analysis::diagnostics(&self.name, &self.text, &self.compiled, self.encoding)
    }

    fn document_symbols(&self, _params: &DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        Some(analysis::document_symbols(
            &self.name,
            &self.text,
            &self.compiled,
            self.encoding,
        ))
    }

    fn hover(&self, params: &HoverParams) -> Option<Hover> {
        let position = params.text_document_position_params.position;
        let source = SourceFile::new(&self.name, &self.text);
        let offset = offset_at(source, position, self.encoding);
        analysis::hover(
            &self.name,
            &self.text,
            &self.compiled,
            offset,
            self.encoding,
        )
    }

    fn goto_definition(&self, params: &GotoDefinitionParams) -> Option<GotoDefinitionResponse> {
        let position = params.text_document_position_params.position;
        let source = SourceFile::new(&self.name, &self.text);
        let offset = offset_at(source, position, self.encoding);
        analysis::goto_definition(
            &self.name,
            &self.text,
            &self.compiled,
            offset,
            self.encoding,
            self.uri.clone(),
        )
    }

    fn completion(&self, params: &CompletionParams) -> Option<CompletionResponse> {
        let position = params.text_document_position.position;
        let source = SourceFile::new(&self.name, &self.text);
        let offset = offset_at(source, position, self.encoding);
        Some(analysis::completion(&self.text, &self.compiled, offset))
    }

    fn semantic_tokens_full(&self, _params: &SemanticTokensParams) -> Option<SemanticTokensResult> {
        Some(SemanticTokensResult::Tokens(tokens::semantic_tokens(
            &self.name,
            &self.text,
            &self.compiled.document,
            self.encoding,
            None,
        )))
    }

    fn semantic_tokens_range(
        &self,
        params: &SemanticTokensRangeParams,
    ) -> Option<SemanticTokensRangeResult> {
        Some(SemanticTokensRangeResult::Tokens(tokens::semantic_tokens(
            &self.name,
            &self.text,
            &self.compiled.document,
            self.encoding,
            Some(params.range),
        )))
    }

    fn inspect_regions(&self) -> Option<Vec<InspectedRegion>> {
        let source = SourceFile::new(&self.name, &self.text);
        let tree = regions::extract_rocci_regions(&self.name, &self.text, &self.compiled.document);
        Some(regions::inspect_regions(source, &tree, self.encoding))
    }
}

pub struct RocciAnalyzer;

impl DocumentAnalyzer for RocciAnalyzer {
    fn can_analyze(&self, uri: &Uri, language_id: Option<&str>) -> bool {
        let path = uri.path().as_str();
        if path.ends_with(".rocdown") || path.ends_with(".md") || path.ends_with(".markdown") {
            return false;
        }
        match language_id {
            Some("rocci") => true,
            Some(_) => false,
            None => path.ends_with(".rocci"),
        }
    }

    fn analyze(
        &self,
        name: &str,
        uri: &Uri,
        text: &str,
        encoding: PositionEncoding,
    ) -> Box<dyn DocumentAnalysis> {
        let compiled = analysis::compile_text(name, text);
        Box::new(RocciAnalysis {
            name: name.to_string(),
            uri: uri.clone(),
            text: text.to_string(),
            compiled,
            encoding,
        })
    }
}
