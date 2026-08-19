use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use rocci_desktop::{PreviewOptions, preview};

use crate::serve::ServeOptions;

pub static APP_JS: &[u8] = include_bytes!("../../../playground/dist/app.js");
pub static WORKER_JS: &[u8] = include_bytes!("../../../playground/dist/compiler-worker.js");
pub static STYLES_CSS: &[u8] = include_bytes!("../../../playground/dist/styles.css");
pub static COMPILER_WASM: &[u8] = include_bytes!("../../../playground/dist/compiler.wasm");

pub const PLAYGROUND_CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-src 'self'";
const BOOT_JS: &[u8] = br#"import { PlaygroundApp } from "/app.js";

async function init() {
  try {
    const resp = await fetch("/api/session");
    const bootstrap = await resp.json();
    const root = document.getElementById("playground-root");
    new PlaygroundApp({ container: root, bootstrap });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    document.body.textContent = "Failed to load playground session: " + message;
    document.body.style.padding = "24px";
    document.body.style.color = "red";
  }
}
init();
"#;
const WASM_HTML_REASON: &str = "HTML preview is not available in WASM mode. The browser cannot dynamically compile generated Roc to WebAssembly.";
const LOCAL_HTML_REASON: &str = "HTML is a static Html.render snapshot of the first fixture or a component whose required parameters all have defaults.";
pub const ROCDOWN_LOCAL_HTML_REASON: &str =
    "HTML preview for Rocdown documents is not available in local playground mode yet.";
const MAX_HEADER_BYTES: usize = 16_384;
const MAX_BODY_BYTES: usize = 1_048_576;

pub type PlaygroundCompileHook = Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

pub fn rocci_local_compile_hook(src_dir: PathBuf) -> PlaygroundCompileHook {
    Arc::new(move |body| crate::playground_compile::compile_rocci(body, Some(&src_dir)))
}

pub struct PlaygroundServerHandle {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    pub url: String,
}

