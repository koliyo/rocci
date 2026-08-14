use std::{
    net::TcpStream,
    process::Child,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use rocci_wry::{PreviewOptions, preview};

const SERVER_WAIT: Duration = Duration::from_secs(120);
const DEFAULT_PORT: u16 = 8000;

pub fn parse_basic_webserver_port(value: Option<&str>) -> Result<u16> {
    match value {
        Some(value) => value
            .parse()
            .with_context(|| format!("invalid ROC_BASIC_WEBSERVER_PORT `{value}`")),
        None => Ok(DEFAULT_PORT),
    }
}

pub fn basic_webserver_port() -> Result<u16> {
    parse_basic_webserver_port(std::env::var("ROC_BASIC_WEBSERVER_PORT").ok().as_deref())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port_is_8000() {
        assert_eq!(parse_basic_webserver_port(None).unwrap(), 8000);
    }

    #[test]
    fn honors_explicit_port() {
        assert_eq!(parse_basic_webserver_port(Some("9001")).unwrap(), 9001);
    }

    #[test]
    fn rejects_invalid_port() {
        let err = parse_basic_webserver_port(Some("nope"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("invalid ROC_BASIC_WEBSERVER_PORT"));
    }
}
