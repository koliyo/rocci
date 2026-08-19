use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use serde_json::Value;

use crate::{
    Error, Result,
    protocol::{
        Document, InitializeResult, ListDocumentsResult, OpenParams, OpenResult, PROTOCOL_VERSION,
        ProbeResult,
    },
};

pub trait AdapterHandler {
    fn adapter_id(&self) -> &str;
    fn probe(&mut self, path: &str) -> Result<ProbeResult>;
    fn list_documents(&mut self, root: &str) -> Result<ListDocumentsResult>;
    fn open(&mut self, params: OpenParams) -> Result<OpenResult>;
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

pub fn serve_stdio(mut handler: impl AdapterHandler) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let params = request
            .get("params")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        match method {
            "initialize" => {
                write_result(
                    &mut stdout,
                    id,
                    InitializeResult {
                        protocol_version: PROTOCOL_VERSION,
                        adapter_id: handler.adapter_id().to_string(),
                        capabilities: vec![
                            "probe".into(),
                            "listDocuments".into(),
                            "open".into(),
                            "shutdown".into(),
                        ],
                    },
                )?;
            }
            "probe" => {
                let path = params.get("path").and_then(Value::as_str).unwrap_or("");
                write_result(&mut stdout, id, handler.probe(path)?)?;
            }
            "listDocuments" => {
                let root = params.get("root").and_then(Value::as_str).unwrap_or("");
                write_result(&mut stdout, id, handler.list_documents(root)?)?;
            }
            "open" => {
                let parsed: OpenParams = serde_json::from_value(params)?;
                write_result(&mut stdout, id, handler.open(parsed)?)?;
            }
            "shutdown" => {
                handler.shutdown()?;
                write_result(&mut stdout, id, Value::Object(Default::default()))?;
                return Ok(());
            }
            _ => continue,
        }
    }
    handler.shutdown()?;
    Ok(())
}

fn write_result<T: Serialize>(out: &mut impl Write, id: Value, result: T) -> Result<()> {
    let _ = id;
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    writeln!(out, "{}", serde_json::to_string(&response)?)?;
    out.flush()?;
    Ok(())
}

pub fn extract_http_url(text: &str) -> Option<String> {
    let plain = strip_ansi(text);
    let start = plain.find("http://")?;
    let rest = &plain[start..];
    let end = rest
        .find(|ch: char| ch.is_whitespace() || ch == '\u{1b}')
        .unwrap_or(rest.len());
    let url = rest[..end].trim_end_matches(['.', ',', ';']);
    if url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:") {
        Some(url.to_string())
    } else {
        None
    }
}

pub fn inspector_url_for(origin: &str) -> String {
    format!("{}/__rocci/dev", origin.trim_end_matches('/'))
}

pub fn origin_from_url(url: &str) -> String {
    let Some(start) = url.find("http://").or_else(|| url.find("https://")) else {
        return url.trim_end_matches('/').to_string();
    };
    let rest = &url[start..];
    let scheme_end = rest.find("://").map(|index| index + 3).unwrap_or(0);
    match rest[scheme_end..].find('/') {
        Some(index) => rest[..scheme_end + index].to_string(),
        None => rest.trim_end_matches('/').to_string(),
    }
}

pub fn url_on_origin(origin: &str, path: &str) -> String {
    let origin = origin.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        format!("{origin}/")
    } else if path.starts_with('/') {
        format!("{origin}{path}")
    } else {
        format!("{origin}/{path}")
    }
}

pub const RUN_SESSION_GRACE: Duration = Duration::from_secs(30);

struct ActiveRun {
    key: String,
    child: Child,
    origin: String,
    inspector_url: String,
    retired_at: Option<Instant>,
}

pub struct RunSessions {
    sessions: Vec<ActiveRun>,
    current: Option<String>,
    grace: Duration,
}

impl Default for RunSessions {
    fn default() -> Self {
        Self::new()
    }
}

impl RunSessions {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            current: None,
            grace: RUN_SESSION_GRACE,
        }
    }

    pub fn open(
        &mut self,
        key: &str,
        bin: &str,
        args: &[String],
        title: String,
        path: &str,
    ) -> Result<OpenResult> {
        self.reap();
        if let Some(index) = self.alive_index(key) {
            self.sessions[index].retired_at = None;
            self.current = Some(key.to_string());
            let session = &self.sessions[index];
            return Ok(OpenResult {
                url: url_on_origin(&session.origin, path),
                title,
                inspector_url: Some(session.inspector_url.clone()),
            });
        }
        let (child, opened) = spawn_run_no_window(bin, args)?;
        let origin = origin_from_url(&opened.url);
        let inspector = opened
            .inspector_url
            .clone()
            .unwrap_or_else(|| inspector_url_for(&origin));
        if let Some(previous) = self.current.take() {
            if previous != key
                && let Some(index) = self
                    .sessions
                    .iter()
                    .position(|session| session.key == previous)
            {
                self.sessions[index].retired_at = Some(Instant::now());
            }
        }
        self.sessions.push(ActiveRun {
            key: key.to_string(),
            child,
            origin: origin.clone(),
            inspector_url: inspector.clone(),
            retired_at: None,
        });
        self.current = Some(key.to_string());
        Ok(OpenResult {
            url: url_on_origin(&origin, path),
            title,
            inspector_url: Some(inspector),
        })
    }

    pub fn shutdown(&mut self) {
        for session in &mut self.sessions {
            let _ = session.child.kill();
            let _ = session.child.wait();
        }
        self.sessions.clear();
        self.current = None;
    }

    fn alive_index(&mut self, key: &str) -> Option<usize> {
        let index = self
            .sessions
            .iter()
            .position(|session| session.key == key)?;
        match self.sessions[index].child.try_wait() {
            Ok(None) => Some(index),
            _ => {
                self.sessions.remove(index);
                None
            }
        }
    }

    fn reap(&mut self) {
        let now = Instant::now();
        let grace = self.grace;
        let current = self.current.clone();
        self.sessions.retain_mut(|session| {
            if current.as_deref() == Some(session.key.as_str()) {
                return true;
            }
            let Some(retired_at) = session.retired_at else {
                return true;
            };
            if now.duration_since(retired_at) < grace {
                return true;
            }
            let _ = session.child.kill();
            let _ = session.child.wait();
            false
        });
    }
}

