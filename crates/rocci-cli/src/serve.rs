use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use clap::Args;
use rocci_desktop::{PreviewOptions, preview};

use crate::logs::{LogHub, LogLevel, LogLine, Progress};
use crate::style;

const SERVER_WAIT: Duration = Duration::from_secs(120);
const LISTEN_HEARTBEAT: Duration = Duration::from_secs(2);

pub fn wait_listen_starting(port: u16) -> String {
    format!("waiting for roc on :{port}")
}

pub fn wait_listen_heartbeat(port: u16, elapsed: Duration) -> String {
    format!("waiting for roc on :{port} ({}s)", elapsed.as_secs())
}

pub fn wait_listen_timeout_message(port: u16, elapsed: Duration, still_running: bool) -> String {
    format!(
        "timed out waiting for roc server on port {port} after {}s (process still running: {still_running})",
        elapsed.as_secs()
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortArg {
    Auto,
    Exact(u16),
}

impl PortArg {
    pub fn resolve(self) -> Result<u16> {
        match self {
            Self::Auto => free_port(),
            Self::Exact(port) => {
                if port_in_use(port) {
                    bail!("port {port} is already in use; pass --port auto or choose another port");
                }
                Ok(port)
            }
        }
    }
}

#[derive(Args, Clone, Copy, Debug)]
pub struct ServeOptions {
    /// Skip the preview window; print the URL and keep the Roc server.
    /// Open that URL with `?reload=0` to pause automatic page refresh.
    #[arg(long)]
    pub no_window: bool,

    /// Pause automatic page refresh. Watch and rebuild still run.
    #[arg(long)]
    pub no_live_reload: bool,

    /// Log each matched `@on` handler to stderr (CLI and Dev Console).
    #[arg(long)]
    pub log_handlers: bool,

    /// Print compile, inspect, and wait phases to stderr.
    #[arg(short, long)]
    pub verbose: bool,

    /// TCP port to listen on. Defaults to a free port with the preview window,
    /// or 8000 with `--no-window`. Pass `auto` to pick a free port.
    #[arg(
        long,
        default_value = "auto",
        default_value_if("no_window", "true", "8000"),
        value_name = "PORT",
        value_parser = parse_port_arg,
        env = "ROC_BASIC_WEBSERVER_PORT"
    )]
    pub port: PortArg,
}

impl ServeOptions {
    pub fn live_reload(self) -> bool {
        !self.no_live_reload
    }
}

pub fn note_live_reload_paused(live_reload: bool) {
    if !live_reload {
        eprintln!(
            "{}",
            crate::style::note("live reload paused; watch/rebuild still runs")
        );
        eprintln!(
            "{}",
            crate::style::note("in a browser, open the URL with ?reload=0")
        );
    }
}

pub fn parse_port_arg(value: &str) -> Result<PortArg, String> {
    if value.eq_ignore_ascii_case("auto") {
        return Ok(PortArg::Auto);
    }
    match value.parse::<u16>() {
        Ok(0) => Err("port 0 is invalid; pass --port auto to pick a free port".into()),
        Ok(port) => Ok(PortArg::Exact(port)),
        Err(_) => Err(format!(
            "invalid port `{value}`; expected a number 1-65535 or `auto`"
        )),
    }
}

pub fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("failed to allocate a local port")?;
    Ok(listener.local_addr()?.port())
}

fn port_in_use(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).is_ok()
}

pub enum ListenWait {
    Ready,
    Exited(ExitStatus),
}

pub struct StderrTee {
    buf: Arc<Mutex<String>>,
    feed: Option<Arc<StderrHubFeed>>,
    thread: Option<JoinHandle<()>>,
}

struct StderrHubFeed {
    hub: Arc<LogHub>,
    state: Mutex<StderrFeedState>,
}

struct StderrFeedState {
    offset: usize,
    remainder: String,
}

impl StderrTee {
    pub fn snapshot(&self) -> String {
        self.buf
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    pub fn flush_to_hub(&self) {
        if let Some(feed) = &self.feed {
            feed.drain(&self.snapshot());
        }
    }

    pub fn finish(&mut self) -> String {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.snapshot()
    }
}

impl StderrHubFeed {
    fn drain(&self, buf: &str) {
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        if state.offset > buf.len() {
            state.offset = buf.len();
        }
        if state.offset < buf.len() {
            let offset = state.offset;
            state.remainder.push_str(&buf[offset..]);
            state.offset = buf.len();
        }
        loop {
            let Some(idx) = state.remainder.find('\n') else {
                break;
            };
            let mut line: String = state.remainder[..idx].to_string();
            state.remainder.replace_range(..=idx, "");
            if line.ends_with('\r') {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }
            self.hub
                .push_line(LogLine::runtime(level_for_stderr_line(&line), line));
        }
    }

