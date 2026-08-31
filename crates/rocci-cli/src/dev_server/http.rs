use std::{
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::inspector;
use crate::logs::{self, LogHub, LogLevel};
use crate::style;

use super::routing::{
    resolve_request, serve_file, write_build_error_shell, write_error_html, write_log_sse,
    write_redirect, write_response, write_sse,
};
use super::{ExtraHttpHandler, RELOAD_JS, ReloadHub, ServeTarget};

pub(crate) struct ServeLoop {
    pub listener: TcpListener,
    pub output: PathBuf,
    pub hub: Arc<ReloadHub>,
    pub last_error: Arc<Mutex<Option<String>>>,
    pub has_build: Arc<AtomicBool>,
    pub backend_port: Option<Arc<AtomicU16>>,
    pub log_handlers: bool,
    pub extra_http: Option<ExtraHttpHandler>,
    pub stop: Arc<AtomicBool>,
}

pub(crate) fn serve_loop(req: ServeLoop) {
    let ServeLoop {
        listener,
        output,
        hub,
        last_error,
        has_build,
        backend_port,
        log_handlers,
        extra_http,
        stop,
    } = req;
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, peer)) => {
                let output = output.clone();
                let hub = hub.clone();
                let last_error = last_error.clone();
                let has_build = has_build.clone();
                let backend_port = backend_port.clone();
                let extra_http = extra_http.clone();
                thread::spawn(move || {
                    let _ = handle_client(HandleClient {
                        stream,
                        loopback_peer: peer.ip().is_loopback(),
                        output: &output,
                        hub: &hub,
                        last_error: &last_error,
                        has_build: &has_build,
                        backend_port: backend_port.as_deref(),
                        log_handlers,
                        extra_http: extra_http.as_ref(),
                    });
                });
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

pub(crate) struct HandleClient<'a> {
    pub stream: TcpStream,
    pub loopback_peer: bool,
    pub output: &'a Path,
    pub hub: &'a ReloadHub,
    pub last_error: &'a Mutex<Option<String>>,
    pub has_build: &'a AtomicBool,
    pub backend_port: Option<&'a AtomicU16>,
    pub log_handlers: bool,
    pub extra_http: Option<&'a ExtraHttpHandler>,
}

pub(crate) fn handle_client(req: HandleClient<'_>) -> io::Result<()> {
    let HandleClient {
        mut stream,
        loopback_peer,
        output,
        hub,
        last_error,
        has_build,
        backend_port,
        log_handlers,
        extra_http,
    } = req;
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request_path(&request).unwrap_or("/");
    let method = request_method(&request).unwrap_or("GET");
    if !loopback_peer && is_loopback_only_preview(path) {
        return write_response(
            &mut stream,
            403,
            "text/plain; charset=utf-8",
            false,
            b"forbidden",
        );
    }
    if let Some(handler) = extra_http
        && let Some((status, content_type, body)) = handler(method, path, &buf[..n])
    {
        return write_response(&mut stream, status, content_type, false, &body);
    }
    let backend = backend_port
        .map(|port| port.load(Ordering::Relaxed))
        .unwrap_or(0);
    let target = resolve_request(output, path);
    if should_proxy(method, path, &target, backend, output) {
        return proxy_to_backend(
            &mut stream,
            &buf[..n],
            backend,
            method,
            path,
            log_handlers,
            &hub.logs,
        );
    }
    match target {
        ServeTarget::ReloadJs => write_response(
            &mut stream,
            200,
            "application/javascript; charset=utf-8",
            false,
            RELOAD_JS.as_bytes(),
        ),
        ServeTarget::Events => write_sse(&mut stream, hub),
        ServeTarget::Logs => write_response(
            &mut stream,
            200,
            "application/json; charset=utf-8",
            false,
            hub.logs.to_json().as_bytes(),
        ),
        ServeTarget::LogEvents => write_log_sse(&mut stream, &hub.logs),
        ServeTarget::LogClear => {
            if method != "POST" {
                return write_response(
                    &mut stream,
                    404,
                    "text/plain; charset=utf-8",
                    false,
                    b"not found",
                );
            }
            hub.logs.clear();
            write_response(&mut stream, 204, "text/plain; charset=utf-8", false, b"")
        }
        ServeTarget::Profile => {
            let body = hub
                .profile()
                .map(|snapshot| snapshot.to_json())
                .unwrap_or_else(|| "{\"total_ms\":0,\"spans\":[]}".to_string());
            write_response(
                &mut stream,
                200,
                "application/json; charset=utf-8",
                false,
                body.as_bytes(),
            )
        }
        ServeTarget::Inspect => {
            let (status, body) = crate::inspect::inspect_json(hub.inspect().as_ref(), path);
            write_response(
                &mut stream,
                status,
                "application/json; charset=utf-8",
                false,
                body.as_bytes(),
            )
        }
        ServeTarget::Dev => {
            let html = inspector::render_panel_with_logs(
                hub.inspect().as_ref(),
                path,
                &hub.logs.snapshot(),
            );
            write_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                true,
                html.as_bytes(),
            )
        }
        ServeTarget::Redirect(location) => write_redirect(&mut stream, &location),
        ServeTarget::File { relative } => {
            let build_error = last_error
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .clone()
                .filter(|_| is_html_file(&relative));
            serve_file(&mut stream, output, &relative, 200, build_error.as_deref())
        }
        ServeTarget::NotFound => {
            let build_error = last_error
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .clone();
            if has_build.load(Ordering::Relaxed) && output.join("404.html").is_file() {
                serve_file(&mut stream, output, "404.html", 404, build_error.as_deref())
            } else if let Some(error) = build_error {
                write_build_error_shell(&mut stream, &error)
            } else if output.join("404.html").is_file() {
                serve_file(&mut stream, output, "404.html", 404, None)
            } else {
                write_error_html(
                    &mut stream,
                    missing_page_message(has_build.load(Ordering::Relaxed)),
                )
            }
        }
    }
}

