use std::{
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Context, Result};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rocci_desktop::{PreviewOptions, preview};

use crate::inspect::InspectSnapshot;
use crate::inspector;
use crate::logs::{self, LogHub, LogLevel};
use crate::profile::ProfileSnapshot;

const DEBOUNCE: Duration = Duration::from_millis(200);

pub type PathFilter = Arc<dyn Fn(&Path) -> bool + Send + Sync>;

const RELOAD_JS: &str = r#"(function () {
  function connect() {
    var es = new EventSource("/__rocci/events");
    es.addEventListener("reload", function () { location.reload(); });
    es.onerror = function () {
      es.close();
      setTimeout(connect, 1000);
    };
  }
  connect();
})();
"#;

const LIVE_RELOAD_TAG: &str = r#"<script src="/__rocci/reload.js" defer></script>"#;

pub struct DevServer {
    pub url: String,
    pub title: String,
    pub inspector_url: String,
    pub logs: Arc<LogHub>,
    pub output: PathBuf,
    stop: Arc<AtomicBool>,
    owns_output: bool,
    on_stop: Option<Arc<dyn Fn() + Send + Sync>>,
    _watcher: Option<RecommendedWatcher>,
    _threads: Vec<JoinHandle<()>>,
}

impl DevServer {
    pub fn wait(&self) {
        while !self.stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for DevServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(on_stop) = &self.on_stop {
            on_stop();
        }
        if self.owns_output {
            let _ = fs::remove_dir_all(&self.output);
        }
    }
}

#[derive(Debug)]
pub struct ReloadHub {
    waiters: Mutex<Vec<mpsc::Sender<u64>>>,
    generation: AtomicU64,
    inspect: Mutex<Option<InspectSnapshot>>,
    pub logs: Arc<LogHub>,
}

impl ReloadHub {
    pub fn new() -> Self {
        Self {
            waiters: Mutex::new(Vec::new()),
            generation: AtomicU64::new(0),
            inspect: Mutex::new(None),
            logs: Arc::new(LogHub::new()),
        }
    }

    pub fn subscribe(&self) -> mpsc::Receiver<u64> {
        let (tx, rx) = mpsc::channel();
        self.waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push(tx);
        rx
    }

    pub fn broadcast(&self) {
        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        self.waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .retain(|tx| tx.send(generation).is_ok());
    }

    pub fn set_inspect(&self, snapshot: Option<InspectSnapshot>) {
        *self.inspect.lock().unwrap_or_else(|err| err.into_inner()) = snapshot;
    }

    pub fn inspect(&self) -> Option<InspectSnapshot> {
        self.inspect
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    pub fn profile(&self) -> Option<ProfileSnapshot> {
        self.inspect().map(|snapshot| snapshot.profile)
    }
}

impl Default for ReloadHub {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StaticDevServerConfig {
    pub title: String,
    pub port: u16,
    pub open_path: String,
    pub output: Option<PathBuf>,
    pub watch_paths: Vec<PathBuf>,
    pub custom_filter: Option<PathFilter>,
    pub log_prefix: String,
    pub backend_port: Option<Arc<AtomicU16>>,
    pub on_stop: Option<Arc<dyn Fn() + Send + Sync>>,
}

pub fn serve_static_site<F>(config: StaticDevServerConfig, mut rebuild: F) -> Result<DevServer>
where
    F: FnMut(&Path) -> Result<Option<InspectSnapshot>> + Send + 'static,
{
    let (output, owns_output) = match config.output {
        Some(path) => {
            let abs = if path.is_absolute() {
                path
            } else {
                std::env::current_dir()?.join(path)
            };
            fs::create_dir_all(&abs)
                .with_context(|| format!("failed to create output directory {}", abs.display()))?;
            (abs, false)
        }
        None => {
            let temp = std::env::temp_dir().join(format!(
                "rocci-dev-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            fs::create_dir_all(&temp).with_context(|| {
                format!("failed to create temp output directory {}", temp.display())
            })?;
            (temp, true)
        }
    };

    let hub = Arc::new(ReloadHub::new());
    let last_error = Arc::new(Mutex::new(None));
    let has_build = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));