    fn finish(&self, buf: &str) {
        self.drain(buf);
        let leftover = {
            let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
            let leftover = state.remainder.trim_end_matches('\r').to_string();
            state.remainder.clear();
            leftover
        };
        if leftover.is_empty() {
            return;
        }
        self.hub
            .push_line(LogLine::runtime(level_for_stderr_line(&leftover), leftover));
    }
}

pub fn stderr_log_lines(text: &str) -> Vec<LogLine> {
    text.split('\n')
        .filter_map(|raw| {
            let line = raw.trim_end_matches('\r');
            if line.is_empty() {
                None
            } else {
                Some(LogLine::runtime(
                    level_for_stderr_line(line),
                    line.to_string(),
                ))
            }
        })
        .collect()
}

fn level_for_stderr_line(line: &str) -> LogLevel {
    let line = style::strip_ansi(line);
    if roc_output_is_failure(&line) {
        LogLevel::Error
    } else if line.ends_with(" -> err") || line.contains(" -> proxy error:") {
        LogLevel::Error
    } else if line.to_ascii_lowercase().contains("warning")
        || line.ends_with(" -> island unavailable")
    {
        LogLevel::Warn
    } else {
        LogLevel::Info
    }
}

pub fn spawn_roc(cmd: Command) -> Result<(Child, StderrTee)> {
    spawn_roc_with_logs(cmd, None)
}

pub fn spawn_roc_with_logs(
    mut cmd: Command,
    logs: Option<Arc<LogHub>>,
) -> Result<(Child, StderrTee)> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd
        .spawn()
        .context("failed to start `roc`; is it on PATH?")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture roc stderr")?;
    let buf = Arc::new(Mutex::new(String::new()));
    let feed = logs.map(|hub| {
        Arc::new(StderrHubFeed {
            hub,
            state: Mutex::new(StderrFeedState {
                offset: 0,
                remainder: String::new(),
            }),
        })
    });
    let thread_buf = buf.clone();
    let thread_feed = feed.clone();
    let thread = thread::spawn(move || {
        let mut reader = io::BufReader::new(stderr);
        let mut chunk = [0u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => {
                    if let Some(feed) = &thread_feed {
                        let snap = thread_buf
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .clone();
                        feed.finish(&snap);
                    }
                    break;
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&chunk[..n]);
                    {
                        let mut buf = thread_buf.lock().unwrap_or_else(|err| err.into_inner());
                        buf.push_str(&text);
                        if let Some(feed) = &thread_feed {
                            feed.drain(&buf);
                        }
                    }
                    eprint!("{text}");
                    let _ = io::stderr().flush();
                }
                Err(_) => break,
            }
        }
    });
    Ok((
        child,
        StderrTee {
            buf,
            feed,
            thread: Some(thread),
        },
    ))
}