pub(crate) fn missing_page_message(has_build: bool) -> &'static str {
    if has_build {
        "page not found"
    } else {
        "no built site yet"
    }
}

pub(crate) fn is_html_file(relative: &str) -> bool {
    Path::new(relative)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("html"))
}

pub(crate) fn request_path(request: &str) -> Option<&str> {
    let mut lines = request.split("\r\n");
    let first = lines.next()?;
    let mut parts = first.split_whitespace();
    let _method = parts.next()?;
    parts
        .next()
        .map(|path| path.split(['?', '#']).next().unwrap_or(path))
}

pub(crate) fn request_method(request: &str) -> Option<&str> {
    request.split([' ', '\r', '\n']).next()
}

pub(crate) fn is_preview_internal(path: &str) -> bool {
    path.starts_with("/__rocci")
        || path.starts_with("/__rocdown")
        || path.starts_with("/__rocci_okf")
}

pub(crate) fn is_loopback_only_preview(path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    matches!(
        path,
        "/__rocci/dev"
            | "/__rocdown/dev"
            | "/__rocci_okf/dev"
            | "/__rocci/inspect"
            | "/__rocdown/inspect"
            | "/__rocci_okf/inspect"
            | "/__rocci/logs"
            | "/__rocdown/logs"
            | "/__rocci_okf/logs"
            | "/__rocci/logs/events"
            | "/__rocdown/logs/events"
            | "/__rocci_okf/logs/events"
            | "/__rocci/logs/clear"
            | "/__rocdown/logs/clear"
            | "/__rocci_okf/logs/clear"
            | "/__rocci/profile"
            | "/__rocdown/profile"
            | "/__rocci_okf/profile"
            | "/__rocci_okf/settings"
    )
}

pub(crate) fn is_cdn_owned_get(path: &str) -> bool {
    path == "/" || path == "/index.html"
}

pub(crate) fn should_proxy(
    method: &str,
    path: &str,
    target: &ServeTarget,
    backend: u16,
    output: &Path,
) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    if backend == 0 || is_preview_internal(path) {
        return false;
    }
    if method == "GET" || method == "HEAD" {
        if is_cdn_owned_get(path) {
            return false;
        }
        if matches!(target, ServeTarget::NotFound) {
            if output.join("404.html").is_file()
                && !path_matches_any_island_route(output, method, path)
            {
                return false;
            }
            return true;
        }
        return false;
    }
    true
}

pub(crate) fn path_matches_any_island_route(output: &Path, method: &str, path: &str) -> bool {
    if method != "GET" && method != "HEAD" {
        return true;
    }
    island_get_paths(output)
        .iter()
        .any(|route| path_matches_island_route(path, route))
}

