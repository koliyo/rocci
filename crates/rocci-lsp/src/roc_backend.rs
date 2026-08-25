use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use lsp_server::{Message, Notification, Request, RequestId};
use lsp_types::notification::{Notification as _, PublishDiagnostics};
use lsp_types::{
    ClientCapabilities, Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverParams, InitializeParams, InitializedParams, Position, PublishDiagnosticsParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
};
use serde_json::Value;

const INIT_TIMEOUT: Duration = Duration::from_secs(8);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const DIAGNOSTIC_WAIT: Duration = Duration::from_millis(800);

pub trait RocBackend: Send {
    fn sync_projection(&mut self, path: &Path, text: &str) -> Result<(), String>;
    fn hover(&mut self, path: &Path, position: Position) -> Option<Hover>;
    fn diagnostics(&mut self, path: &Path) -> Vec<Diagnostic> {
        let _ = path;
        Vec::new()
    }
}

pub struct NullRocBackend;

impl RocBackend for NullRocBackend {
    fn sync_projection(&mut self, _path: &Path, _text: &str) -> Result<(), String> {
        Ok(())
    }

    fn hover(&mut self, _path: &Path, _position: Position) -> Option<Hover> {
        None
    }
}

#[derive(Default)]
pub struct FakeRocBackend {
    pub synced: Option<(PathBuf, String)>,
    hovers: HashMap<(u32, u32), Hover>,
    any: Option<Hover>,
    diagnostics: Vec<Diagnostic>,
}

impl FakeRocBackend {
    pub fn set_hover(&mut self, line: u32, character: u32, hover: Hover) {
        self.hovers.insert((line, character), hover);
    }

    pub fn set_any_hover(&mut self, hover: Hover) {
        self.any = Some(hover);
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics = diagnostics;
    }
}

impl RocBackend for FakeRocBackend {
    fn sync_projection(&mut self, path: &Path, text: &str) -> Result<(), String> {
        self.synced = Some((path.to_path_buf(), text.to_string()));
        Ok(())
    }

    fn hover(&mut self, _path: &Path, position: Position) -> Option<Hover> {
        if let Some(hover) = &self.any {
            return Some(hover.clone());
        }
        self.hovers
            .get(&(position.line, position.character))
            .cloned()
    }

    fn diagnostics(&mut self, _path: &Path) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }
}

pub struct ChildRocBackend {
    child: Child,
    stdin: ChildStdin,
    incoming: Receiver<Message>,
    next_id: i32,
    opened: HashMap<PathBuf, i32>,
    diagnostics: HashMap<Uri, Vec<Diagnostic>>,
}

impl ChildRocBackend {
    pub fn spawn(roc_path: &str) -> Result<Self, String> {
        let mut child = Command::new(roc_path)
            .args(["experimental-lsp", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("spawn {roc_path} experimental-lsp: {err}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "roc experimental-lsp stdout missing".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "roc experimental-lsp stderr missing".to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "roc experimental-lsp stdin missing".to_string())?;
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() && !line.is_empty() {
                line.clear();
            }
        });
        let (tx, incoming) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Ok(Some(msg)) = Message::read(&mut reader) {
                if tx.send(msg).is_err() {
                    break;
                }
            }
        });
        let mut backend = Self {
            child,
            stdin,
            incoming,
            next_id: 1,
            opened: HashMap::new(),
            diagnostics: HashMap::new(),
        };
        backend.initialize()?;
        Ok(backend)
    }

    pub fn spawn_from_env() -> Result<Self, String> {
        let path = std::env::var("ROCCI_ROC_PATH").unwrap_or_else(|_| "roc".to_string());
        Self::spawn(&path)
    }

    fn initialize(&mut self) -> Result<(), String> {
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            capabilities: ClientCapabilities::default(),
            ..InitializeParams::default()
        };
        let value = self.request(
            "initialize",
            serde_json::to_value(params).map_err(|err| err.to_string())?,
            INIT_TIMEOUT,
        )?;
        if value.is_null() {
            return Err("roc experimental-lsp initialize returned null".into());
        }
        self.notify(
            "initialized",
            serde_json::to_value(InitializedParams {}).map_err(|err| err.to_string())?,
        )
    }

    fn request(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let req = Request::new(RequestId::from(id), method.to_string(), params);
        Message::Request(req)
            .write(&mut self.stdin)
            .map_err(|err| format!("write {method}: {err}"))?;
        self.stdin
            .flush()
            .map_err(|err| format!("flush {method}: {err}"))?;
        let deadline = Instant::now() + timeout;
        let want = RequestId::from(id);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(format!("{method} timed out"));
            }
            match self.incoming.recv_timeout(remaining) {
                Ok(Message::Response(response)) if response.id == want => {
                    return response
                        .response_result
                        .map_err(|err| format!("{method}: {}", err.message));
                }
                Ok(Message::Notification(not)) => self.ingest_notification(not),
                Ok(Message::Response(_)) => {}
                Ok(Message::Request(_)) => {}
                Err(RecvTimeoutError::Timeout) => return Err(format!("{method} timed out")),
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("roc experimental-lsp disconnected".into());
                }
            }
        }
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let not = Notification::new(method.to_string(), params);
        Message::Notification(not)
            .write(&mut self.stdin)
            .map_err(|err| format!("write {method}: {err}"))?;
        self.stdin
            .flush()
            .map_err(|err| format!("flush {method}: {err}"))
    }

    fn ingest_notification(&mut self, not: Notification) {
        if not.method != PublishDiagnostics::METHOD {
            return;
        }
        let Ok(params) = serde_json::from_value::<PublishDiagnosticsParams>(not.params) else {
            return;
        };
        self.diagnostics.insert(params.uri, params.diagnostics);
    }

    fn wait_diagnostics(&mut self, uri: &Uri) {
        if self.diagnostics.contains_key(uri) {
            return;
        }
        let deadline = Instant::now() + DIAGNOSTIC_WAIT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return;
            }
            match self.incoming.recv_timeout(remaining) {
                Ok(Message::Notification(not)) => {
                    self.ingest_notification(not);
                    if self.diagnostics.contains_key(uri) {
                        return;
                    }
                }
                Ok(Message::Response(_)) | Ok(Message::Request(_)) => {}
                Err(_) => return,
            }
        }
    }

    fn uri_for(&self, path: &Path) -> Result<Uri, String> {
        file_uri(path)
    }
}

