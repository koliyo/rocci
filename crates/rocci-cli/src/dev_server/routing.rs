use std::{
    fs,
    io::{self, Write},
    net::TcpStream,
    path::Path,
};

use crate::error_page;
use crate::logs::LogHub;

use super::{LIVE_RELOAD_TAG, PREVIEW_HTML_CSP, ReloadHub};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServeTarget {
    ReloadJs,
    Events,
    Logs,
    LogEvents,
    LogClear,
    Profile,
    Inspect,
    Dev,
    Redirect(String),
    File { relative: String },
    NotFound,
}

pub fn resolve_request(output: &Path, url_path: &str) -> ServeTarget {
    let path = url_path.split(['?', '#']).next().unwrap_or(url_path);
    let path = if path.is_empty() { "/" } else { path };
    if path == "/__rocci/reload.js"
        || path == "/__rocdown/reload.js"
        || path == "/__rocci_okf/reload.js"
    {
        return ServeTarget::ReloadJs;
    }
    if path == "/__rocci/events" || path == "/__rocdown/events" || path == "/__rocci_okf/events" {
        return ServeTarget::Events;
    }
    if path == "/__rocci/logs" || path == "/__rocdown/logs" || path == "/__rocci_okf/logs" {
        return ServeTarget::Logs;
    }
    if path == "/__rocci/logs/events"
        || path == "/__rocdown/logs/events"
        || path == "/__rocci_okf/logs/events"
    {
        return ServeTarget::LogEvents;
    }
    if path == "/__rocci/logs/clear"
        || path == "/__rocdown/logs/clear"
        || path == "/__rocci_okf/logs/clear"
    {
        return ServeTarget::LogClear;
    }
    if path == "/__rocci/profile" || path == "/__rocdown/profile" || path == "/__rocci_okf/profile"
    {
        return ServeTarget::Profile;
    }
    if path == "/__rocci/inspect" || path == "/__rocdown/inspect" || path == "/__rocci_okf/inspect"
    {
        return ServeTarget::Inspect;
    }
    if path == "/__rocci/dev" || path == "/__rocdown/dev" || path == "/__rocci_okf/dev" {
        return ServeTarget::Dev;
    }
    resolve_file_request(output, path)
}

pub fn resolve_published_request(output: &Path, url_path: &str) -> ServeTarget {
    let path = url_path.split(['?', '#']).next().unwrap_or(url_path);
    let path = if path.is_empty() { "/" } else { path };
    resolve_file_request(output, path)
}

pub(crate) fn resolve_file_request(output: &Path, path: &str) -> ServeTarget {
    if path.split('/').any(|segment| segment == "..") {
        return ServeTarget::NotFound;
    }
    let trimmed = path.trim_start_matches('/');
    if path.ends_with('/') {
        let relative = if trimmed.is_empty() {
            "index.html".to_string()
        } else {
            format!("{trimmed}index.html")
        };
        if output.join(&relative).is_file() {
            return ServeTarget::File { relative };
        }
        return ServeTarget::NotFound;
    }
    if !trimmed.is_empty() && output.join(trimmed).is_file() {
        return ServeTarget::File {
            relative: trimmed.to_string(),
        };
    }
    if output.join(trimmed).join("index.html").is_file() {
        return ServeTarget::Redirect(format!("{path}/"));
    }
    if output.join(format!("{trimmed}.html")).is_file() {
        return ServeTarget::File {
            relative: format!("{trimmed}.html"),
        };
    }
    ServeTarget::NotFound
}

pub(crate) fn serve_file(
    stream: &mut TcpStream,
    output: &Path,
    relative: &str,
    status: u16,
    build_error: Option<&str>,
) -> io::Result<()> {
    let path = output.join(relative);
    let bytes = fs::read(&path)?;
    let mime = mime_type(&path);
    let inject = mime.starts_with("text/html");
    let body = if inject {
        let mut html = String::from_utf8_lossy(&bytes).into_owned();
        if let Some(error) = build_error {
            html = error_page::inject_build_error_dialog(&html, error);
        }
        inject_live_reload(&html).into_bytes()
    } else {
        bytes
    };
    write_response(stream, status, mime, inject, &body)
}