pub fn wait_for_listen(child: &mut Child, port: u16, progress: Progress) -> Result<ListenWait> {
    let start = Instant::now();
    let mut last_beat = start;
    progress.detail(wait_listen_starting(port));
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(ListenWait::Exited(status)),
            Ok(None) => {}
            Err(err) => bail!("failed to poll roc: {err}"),
        }
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(ListenWait::Ready);
        }
        let elapsed = start.elapsed();
        if elapsed > SERVER_WAIT {
            let still_running = matches!(child.try_wait(), Ok(None));
            bail!(
                "{}",
                wait_listen_timeout_message(port, elapsed, still_running)
            );
        }
        if last_beat.elapsed() >= LISTEN_HEARTBEAT {
            progress.detail(wait_listen_heartbeat(port, elapsed));
            last_beat = Instant::now();
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn wait_for_server(child: &mut Child, port: u16, progress: Progress) -> Result<()> {
    match wait_for_listen(child, port, progress)? {
        ListenWait::Ready => Ok(()),
        ListenWait::Exited(status) => bail!("roc exited before serving ({status})"),
    }
}

pub enum RocStart {
    Ready,
    Failed(String),
}

pub fn roc_output_is_failure(output: &str) -> bool {
    if output.contains("[ROC CRASHED]") {
        return true;
    }
    found_error_count(output).is_some_and(|count| count > 0)
}

fn found_error_count(output: &str) -> Option<u32> {
    let mut rest = output;
    while let Some(idx) = rest.find("Found ") {
        let after = &rest[idx + 6..];
        let digits = after.chars().take_while(|ch| ch.is_ascii_digit()).count();
        if digits > 0 {
            let count = after[..digits].parse().ok()?;
            if after[digits..].starts_with(" error") {
                return Some(count);
            }
        }
        rest = after.get(1..).unwrap_or("");
        if rest.is_empty() {
            break;
        }
    }
    None
}

pub fn wait_for_roc(
    child: &mut Child,
    tee: &mut StderrTee,
    port: u16,
    probe_path: &str,
    progress: Progress,
) -> Result<RocStart> {
    match wait_for_listen(child, port, progress)? {
        ListenWait::Exited(_) => Ok(RocStart::Failed(tee.finish())),
        ListenWait::Ready => {
            let output = wait_for_roc_diagnostics(tee);
            if roc_output_is_failure(&output) {
                return Ok(RocStart::Failed(stop_roc(child, tee, port)));
            }
            let path = normalize_probe_path(probe_path);
            let _ = probe_http(port, &path);
            thread::sleep(Duration::from_millis(50));
            if matches!(child.try_wait(), Ok(Some(_))) || roc_output_is_failure(&tee.snapshot()) {
                return Ok(RocStart::Failed(stop_roc(child, tee, port)));
            }
            Ok(RocStart::Ready)
        }
    }
}

fn wait_for_roc_diagnostics(tee: &StderrTee) -> String {
    let start = Instant::now();
    loop {
        let snap = tee.snapshot();
        if roc_output_is_failure(&snap)
            || snap.contains("Found 0 error")
            || snap.contains("Listening on")
        {
            return snap;
        }
        if start.elapsed() > Duration::from_millis(750) {
            return snap;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn stop_roc(child: &mut Child, tee: &mut StderrTee, port: u16) -> String {
    stop_child(child);
    wait_port_free(port, Duration::from_secs(2));
    tee.finish()
}

pub fn wait_port_free(port: u16, budget: Duration) {
    let start = Instant::now();
    while port_in_use(port) && start.elapsed() < budget {
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn stop_child(child: &mut Child) {
    #[cfg(unix)]
    kill_process_group(child.id());
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    unsafe extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    let _ = unsafe { kill(-(pid as i32), 9) };
}

fn normalize_probe_path(path: &str) -> String {
    if path.is_empty() {
        "/".to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn probe_http(port: u16, path: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(2),
    )?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;
    let mut buf = [0u8; 256];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "empty HTTP response",
        ));
    }
    Ok(())
}

pub fn serve_html(
    port: u16,
    status: u16,
    html: &str,
    title: &str,
    no_window: bool,
    live_reload: bool,
) -> Result<()> {
    let url = format!("http://127.0.0.1:{port}/");
    let html = html.to_string();
    let stop = Arc::new(AtomicBool::new(false));
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("failed to bind 127.0.0.1:{port}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to set error-page listener non-blocking")?;

    let stop_flag = stop.clone();
    let html_clone = html.clone();
    let thread = thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = write_html_response(stream, status, &html_clone);
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });

    println!("{}", style::serving(title, &url));
    note_live_reload_paused(live_reload);
    if no_window {
        let _ = thread.join();
        return Ok(());
    }
    let preview = open_preview(&url, title, live_reload);
    stop.store(true, Ordering::Relaxed);
    let _ = thread.join();
    preview
}

fn write_html_response(mut stream: TcpStream, status: u16, html: &str) -> io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buf = [0u8; 8192];
    let _ = stream.read(&mut buf);
    let reason = match status {
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        html.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(html.as_bytes())?;
    Ok(())
}

pub fn open_preview(url: &str, title: &str, live_reload: bool) -> Result<()> {
    preview(PreviewOptions {
        url: url.to_string(),
        title: title.to_string(),
        state_key: Some("rocci:view".to_string()),
        live_reload,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"))
}

pub fn with_window(
    child: &mut Child,
    url: &str,
    title: &str,
    no_window: bool,
    live_reload: bool,
) -> Result<()> {
    with_window_and_inspector(child, url, title, no_window, live_reload, None, None, None)
}

#[allow(clippy::too_many_arguments)]
pub fn with_window_and_inspector(
    child: &mut Child,
    url: &str,
    title: &str,
    no_window: bool,
    live_reload: bool,
    inspect: Option<crate::inspect::InspectSnapshot>,
    state_key: Option<String>,
    logs: Option<Arc<LogHub>>,
) -> Result<()> {
    note_live_reload_paused(live_reload);
    let inspector = match inspect {
        Some(snapshot) => Some(crate::inspector::InspectorServer::spawn_with_logs(
            snapshot,
            logs.unwrap_or_else(|| Arc::new(LogHub::new())),
        )?),
        None => None,
    };
    if no_window {
        let status = child.wait().context("roc server exited unexpectedly")?;
        drop(inspector);
        if !status.success() {
            bail!("roc exited with {status}");
        }
        return Ok(());
    }

    let preview_result = preview(PreviewOptions {
        url: url.to_string(),
        title: title.to_string(),
        inspector_url: inspector.as_ref().map(|server| server.url.clone()),
        state_key: Some(state_key.unwrap_or_else(|| "rocci:view".to_string())),
        live_reload,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(inspector);
    stop_child(child);
    preview_result
}

/// Roc helper: basic-webserver 0.16 binds `Server.Config.listen`, not the env var.
/// `rocci --port` still sets `ROC_BASIC_WEBSERVER_PORT` on the child.
pub const ROC_LISTEN_PORT_HELPER: &str = r#"
listen_port! : {} => U16
listen_port! = |_| {
    match Env.var_str!("ROC_BASIC_WEBSERVER_PORT") {
        Ok(value) =>
            match U16.from_str(value) {
                Ok(0) => 8000
                Ok(port) => port
                Err(_) => 8000
            }
        Err(_) => 8000
    }
}
"#;

/// Bind host for generated `with_listen`. Defaults to loopback. Set
/// `ROC_BASIC_WEBSERVER_HOST=0.0.0.0` so another container can reverse-proxy.
pub const ROC_LISTEN_HOST_HELPER: &str = r#"
listen_host! : {} => Str
listen_host! = |_| {
    match Env.var_str!("ROC_BASIC_WEBSERVER_HOST") {
        Ok("") => "127.0.0.1"
        Ok(value) => value
        Err(_) => "127.0.0.1"
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct ServeCli {
        #[command(flatten)]
        serve: ServeOptions,
    }

    #[test]
    fn parses_auto_port() {
        assert_eq!(parse_port_arg("auto").unwrap(), PortArg::Auto);
        assert_eq!(parse_port_arg("AUTO").unwrap(), PortArg::Auto);
    }

    #[test]
    fn parses_explicit_port() {
        assert_eq!(parse_port_arg("9001").unwrap(), PortArg::Exact(9001));
    }

    #[test]
    fn rejects_invalid_port() {
        let err = parse_port_arg("nope").unwrap_err();
        assert!(err.contains("invalid port `nope`"));
        let err = parse_port_arg("0").unwrap_err();
        assert!(err.contains("port 0 is invalid"));
    }

    #[test]
    fn clap_defaults_to_auto_with_window() {
        if std::env::var_os("ROC_BASIC_WEBSERVER_PORT").is_some() {
            return;
        }
        let cli = ServeCli::try_parse_from(["rocci"]).unwrap();
        assert!(!cli.serve.no_window);
        assert!(!cli.serve.no_live_reload);
        assert_eq!(cli.serve.port, PortArg::Auto);
    }

    #[test]
    fn clap_defaults_to_8000_without_window() {
        if std::env::var_os("ROC_BASIC_WEBSERVER_PORT").is_some() {
            return;
        }
        let cli = ServeCli::try_parse_from(["rocci", "--no-window"]).unwrap();
        assert!(cli.serve.no_window);
        assert_eq!(cli.serve.port, PortArg::Exact(8000));
    }

    #[test]
    fn clap_accepts_no_live_reload() {
        let cli = ServeCli::try_parse_from(["rocci", "--no-live-reload"]).unwrap();
        assert!(cli.serve.no_live_reload);
        assert!(!cli.serve.live_reload());
    }

    #[test]
    fn clap_accepts_log_handlers() {
        let cli = ServeCli::try_parse_from(["rocci", "--log-handlers"]).unwrap();
        assert!(cli.serve.log_handlers);
        let cli = ServeCli::try_parse_from(["rocci"]).unwrap();
        assert!(!cli.serve.log_handlers);
    }

    #[test]
    fn clap_accepts_verbose() {
        let cli = ServeCli::try_parse_from(["rocci", "--verbose"]).unwrap();
        assert!(cli.serve.verbose);
        let cli = ServeCli::try_parse_from(["rocci", "-v"]).unwrap();
        assert!(cli.serve.verbose);
        let cli = ServeCli::try_parse_from(["rocci"]).unwrap();
        assert!(!cli.serve.verbose);
    }

    #[test]
    fn clap_accepts_port_auto() {
        let cli = ServeCli::try_parse_from(["rocci", "--port", "auto"]).unwrap();
        assert_eq!(cli.serve.port, PortArg::Auto);
    }

    #[test]
    fn clap_accepts_numeric_port() {
        let cli = ServeCli::try_parse_from(["rocci", "--port", "9001"]).unwrap();
        assert_eq!(cli.serve.port, PortArg::Exact(9001));
    }

    #[test]
    fn no_window_help_mentions_reload_query() {
        use clap::CommandFactory;
        let cmd = ServeCli::command();
        let arg = cmd
            .get_arguments()
            .find(|arg| arg.get_long() == Some("no-window"))
            .expect("no-window");
        let help = format!(
            "{}{}",
            arg.get_help().map(|h| h.to_string()).unwrap_or_default(),
            arg.get_long_help()
                .map(|h| h.to_string())
                .unwrap_or_default()
        );
        assert!(help.contains("?reload=0"), "{help}");
    }

    #[test]
    fn free_port_is_bindable() {
        let port = free_port().unwrap();
        assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
    }

    #[test]
    fn exact_port_fails_when_in_use() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let err = PortArg::Exact(port).resolve().unwrap_err().to_string();
        assert!(err.contains("already in use"));
    }

    #[test]
    fn auto_port_resolves_to_a_free_port() {
        let port = PortArg::Auto.resolve().unwrap();
        assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
    }

    #[test]
    fn html_error_response_includes_status_and_body() {
        use std::io::{Read, Write};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            write_html_response(stream, 500, "<html>boom</html>").unwrap();
        });
        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut buf = String::new();
        client.read_to_string(&mut buf).unwrap();
        server.join().unwrap();
        assert!(buf.contains("500 Internal Server Error"));
        assert!(buf.contains("text/html; charset=utf-8"));
        assert!(buf.contains("<html>boom</html>"));
    }

    #[test]
    fn roc_output_treats_error_summary_as_failure() {
        assert!(roc_output_is_failure(
            "Found 20 errors and 2 warnings for main.roc.\n"
        ));
        assert!(roc_output_is_failure(
            "Found 1 error and 0 warnings for main.roc.\n"
        ));
        assert!(!roc_output_is_failure(
            "Found 0 errors and 5 warnings for main.roc.\n"
        ));
        assert!(!roc_output_is_failure(
            "Listening on http://127.0.0.1:8000\n"
        ));
    }

    #[test]
    fn roc_output_treats_crash_marker_as_failure() {
        assert!(roc_output_is_failure("[ROC CRASHED] runtime error\n"));
        assert!(roc_output_is_failure(
            "Found 0 errors and 0 warnings for main.roc.\n[ROC CRASHED] runtime error\n"
        ));
    }

    #[test]
    fn stderr_snapshot_does_not_consume_the_buffer() {
        let tee = StderrTee {
            buf: Arc::new(Mutex::new(String::from("Found 2 errors\n"))),
            feed: None,
            thread: None,
        };
        assert_eq!(tee.snapshot(), "Found 2 errors\n");
        assert_eq!(tee.snapshot(), "Found 2 errors\n");
        assert!(roc_output_is_failure(&tee.snapshot()));
    }

    #[test]
    fn wait_for_roc_diagnostics_returns_existing_summary() {
        let tee = StderrTee {
            buf: Arc::new(Mutex::new(String::from(
                "Found 2 errors and 0 warnings for main.roc.\nListening on http://127.0.0.1:1\n",
            ))),
            feed: None,
            thread: None,
        };
        let snap = wait_for_roc_diagnostics(&tee);
        assert!(roc_output_is_failure(&snap));
    }

    #[test]
    fn probe_http_succeeds_when_server_responds() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 256];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
        });
        probe_http(port, "/").unwrap();
        server.join().unwrap();
    }

    #[test]
    fn listen_helpers_read_port_and_host_env() {
        assert!(ROC_LISTEN_PORT_HELPER.contains("ROC_BASIC_WEBSERVER_PORT"));
        assert!(ROC_LISTEN_HOST_HELPER.contains("ROC_BASIC_WEBSERVER_HOST"));
        assert!(ROC_LISTEN_HOST_HELPER.contains("127.0.0.1"));
    }

    #[test]
    fn normalize_probe_path_adds_a_leading_slash() {
        assert_eq!(normalize_probe_path(""), "/");
        assert_eq!(normalize_probe_path("/all-syntax/"), "/all-syntax/");
        assert_eq!(normalize_probe_path("all-syntax/"), "/all-syntax/");
    }

    #[test]
    fn wait_listen_messages_include_port_and_elapsed() {
        assert_eq!(wait_listen_starting(8123), "waiting for roc on :8123");
        assert_eq!(
            wait_listen_heartbeat(8123, Duration::from_secs(4)),
            "waiting for roc on :8123 (4s)"
        );
        let timeout = wait_listen_timeout_message(8123, Duration::from_secs(120), true);
        assert!(timeout.contains("waiting for roc"));
        assert!(timeout.contains("8123"));
        assert!(timeout.contains("120s"));
        assert!(timeout.contains("process still running: true"));
    }

    #[test]
    fn stderr_bytes_become_runtime_log_lines() {
        let lines = stderr_log_lines(
            "compiling main.roc\nFound 1 error and 0 warnings for main.roc.\nwarning: unused\nFound 0 errors and 2 warnings for main.roc.\n",
        );
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].source, "runtime");
        assert_eq!(lines[0].level, "info");
        assert_eq!(lines[0].text, "compiling main.roc");
        assert_eq!(lines[1].source, "runtime");
        assert_eq!(lines[1].level, "error");
        assert_eq!(lines[2].level, "warn");
        assert_eq!(lines[2].text, "warning: unused");
        assert_eq!(lines[3].level, "warn");
        assert_eq!(lines[3].text, "Found 0 errors and 2 warnings for main.roc.");
        let handlers = stderr_log_lines(
            "POST /actions/counter/increment -> ok\nPOST /actions/counter/increment -> err\n",
        );
        assert_eq!(handlers[0].level, "info");
        assert_eq!(handlers[1].level, "error");
    }

    #[test]
    fn stderr_feed_skips_already_drained_bytes() {
        let hub = Arc::new(LogHub::new());
        let feed = StderrHubFeed {
            hub: hub.clone(),
            state: Mutex::new(StderrFeedState {
                offset: 0,
                remainder: String::new(),
            }),
        };
        feed.drain("first\n");
        feed.drain("first\nsecond\n");
        let lines = hub.snapshot();
        assert_eq!(
            lines
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert!(lines.iter().all(|line| line.source == "runtime"));
    }

    #[test]
    fn flush_to_hub_copies_snapshot_without_duplicates() {
        let hub = Arc::new(LogHub::new());
        let feed = Arc::new(StderrHubFeed {
            hub: hub.clone(),
            state: Mutex::new(StderrFeedState {
                offset: 0,
                remainder: String::new(),
            }),
        });
        let tee = StderrTee {
            buf: Arc::new(Mutex::new(String::from(
                "Listening on http://127.0.0.1:8000\n",
            ))),
            feed: Some(feed),
            thread: None,
        };
        tee.flush_to_hub();
        tee.flush_to_hub();
        let lines = hub.snapshot();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "Listening on http://127.0.0.1:8000");
        assert_eq!(lines[0].source, "runtime");
        assert_eq!(lines[0].level, "info");
    }

    #[test]
    fn no_window_spawns_sibling_inspector_before_wait() {
        let src = include_str!("serve.rs");
        let start = src
            .find("pub fn with_window_and_inspector")
            .expect("with_window_and_inspector");
        let body = &src[start..];
        let spawn = body.find("InspectorServer::spawn").expect("spawn");
        let no_window = body.find("if no_window").expect("no_window");
        let wait = body.find("child.wait()").expect("wait");
        assert!(spawn < no_window);
        assert!(no_window < wait);
    }
}
