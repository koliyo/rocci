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

use crate::build::{BuildSession, absolute, unique_temp};
use crate::config::{CONFIG_FILE, load_config};

const DEBOUNCE: Duration = Duration::from_millis(200);
const RELOAD_JS: &str = r#"(function () {
  function connect() {
    var es = new EventSource("/__rocs/events");
    es.addEventListener("reload", function () { location.reload(); });
    es.onerror = function () {
      es.close();
      setTimeout(connect, 1000);
    };
  }
  connect();
})();
"#;
const LIVE_RELOAD_TAG: &str = r#"<script src="/__rocs/reload.js" defer></script>"#;

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

pub fn run(root: &Path, output: Option<&Path>, port: u16) -> Result<DevServer> {
    let root = absolute(root)?;
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let (output, owns_output) = match output {
        Some(path) => (absolute(path)?, false),
        None => (unique_temp("run-out")?, true),
    };

    let title = load_config(&root)
        .map(|config| config.site.title)
        .unwrap_or_else(|_| "Documentation".into());
    let assets = load_config(&root)
        .map(|config| config.build.assets)
        .unwrap_or_else(|_| "assets".into());

    let hub = Arc::new(ReloadHub::new());
    let last_error = Arc::new(Mutex::new(None));
    let has_build = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));

    let mut session = BuildSession::create()?;
    match session.rebuild(&root, &output) {
        Ok(_) => {
            has_build.store(true, Ordering::Relaxed);
        }
        Err(err) => {
            eprintln!("rocs: {err:#}");
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
    let watch_has_build = has_build;
    let watch = thread::spawn(move || {
        watch_loop(
            rx,
            session,
            watch_root,
            watch_output,
            assets,
            watch_hub,
            watch_error,
            watch_has_build,
            watch_stop,
        );
    });

    Ok(DevServer {
        url,
        title,
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

fn watch_loop(
    rx: mpsc::Receiver<notify::Result<notify::Event>>,
    mut session: BuildSession,
    root: PathBuf,
    output: PathBuf,
    mut assets: String,
    hub: Arc<ReloadHub>,
    last_error: Arc<Mutex<Option<String>>>,
    has_build: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let event = match rx.recv_timeout(DEBOUNCE) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let mut rebuild = event_is_relevant(&event, &root, &output, &assets);
        loop {
            match rx.recv_timeout(DEBOUNCE) {
                Ok(next) => {
                    rebuild = rebuild || event_is_relevant(&next, &root, &output, &assets);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        if !rebuild {
            continue;
        }
        if let Ok(config) = load_config(&root) {
            assets = config.build.assets;
        }
        match session.rebuild(&root, &output) {
            Ok(_) => {
                has_build.store(true, Ordering::Relaxed);
                *last_error.lock().unwrap_or_else(|err| err.into_inner()) = None;
                hub.broadcast();
            }
            Err(err) => {
                eprintln!("rocs: rebuild failed: {err:#}");
                if !has_build.load(Ordering::Relaxed) {
                    *last_error.lock().unwrap_or_else(|err| err.into_inner()) =
                        Some(format!("{err:#}"));
                    hub.broadcast();
                }
            }
        }
    }
}

fn event_is_relevant(
    event: &notify::Result<notify::Event>,
    root: &Path,
    output: &Path,
    assets: &str,
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
        .any(|path| path_is_relevant(path, root, output, assets))
}

pub(crate) fn path_is_relevant(path: &Path, root: &Path, output: &Path, assets: &str) -> bool {
    if path.starts_with(output) {
        return false;
    }
    if path
        .components()
        .any(|component| component.as_os_str() == ".git")
    {
        return false;
    }
    if is_temp_name(path) {
        return false;
    }
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    if relative.as_os_str().is_empty() {
        return false;
    }
    if path.file_name().is_some_and(|name| name == CONFIG_FILE) {
        return true;
    }
    if path.extension().is_some_and(|ext| ext == "rocdown") {
        return true;
    }
    relative.starts_with(assets)
}

fn is_temp_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name == ".DS_Store"
        || name.ends_with('~')
        || name.ends_with(".tmp")
        || name.starts_with('.')
            && (name.ends_with(".swp") || name.ends_with(".swx") || name.ends_with(".swo"))
        || name.starts_with('#') && name.ends_with('#')
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
                    let _ = handle_client(stream, &output, &hub, &last_error, &has_build);
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
    match resolve_request(output, path) {
        ServeTarget::ReloadJs => write_response(
            &mut stream,
            200,
            "text/javascript; charset=utf-8",
            false,
            RELOAD_JS.as_bytes(),
        ),
        ServeTarget::Events => write_sse(&mut stream, hub),
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
                write_error_html(&mut stream, "no built site yet")
            }
        }
    }
}

fn request_path(request: &str) -> Option<&str> {
    let mut lines = request.split("\r\n");
    let first = lines.next()?;
    let mut parts = first.split_whitespace();
    let _method = parts.next()?;
    parts.next()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServeTarget {
    ReloadJs,
    Events,
    Redirect(String),
    File { relative: String },
    NotFound,
}

pub(crate) fn resolve_request(output: &Path, url_path: &str) -> ServeTarget {
    let path = url_path.split(['?', '#']).next().unwrap_or(url_path);
    let path = if path.is_empty() { "/" } else { path };
    if path == "/__rocs/reload.js" {
        return ServeTarget::ReloadJs;
    }
    if path == "/__rocs/events" {
        return ServeTarget::Events;
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

pub(crate) fn inject_live_reload(html: &str) -> String {
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
    let escaped = message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Rocs build failed</title></head><body><h1>Build failed</h1><pre>{escaped}</pre></body></html>"
    )
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    html: bool,
    body: &[u8],
) -> io::Result<()> {
    let reason = match status {
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let cache = if html {
        "Cache-Control: no-store\r\n"
    } else {
        ""
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n{cache}Connection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn write_redirect(stream: &mut TcpStream, location: &str) -> io::Result<()> {
    let header = format!(
        "HTTP/1.1 308 Permanent Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(header.as_bytes())
}

fn write_sse(stream: &mut TcpStream, hub: &ReloadHub) -> io::Result<()> {
    stream.set_read_timeout(None)?;
    stream.set_write_timeout(Some(Duration::from_secs(60)))?;
    let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: keep-alive\r\n\r\nretry: 1000\r\n\r\n";
    stream.write_all(header.as_bytes())?;
    stream.flush()?;
    let rx = hub.subscribe();
    loop {
        match rx.recv_timeout(Duration::from_secs(15)) {
            Ok(_) => {
                stream.write_all(b"event: reload\ndata: {}\n\n")?;
                stream.flush()?;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                stream.write_all(b": keepalive\n\n")?;
                stream.flush()?;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        unique_temp(name).unwrap()
    }

    fn write_tree(root: &Path) {
        fs::create_dir_all(root.join("guide")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(
            root.join("index.html"),
            "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"default-src &#39;none&#39;; script-src &#39;none&#39;; connect-src &#39;none&#39;\"></head><body>home</body></html>",
        )
        .unwrap();
        fs::write(
            root.join("guide/index.html"),
            "<html><body>guide</body></html>",
        )
        .unwrap();
        fs::write(root.join("404.html"), "<html><body>missing</body></html>").unwrap();
        fs::write(root.join("assets/theme.css"), "body{color:black}").unwrap();
    }

    #[test]
    fn resolve_maps_indexes_redirects_and_reserved_routes() {
        let root = temp("serve-map");
        write_tree(&root);
        assert_eq!(
            resolve_request(&root, "/"),
            ServeTarget::File {
                relative: "index.html".into()
            }
        );
        assert_eq!(
            resolve_request(&root, "/guide/"),
            ServeTarget::File {
                relative: "guide/index.html".into()
            }
        );
        assert_eq!(
            resolve_request(&root, "/guide"),
            ServeTarget::Redirect("/guide/".into())
        );
        assert_eq!(
            resolve_request(&root, "/assets/theme.css"),
            ServeTarget::File {
                relative: "assets/theme.css".into()
            }
        );
        assert_eq!(resolve_request(&root, "/missing"), ServeTarget::NotFound);
        assert_eq!(
            resolve_request(&root, "/__rocs/reload.js"),
            ServeTarget::ReloadJs
        );
        assert_eq!(
            resolve_request(&root, "/__rocs/events"),
            ServeTarget::Events
        );
        assert_eq!(resolve_request(&root, "/../secret"), ServeTarget::NotFound);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn inject_rewrites_csp_and_inserts_script() {
        let html = "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'none'; connect-src 'none'\"></head><body>hi</body></html>";
        let injected = inject_live_reload(html);
        assert!(injected.contains("script-src 'self'"));
        assert!(injected.contains("connect-src 'self'"));
        assert!(!injected.contains("script-src 'none'"));
        assert!(injected.contains("/__rocs/reload.js"));
        assert!(injected.contains("</body>"));
        assert!(!html.contains("/__rocs/reload.js"));
    }

    #[test]
    fn inject_rewrites_html_escaped_csp_quotes() {
        let html = "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"default-src &#39;none&#39;; script-src &#39;none&#39;; connect-src &#39;none&#39;\"></head><body>hi</body></html>";
        let injected = inject_live_reload(html);
        assert!(injected.contains("script-src &#39;self&#39;"));
        assert!(injected.contains("connect-src &#39;self&#39;"));
        assert!(!injected.contains("script-src &#39;none&#39;"));
        assert!(injected.contains("default-src &#39;none&#39;"));
        assert!(injected.contains("/__rocs/reload.js"));
    }

    #[test]
    fn path_filter_keeps_content_and_ignores_noise() {
        let root = PathBuf::from("/docs");
        let output = PathBuf::from("/docs/dist");
        assert!(path_is_relevant(
            Path::new("/docs/index.rocdown"),
            &root,
            &output,
            "assets"
        ));
        assert!(path_is_relevant(
            Path::new("/docs/rocs.toml"),
            &root,
            &output,
            "assets"
        ));
        assert!(path_is_relevant(
            Path::new("/docs/assets/og.png"),
            &root,
            &output,
            "assets"
        ));
        assert!(!path_is_relevant(
            Path::new("/docs/dist/index.html"),
            &root,
            &output,
            "assets"
        ));
        assert!(!path_is_relevant(
            Path::new("/docs/.git/HEAD"),
            &root,
            &output,
            "assets"
        ));
        assert!(!path_is_relevant(
            Path::new("/docs/index.rocdown~"),
            &root,
            &output,
            "assets"
        ));
        assert!(!path_is_relevant(
            Path::new("/docs/notes.txt"),
            &root,
            &output,
            "assets"
        ));
    }

    #[test]
    fn html_response_injects_reload_and_css_does_not() {
        let root = temp("serve-http");
        write_tree(&root);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let stop = Arc::new(AtomicBool::new(false));
        let hub = Arc::new(ReloadHub::new());
        let error = Arc::new(Mutex::new(None));
        let has_build = Arc::new(AtomicBool::new(true));
        listener.set_nonblocking(true).unwrap();
        let output = root.clone();
        let server_stop = stop.clone();
        let server = thread::spawn(move || {
            serve_loop(listener, output, hub, error, has_build, server_stop);
        });

        let html = http_get(port, "/");
        assert!(html.contains("200 OK"));
        assert!(html.contains("script-src &#39;self&#39;"));
        assert!(!html.contains("script-src &#39;none&#39;"));
        assert!(html.contains("/__rocs/reload.js"));
        assert!(html.contains("Cache-Control: no-store"));

        let redirect = http_get(port, "/guide");
        assert!(redirect.contains("308"));
        assert!(redirect.contains("Location: /guide/"));

        let css = http_get(port, "/assets/theme.css");
        assert!(css.contains("200 OK"));
        assert!(css.contains("body{color:black}"));
        assert!(!css.contains("/__rocs/reload.js"));

        let missing = http_get(port, "/nope");
        assert!(missing.contains("404"));
        assert!(missing.contains("missing"));
        assert!(missing.contains("/__rocs/reload.js"));

        let js = http_get(port, "/__rocs/reload.js");
        assert!(js.contains("EventSource"));

        stop.store(true, Ordering::Relaxed);
        let _ = server.join();
        let _ = fs::remove_dir_all(root);
    }

    fn http_get(port: u16, path: &str) -> String {
        let mut last = String::new();
        for _ in 0..50 {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(mut client) => {
                    if client
                        .write_all(
                            format!(
                                "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
                            )
                            .as_bytes(),
                        )
                        .is_err()
                    {
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    let _ = client.set_read_timeout(Some(Duration::from_secs(2)));
                    let mut buf = String::new();
                    let _ = client.read_to_string(&mut buf);
                    if !buf.is_empty() {
                        return buf;
                    }
                    last = buf;
                }
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
        last
    }
}
