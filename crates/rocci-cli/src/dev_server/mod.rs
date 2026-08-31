use std::{
    fs,
    io::{self, Read},
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
use crate::logs::{self, LogHub, LogLevel};
use crate::profile::ProfileSnapshot;

const DEBOUNCE: Duration = Duration::from_millis(200);

pub type PathFilter = Arc<dyn Fn(&Path) -> bool + Send + Sync>;
pub type ExtraHttpHandler =
    Arc<dyn Fn(&str, &str, &[u8]) -> Option<(u16, &'static str, Vec<u8>)> + Send + Sync>;

const RELOAD_JS: &str = r#"(function () {
  if (window.__rocciLiveReload) {
    return;
  }
  var KEY = "rocci-live-reload";
  var es = null;
  var dirty = false;
  function seedFromQuery() {
    try {
      if (new URLSearchParams(window.location.search).get("reload") === "0") {
        sessionStorage.setItem(KEY, "0");
      }
    } catch (err) {}
  }
  function enabled() {
    try {
      return sessionStorage.getItem(KEY) !== "0";
    } catch (err) {
      return true;
    }
  }
  function setEnabled(on) {
    try {
      sessionStorage.setItem(KEY, on ? "1" : "0");
    } catch (err) {}
    if (on && dirty) {
      location.reload();
    }
  }
  function connect() {
    if (es) {
      return;
    }
    es = new EventSource("/__rocci/events");
    es.addEventListener("reload", function () {
      if (enabled()) {
        location.reload();
      } else {
        dirty = true;
      }
    });
    es.onerror = function () {
      if (es) {
        es.close();
      }
      es = null;
      setTimeout(connect, 1000);
    };
  }
  window.__rocciLiveReload = { enabled: enabled, set: setEnabled };
  seedFromQuery();
  connect();
})();
"#;

const LIVE_RELOAD_TAG: &str = r#"<script src="/__rocci/reload.js" defer></script>"#;
/// HTTP CSP for HTML responses that inject live-reload. Intersects with any
/// page meta CSP; must keep `'unsafe-eval'` so Datastar can compile expressions.
const PREVIEW_HTML_CSP: &str = "Content-Security-Policy: default-src 'self' 'unsafe-inline' 'unsafe-eval'; connect-src 'self'; frame-src 'self';\r\n";

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
    pub log_handlers: bool,
    pub on_stop: Option<Arc<dyn Fn() + Send + Sync>>,
    pub public: bool,
    pub extra_http: Option<ExtraHttpHandler>,
}