impl PlaygroundServerHandle {
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn join(mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for PlaygroundServerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_playground_server(
    filename: &str,
    source: &str,
    language: &str,
    port: u16,
    mode: PlaygroundMode,
    compile_hook: Option<PlaygroundCompileHook>,
    native_languages: &[&str],
) -> Result<PlaygroundServerHandle> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("failed to bind playground server to 127.0.0.1:{port}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to set playground listener to non-blocking")?;

    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();

    let html_runtime = match (mode, native_languages) {
        (PlaygroundMode::Wasm, _) => serde_json::json!({
            "available": false,
            "reason": WASM_HTML_REASON,
        }),
        (PlaygroundMode::Local, ["rocci"]) => serde_json::json!({
            "available": true,
            "reason": LOCAL_HTML_REASON,
        }),
        (PlaygroundMode::Local, _) => serde_json::json!({
            "available": false,
            "reason": ROCDOWN_LOCAL_HTML_REASON,
        }),
    };

    let native_languages_json: Vec<&str> = native_languages.to_vec();

    let session_json = serde_json::json!({
        "protocol_version": 1,
        "documents": [
            {
                "id": "doc1",
                "filename": filename,
                "language": language,
                "source": source
            }
        ],
        "selected_document": "doc1",
        "compiler_wasm_url": "/compiler.wasm",
        "worker_url": "/compiler-worker.js",
        "mode": mode.as_str(),
        "compile_url": if mode == PlaygroundMode::Local { "/api/compile" } else { "" },
        "native_languages": native_languages_json,
        "html_runtime": html_runtime
    })
    .to_string();

    let session_bytes = session_json.into_bytes();
    let compile_hook = compile_hook.filter(|_| mode == PlaygroundMode::Local);

    let handle = thread::spawn(move || {
        while running_thread.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    handle_connection(stream, &session_bytes, compile_hook.as_ref());
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    let url = format!("http://127.0.0.1:{port}");

    Ok(PlaygroundServerHandle {
        running,
        handle: Some(handle),
        url,
    })
}

fn handle_connection(
    mut stream: TcpStream,
    session_bytes: &[u8],
    compile_hook: Option<&PlaygroundCompileHook>,
) {
    let Some(req) = read_http_request(&mut stream) else {
        return;
    };

    if req.path.contains("..") {
        let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    if req.method == "POST" {
        if req.path != "/api/compile" {
            let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
            return;
        }
        let Some(hook) = compile_hook else {
            let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
            return;
        };
        if req.body.len() > MAX_BODY_BYTES {
            send_empty(&mut stream, 413, "Payload Too Large");
            return;
        }
        let body = hook(&req.body);
        send_response(
            &mut stream,
            200,
            "OK",
            "application/json; charset=utf-8",
            &body,
            PLAYGROUND_CSP,
            "no-store, no-cache, must-revalidate",
        );
        return;
    }

    if req.method != "GET" {
        let resp =
            "HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    match req.path.as_str() {
        "/" | "/index.html" => {
            let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rocci Playground</title>
  <link rel="stylesheet" href="/styles.css">
</head>
<body>
  <div id="playground-root"></div>
  <script type="module" src="/boot.js"></script>
</body>
</html>"#;
            send_response(
                &mut stream,
                200,
                "OK",
                "text/html; charset=utf-8",
                html.as_bytes(),
                PLAYGROUND_CSP,
                "no-cache",
            );
        }
        "/boot.js" => {
            send_response(
                &mut stream,
                200,
                "OK",
                "application/javascript; charset=utf-8",
                BOOT_JS,
                PLAYGROUND_CSP,
                "no-cache",
            );
        }
        "/app.js" => {
            send_response(
                &mut stream,
                200,
                "OK",
                "application/javascript; charset=utf-8",
                APP_JS,
                PLAYGROUND_CSP,
                "public, max-age=31536000, immutable",
            );
        }
        "/compiler-worker.js" => {
            send_response(
                &mut stream,
                200,
                "OK",
                "application/javascript; charset=utf-8",
                WORKER_JS,
                PLAYGROUND_CSP,
                "public, max-age=31536000, immutable",
            );
        }
        "/styles.css" => {
            send_response(
                &mut stream,
                200,
                "OK",
                "text/css; charset=utf-8",
                STYLES_CSS,
                PLAYGROUND_CSP,
                "public, max-age=31536000, immutable",
            );
        }
        "/compiler.wasm" => {
            send_response(
                &mut stream,
                200,
                "OK",
                "application/wasm",
                COMPILER_WASM,
                PLAYGROUND_CSP,
                "public, max-age=31536000, immutable",
            );
        }
        "/api/session" => {
            send_response(
                &mut stream,
                200,
                "OK",
                "application/json; charset=utf-8",
                session_bytes,
                PLAYGROUND_CSP,
                "no-store, no-cache, must-revalidate",
            );
        }
        "/api/compile" => {
            send_empty(&mut stream, 405, "Method Not Allowed");
        }
        _ => {
            let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
        }
    }
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));

    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let header_end = loop {
        let n = stream.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > MAX_HEADER_BYTES + MAX_BODY_BYTES {
            send_empty(stream, 413, "Payload Too Large");
            return None;
        }
        if let Some(idx) = find_double_crlf(&buf) {
            break idx;
        }
        if buf.len() > MAX_HEADER_BYTES {
            send_empty(stream, 431, "Request Header Fields Too Large");
            return None;
        }
    };

    let header_bytes = &buf[..header_end];
    let header_str = String::from_utf8_lossy(header_bytes);
    let mut lines = header_str.split("\r\n");
    let first = lines.next().unwrap_or("");
    let parts: Vec<&str> = first.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let method = parts[0].to_string();
    let raw_path = parts[1];
    let path = raw_path.split('?').next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.trim().parse().unwrap_or(0);
        }
        if name.eq_ignore_ascii_case("transfer-encoding")
            && value.to_ascii_lowercase().contains("chunked")
        {
            send_empty(stream, 411, "Length Required");
            return None;
        }
    }

