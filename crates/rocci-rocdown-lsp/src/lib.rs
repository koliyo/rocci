use rocci_lsp::{LanguageServer, RocciAnalyzer};
use rocci_rocdown::RocdownAnalyzer;

pub fn composed_server() -> LanguageServer {
    LanguageServer::with_analyzers(vec![Box::new(RocciAnalyzer), Box::new(RocdownAnalyzer)])
}
