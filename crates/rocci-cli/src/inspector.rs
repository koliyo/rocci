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
use crate::logs::{LogHub, LogLine};
use crate::profile::ProfileSnapshot;

const METRICS_PANEL: &str = include_str!("../templates/dev/MetricsPanel.rocci");

const DOCUMENT_CSS: &str = "html, body { height: 100%; margin: 0; overflow: hidden; }";

const INSPECTOR_NOTIFY: &str = r#"<script>(function(){var p=new URLSearchParams(location.search);parent.postMessage({type:"rocci-inspector",tab:p.get("tab")||"performance",view:p.get("view")||"source"},"*");})();</script>"#;

const CONSOLE_JS: &str = r#"<script>(function(){var root=document.querySelector("[data-logs-root]");if(!root)return;var api=root.getAttribute("data-logs-root")||"/__rocci";var pane=document.getElementById("console-log");var body=pane&&pane.querySelector("tbody");if(!pane||!body)return;var levels={debug:true,info:true,warn:true,error:true};var stick=true;function near(){return pane.scrollHeight-pane.scrollTop-pane.clientHeight<32;}function apply(){var rows=body.querySelectorAll("tr[data-level]");for(var i=0;i<rows.length;i++){rows[i].hidden=!levels[rows[i].getAttribute("data-level")];}}function row(line){var tr=document.createElement("tr");tr.setAttribute("data-level",line.level||"info");var t=new Date(Number(line.t)||0);var time=isNaN(t.getTime())?String(line.t||""):t.toLocaleTimeString();tr.innerHTML="<td>"+time+"</td><td>"+esc(line.level)+"</td><td>"+esc(line.source)+"</td><td>"+esc(line.text)+"</td>";return tr;}function esc(v){return String(v==null?"":v).replace(/[&<>\"]/g,function(ch){return ch==="&"?"&amp;":ch==="<"?"&lt;":ch===">"?"&gt;":"&quot;";});}pane.addEventListener("scroll",function(){stick=near();});var chips=root.querySelectorAll("[data-level]");for(var c=0;c<chips.length;c++){(function(btn){btn.addEventListener("click",function(){var level=btn.getAttribute("data-level");levels[level]=!levels[level];btn.setAttribute("aria-pressed",levels[level]?"true":"false");apply();});})(chips[c]);}var clear=root.querySelector(".console-clear");if(clear){clear.addEventListener("click",function(){fetch(api+"/logs/clear",{method:"POST"}).then(function(){body.innerHTML="";});});}try{var es=new EventSource(api+"/logs/events");es.addEventListener("log",function(ev){var keep=stick||near();try{body.appendChild(row(JSON.parse(ev.data)));}catch(err){}apply();if(keep){pane.scrollTop=pane.scrollHeight;}});}catch(err){}})();</script>"#;

pub fn render_panel_html(snapshot: Option<&InspectSnapshot>, target: &str) -> String {
    render_panel_with_logs(snapshot, target, &[])
}

