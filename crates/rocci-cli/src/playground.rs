use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
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

pub const PLAYGROUND_CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'";

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
) -> Result<PlaygroundServerHandle> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("failed to bind playground server to 127.0.0.1:{port}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to set playground listener to non-blocking")?;

    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();

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
        "html_runtime": {
            "available": false,
            "reason": "HTML preview is not available yet. Rocci can parse and lower this file in Rust/WASM, but rendering the generated Roc also requires a Roc runtime in WebAssembly."
        }
    })
    .to_string();

    let session_bytes = session_json.into_bytes();

    let handle = thread::spawn(move || {
        while running_thread.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    handle_connection(stream, &session_bytes);
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

fn handle_connection(mut stream: TcpStream, session_bytes: &[u8]) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));

    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buf[..n]);
    let first_line = req_str.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() < 2 || parts[0] != "GET" {
        let resp =
            "HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let raw_path = parts[1];
    let path = raw_path.split('?').next().unwrap_or("/");

    if path.contains("..") {
        let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    match path {
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
  <script type="module">
    import { PlaygroundApp } from "/app.js";

    async function init() {
      try {
        const resp = await fetch("/api/session");
        const bootstrap = await resp.json();
        const root = document.getElementById("playground-root");
        new PlaygroundApp({ container: root, bootstrap });
      } catch (err) {
        document.body.innerHTML = '<div style="padding: 24px; color: red;">Failed to load playground session: ' + err.message + '</div>';
      }
    }
    init();
  </script>
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
        _ => {
            let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
        }
    }
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

pub fn run_playground_cli(input: &Path, serve: ServeOptions, expected_cli: &str) -> Result<()> {
    if !input.exists() {
        bail!("file not found: {}", input.display());
    }

    let filename = input
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".to_string());

    let lower_name = filename.to_ascii_lowercase();

    if expected_cli == "rocci" {
        if lower_name.ends_with(".rocdown")
            || lower_name.ends_with(".md")
            || lower_name.ends_with(".markdown")
        {
            bail!(
                "'rocci playground' only accepts .rocci templates.\nHint: Run 'rocdown playground {}' for Markdown and Rocdown documents.",
                input.display()
            );
        }
        if !lower_name.ends_with(".rocci") {
            bail!(
                "expected a .rocci file, got '{}'. Run 'rocci playground Foo.rocci'",
                input.display()
            );
        }
    } else if expected_cli == "rocdown" {
        if lower_name.ends_with(".rocci") {
            bail!(
                "'rocdown playground' only accepts .rocdown and .md documents.\nHint: Run 'rocci playground {}' for Rocci templates.",
                input.display()
            );
        }
        if !lower_name.ends_with(".rocdown")
            && !lower_name.ends_with(".md")
            && !lower_name.ends_with(".markdown")
        {
            bail!(
                "expected a .rocdown, .md, or .markdown file, got '{}'. Run 'rocdown playground Guide.rocdown'",
                input.display()
            );
        }
    }

    let source = fs::read_to_string(input)
        .with_context(|| format!("failed to read input file: {}", input.display()))?;

    let language = if lower_name.ends_with(".rocci") {
        "rocci"
    } else {
        "rocdown"
    };

    let port = serve.port.resolve()?;
    let mut server = start_playground_server(&filename, &source, language, port)?;

    eprintln!(
        "Rocci Playground running at {}\nAll parsing, lowering, AST formatting, and diagnostics run in WebAssembly.\nNote: Edits in the playground are in-memory and will not modify {}.",
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
        })
        .context("failed to open playground desktop preview window")?;

        server.stop();
    }

    Ok(())
}
