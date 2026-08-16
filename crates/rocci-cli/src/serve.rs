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
use rocci_wry::{PreviewOptions, preview};

use crate::style;

const SERVER_WAIT: Duration = Duration::from_secs(120);

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
    /// Skip the embedded window; print the URL and keep the Roc server.
    #[arg(long)]
    pub no_window: bool,

    /// TCP port to listen on. Defaults to a free port with the embedded window,
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
    thread: Option<JoinHandle<()>>,
}

impl StderrTee {
    pub fn finish(&mut self) -> String {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.buf
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }
}

pub fn spawn_roc(mut cmd: Command) -> Result<(Child, StderrTee)> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .context("failed to start `roc`; is it on PATH?")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture roc stderr")?;
    let buf = Arc::new(Mutex::new(String::new()));
    let thread_buf = buf.clone();
    let thread = thread::spawn(move || {
        let mut reader = io::BufReader::new(stderr);
        let mut chunk = [0u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&chunk[..n]);
                    eprint!("{text}");
                    let _ = io::stderr().flush();
                    thread_buf
                        .lock()
                        .unwrap_or_else(|err| err.into_inner())
                        .push_str(&text);
                }
                Err(_) => break,
            }
        }
    });
    Ok((
        child,
        StderrTee {
            buf,
            thread: Some(thread),
        },
    ))
}

pub fn wait_for_listen(child: &mut Child, port: u16) -> Result<ListenWait> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(ListenWait::Exited(status)),
            Ok(None) => {}
            Err(err) => bail!("failed to poll roc: {err}"),
        }
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(ListenWait::Ready);
        }
        if start.elapsed() > SERVER_WAIT {
            bail!("timed out waiting for roc server on port {port}");
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn wait_for_server(child: &mut Child, port: u16) -> Result<()> {
    match wait_for_listen(child, port)? {
        ListenWait::Ready => Ok(()),
        ListenWait::Exited(status) => bail!("roc exited before serving ({status})"),
    }
}

pub fn serve_html(port: u16, status: u16, html: &str, title: &str, no_window: bool) -> Result<()> {
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
    if no_window {
        let _ = thread.join();
        return Ok(());
    }
    let preview = open_preview(&url, title);
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

pub fn open_preview(url: &str, title: &str) -> Result<()> {
    preview(PreviewOptions {
        url: url.to_string(),
        title: title.to_string(),
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"))
}

pub fn with_window(child: &mut Child, url: &str, title: &str, no_window: bool) -> Result<()> {
    if no_window {
        let status = child.wait().context("roc server exited unexpectedly")?;
        if !status.success() {
            bail!("roc exited with {status}");
        }
        return Ok(());
    }

    let preview_result = open_preview(url, title);
    let _ = child.kill();
    let _ = child.wait();
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
}