impl RocBackend for ChildRocBackend {
    fn sync_projection(&mut self, path: &Path, text: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("create projection dir: {err}"))?;
        }
        std::fs::write(path, text).map_err(|err| format!("write projection: {err}"))?;
        let uri = self.uri_for(path)?;
        self.diagnostics.remove(&uri);
        if let Some(version) = self.opened.get_mut(path) {
            *version += 1;
            let version = *version;
            self.notify(
                "textDocument/didChange",
                serde_json::to_value(DidChangeTextDocumentParams {
                    text_document: VersionedTextDocumentIdentifier {
                        uri: uri.clone(),
                        version,
                    },
                    content_changes: vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: text.to_string(),
                    }],
                })
                .map_err(|err| err.to_string())?,
            )?;
        } else {
            self.opened.insert(path.to_path_buf(), 1);
            self.notify(
                "textDocument/didOpen",
                serde_json::to_value(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: uri.clone(),
                        language_id: "roc".to_string(),
                        version: 1,
                        text: text.to_string(),
                    },
                })
                .map_err(|err| err.to_string())?,
            )?;
        }
        self.wait_diagnostics(&uri);
        Ok(())
    }

    fn hover(&mut self, path: &Path, position: Position) -> Option<Hover> {
        let uri = self.uri_for(path).ok()?;
        let value = self
            .request(
                "textDocument/hover",
                serde_json::to_value(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position,
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                })
                .ok()?,
                REQUEST_TIMEOUT,
            )
            .ok()?;
        if value.is_null() {
            return None;
        }
        serde_json::from_value(value).ok()
    }

    fn diagnostics(&mut self, path: &Path) -> Vec<Diagnostic> {
        let Ok(uri) = self.uri_for(path) else {
            return Vec::new();
        };
        self.diagnostics.get(&uri).cloned().unwrap_or_default()
    }
}

impl Drop for ChildRocBackend {
    fn drop(&mut self) {
        let _ = self.notify("exit", Value::Null);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn file_uri(path: &Path) -> Result<Uri, String> {
    let abs = path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    });
    let mut raw = abs.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw.insert(0, '/');
    }
    format!("file://{raw}")
        .parse()
        .map_err(|err| format!("projection uri: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{HoverContents, MarkupContent, MarkupKind};

    fn sample_hover() -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "```roc\na -> a\n```".to_string(),
            }),
            range: None,
        }
    }

    #[test]
    fn null_backend_returns_no_hover() {
        let mut backend = NullRocBackend;
        backend
            .sync_projection(Path::new("Hello.roc"), "hello")
            .unwrap();
        assert!(
            backend
                .hover(Path::new("Hello.roc"), Position::new(0, 0))
                .is_none()
        );
    }

    #[test]
    fn fake_backend_returns_scripted_hover() {
        let mut backend = FakeRocBackend::default();
        backend.set_hover(1, 4, sample_hover());
        backend
            .sync_projection(
                Path::new("Hello.roc"),
                "Hello := [].{\n    greet = |name| name\n}\n",
            )
            .unwrap();
        let hover = backend
            .hover(Path::new("Hello.roc"), Position::new(1, 4))
            .expect("scripted hover");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("markup");
        };
        assert!(markup.value.contains("a -> a"));
    }
}