    if content_length > MAX_BODY_BYTES {
        send_empty(stream, 413, "Payload Too Large");
        return None;
    }

    let mut body = buf[header_end + 4..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut tmp).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
        if body.len() > MAX_BODY_BYTES {
            send_empty(stream, 413, "Payload Too Large");
            return None;
        }
    }
    body.truncate(content_length);

    Some(HttpRequest { method, path, body })
}

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn send_empty(stream: &mut TcpStream, status_code: u16, status_text: &str) {
    let header = format!(
        "HTTP/1.1 {status_code} {status_text}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    let _ = stream.write_all(header.as_bytes());
}

fn send_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    csp: &str,
    cache_control: &str,
) {
    let header = format!(
        "HTTP/1.1 {status_code} {status_text}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Content-Security-Policy: {csp}\r\n\
         Cache-Control: {cache_control}\r\n\
         X-Content-Type-Options: nosniff\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );

    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

pub fn playground_source_language(filename: &str) -> Result<&'static str> {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".rocci") {
        Ok("rocci")
    } else if lower.ends_with(".rocdown") || lower.ends_with(".md") || lower.ends_with(".markdown")
    {
        Ok("rocdown")
    } else {
        bail!("expected a .rocci, .rocdown, .md, or .markdown file, got '{filename}'")
    }
}

pub fn run_playground_cli(
    input: &Path,
    serve: ServeOptions,
    expected_cli: &str,
    mode: PlaygroundMode,
    compile_hook: Option<PlaygroundCompileHook>,
) -> Result<()> {
    if !input.exists() {
        bail!("file not found: {}", input.display());
    }

    let filename = input
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".to_string());

    let language = playground_source_language(&filename)?;
    let source = fs::read_to_string(input)
        .with_context(|| format!("failed to read input file: {}", input.display()))?;

    if mode == PlaygroundMode::Local && compile_hook.is_none() {
        bail!("local playground mode requires a native compile hook");
    }

    let native_languages: &[&str] = match (mode, expected_cli) {
        (PlaygroundMode::Wasm, _) => &[],
        (PlaygroundMode::Local, "rocci") => &["rocci"],
        (PlaygroundMode::Local, _) => &["rocci", "rocdown"],
    };

    let port = serve.port.resolve()?;
    let mut server = start_playground_server(
        &filename,
        &source,
        language,
        port,
        mode,
        compile_hook,
        native_languages,
    )?;

    let compiler_note = match mode {
        PlaygroundMode::Wasm => {
            "All parsing, lowering, AST formatting, and diagnostics run in WebAssembly."
        }
        PlaygroundMode::Local => {
            "Parsing and lowering run natively. HTML is a static Html.render snapshot."
        }
    };
    eprintln!(
        "Rocci Playground ({}) running at {}\n{compiler_note}\nNote: Edits in the playground are in-memory and will not modify {}.",
        mode.as_str(),
        server.url,
        input.display()
    );

    if serve.no_window {
        eprintln!("Serving at {} (press Ctrl+C to stop)...", server.url);
        while server.is_running() {
            thread::sleep(Duration::from_millis(100));
        }
    } else {
        let title = format!("{filename} - Rocci Playground");
        preview(PreviewOptions {
            title,
            url: server.url.clone(),
            width: 1200.0,
            height: 800.0,
            devtools: true,
            state_key: None,
            inspector_url: None,
            source_root: None,
        })
        .context("failed to open playground desktop preview window")?;

        server.stop();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::playground_source_language;

    #[test]
    fn playground_source_language_accepts_rocci_and_rocdown() {
        assert_eq!(playground_source_language("Foo.rocci").unwrap(), "rocci");
        assert_eq!(
            playground_source_language("Guide.rocdown").unwrap(),
            "rocdown"
        );
        assert_eq!(playground_source_language("Note.md").unwrap(), "rocdown");
        assert_eq!(
            playground_source_language("Note.markdown").unwrap(),
            "rocdown"
        );
        assert!(playground_source_language("main.roc").is_err());
    }
}
