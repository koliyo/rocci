use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Context, Result};
use rocci_template::{LowerOptions, SourceFile, compile, file_scope_id};

use crate::error_page;
use crate::profile::ProfileSnapshot;

const METRICS_PANEL: &str = include_str!("../templates/dev/MetricsPanel.rocci");

pub fn render_panel_html(snapshot: Option<&ProfileSnapshot>) -> String {
    let css = panel_css();
    let scope = file_scope_id("MetricsPanel.rocci");
    let body = match snapshot {
        Some(snapshot) if !snapshot.spans.is_empty() => render_spans(snapshot),
        Some(snapshot) => format!(
            "<p class=\"total\"><span class=\"value\">{}</span><span class=\"unit\">ms total</span></p><p class=\"empty\">No timing spans recorded.</p>",
            snapshot.total_ms
        ),
        None => "<p class=\"empty\">No timing spans recorded.</p>".to_string(),
    };
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta name=\"color-scheme\" content=\"light dark\"><title>Profiling</title><style>{css}</style></head><body><section class=\"metrics-panel\" data-rocci-css=\"{scope}\"><h1>Profiling</h1>{body}</section></body></html>\n"
    )
}

fn panel_css() -> &'static str {
    static CSS: OnceLock<String> = OnceLock::new();
    CSS.get_or_init(|| {
        let compiled = compile(
            SourceFile::new("MetricsPanel.rocci", METRICS_PANEL),
            &LowerOptions::default(),
        );
        compiled
            .styles
            .iter()
            .map(|style| style.css.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn render_spans(snapshot: &ProfileSnapshot) -> String {
    let mut rows = String::new();
    for span in &snapshot.spans {
        let note = span
            .note
            .as_deref()
            .map(|note| {
                format!(
                    "<span class=\"note\">{}</span>",
                    error_page::html_escape(note)
                )
            })
            .unwrap_or_default();
        rows.push_str(&format!(
            "<tr><td>{}</td><td class=\"ms\">{}</td><td>{note}</td></tr>",
            error_page::html_escape(&span.name),
            span.duration_ms
        ));
    }
    format!(
        "<p class=\"total\"><span class=\"value\">{}</span><span class=\"unit\">ms total</span></p><table><thead><tr><th>Stage</th><th>ms</th><th></th></tr></thead><tbody>{rows}</tbody></table>",
        snapshot.total_ms
    )
}

pub fn metrics_panel_compiles() -> bool {
    let compiled = compile(
        SourceFile::new("MetricsPanel.rocci", METRICS_PANEL),
        &LowerOptions::default(),
    );
    !compiled.has_errors()
}

pub struct InspectorServer {
    pub url: String,
    stop: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
}

impl InspectorServer {
    pub fn spawn(snapshot: ProfileSnapshot) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").context("failed to bind inspector")?;
        listener
            .set_nonblocking(true)
            .context("failed to set inspector listener non-blocking")?;
        let port = listener.local_addr()?.port();
        let url = format!("http://127.0.0.1:{port}/__rocci/dev");
        let stop = Arc::new(AtomicBool::new(false));
        let store = Arc::new(Mutex::new(snapshot));
        let thread_stop = stop.clone();
        let thread_store = store;
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let snapshot = thread_store
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .clone();
                        let _ = handle_inspector(stream, Some(&snapshot));
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            url,
            stop,
            _thread: Some(thread),
        })
    }
}

impl Drop for InspectorServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn handle_inspector(
    mut stream: TcpStream,
    snapshot: Option<&ProfileSnapshot>,
) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let path = path.split(['?', '#']).next().unwrap_or(path);
    match path {
        "/__rocci/profile" | "/__rocdown/profile" | "/__rocci_okf/profile" => {
            let body = snapshot
                .map(ProfileSnapshot::to_json)
                .unwrap_or_else(|| "{\"total_ms\":0,\"spans\":[]}".to_string());
            write_response(
                &mut stream,
                200,
                "application/json; charset=utf-8",
                body.as_bytes(),
            )
        }
        "/__rocci/dev" | "/__rocdown/dev" | "/__rocci_okf/dev" | "/" => {
            let html = render_panel_html(snapshot);
            write_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                html.as_bytes(),
            )
        }
        _ => write_response(&mut stream, 404, "text/plain; charset=utf-8", b"not found"),
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = if status == 200 { "OK" } else { "Not Found" };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{ProfileSpan, SpanRecorder};

    #[test]
    fn metrics_panel_template_compiles() {
        assert!(metrics_panel_compiles());
    }

    #[test]
    fn panel_html_includes_spans() {
        let mut recorder = SpanRecorder::new();
        recorder.push("parse", 4, None);
        recorder.push("compile", 10, Some("cached".into()));
        let snapshot = recorder.finish();
        let html = render_panel_html(Some(&snapshot));
        assert!(html.contains("Profiling"));
        assert!(html.contains("parse"));
        assert!(html.contains("cached"));
        assert!(html.contains("14"));
        let empty = render_panel_html(None);
        assert!(empty.contains("No timing spans recorded."));
    }

    #[test]
    fn profile_span_serializes() {
        let span = ProfileSpan {
            name: "read".into(),
            duration_ms: 1,
            note: None,
        };
        assert_eq!(span.name, "read");
    }
}