pub fn serve_static_site<F>(config: StaticDevServerConfig, mut rebuild: F) -> Result<DevServer>
where
    F: FnMut(&Path, Arc<LogHub>) -> Result<Option<InspectSnapshot>> + Send + 'static,
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
    logs::tee(
        &hub.logs,
        LogLevel::Info,
        format!(
            "{}: preview files at {}",
            config.log_prefix,
            output.display()
        ),
    );
    let last_error = Arc::new(Mutex::new(None));
    let has_build = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));

    match rebuild(&output, hub.logs.clone()) {
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

    let host = crate::serve::bind_host(config.public);
    let listener = TcpListener::bind((host, config.port))
        .with_context(|| format!("failed to bind {host}:{}", config.port))?;
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
    crate::serve::note_public_listen(config.public, bound);

    let server_stop = stop.clone();
    let server_hub = hub.clone();
    let server_output = output.clone();
    let server_error = last_error.clone();
    let server_has_build = has_build.clone();
    let server_backend = config.backend_port.clone();
    let server_log_handlers = config.log_handlers;
    let server_extra_http = config.extra_http.clone();
    let server = thread::spawn(move || {
        serve_loop(ServeLoop {
            listener,
            output: server_output,
            hub: server_hub,
            last_error: server_error,
            has_build: server_has_build,
            backend_port: server_backend,
            log_handlers: server_log_handlers,
            extra_http: server_extra_http,
            stop: server_stop,
        });
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
    live_reload: bool,
    state_key: Option<String>,
    rebuild: F,
) -> Result<()>
where
    F: FnMut(&Path, Arc<LogHub>) -> Result<Option<InspectSnapshot>> + Send + 'static,
{
    let prefix = config.log_prefix.clone();
    let title = config.title.clone();
    let server = serve_static_site(config, rebuild)?;
    logs::tee(
        &server.logs,
        LogLevel::Info,
        format!("{prefix}: serving {title} at {}", server.url),
    );
    crate::serve::emit_preview_ready(&server.url);
    crate::serve::emit_inspector_ready(&server.inspector_url);
    crate::serve::note_live_reload_paused(live_reload);
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
        live_reload,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

pub struct PublishedTreeConfig {
    pub title: String,
    pub port: u16,
    pub dist: PathBuf,
    pub open_path: String,
    pub log_prefix: String,
    pub public: bool,
}

pub struct PublishedServer {
    pub url: String,
    pub title: String,
    pub logs: Arc<LogHub>,
    stop: Arc<AtomicBool>,
    _thread: JoinHandle<()>,
}

impl PublishedServer {
    pub fn wait(&self) {
        while !self.stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for PublishedServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

pub fn serve_published_tree(config: PublishedTreeConfig) -> Result<PublishedServer> {
    let dist = if config.dist.is_absolute() {
        config.dist
    } else {
        std::env::current_dir()?.join(config.dist)
    };
    if !dist.join("index.html").is_file() {
        anyhow::bail!(
            "`{}` is not a built site tree (missing index.html)",
            dist.display()
        );
    }
    let hub = Arc::new(ReloadHub::new());
    logs::tee(
        &hub.logs,
        LogLevel::Info,
        format!("{}: serving files at {}", config.log_prefix, dist.display()),
    );
    let stop = Arc::new(AtomicBool::new(false));
    let host = crate::serve::bind_host(config.public);
    let listener = TcpListener::bind((host, config.port))
        .with_context(|| format!("failed to bind {host}:{}", config.port))?;
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
    crate::serve::note_public_listen(config.public, bound);

    let server_stop = stop.clone();
    let server_dist = dist;
    let server = thread::spawn(move || {
        serve_published_loop(listener, server_dist, server_stop);
    });

    Ok(PublishedServer {
        url,
        title: config.title,
        logs: hub.logs.clone(),
        stop,
        _thread: server,
    })
}

pub fn preview_published_tree(
    config: PublishedTreeConfig,
    no_window: bool,
    state_key: Option<String>,
) -> Result<()> {
    let prefix = config.log_prefix.clone();
    let title = config.title.clone();
    let server = serve_published_tree(config)?;
    logs::tee(
        &server.logs,
        LogLevel::Info,
        format!("{prefix}: serving {title} at {}", server.url),
    );
    crate::serve::emit_preview_ready(&server.url);
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
        live_reload: false,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

fn serve_published_loop(listener: TcpListener, dist: PathBuf, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let dist = dist.clone();
                thread::spawn(move || {
                    let _ = handle_published_client(stream, &dist);
                });
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn handle_published_client(mut stream: TcpStream, dist: &Path) -> io::Result<()> {
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
    match resolve_published_request(dist, path) {
        ServeTarget::Redirect(location) => write_redirect(&mut stream, &location),
        ServeTarget::File { relative } => serve_published_file(&mut stream, dist, &relative, 200),
        ServeTarget::NotFound => {
            if dist.join("404.html").is_file() {
                serve_published_file(&mut stream, dist, "404.html", 404)
            } else {
                write_response(
                    &mut stream,
                    404,
                    "text/plain; charset=utf-8",
                    false,
                    b"not found",
                )
            }
        }
        _ => write_response(
            &mut stream,
            404,
            "text/plain; charset=utf-8",
            false,
            b"not found",
        ),
    }
}

fn serve_published_file(
    stream: &mut TcpStream,
    output: &Path,
    relative: &str,
    status: u16,
) -> io::Result<()> {
    let path = output.join(relative);
    let bytes = fs::read(&path)?;
    let mime = mime_type(&path);
    write_response(stream, status, mime, false, &bytes)
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
    F: FnMut(&Path, Arc<LogHub>) -> Result<Option<InspectSnapshot>> + Send + 'static,
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
        match rebuild(&output, ctl.hub.logs.clone()) {
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

mod http;
mod routing;
use http::{ServeLoop, request_path, serve_loop};
use routing::{mime_type, write_redirect, write_response};

pub use routing::{ServeTarget, inject_live_reload, resolve_published_request, resolve_request};

#[cfg(test)]
mod tests;
