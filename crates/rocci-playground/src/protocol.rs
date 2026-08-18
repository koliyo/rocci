use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const HTML_UNAVAILABLE_REASON: &str = "HTML preview is not available in WASM mode. The browser cannot dynamically compile generated Roc to WebAssembly.";
pub const HTML_NO_TARGET_REASON: &str =
    "HTML preview needs a @fixture or a component whose required parameters all have defaults.";

fn default_protocol_version() -> u32 {
    PROTOCOL_VERSION
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaygroundMode {
    #[default]
    Wasm,
    Local,
}

impl PlaygroundMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wasm => "wasm",
            Self::Local => "local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rocci,
    Rocdown,
}

impl Language {
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_ascii_lowercase();
        if lower.ends_with(".rocci") {
            Some(Language::Rocci)
        } else if lower.ends_with(".rocdown")
            || lower.ends_with(".md")
            || lower.ends_with(".markdown")
        {
            Some(Language::Rocdown)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rocci => "rocci",
            Language::Rocdown => "rocdown",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualFile {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualWorkspace {
    pub files: Vec<VirtualFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompileRequest {
    #[serde(default = "default_protocol_version")]
    pub protocol_version: u32,
    #[serde(default)]
    pub revision: u64,
    pub filename: String,
    #[serde(default)]
    pub language: Option<Language>,
    pub source: String,
    #[serde(default)]
    pub workspace: Option<VirtualWorkspace>,
}

impl CompileRequest {
    pub fn resolved_language(&self) -> Language {
        self.language
            .or_else(|| Language::from_filename(&self.filename))
            .unwrap_or(Language::Rocci)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub from: usize,
    pub to: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundHighlightSpan {
    pub from: usize,
    pub to: usize,
    pub kind: String,
    #[serde(default)]
    pub modifiers: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundHighlights {
    pub source: Vec<PlaygroundHighlightSpan>,
    pub roc: Vec<PlaygroundHighlightSpan>,
    pub ast: Vec<PlaygroundHighlightSpan>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HtmlCapability {
    pub available: bool,
    pub reason: String,
}

impl Default for HtmlCapability {
    fn default() -> Self {
        Self {
            available: false,
            reason: HTML_UNAVAILABLE_REASON.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundCapabilities {
    pub roc: Capability,
    pub ast: Capability,
    pub html: HtmlCapability,
}

impl Default for PlaygroundCapabilities {
    fn default() -> Self {
        Self {
            roc: Capability { available: true },
            ast: Capability { available: true },
            html: HtmlCapability::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompileResponse {
    pub protocol_version: u32,
    pub revision: u64,
    pub language: Language,
    pub roc: String,
    pub ast: String,
    #[serde(default)]
    pub html: String,
    pub diagnostics: Vec<PlaygroundDiagnostic>,
    pub highlights: PlaygroundHighlights,
    pub capabilities: PlaygroundCapabilities,
    pub has_errors: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundBootstrapDocument {
    pub id: String,
    pub filename: String,
    pub language: Language,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaygroundBootstrap {
    pub protocol_version: u32,
    pub documents: Vec<PlaygroundBootstrapDocument>,
    pub selected_document: String,
    pub compiler_wasm_url: String,
    pub worker_url: String,
    #[serde(default)]
    pub mode: PlaygroundMode,
    #[serde(default)]
    pub compile_url: String,
    #[serde(default)]
    pub native_languages: Vec<Language>,
    pub html_runtime: HtmlCapability,
}

impl Default for PlaygroundBootstrap {
    fn default() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            documents: Vec::new(),
            selected_document: String::new(),
            compiler_wasm_url: String::new(),
            worker_url: String::new(),
            mode: PlaygroundMode::Wasm,
            compile_url: String::new(),
            native_languages: Vec::new(),
            html_runtime: HtmlCapability::default(),
        }
    }
}