pub fn render_panel_with_logs(
    snapshot: Option<&InspectSnapshot>,
    target: &str,
    logs: &[LogLine],
) -> String {
    let css = panel_css();
    let scope = file_scope_id("MetricsPanel.rocci");
    let query = inspect::parse_inspect_query(target);
    let tab = panel_tab(target);
    let action = panel_form_action(target);
    let route = query.route.as_deref().unwrap_or("/");
    let tabs = render_tablist(tab, action, route, query.view.as_str());
    let body = match tab {
        "source" => format!(
            "<div class=\"inspector-body tab-source\">{}</div>",
            render_source_pane(snapshot, &query, target)
        ),
        "console" => render_console_pane(target, logs),
        _ => format!(
            "<div class=\"inspector-body tab-performance\"><h1>Profiling</h1>{}</div>",
            render_performance(snapshot.map(|snapshot| &snapshot.profile))
        ),
    };
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta name=\"color-scheme\" content=\"light dark\"><title>Inspector</title><style>{DOCUMENT_CSS}{css}</style></head><body><section class=\"inspector-panel\" data-rocci-css=\"{scope}\">{tabs}{body}</section>{INSPECTOR_NOTIFY}</body></html>\n"
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

fn render_source_pane(
    snapshot: Option<&InspectSnapshot>,
    query: &inspect::InspectQuery,
    target: &str,
) -> String {
    let view = query.view;
    let route = query.route.as_deref().unwrap_or("/");
    let action = panel_form_action(target);
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
    let pane_html = if available {
        format!(
            "<div class=\"code-pane\"><pre><code>{}</code></pre></div>",
            error_page::html_escape(body)
        )
    } else {
        "<div class=\"code-pane\"></div>".to_string()
    };
    format!(
        "<div class=\"source-chrome\"><form class=\"source-form\" method=\"get\" action=\"{action}\"><input type=\"hidden\" name=\"route\" value=\"{}\" /><input type=\"hidden\" name=\"tab\" value=\"source\" /><label class=\"view-label\">View<select name=\"view\" onchange=\"this.form.submit()\">{}</select></label><noscript><button type=\"submit\">Show</button></noscript></form><p class=\"file-path\">{}</p>{reason_html}</div>{pane_html}",
        error_page::html_escape(route),
        view_options(view),
        error_page::html_escape(path),
    )
}

fn render_tablist(selected: &str, action: &str, route: &str, view: &str) -> String {
    const TABS: [(&str, &str); 3] = [
        ("performance", "Performance"),
        ("source", "Source"),
        ("console", "Console"),
    ];
    let mut html = String::from("<nav class=\"inspector-tabs\" role=\"tablist\">");
    for (id, label) in TABS {
        let selected_attr = if selected == id { "true" } else { "false" };
        let href = format!(
            "{action}?tab={id}&route={}&view={}",
            query_encode(route),
            query_encode(view)
        );
        html.push_str(&format!(
            "<a role=\"tab\" aria-selected=\"{selected_attr}\" href=\"{}\">{label}</a>",
            error_page::html_escape(&href)
        ));
    }
    html.push_str("</nav>");
    html
}

fn panel_api_root(target: &str) -> &'static str {
    match panel_form_action(target) {
        "/__rocdown/dev" => "/__rocdown",
        "/__rocci_okf/dev" => "/__rocci_okf",
        _ => "/__rocci",
    }
}

fn render_console_pane(target: &str, logs: &[LogLine]) -> String {
    let root = panel_api_root(target);
    let mut rows = String::new();
    for line in logs {
        rows.push_str(&format!(
            "<tr data-level=\"{}\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            error_page::html_escape(&line.level),
            line.t,
            error_page::html_escape(&line.level),
            error_page::html_escape(&line.source),
            error_page::html_escape(&line.text),
        ));
    }
    if rows.is_empty() {
        rows.push_str(
            "<tr class=\"console-empty\"><td colspan=\"4\">No runtime messages yet.</td></tr>",
        );
    }
    format!(
        "<div class=\"inspector-body tab-console\" data-logs-root=\"{root}\"><div class=\"console-toolbar\"><div class=\"console-filters\" role=\"group\" aria-label=\"Log level\"><button type=\"button\" data-level=\"debug\" aria-pressed=\"true\">debug</button><button type=\"button\" data-level=\"info\" aria-pressed=\"true\">info</button><button type=\"button\" data-level=\"warn\" aria-pressed=\"true\">warn</button><button type=\"button\" data-level=\"error\" aria-pressed=\"true\">error</button></div><button type=\"button\" class=\"console-clear\">Clear</button></div><div class=\"console-log\" id=\"console-log\"><table><thead><tr><th>Time</th><th>Level</th><th>Source</th><th>Message</th></tr></thead><tbody>{rows}</tbody></table></div></div>{CONSOLE_JS}"
    )
}

