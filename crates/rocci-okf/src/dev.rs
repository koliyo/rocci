use std::{
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use okf::{Profile, load};

use crate::presentation::build_review_site;

const DEBOUNCE: Duration = Duration::from_millis(200);
const RELOAD_JS: &str = r#"(function () {
  function connect() {
    var es = new EventSource("/__rocci_okf/events");
    es.addEventListener("reload", function () { location.reload(); });
    es.onerror = function () {
      es.close();
      setTimeout(connect, 1000);
    };
  }
  connect();
})();
"#;

const DEFAULT_CSS: &str = r#"
:root {
  --rd-font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;
  --rd-font-mono: ui-monospace, "SF Mono", Menlo, Monaco, Consolas, "Liberation Mono", monospace;
  --rd-bg: #ffffff;
  --rd-fg: #1f2328;
  --rd-border: #d0d7de;
  --rd-border-subtle: #eaeef2;
  --rd-bg-subtle: #f6f8fa;
  --rd-primary: #0969da;
}
@media (prefers-color-scheme: dark) {
  :root {
    --rd-bg: #0d1117;
    --rd-fg: #e6edf3;
    --rd-border: #30363d;
    --rd-border-subtle: #21262d;
    --rd-bg-subtle: #161b22;
    --rd-primary: #4493f8;
  }
}
body {
  font-family: var(--rd-font-sans);
  background: var(--rd-bg);
  color: var(--rd-fg);
  margin: 0;
  padding: 2rem;
  line-height: 1.5;
}
main.rd-document {
  max-width: 960px;
  margin: 0 auto;
}
a { color: var(--rd-primary); text-decoration: none; }
a:hover { text-decoration: underline; }
code { font-family: var(--rd-font-mono); font-size: 0.9em; background: var(--rd-bg-subtle); padding: 0.2em 0.4em; border-radius: 4px; }
.okf-badge-group { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 1rem; }
.okf-badge { font-size: 0.8rem; padding: 0.2rem 0.5rem; border-radius: 9999px; border: 1px solid var(--rd-border); font-weight: 500; }
.okf-type { background: var(--rd-bg-subtle); }
.okf-status-stable { background: #dafbe1; color: #1a7f37; border-color: #4ac26b; }
.okf-status-draft { background: #fff8c5; color: #9a6700; border-color: #d4a72c; }
.okf-status-deprecated { background: #ffebe9; color: #cf222e; border-color: #ff8182; }
.okf-alert-banner { display: flex; gap: 0.5rem; background: #fff8c5; border: 1px solid #d4a72c; padding: 0.75rem 1rem; border-radius: 6px; margin: 1rem 0; }
.okf-meta-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.5rem; background: var(--rd-bg-subtle); padding: 1rem; border-radius: 6px; margin-bottom: 1rem; font-size: 0.9rem; }
.okf-meta-label { font-weight: 600; margin-right: 0.5rem; }
.okf-stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 1rem; margin-bottom: 2rem; }
.okf-stat-card { background: var(--rd-bg-subtle); border: 1px solid var(--rd-border); padding: 1rem; border-radius: 8px; text-align: center; }
.okf-stat-value { font-size: 1.8rem; font-weight: bold; }
.okf-stat-label { font-size: 0.85rem; color: #656d76; }
.okf-stat-card.is-action .okf-stat-value { color: #cf222e; }
.okf-review-table { width: 100%; border-collapse: collapse; margin-top: 1rem; font-size: 0.9rem; }
.okf-review-table th, .okf-review-table td { padding: 0.75rem; border: 1px solid var(--rd-border); text-align: left; vertical-align: top; }
.okf-review-table th { background: var(--rd-bg-subtle); }
.okf-action-pill { display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.8rem; font-weight: 600; }
.pill-action { background: #fff8c5; color: #9a6700; border: 1px solid #d4a72c; }
.pill-error { background: #ffebe9; color: #cf222e; border: 1px solid #ff8182; }
.pill-clean { background: #dafbe1; color: #1a7f37; border: 1px solid #4ac26b; }
.pill-info { background: #ddf4ff; color: #0969da; border: 1px solid #54aeff; }
.okf-action-detail-text { font-size: 0.8rem; color: #656d76; margin-top: 0.25rem; }
.okf-filter-bar { display: flex; gap: 0.5rem; margin-bottom: 1rem; align-items: center; }
.okf-filter-btn { padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--rd-border); background: var(--rd-bg); cursor: pointer; font-size: 0.85rem; }
.okf-filter-btn.is-active { background: var(--rd-primary); color: #fff; border-color: var(--rd-primary); }
.okf-search-input { flex: 1; padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--rd-border); background: var(--rd-bg); color: var(--rd-fg); }
.okf-cta-row { display: flex; gap: 1rem; align-items: center; margin: 1.5rem 0; }
.okf-cta-btn { background: var(--rd-primary); color: #fff; padding: 0.6rem 1.2rem; border-radius: 6px; font-weight: 500; }
.okf-cta-btn:hover { text-decoration: none; opacity: 0.9; }
"#;

pub struct DevServer {
    pub url: String,
    pub title: String,
    stop: Arc<AtomicBool>,
    output: PathBuf,
    owns_output: bool,
    _watcher: Option<RecommendedWatcher>,
    _threads: Vec<JoinHandle<()>>,
}

impl DevServer {
    pub fn wait(&self) {
        while !self.stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(60));
        }
    }
}

impl Drop for DevServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if self.owns_output {
            let _ = fs::remove_dir_all(&self.output);
        }
    }
}

pub fn run_knowledge(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    profile: Profile,
) -> Result<DevServer> {
    let root = okf::absolute(root)?;
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let root = fs::canonicalize(&root)
        .with_context(|| format!("failed to resolve knowledge root {}", root.display()))?;
    let (output, owns_output) = match output {
        Some(path) => (okf::absolute(path)?, false),
        None => (okf::unique_temp("knowledge-run-out")?, true),
    };

    let hub = Arc::new(ReloadHub::new());
    let last_error = Arc::new(Mutex::new(None));
    let has_build = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));

    match rebuild_site(&root, &output, profile) {
        Ok(_) => {
            has_build.store(true, Ordering::Relaxed);
        }
        Err(err) => {
            eprintln!("rocci-okf: {err:#}");
            *last_error.lock().unwrap_or_else(|err| err.into_inner()) = Some(format!("{err:#}"));
        }
    }

    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("failed to bind 127.0.0.1:{port}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to set listener non-blocking")?;
    let bound = listener.local_addr()?.port();
    let url = format!("http://127.0.0.1:{bound}/");

    let server_stop = stop.clone();
    let server_hub = hub.clone();
    let server_output = output.clone();
    let server_error = last_error.clone();
    let server_has_build = has_build.clone();
    let server = thread::spawn(move || {
        serve_loop(
            listener,
            server_output,
            server_hub,
            server_error,
            server_has_build,
            server_stop,
        );
    });

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |result| {
        let _ = tx.send(result);
    })
    .context("failed to start file watcher")?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", root.display()))?;

    let watch_stop = stop.clone();
    let watch_root = root.clone();
    let watch_output = output.clone();
    let watch_hub = hub;
    let watch_error = last_error;
    let watch_ctx = WatchContext {
        root: watch_root,
        output: watch_output,
        profile,
        hub: watch_hub,
        last_error: watch_error,
        has_build,
        stop: watch_stop,
    };
    let watch = thread::spawn(move || {
        knowledge_watch_loop(rx, watch_ctx);
    });

    Ok(DevServer {
        url,
        title: "Knowledge".into(),
        stop,
        output,
        owns_output,
        _watcher: Some(watcher),
        _threads: vec![server, watch],
    })
}

struct ReloadHub {
    waiters: Mutex<Vec<mpsc::Sender<u64>>>,
    generation: AtomicU64,
}

impl ReloadHub {
    fn new() -> Self {
        Self {
            waiters: Mutex::new(Vec::new()),
            generation: AtomicU64::new(0),
        }
    }

    fn subscribe(&self) -> mpsc::Receiver<u64> {
        let (tx, rx) = mpsc::channel();
        self.waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push(tx);
        rx
    }

    fn broadcast(&self) {
        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        self.waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .retain(|tx| tx.send(generation).is_ok());
    }
}

struct WatchContext {
    root: PathBuf,
    output: PathBuf,
    profile: Profile,
    hub: Arc<ReloadHub>,
    last_error: Arc<Mutex<Option<String>>>,
    has_build: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

fn knowledge_watch_loop(rx: mpsc::Receiver<notify::Result<notify::Event>>, ctx: WatchContext) {
    loop {
        if ctx.stop.load(Ordering::Relaxed) {
            break;
        }
        let event = match rx.recv_timeout(DEBOUNCE) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let mut rebuild = knowledge_event_is_relevant(&event, &ctx.root, &ctx.output);
        loop {
            match rx.recv_timeout(DEBOUNCE) {
                Ok(next) => {
                    rebuild = rebuild || knowledge_event_is_relevant(&next, &ctx.root, &ctx.output);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        if !rebuild {
            continue;
        }
        match rebuild_site(&ctx.root, &ctx.output, ctx.profile) {
            Ok(_) => {
                ctx.has_build.store(true, Ordering::Relaxed);
                *ctx.last_error.lock().unwrap_or_else(|err| err.into_inner()) = None;
                ctx.hub.broadcast();
            }
            Err(err) => {
                eprintln!("rocci-okf: rebuild failed: {err:#}");
                if !ctx.has_build.load(Ordering::Relaxed) {
                    *ctx.last_error.lock().unwrap_or_else(|err| err.into_inner()) =
                        Some(format!("{err:#}"));
                    ctx.hub.broadcast();
                }
            }
        }
    }
}

fn rebuild_site(root: &Path, output: &Path, profile: Profile) -> Result<()> {
    let bundle = load(root, profile)?;
    if bundle.has_errors() {
        bail!("knowledge bundle has validation errors");
    }
    build_review_site(&bundle, output)?;
    Ok(())
}

fn knowledge_event_is_relevant(
    event: &notify::Result<notify::Event>,
    root: &Path,
    output: &Path,
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
        .any(|path| knowledge_path_is_relevant(path, root, output))
}

fn knowledge_path_is_relevant(path: &Path, root: &Path, output: &Path) -> bool {
    if path.starts_with(output)
        || path
            .components()
            .any(|component| component.as_os_str() == ".git")
    {
        return false;
    }
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    if relative.as_os_str().is_empty() {
        return false;
    }
    true
}

fn serve_loop(
    listener: TcpListener,
    output: PathBuf,
    hub: Arc<ReloadHub>,
    last_error: Arc<Mutex<Option<String>>>,
    has_build: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let output = output.clone();
                let hub = hub.clone();
                let last_error = last_error.clone();
                let has_build = has_build.clone();
                thread::spawn(move || {
                    let _ = handle_conn(stream, &output, &hub, &last_error, &has_build);
                });
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn handle_conn(
    mut stream: TcpStream,
    output: &Path,
    hub: &Arc<ReloadHub>,
    last_error: &Arc<Mutex<Option<String>>>,
    has_build: &Arc<AtomicBool>,
) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = [0u8; 4096];
    let count = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..count]);
    let mut lines = request.lines();
    let Some(first) = lines.next() else {
        return Ok(());
    };
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        return send_status(&mut stream, 405, "Method Not Allowed");
    }

    if path == "/__rocci_okf/events" {
        return stream_events(&mut stream, hub);
    }
    if path == "/__rocci_okf/reload.js" {
        return send_js(&mut stream, RELOAD_JS);
    }
    if path == "/__rocci_okf/app.css" {
        return send_css(&mut stream, DEFAULT_CSS);
    }

    if !has_build.load(Ordering::Relaxed) {
        let err = last_error.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let message = err.unwrap_or_else(|| "build failed".into());
        return send_html(
            &mut stream,
            500,
            &format!("<h1>Build Error</h1><pre>{}</pre>", message),
        );
    }

    let url_path = path.split('?').next().unwrap_or(path);
    let trimmed = url_path.trim_start_matches('/');
    let target = if trimmed.is_empty() {
        output.join("index.html")
    } else {
        let candidate = output.join(trimmed);
        if candidate.is_dir() {
            candidate.join("index.html")
        } else if candidate.exists() {
            candidate
        } else {
            output.join(format!("{trimmed}/index.html"))
        }
    };

    if target.is_file() {
        let bytes = fs::read(&target)?;
        let mime = mime_type(&target);
        send_bytes(&mut stream, 200, "OK", mime, &bytes)
    } else {
        send_status(&mut stream, 404, "Not Found")
    }
}

fn stream_events(stream: &mut TcpStream, hub: &Arc<ReloadHub>) -> Result<()> {
    let rx = hub.subscribe();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\n\r\n"
    )?;
    stream.flush()?;
    while rx.recv().is_ok() {
        if write!(stream, "event: reload\ndata: {}\n\n", 1).is_err() {
            break;
        }
        if stream.flush().is_err() {
            break;
        }
    }
    Ok(())
}

fn send_html(stream: &mut TcpStream, status: u16, body: &str) -> Result<()> {
    send_bytes(
        stream,
        status,
        "OK",
        "text/html; charset=utf-8",
        body.as_bytes(),
    )
}

fn send_js(stream: &mut TcpStream, body: &str) -> Result<()> {
    send_bytes(
        stream,
        200,
        "OK",
        "application/javascript; charset=utf-8",
        body.as_bytes(),
    )
}

fn send_css(stream: &mut TcpStream, body: &str) -> Result<()> {
    send_bytes(
        stream,
        200,
        "OK",
        "text/css; charset=utf-8",
        body.as_bytes(),
    )
}

fn send_status(stream: &mut TcpStream, status: u16, message: &str) -> Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status} {message}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )?;
    stream.flush()?;
    Ok(())
}

fn send_bytes(
    stream: &mut TcpStream,
    status: u16,
    status_text: &str,
    mime: &str,
    bytes: &[u8],
) -> Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    )?;
    stream.write_all(bytes)?;
    stream.flush()?;
    Ok(())
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