pub(crate) fn island_get_paths(output: &Path) -> Vec<String> {
    let path = output.join("islands.json");
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Vec::new(),
    };
    #[derive(serde::Deserialize)]
    pub(crate) struct RouteRow {
        method: String,
        path: String,
    }
    #[derive(serde::Deserialize)]
    pub(crate) struct IslandsFile {
        routes: Vec<RouteRow>,
    }
    match serde_json::from_slice::<IslandsFile>(&bytes) {
        Ok(file) => file
            .routes
            .into_iter()
            .filter(|route| route.method == "GET" || route.method == "HEAD")
            .map(|route| route.path)
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn normalize_route_path(path: &str) -> &str {
    if path.len() > 1 && path.ends_with('/') {
        path.trim_end_matches('/')
    } else {
        path
    }
}

pub(crate) fn path_matches_island_route(path: &str, route: &str) -> bool {
    normalize_route_path(path) == normalize_route_path(route)
}

pub(crate) fn remaining_body(initial: &[u8]) -> usize {
    let Some(idx) = initial.windows(4).position(|window| window == b"\r\n\r\n") else {
        return 0;
    };
    let headers = &initial[..idx];
    let body = &initial[idx + 4..];
    let headers_text = String::from_utf8_lossy(headers);
    let length = headers_text
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    length.saturating_sub(body.len())
}

pub(crate) fn force_connection_close(headers: &str) -> String {
    let mut out = String::with_capacity(headers.len() + 24);
    let mut saw_connection = false;
    for line in headers.lines() {
        if line.is_empty() {
            continue;
        }
        let Some((name, _)) = line.split_once(':') else {
            out.push_str(line);
            out.push_str("\r\n");
            continue;
        };
        if name.eq_ignore_ascii_case("connection") {
            if !saw_connection {
                out.push_str("Connection: close\r\n");
                saw_connection = true;
            }
            continue;
        }
        if name.eq_ignore_ascii_case("keep-alive") {
            continue;
        }
        out.push_str(line);
        out.push_str("\r\n");
    }
    if !saw_connection {
        out.push_str("Connection: close\r\n");
    }
    out
}

pub(crate) fn rewrite_message_connection_close(message: &[u8]) -> Vec<u8> {
    let Some(idx) = message.windows(4).position(|window| window == b"\r\n\r\n") else {
        return message.to_vec();
    };
    let headers = String::from_utf8_lossy(&message[..idx]);
    let rewritten = force_connection_close(&headers);
    let mut out = Vec::with_capacity(rewritten.len() + 4 + message.len() - idx - 4);
    out.extend_from_slice(rewritten.as_bytes());
    out.extend_from_slice(b"\r\n");
    out.extend_from_slice(&message[idx + 4..]);
    out
}

pub(crate) fn read_headers(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte)?;
        buf.push(byte[0]);
        if buf.len() >= 4 && &buf[buf.len() - 4..] == b"\r\n\r\n" {
            return Ok(buf);
        }
        if buf.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "proxy response headers too large",
            ));
        }
    }
}

pub(crate) fn proxy_to_backend(
    client: &mut TcpStream,
    initial: &[u8],
    port: u16,
    method: &str,
    path: &str,
    log_handlers: bool,
    logs: &LogHub,
) -> io::Result<()> {
    let started = Instant::now();
    let mut backend = match TcpStream::connect(("127.0.0.1", port)) {
        Ok(stream) => stream,
        Err(_) => {
            if log_handlers {
                logs::tee(
                    logs,
                    LogLevel::Warn,
                    style::handler_unavailable(method, path),
                );
            }
            return write_error_html(
                client,
                "island service is not running; static preview is still available",
            );
        }
    };
    let _ = backend.set_nodelay(true);
    let _ = client.set_nodelay(true);
    backend.set_read_timeout(Some(Duration::from_secs(30)))?;
    backend.set_write_timeout(Some(Duration::from_secs(30)))?;
    client.set_write_timeout(None)?;
    let request = rewrite_message_connection_close(initial);
    backend.write_all(&request)?;
    let remaining = remaining_body(&request);
    if remaining > 0 {
        let mut rest = vec![0u8; remaining];
        client.read_exact(&mut rest)?;
        backend.write_all(&rest)?;
    }
    let headers = read_headers(&mut backend)?;
    let rewritten = rewrite_message_connection_close(&headers);
    client.write_all(&rewritten)?;
    client.flush()?;
    match stream_proxy_body(&mut backend, client) {
        Ok(()) => {
            if log_handlers {
                let ms = started.elapsed().as_millis();
                logs::tee(
                    logs,
                    LogLevel::Info,
                    style::handler_proxied(method, path, ms),
                );
            }
            Ok(())
        }
        Err(err) if is_client_abort(&err) => {
            if log_handlers {
                logs::tee(
                    logs,
                    LogLevel::Info,
                    style::handler_proxy_closed(method, path),
                );
            }
            Ok(())
        }
        Err(err) => {
            if log_handlers {
                logs::tee(
                    logs,
                    LogLevel::Error,
                    style::handler_proxy_error(method, path, &err),
                );
            }
            Err(err)
        }
    }
}

pub(crate) fn is_client_abort(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset | io::ErrorKind::UnexpectedEof
    ) || err.to_string().contains("Broken pipe")
}

pub(crate) fn stream_proxy_body(backend: &mut TcpStream, client: &mut TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        match backend.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                client.write_all(&buf[..n])?;
                client.flush()?;
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }
}