fn render_performance(profile: Option<&ProfileSnapshot>) -> String {
    match profile {
        Some(profile) if !profile.spans.is_empty() => render_spans(profile),
        Some(profile) => format!(
            "<p class=\"total\"><span class=\"value\">{}</span><span class=\"unit\">ms total</span></p><p class=\"empty\">No timing spans recorded.</p>",
            profile.total_ms
        ),
        None => "<p class=\"empty\">No timing spans recorded.</p>".to_string(),
    }
}

fn query_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn panel_form_action(target: &str) -> &'static str {
    let path = target.split(['?', '#']).next().unwrap_or(target);
    match path {
        "/__rocdown/dev" => "/__rocdown/dev",
        "/__rocci_okf/dev" => "/__rocci_okf/dev",
        _ => "/__rocci/dev",
    }
}

fn panel_tab(target: &str) -> &'static str {
    let query = target.split_once('?').map(|(_, query)| query).unwrap_or("");
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "tab" {
            return match value {
                "source" => "source",
                "console" => "console",
                "performance" => "performance",
                _ => "performance",
            };
        }
    }
    "performance"
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
    pub logs: Arc<LogHub>,
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
        let logs = Arc::new(LogHub::new());
        let thread_stop = stop.clone();
        let thread_store = store;
        let thread_logs = logs.clone();
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let snapshot = thread_store
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .clone();
                        let logs = thread_logs.clone();
                        thread::spawn(move || {
                            let _ = handle_inspector(stream, Some(&snapshot), &logs);
                        });
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
            logs,
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
    logs: &LogHub,
) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    let first = request.lines().next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let target = parts.next().unwrap_or("/");
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
        "/__rocci/logs" | "/__rocdown/logs" | "/__rocci_okf/logs" => write_response(
            &mut stream,
            200,
            "application/json; charset=utf-8",
            logs.to_json().as_bytes(),
        ),
        "/__rocci/logs/events" | "/__rocdown/logs/events" | "/__rocci_okf/logs/events" => {
            write_inspector_log_sse(&mut stream, logs)
        }
        "/__rocci/logs/clear" | "/__rocdown/logs/clear" | "/__rocci_okf/logs/clear" => {
            if method != "POST" {
                return write_response(&mut stream, 404, "text/plain; charset=utf-8", b"not found");
            }
            logs.clear();
            write_response(&mut stream, 204, "text/plain; charset=utf-8", b"")
        }
        "/__rocci/dev" | "/__rocdown/dev" | "/__rocci_okf/dev" | "/" => {
            let html = render_panel_with_logs(snapshot, target, &logs.snapshot());
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

fn write_inspector_log_sse(stream: &mut TcpStream, hub: &LogHub) -> std::io::Result<()> {
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

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        _ => "Not Found",
    };
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
        assert!(html.contains("<title>Inspector</title>"));
        assert!(html.contains("Profiling"));
        assert!(html.contains("parse"));
        assert!(html.contains("cached"));
        assert!(html.contains("14"));
        assert!(html.contains("<table>"));
        assert!(!html.contains("<pre><code>"));
        let empty = render_panel_html(None, "/__rocci/dev");
        assert!(empty.contains("No timing spans recorded."));
    }

    #[test]
    fn panel_html_includes_tabs() {
        let snapshot = sample_inspect();
        let html = render_panel_html(Some(&snapshot), "/__rocci/dev");
        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains(">Performance<"));
        assert!(html.contains(">Source<"));
        assert!(html.contains(">Console<"));
        assert!(html.contains("aria-selected=\"true\""));
        assert!(html.contains("tab=performance"));

        let source = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=source&view=ast");
        assert!(source.contains("aria-selected=\"true\""));
        assert!(source.contains("(Document ...)"));
        assert!(!source.contains("<table>"));
        assert!(source.contains("<pre><code>"));

        let performance = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=performance");
        assert!(performance.contains("<table>"));
        assert!(!performance.contains("<pre><code>"));

        let console = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=console");
        assert!(console.contains("No runtime messages yet."));
        assert!(!console.contains("<table><thead><tr><th>Stage</th>"));
        assert!(!console.contains("<pre><code>"));
        let logged = render_panel_with_logs(
            Some(&snapshot),
            "/__rocci/dev?tab=console",
            &[LogLine::runtime(
                crate::logs::LogLevel::Info,
                "serving at 127.0.0.1",
            )],
        );
        assert!(logged.contains("serving at 127.0.0.1"));
        assert!(logged.contains("data-level=\"info\""));

        let unknown = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=nope");
        assert!(unknown.contains("<table>"));
        assert!(unknown.contains("tab=performance"));
    }

    #[test]
    fn panel_html_includes_source_views() {
        let snapshot = sample_inspect();
        let html = render_panel_html(
            Some(&snapshot),
            "/__rocci/dev?tab=source&route=/&view=source",
        );
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
        assert!(html.contains("action=\"/__rocci/dev\""));
        assert!(html.contains("name=\"tab\" value=\"source\""));
        assert!(html.contains("class=\"code-pane\""));
        assert!(html.contains("html, body { height: 100%; margin: 0; overflow: hidden; }"));
        assert!(html.contains("rocci-inspector"));
        assert!(!html.contains("<table>"));

        let ast = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=source&view=ast");
        assert!(ast.contains("<option value=\"ast\" selected=\"\">"));
        assert!(ast.contains("(Document ...)"));
        assert!(ast.contains("<div class=\"code-pane\"><pre><code>"));

        let html_view = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=source&view=html");
        assert!(html_view.contains("<option value=\"html\" selected=\"\">"));
        assert!(html_view.contains("HTML snapshot was not captured for this route."));
        assert!(html_view.contains("<div class=\"code-pane\"></div>"));
        assert!(!html_view.contains("<pre><code>"));

        let unknown = render_panel_html(Some(&snapshot), "/__rocci/dev?tab=source&view=nope");
        assert!(unknown.contains("<option value=\"source\" selected=\"\">"));
        assert!(unknown.contains("&lt;div class=&quot;x&quot;&gt;&amp; more&lt;/div&gt;"));

        let alias = render_panel_html(
            Some(&snapshot),
            "/__rocdown/dev?tab=source&route=/&view=ast",
        );
        assert!(alias.contains("action=\"/__rocdown/dev\""));
        assert!(alias.contains("name=\"tab\" value=\"source\""));
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

    #[test]
    fn inspector_server_serves_logs() {
        use std::io::{Read, Write};

        let server = InspectorServer::spawn(sample_inspect()).unwrap();
        server
            .logs
            .push(crate::logs::LogLevel::Info, "rebuild done");
        let port: u16 = server
            .url
            .trim_start_matches("http://127.0.0.1:")
            .trim_end_matches("/__rocci/dev")
            .parse()
            .unwrap();

        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client
            .write_all(
                b"GET /__rocci/logs HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
        assert!(response.contains("application/json"));
        let body = response.split("\r\n\r\n").nth(1).unwrap_or("");
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(value[0]["text"], "rebuild done");
        assert_eq!(value[0]["source"], "runtime");
        assert_eq!(value[0]["level"], "info");

        let mut panel = TcpStream::connect(("127.0.0.1", port)).unwrap();
        panel
            .write_all(b"GET /__rocci/dev?tab=console HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut html = String::new();
        panel.read_to_string(&mut html).unwrap();
        assert!(html.contains("rebuild done"), "{html}");
        assert!(html.contains("data-level=\"info\""));

        let mut events = TcpStream::connect(("127.0.0.1", port)).unwrap();
        events
            .set_read_timeout(Some(Duration::from_millis(400)))
            .unwrap();
        events
            .write_all(b"GET /__rocci/logs/events HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut headers = [0u8; 512];
        let n = events.read(&mut headers).unwrap_or(0);
        let head = String::from_utf8_lossy(&headers[..n]);
        assert!(head.contains("text/event-stream"), "{head}");
        assert!(head.contains("event: log") || head.contains("HTTP/1.1 200 OK"));
    }
}