    match rebuild(&output) {
        Ok(snapshot) => {
            has_build.store(true, Ordering::Relaxed);
            hub.set_inspect(snapshot);
        }
        Err(err) => {
            logs::tee(
                &hub.logs,
                LogLevel::Error,
                format!("{}: {err:#}", config.log_prefix),
            );
            *last_error.lock().unwrap_or_else(|e| e.into_inner()) = Some(format!("{err:#}"));
        }
    }

    let listener = TcpListener::bind(("127.0.0.1", config.port))
        .with_context(|| format!("failed to bind 127.0.0.1:{}", config.port))?;
    listener
        .set_nonblocking(true)
        .context("failed to set listener non-blocking")?;
    let bound = listener.local_addr()?.port();
    let open_path = if config.open_path.is_empty() {
        "/"
    } else {
        &config.open_path
    };
    let open_path = if open_path.starts_with('/') {
        open_path.to_string()
    } else {
        format!("/{open_path}")
    };
    let url = format!("http://127.0.0.1:{bound}{open_path}");
    let inspector_url = format!("http://127.0.0.1:{bound}/__rocci/dev");

    let server_stop = stop.clone();
    let server_hub = hub.clone();
    let server_output = output.clone();
    let server_error = last_error.clone();
    let server_has_build = has_build.clone();
    let server_backend = config.backend_port.clone();
    let server = thread::spawn(move || {
        serve_loop(
            listener,
            server_output,
            server_hub,
            server_error,
            server_has_build,
            server_backend,
            server_stop,
        );
    });

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |result| {
        let _ = tx.send(result);
    })
    .context("failed to start file watcher")?;

    for watch_path in &config.watch_paths {
        if watch_path.exists() {
            let canonical = fs::canonicalize(watch_path).unwrap_or_else(|_| watch_path.clone());
            watcher
                .watch(&canonical, RecursiveMode::Recursive)
                .with_context(|| format!("failed to watch {}", canonical.display()))?;
        }
    }

    let watch_stop = stop.clone();
    let watch_output = output.clone();
    let logs = hub.logs.clone();
    let watch_hub = hub;
    let watch_error = last_error;
    let watch_has_build = has_build;
    let custom_filter = config.custom_filter;
    let log_prefix = config.log_prefix;
    let watch = thread::spawn(move || {
        watch_loop(
            rx,
            rebuild,
            watch_output,
            custom_filter,
            log_prefix,
            WatchCtl {
                hub: watch_hub,
                last_error: watch_error,
                has_build: watch_has_build,
                stop: watch_stop,
            },
        );
    });

    Ok(DevServer {
        url,
        title: config.title,
        inspector_url,
        logs,
        stop,
        output,
        owns_output,
        on_stop: config.on_stop,
        _watcher: Some(watcher),
        _threads: vec![server, watch],
    })
}

