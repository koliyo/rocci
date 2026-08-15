use std::{
    net::{TcpListener, TcpStream},
    process::Child,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use clap::Args;
use rocci_wry::{PreviewOptions, preview};

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

pub fn wait_for_server(child: &mut Child, port: u16) -> Result<()> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => bail!("roc exited before serving ({status})"),
            Ok(None) => {}
            Err(err) => bail!("failed to poll roc: {err}"),
        }
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        if start.elapsed() > SERVER_WAIT {
            bail!("timed out waiting for roc server on port {port}");
        }
        thread::sleep(Duration::from_millis(100));
    }
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
}