pub fn inject_live_reload(html: &str) -> String {
    let html = relax_csp(html);
    if let Some(idx) = html.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + LIVE_RELOAD_TAG.len());
        out.push_str(&html[..idx]);
        out.push_str(LIVE_RELOAD_TAG);
        out.push_str(&html[idx..]);
        out
    } else {
        format!("{html}{LIVE_RELOAD_TAG}")
    }
}

pub(crate) fn relax_csp(html: &str) -> String {
    let html = html
        .replace("script-src 'none'", "script-src 'self'")
        .replace("script-src &#39;none&#39;", "script-src &#39;self&#39;")
        .replace("connect-src 'none'", "connect-src 'self'")
        .replace("connect-src &#39;none&#39;", "connect-src &#39;self&#39;");
    if html.contains("frame-src") {
        html
    } else {
        html.replace("default-src 'none'", "default-src 'none'; frame-src 'self'")
            .replace(
                "default-src &#39;none&#39;",
                "default-src &#39;none&#39;; frame-src &#39;self&#39;",
            )
    }
}

pub(crate) fn write_error_html(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    let html = inject_live_reload(&error_page(message));
    write_response(
        stream,
        500,
        "text/html; charset=utf-8",
        true,
        html.as_bytes(),
    )
}

pub(crate) fn write_build_error_shell(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    let html = inject_live_reload(&error_page::render_build_error_shell(message));
    write_response(
        stream,
        200,
        "text/html; charset=utf-8",
        true,
        html.as_bytes(),
    )
}

pub(crate) fn error_page(message: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Build error</title>
  <style>
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: #111418;
      color: #f1f3f5;
      margin: 0;
      padding: 3rem 2rem;
    }}
    .box {{
      max-width: 48rem;
      margin: 0 auto;
      background: #1c2128;
      border: 1px solid #e06c75;
      border-radius: 8px;
      padding: 2rem;
    }}
    h1 {{
      margin: 0 0 1rem;
      font-size: 1.25rem;
      color: #e06c75;
    }}
    pre {{
      font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
      font-size: 0.9rem;
      white-space: pre-wrap;
      word-break: break-word;
      background: #15181e;
      border-radius: 4px;
      padding: 1rem;
      margin: 0;
      line-height: 1.5;
    }}
  </style>
</head>
<body>
  <div class="box">
    <h1>Build error</h1>
    <pre>{}</pre>
  </div>
</body>
</html>"#,
        html_escape(message)
    )
}

pub(crate) fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn write_response(
    stream: &mut TcpStream,
    status: u16,
    mime: &str,
    inject: bool,
    body: &[u8],
) -> io::Result<()> {
    let status_text = match status {
        200 => "OK",
        204 => "No Content",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Status",
    };
    let csp = if inject { PREVIEW_HTML_CSP } else { "" };
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n{csp}Connection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}

pub(crate) fn write_redirect(stream: &mut TcpStream, location: &str) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )?;
    stream.flush()
}

pub(crate) fn write_log_sse(stream: &mut TcpStream, hub: &LogHub) -> io::Result<()> {
    let rx = hub.subscribe();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
    )?;
    stream.flush()?;
    while let Ok(line) = rx.recv() {
        let data = serde_json::to_string(&line).unwrap_or_else(|_| "{}".into());
        if write!(stream, "event: log\ndata: {data}\n\n").is_err() {
            break;
        }
        if stream.flush().is_err() {
            break;
        }
    }
    Ok(())
}

pub(crate) fn write_sse(stream: &mut TcpStream, hub: &ReloadHub) -> io::Result<()> {
    let rx = hub.subscribe();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
    )?;
    stream.flush()?;
    while let Ok(generation) = rx.recv() {
        if write!(stream, "event: reload\ndata: {generation}\n\n").is_err() {
            break;
        }
        if stream.flush().is_err() {
            break;
        }
    }
    Ok(())
}

pub(crate) fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