pub fn preview_static_site<F>(
    config: StaticDevServerConfig,
    no_window: bool,
    state_key: Option<String>,
    rebuild: F,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<Option<InspectSnapshot>> + Send + 'static,
{
    let prefix = config.log_prefix.clone();
    let title = config.title.clone();
    let server = serve_static_site(config, rebuild)?;
    logs::tee(
        &server.logs,
        LogLevel::Info,
        format!("{prefix}: serving {title} at {}", server.url),
    );
    if no_window {
        server.wait();
        return Ok(());
    }
    let result = preview(PreviewOptions {
        url: server.url.clone(),
        title: format!("{title} — Rocci"),
        state_key,
        width: 1200.0,
        height: 800.0,
        inspector_url: Some(server.inspector_url.clone()),
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

struct WatchCtl {
    hub: Arc<ReloadHub>,
    last_error: Arc<Mutex<Option<String>>>,
    has_build: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

fn watch_loop<F>(
    rx: mpsc::Receiver<notify::Result<notify::Event>>,
    mut rebuild: F,
    output: PathBuf,
    custom_filter: Option<PathFilter>,
    log_prefix: String,
    ctl: WatchCtl,
) where
    F: FnMut(&Path) -> Result<Option<InspectSnapshot>> + Send + 'static,
{
    loop {
        if ctl.stop.load(Ordering::Relaxed) {
            break;
        }
        let event = match rx.recv_timeout(DEBOUNCE) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let mut is_relevant = event_is_relevant(&event, &output, custom_filter.as_deref());
        loop {
            match rx.recv_timeout(DEBOUNCE) {
                Ok(next) => {
                    is_relevant =
                        is_relevant || event_is_relevant(&next, &output, custom_filter.as_deref());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        if !is_relevant {
            continue;
        }
        logs::tee(
            &ctl.hub.logs,
            LogLevel::Info,
            format!("{log_prefix}: rebuilding"),
        );
        match rebuild(&output) {
            Ok(snapshot) => {
                ctl.has_build.store(true, Ordering::Relaxed);
                ctl.hub.set_inspect(snapshot);
                *ctl.last_error.lock().unwrap_or_else(|err| err.into_inner()) = None;
                logs::tee(
                    &ctl.hub.logs,
                    LogLevel::Info,
                    format!("{log_prefix}: rebuilt"),
                );
                ctl.hub.broadcast();
            }
            Err(err) => {
                logs::tee(
                    &ctl.hub.logs,
                    LogLevel::Error,
                    format!("{log_prefix}: {err:#}"),
                );
                *ctl.last_error.lock().unwrap_or_else(|e| e.into_inner()) =
                    Some(format!("{err:#}"));
                ctl.hub.broadcast();
            }
        }
    }
}

fn event_is_relevant(
    event: &notify::Result<notify::Event>,
    output: &Path,
    custom_filter: Option<&(dyn Fn(&Path) -> bool + Send + Sync)>,
) -> bool {
    let Ok(event) = event else {
        return false;
    };
    if matches!(
        event.kind,
        EventKind::Access(_) | EventKind::Modify(notify::event::ModifyKind::Metadata(_))
    ) {
        return false;
    }
    event
        .paths
        .iter()
        .any(|path| path_is_relevant(path, output, custom_filter))
}

fn path_is_relevant(
    path: &Path,
    output: &Path,
    custom_filter: Option<&(dyn Fn(&Path) -> bool + Send + Sync)>,
) -> bool {
    if path.starts_with(output) {
        return false;
    }
    if path
        .components()
        .any(|component| component.as_os_str() == ".git")
    {
        return false;
    }
    if let Some(filter) = custom_filter {
        return filter(path);
    }
    true
}

fn serve_loop(
    listener: TcpListener,
    output: PathBuf,
    hub: Arc<ReloadHub>,
    last_error: Arc<Mutex<Option<String>>>,
    has_build: Arc<AtomicBool>,
    backend_port: Option<Arc<AtomicU16>>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let output = output.clone();
                let hub = hub.clone();
                let last_error = last_error.clone();
                let has_build = has_build.clone();
                let backend_port = backend_port.clone();
                thread::spawn(move || {
                    let _ = handle_client(
                        stream,
                        &output,
                        &hub,
                        &last_error,
                        &has_build,
                        backend_port.as_deref(),
                    );
                });
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn handle_client(
    mut stream: TcpStream,
    output: &Path,
    hub: &ReloadHub,
    last_error: &Mutex<Option<String>>,
    has_build: &AtomicBool,
    backend_port: Option<&AtomicU16>,
) -> io::Result<()> {
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
    let backend = backend_port
        .map(|port| port.load(Ordering::Relaxed))
        .unwrap_or(0);
    let target = resolve_request(output, path);
    if should_proxy(method, path, &target, backend) {
        return proxy_to_backend(&mut stream, &buf[..n], backend);
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
            let body = inject_live_reload(&html);
            write_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                true,
                body.as_bytes(),
            )
        }
        ServeTarget::Redirect(location) => write_redirect(&mut stream, &location),
        ServeTarget::File { relative } => serve_file(&mut stream, output, &relative, 200),
        ServeTarget::NotFound => {
            if has_build.load(Ordering::Relaxed) && output.join("404.html").is_file() {
                serve_file(&mut stream, output, "404.html", 404)
            } else if let Some(error) = last_error
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .clone()
            {
                write_error_html(&mut stream, &error)
            } else if output.join("404.html").is_file() {
                serve_file(&mut stream, output, "404.html", 404)
            } else {
                write_error_html(
                    &mut stream,
                    missing_page_message(has_build.load(Ordering::Relaxed)),
                )
            }
        }
    }
}

fn missing_page_message(has_build: bool) -> &'static str {
    if has_build {
        "page not found"
    } else {
        "no built site yet"
    }
}

fn request_path(request: &str) -> Option<&str> {
    let mut lines = request.split("\r\n");
    let first = lines.next()?;
    let mut parts = first.split_whitespace();
    let _method = parts.next()?;
    parts.next()
}

fn request_method(request: &str) -> Option<&str> {
    request.split([' ', '\r', '\n']).next()
}

fn is_preview_internal(path: &str) -> bool {
    path.starts_with("/__rocci")
        || path.starts_with("/__rocdown")
        || path.starts_with("/__rocci_okf")
}

fn is_cdn_owned_get(path: &str) -> bool {
    path == "/" || path == "/index.html"
}

pub(crate) fn should_proxy(method: &str, path: &str, target: &ServeTarget, backend: u16) -> bool {
    if backend == 0 || is_preview_internal(path) {
        return false;
    }
    if method == "GET" || method == "HEAD" {
        if is_cdn_owned_get(path) {
            return false;
        }
        return matches!(target, ServeTarget::NotFound);
    }
    true
}

fn remaining_body(initial: &[u8]) -> usize {
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

fn proxy_to_backend(client: &mut TcpStream, initial: &[u8], port: u16) -> io::Result<()> {
    let mut backend = match TcpStream::connect(("127.0.0.1", port)) {
        Ok(stream) => stream,
        Err(_) => {
            return write_error_html(
                client,
                "island service is not running; static preview is still available",
            );
        }
    };
    backend.set_read_timeout(Some(Duration::from_secs(30)))?;
    backend.set_write_timeout(Some(Duration::from_secs(30)))?;
    backend.write_all(initial)?;
    let remaining = remaining_body(initial);
    if remaining > 0 {
        let mut rest = vec![0u8; remaining];
        client.read_exact(&mut rest)?;
        backend.write_all(&rest)?;
    }
    io::copy(&mut backend, client)?;
    Ok(())
}

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

fn serve_file(
    stream: &mut TcpStream,
    output: &Path,
    relative: &str,
    status: u16,
) -> io::Result<()> {
    let path = output.join(relative);
    let bytes = fs::read(&path)?;
    let mime = mime_type(&path);
    let inject = mime.starts_with("text/html");
    let body = if inject {
        inject_live_reload(&String::from_utf8_lossy(&bytes)).into_bytes()
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

fn relax_csp(html: &str) -> String {
    html.replace("script-src 'none'", "script-src 'self'")
        .replace("script-src &#39;none&#39;", "script-src &#39;self&#39;")
        .replace("connect-src 'none'", "connect-src 'self'")
        .replace("connect-src &#39;none&#39;", "connect-src &#39;self&#39;")
}

fn write_error_html(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    let html = inject_live_reload(&error_page(message));
    write_response(
        stream,
        500,
        "text/html; charset=utf-8",
        true,
        html.as_bytes(),
    )
}

fn error_page(message: &str) -> String {
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

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn write_response(
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
    let csp = if inject {
        "Content-Security-Policy: default-src 'self' 'unsafe-inline'; connect-src 'self';\r\n"
    } else {
        ""
    };
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n{csp}Connection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}

fn write_redirect(stream: &mut TcpStream, location: &str) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )?;
    stream.flush()
}

fn write_log_sse(stream: &mut TcpStream, hub: &LogHub) -> io::Result<()> {
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

fn write_sse(stream: &mut TcpStream, hub: &ReloadHub) -> io::Result<()> {
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

fn mime_type(path: &Path) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload_hub_subscription_and_broadcast() {
        let hub = ReloadHub::new();
        let rx1 = hub.subscribe();
        let rx2 = hub.subscribe();

        hub.broadcast();
        assert_eq!(rx1.recv().unwrap(), 1);
        assert_eq!(rx2.recv().unwrap(), 1);

        hub.broadcast();
        assert_eq!(rx1.recv().unwrap(), 2);
        assert_eq!(rx2.recv().unwrap(), 2);
    }

    #[test]
    fn test_inject_live_reload_and_relax_csp() {
        let html = "<!doctype html><html><head><meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'none'; connect-src 'none'\"></head><body><h1>Hello</h1></body></html>";
        let injected = inject_live_reload(html);
        assert!(injected.contains("<script src=\"/__rocci/reload.js\" defer></script>"));
        assert!(injected.contains("script-src 'self'"));
        assert!(injected.contains("connect-src 'self'"));
        assert!(!injected.contains("script-src 'none'"));
    }

    #[test]
    fn test_resolve_request_routing() {
        let temp = std::env::temp_dir().join(format!("rocci-dev-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("guide")).unwrap();
        fs::write(temp.join("index.html"), "<h1>Home</h1>").unwrap();
        fs::write(temp.join("guide").join("index.html"), "<h1>Guide</h1>").unwrap();
        fs::write(temp.join("about.html"), "<h1>About</h1>").unwrap();

        assert_eq!(
            resolve_request(&temp, "/__rocci/events"),
            ServeTarget::Events
        );
        assert_eq!(
            resolve_request(&temp, "/__rocdown/events"),
            ServeTarget::Events
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci_okf/events"),
            ServeTarget::Events
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci/profile"),
            ServeTarget::Profile
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci_okf/profile"),
            ServeTarget::Profile
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci/inspect"),
            ServeTarget::Inspect
        );
        assert_eq!(
            resolve_request(&temp, "/__rocdown/inspect?route=/"),
            ServeTarget::Inspect
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci_okf/inspect"),
            ServeTarget::Inspect
        );
        assert_eq!(resolve_request(&temp, "/__rocci/dev"), ServeTarget::Dev);
        assert_eq!(resolve_request(&temp, "/__rocdown/dev"), ServeTarget::Dev);
        assert_eq!(resolve_request(&temp, "/__rocci/logs"), ServeTarget::Logs);
        assert_eq!(
            resolve_request(&temp, "/__rocdown/logs/events"),
            ServeTarget::LogEvents
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci_okf/logs/clear"),
            ServeTarget::LogClear
        );
        assert_eq!(
            resolve_request(&temp, "/__rocci/reload.js"),
            ServeTarget::ReloadJs
        );
        assert_eq!(
            resolve_request(&temp, "/"),
            ServeTarget::File {
                relative: "index.html".into()
            }
        );
        assert_eq!(
            resolve_request(&temp, "/guide"),
            ServeTarget::Redirect("/guide/".into())
        );
        assert_eq!(
            resolve_request(&temp, "/guide/"),
            ServeTarget::File {
                relative: "guide/index.html".into()
            }
        );
        assert_eq!(
            resolve_request(&temp, "/about"),
            ServeTarget::File {
                relative: "about.html".into()
            }
        );
        assert_eq!(
            resolve_request(&temp, "/nonexistent"),
            ServeTarget::NotFound
        );
        assert_eq!(resolve_request(&temp, "/../outside"), ServeTarget::NotFound);

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn missing_page_message_depends_on_build_state() {
        assert_eq!(missing_page_message(false), "no built site yet");
        assert_eq!(missing_page_message(true), "page not found");
    }

    #[test]
    fn should_proxy_posts_and_missing_gets_to_backend() {
        assert!(!should_proxy("GET", "/", &ServeTarget::NotFound, 0));
        assert!(!should_proxy("GET", "/", &ServeTarget::NotFound, 9000));
        assert!(!should_proxy("HEAD", "/", &ServeTarget::NotFound, 9000));
        assert!(!should_proxy(
            "GET",
            "/index.html",
            &ServeTarget::NotFound,
            9000
        ));
        assert!(should_proxy("GET", "/health", &ServeTarget::NotFound, 9000));
        assert!(should_proxy(
            "POST",
            "/actions/reveal/show",
            &ServeTarget::NotFound,
            9000
        ));
        assert!(should_proxy(
            "POST",
            "/",
            &ServeTarget::File {
                relative: "index.html".into()
            },
            9000
        ));
        assert!(!should_proxy(
            "GET",
            "/",
            &ServeTarget::File {
                relative: "index.html".into()
            },
            9000
        ));
        assert!(!should_proxy(
            "GET",
            "/__rocci/events",
            &ServeTarget::Events,
            9000
        ));
        assert!(!should_proxy(
            "POST",
            "/__rocci/events",
            &ServeTarget::Events,
            9000
        ));
    }

    #[test]
    fn remaining_body_uses_content_length() {
        let request =
            b"POST /actions/x HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 7\r\n\r\nhello";
        assert_eq!(remaining_body(request), 2);
        let complete = b"POST /actions/x HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(remaining_body(complete), 0);
    }

    #[test]
    fn static_server_proxies_unmatched_posts() {
        let output = std::env::temp_dir().join(format!(
            "rocci-proxy-out-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("index.html"), "<h1>cdn</h1>").unwrap();

        let backend_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let backend_port = backend_listener.local_addr().unwrap().port();
        let backend = thread::spawn(move || {
            let (mut stream, _) = backend_listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            assert!(request.starts_with("POST /actions/x"), "{request}");
            let body = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 21\r\nConnection: close\r\n\r\n<div id=\"slot-a\">ok</div>";
            stream.write_all(body).unwrap();
        });

        let port = crate::serve::free_port().unwrap();
        let advertised = Arc::new(AtomicU16::new(backend_port));
        let server = serve_static_site(
            StaticDevServerConfig {
                title: "proxy".into(),
                port,
                open_path: "/".into(),
                output: Some(output.clone()),
                watch_paths: Vec::new(),
                custom_filter: None,
                log_prefix: "test".into(),
                backend_port: Some(advertised),
                on_stop: None,
            },
            |_| Ok(None),
        )
        .unwrap();

        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client
            .write_all(
                b"POST /actions/x HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        assert!(response.contains("slot-a"), "{response}");

        let mut home = TcpStream::connect(("127.0.0.1", port)).unwrap();
        home.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut html = String::new();
        home.read_to_string(&mut html).unwrap();
        assert!(html.contains("<h1>cdn</h1>"), "{html}");

        drop(server);
        let _ = backend.join();
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn static_server_serves_inspect_json_after_rebuild() {
        use crate::inspect::{InspectCapabilities, InspectPage, InspectSnapshot, ViewCapability};
        use crate::profile::ProfileSnapshot;
        use std::process::Command;

        let output = std::env::temp_dir().join(format!(
            "rocci-inspect-out-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&output).unwrap();
        let port = crate::serve::free_port().unwrap();
        let server = serve_static_site(
            StaticDevServerConfig {
                title: "inspect".into(),
                port,
                open_path: "/".into(),
                output: Some(output.clone()),
                watch_paths: Vec::new(),
                custom_filter: None,
                log_prefix: "test".into(),
                backend_port: None,
                on_stop: None,
            },
            |out| {
                fs::write(out.join("index.html"), "<h1>home</h1>").unwrap();
                Ok(Some(InspectSnapshot {
                    pages: vec![InspectPage {
                        route: "/".into(),
                        path: "index.rocdown".into(),
                        language: "rocdown".into(),
                        source: "<p>source & \"quotes\"</p>".into(),
                        ast: "(Document)".into(),
                        roc: "module [] {}".into(),
                        html: "<h1>home</h1>".into(),
                        capabilities: InspectCapabilities {
                            source: ViewCapability::available(),
                            ast: ViewCapability::available(),
                            roc: ViewCapability::available(),
                            html: ViewCapability::available(),
                        },
                    }],
                    profile: ProfileSnapshot {
                        total_ms: 2,
                        spans: Vec::new(),
                    },
                }))
            },
        )
        .unwrap();

        let url = format!("http://127.0.0.1:{port}/__rocci/inspect?route=/");
        let curl = Command::new("curl")
            .args(["-sS", "-w", "\n%{http_code}", &url])
            .output()
            .expect("curl");
        assert!(curl.status.success(), "curl failed: {curl:?}");
        let stdout = String::from_utf8_lossy(&curl.stdout);
        let (body, status) = stdout.rsplit_once('\n').unwrap_or((&stdout, ""));
        assert_eq!(status.trim(), "200", "{stdout}");
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(value["path"], "index.rocdown");
        assert_eq!(value["source"], "<p>source & \"quotes\"</p>");
        assert_eq!(value["html"], "<h1>home</h1>");
        assert_eq!(value["profile"]["total_ms"], 2);

        drop(server);
        let _ = fs::remove_dir_all(&output);
    }
}