pub fn spawn_run_no_window(bin: &str, args: &[String]) -> Result<(Child, OpenResult)> {
    let mut child = Command::new(bin)
        .args(args)
        .arg("--no-window")
        .arg("--port")
        .arg("auto")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Error::message(format!("failed to spawn {bin}: {error}")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::message("run stdout missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::message("run stderr missing"))?;
    let url = match wait_for_url(stdout, stderr, Duration::from_secs(120)) {
        Ok(url) => url,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    let title = args
        .iter()
        .rev()
        .find(|item| !item.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| "preview".into());
    Ok((
        child,
        OpenResult {
            inspector_url: Some(inspector_url_for(&url)),
            url,
            title,
        },
    ))
}

fn wait_for_url(
    stdout: std::process::ChildStdout,
    stderr: std::process::ChildStderr,
    timeout: Duration,
) -> Result<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    spawn_url_reader(stdout, tx.clone());
    spawn_url_reader(stderr, tx);
    let started = Instant::now();
    while started.elapsed() < timeout {
        if let Ok(url) = rx.try_recv() {
            return Ok(url);
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(Error::message("timed out waiting for run origin"))
}

fn spawn_url_reader(
    stream: impl std::io::Read + Send + 'static,
    tx: std::sync::mpsc::Sender<String>,
) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(std::result::Result::ok) {
            if let Some(url) = extract_http_url(&line)
                && tx.send(url).is_err()
            {
                break;
            }
        }
    });
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

pub fn documents_from_pages_json(raw: &str) -> Result<Vec<Document>> {
    let rows: Vec<Value> = serde_json::from_str(raw)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let path = row.get("path")?.as_str()?.to_string();
            let title = row
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(&path)
                .to_string();
            let route = row.get("route").and_then(Value::as_str).map(str::to_string);
            Some(Document {
                id: path.clone(),
                title,
                path,
                route,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_url_from_serving_line() {
        let line = "Serving docs at http://127.0.0.1:8123/guides/";
        assert_eq!(
            extract_http_url(line).as_deref(),
            Some("http://127.0.0.1:8123/guides/")
        );
        let colored =
            "\u{1b}[1;32mServing\u{1b}[0m Guide at \u{1b}[1;36mhttp://127.0.0.1:9/\u{1b}[0m";
        assert_eq!(
            extract_http_url(colored).as_deref(),
            Some("http://127.0.0.1:9/")
        );
        assert_eq!(
            extract_http_url("rocdown: serving Docs at http://127.0.0.1:8123/guides/").as_deref(),
            Some("http://127.0.0.1:8123/guides/")
        );
    }

    #[test]
    fn spawn_run_reads_url_from_stderr() {
        let script = concat!(
            "import sys, time\n",
            "print('Serving stub at http://127.0.0.1:59999/', file=sys.stderr, flush=True)\n",
            "time.sleep(30)\n",
        );
        let (mut child, opened) =
            spawn_run_no_window("python3", &["-u".into(), "-c".into(), script.into()])
                .expect("stub origin");
        assert_eq!(opened.url, "http://127.0.0.1:59999/");
        assert_eq!(
            opened.inspector_url.as_deref(),
            Some("http://127.0.0.1:59999/__rocci/dev")
        );
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn pages_json_becomes_documents() {
        let raw = r#"[{"title":"Home","route":"/","path":"index.md","kind":"static"}]"#;
        let docs = documents_from_pages_json(raw).unwrap();
        assert_eq!(docs[0].id, "index.md");
        assert_eq!(docs[0].route.as_deref(), Some("/"));
    }

    #[test]
    fn origin_and_path_join() {
        assert_eq!(
            origin_from_url("http://127.0.0.1:8123/guides/"),
            "http://127.0.0.1:8123"
        );
        assert_eq!(
            url_on_origin("http://127.0.0.1:8123", "/about"),
            "http://127.0.0.1:8123/about"
        );
    }

    #[test]
    fn run_sessions_reuse_the_same_origin() {
        let script = concat!(
            "import sys, time\n",
            "print('Serving stub at http://127.0.0.1:59998/', file=sys.stderr, flush=True)\n",
            "time.sleep(30)\n",
        );
        let args = vec!["-u".into(), "-c".into(), script.into()];
        let mut sessions = RunSessions::new();
        let first = sessions
            .open("root", "python3", &args, "one".into(), "/")
            .unwrap();
        let second = sessions
            .open("root", "python3", &args, "two".into(), "/about")
            .unwrap();
        assert_eq!(first.url, "http://127.0.0.1:59998/");
        assert_eq!(second.url, "http://127.0.0.1:59998/about");
        sessions.shutdown();
    }
}
