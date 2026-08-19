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
use crate::inspect::{self, InspectSnapshot};
use crate::profile::ProfileSnapshot;

const METRICS_PANEL: &str = include_str!("../templates/dev/MetricsPanel.rocci");

pub fn render_panel_html(snapshot: Option<&InspectSnapshot>, target: &str) -> String {
    let css = panel_css();
    let scope = file_scope_id("MetricsPanel.rocci");
    let query = inspect::parse_inspect_query(target);
    let profile = snapshot.map(|snapshot| &snapshot.profile);
    let profiling = match profile {
        Some(profile) if !profile.spans.is_empty() => render_spans(profile),
        Some(profile) => format!(
            "<p class=\"total\"><span class=\"value\">{}</span><span class=\"unit\">ms total</span></p><p class=\"empty\">No timing spans recorded.</p>",
            profile.total_ms
        ),
        None => "<p class=\"empty\">No timing spans recorded.</p>".to_string(),
    };
    let source_pane = render_source_pane(snapshot, &query);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta name=\"color-scheme\" content=\"light dark\"><title>Profiling</title><style>{css}</style></head><body><section class=\"metrics-panel\" data-rocci-css=\"{scope}\"><h1>Profiling</h1>{profiling}{source_pane}</section></body></html>\n"
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

fn render_source_pane(snapshot: Option<&InspectSnapshot>, query: &inspect::InspectQuery) -> String {
    let view = query.view;
    let route = query.route.as_deref().unwrap_or("/");
    let (path, body, available, reason) =
        match snapshot.and_then(|snapshot| snapshot.resolve(query.route.as_deref()).ok()) {
            Some(page) => {
                let capability = page.capability_for(view);
                if capability.available {
                    (page.path.as_str(), page.body_for(view), true, "")
                } else {
                    (page.path.as_str(), "", false, capability.reason.as_str())
                }
            }
            None => ("", "", false, "No inspect snapshot for this route."),
        };
    let reason_html = if available {
        String::new()
    } else {
        format!(
            "<p class=\"unavailable\">{}</p>",
            error_page::html_escape(reason)
        )
    };
    format!(
        "<form class=\"source-form\" method=\"get\"><input type=\"hidden\" name=\"route\" value=\"{}\" /><label class=\"view-label\">View<select name=\"view\" onchange=\"this.form.submit()\">{}</select></label><noscript><button type=\"submit\">Show</button></noscript></form><p class=\"file-path\">{}</p>{reason_html}<pre><code>{}</code></pre>",
        error_page::html_escape(route),
        view_options(view),
        error_page::html_escape(path),
        error_page::html_escape(body),
    )
}

fn view_options(selected: inspect::InspectView) -> String {
    const OPTIONS: [(&str, &str); 4] = [
        ("source", "Original source"),
        ("ast", "AST"),
        ("roc", "Generated Roc"),
        ("html", "Generated HTML"),
    ];
    let mut html = String::new();
    for (value, label) in OPTIONS {
        if selected.as_str() == value {
            html.push_str(&format!(
                "<option value=\"{value}\" selected=\"\">{label}</option>"
            ));
        } else {
            html.push_str(&format!("<option value=\"{value}\">{label}</option>"));
        }
    }
    html
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
    metrics_panel_diagnostics().is_empty()
}

pub fn metrics_panel_diagnostics() -> Vec<String> {
    let compiled = compile(
        SourceFile::new("MetricsPanel.rocci", METRICS_PANEL),
        &LowerOptions::default(),
    );
    compiled
        .diagnostics
        .iter()
        .map(|d| d.message.clone())
        .collect()
}

pub struct InspectorServer {
    pub url: String,
    stop: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
}

impl InspectorServer {
    pub fn spawn(snapshot: impl Into<InspectSnapshot>) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").context("failed to bind inspector")?;
        listener
            .set_nonblocking(true)
            .context("failed to set inspector listener non-blocking")?;
        let port = listener.local_addr()?.port();
        let url = format!("http://127.0.0.1:{port}/__rocci/dev");
        let stop = Arc::new(AtomicBool::new(false));
        let store = Arc::new(Mutex::new(snapshot.into()));
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
    snapshot: Option<&InspectSnapshot>,
) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    let target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let path = target.split(['?', '#']).next().unwrap_or(target);
    match path {
        "/__rocci/inspect" | "/__rocdown/inspect" | "/__rocci_okf/inspect" => {
            let (status, body) = inspect::inspect_json(snapshot, target);
            write_response(
                &mut stream,
                status,
                "application/json; charset=utf-8",
                body.as_bytes(),
            )
        }
        "/__rocci/profile" | "/__rocdown/profile" | "/__rocci_okf/profile" => {
            let body = snapshot
                .map(|snapshot| snapshot.profile.to_json())
                .unwrap_or_else(|| "{\"total_ms\":0,\"spans\":[]}".to_string());
            write_response(
                &mut stream,
                200,
                "application/json; charset=utf-8",
                body.as_bytes(),
            )
        }
        "/__rocci/dev" | "/__rocdown/dev" | "/__rocci_okf/dev" | "/" => {
            let html = render_panel_html(snapshot, target);
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
        let diagnostics = metrics_panel_diagnostics();
        assert!(
            diagnostics.is_empty(),
            "MetricsPanel.rocci diagnostics: {diagnostics:?}"
        );
    }

    fn sample_inspect() -> InspectSnapshot {
        let mut recorder = SpanRecorder::new();
        recorder.push("parse", 4, None);
        recorder.push("compile", 10, Some("cached".into()));
        InspectSnapshot {
            pages: vec![inspect::InspectPage {
                route: "/".into(),
                path: "App.rocci".into(),
                language: "rocci".into(),
                source: "<div class=\"x\">& more</div>".into(),
                ast: "(Document ...)".into(),
                roc: "app [] {}".into(),
                html: String::new(),
                capabilities: inspect::InspectCapabilities {
                    source: inspect::ViewCapability::available(),
                    ast: inspect::ViewCapability::available(),
                    roc: inspect::ViewCapability::available(),
                    html: inspect::ViewCapability::unavailable(
                        "HTML snapshot was not captured for this route.",
                    ),
                },
            }],
            profile: recorder.finish(),
        }
    }

    #[test]
    fn panel_html_includes_spans() {
        let snapshot = sample_inspect();
        let html = render_panel_html(Some(&snapshot), "/__rocci/dev");
        assert!(html.contains("Profiling"));
        assert!(html.contains("parse"));
        assert!(html.contains("cached"));
        assert!(html.contains("14"));
        assert!(html.contains("<table>"));
        let empty = render_panel_html(None, "/__rocci/dev");
        assert!(empty.contains("No timing spans recorded."));
    }

    #[test]
    fn panel_html_includes_source_views() {
        let snapshot = sample_inspect();
        let html = render_panel_html(Some(&snapshot), "/__rocci/dev?route=/&view=source");
        assert!(html.contains("value=\"source\""));
        assert!(html.contains("Original source"));
        assert!(html.contains("value=\"ast\""));
        assert!(html.contains(">AST<"));
        assert!(html.contains("value=\"roc\""));
        assert!(html.contains("Generated Roc"));
        assert!(html.contains("value=\"html\""));
        assert!(html.contains("Generated HTML"));
        assert!(html.contains("<option value=\"source\" selected=\"\">"));
        assert!(html.contains("App.rocci"));
        assert!(html.contains("&lt;div class=&quot;x&quot;&gt;&amp; more&lt;/div&gt;"));
        assert!(!html.contains("<p class=\"unavailable\">"));

        let ast = render_panel_html(Some(&snapshot), "/__rocci/dev?view=ast");
        assert!(ast.contains("<option value=\"ast\" selected=\"\">"));
        assert!(ast.contains("(Document ...)"));

        let html_view = render_panel_html(Some(&snapshot), "/__rocci/dev?view=html");
        assert!(html_view.contains("<option value=\"html\" selected=\"\">"));
        assert!(html_view.contains("HTML snapshot was not captured for this route."));
        assert!(html_view.contains("<pre><code></code></pre>"));

        let unknown = render_panel_html(Some(&snapshot), "/__rocci/dev?view=nope");
        assert!(unknown.contains("<option value=\"source\" selected=\"\">"));
        assert!(unknown.contains("&lt;div class=&quot;x&quot;&gt;&amp; more&lt;/div&gt;"));
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

    #[test]
    fn inspector_server_serves_inspect_json() {
        use std::io::{Read, Write};

        let snapshot = InspectSnapshot {
            pages: vec![inspect::InspectPage {
                route: "/".into(),
                path: "App.rocci".into(),
                language: "rocci".into(),
                source: "<span>&\"'</span>".into(),
                ast: "(Document)".into(),
                roc: "app [] {}".into(),
                html: String::new(),
                capabilities: inspect::InspectCapabilities {
                    source: inspect::ViewCapability::available(),
                    ast: inspect::ViewCapability::available(),
                    roc: inspect::ViewCapability::available(),
                    html: inspect::ViewCapability::unavailable("no HTML snapshot"),
                },
            }],
            profile: ProfileSnapshot {
                total_ms: 3,
                spans: vec![ProfileSpan {
                    name: "parse".into(),
                    duration_ms: 3,
                    note: None,
                }],
            },
        };
        let server = InspectorServer::spawn(snapshot).unwrap();
        let port: u16 = server
            .url
            .trim_start_matches("http://127.0.0.1:")
            .trim_end_matches("/__rocci/dev")
            .parse()
            .unwrap();

        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client
            .write_all(b"GET /__rocci/inspect?route=/ HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
        assert!(response.contains("application/json"));
        let body = response.split("\r\n\r\n").nth(1).unwrap_or("");
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(value["source"], "<span>&\"'</span>");
        assert_eq!(value["capabilities"]["html"]["reason"], "no HTML snapshot");

        let mut missing = TcpStream::connect(("127.0.0.1", port)).unwrap();
        missing
            .write_all(b"GET /__rocci/inspect?route=/nope HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut not_found = String::new();
        missing.read_to_string(&mut not_found).unwrap();
        assert!(not_found.contains("HTTP/1.1 404 Not Found"), "{not_found}");
    }
}
