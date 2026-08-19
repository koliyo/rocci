use std::{path::Path, time::Duration};

use rocci_template::{CompileOutput, LowerOptions, SourceFile, compile, format_ast};
use serde::{Deserialize, Serialize};

use crate::playground_html;
use crate::profile::ProfileSnapshot;

pub const HTML_NOT_CAPTURED: &str = "HTML snapshot was not captured for this route.";
pub const AST_UNAVAILABLE_OKF: &str = "OKF records are not Rocci or Rocdown syntax trees.";
pub const ROC_UNAVAILABLE_OKF: &str =
    "OKF preview does not expose a Rocci/Rocdown compiled module.";
pub const HTML_NOT_BUILT: &str = "Built HTML was not found for this route.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectView {
    Source,
    Ast,
    Roc,
    Html,
}

impl InspectView {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("ast") => Self::Ast,
            Some("roc") => Self::Roc,
            Some("html") => Self::Html,
            _ => Self::Source,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Ast => "ast",
            Self::Roc => "roc",
            Self::Html => "html",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewCapability {
    pub available: bool,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub reason: String,
}

impl ViewCapability {
    pub fn available() -> Self {
        Self {
            available: true,
            reason: String::new(),
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            available: false,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectCapabilities {
    pub source: ViewCapability,
    pub ast: ViewCapability,
    pub roc: ViewCapability,
    pub html: ViewCapability,
}

impl InspectCapabilities {
    pub fn all_available() -> Self {
        Self {
            source: ViewCapability::available(),
            ast: ViewCapability::available(),
            roc: ViewCapability::available(),
            html: ViewCapability::available(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectPage {
    pub route: String,
    pub path: String,
    pub language: String,
    pub source: String,
    pub ast: String,
    pub roc: String,
    pub html: String,
    pub capabilities: InspectCapabilities,
}

impl InspectPage {
    pub fn from_views(
        route: impl Into<String>,
        path: impl Into<String>,
        language: impl Into<String>,
        source: Result<String, String>,
        ast: Result<String, String>,
        roc: Result<String, String>,
        html: Result<String, String>,
    ) -> Self {
        let (source, source_cap) = split_view(source);
        let (ast, ast_cap) = split_view(ast);
        let (roc, roc_cap) = split_view(roc);
        let (html, html_cap) = split_view(html);
        Self {
            route: route.into(),
            path: path.into(),
            language: language.into(),
            source,
            ast,
            roc,
            html,
            capabilities: InspectCapabilities {
                source: source_cap,
                ast: ast_cap,
                roc: roc_cap,
                html: html_cap,
            },
        }
    }

    pub fn from_rocci_compile(
        route: &str,
        path: &str,
        source: &str,
        compiled: &CompileOutput,
    ) -> Self {
        let ast = format_ast(source, &compiled.document);
        let roc = if compiled.has_errors() {
            Err("Generated Roc is unavailable because the template has errors.".into())
        } else {
            Ok(compiled.roc.clone())
        };
        let html = html_capability_for_rocci(path, compiled);
        Self::from_views(
            route,
            path,
            language_for_path(path),
            Ok(source.to_string()),
            Ok(ast),
            roc,
            html,
        )
    }

    pub fn from_rocci_file(route: &str, path: &Path) -> std::io::Result<Self> {
        let src = std::fs::read_to_string(path)?;
        let name = path.display().to_string();
        let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
        Ok(Self::from_rocci_compile(route, &name, &src, &compiled))
    }

    pub fn from_rocdown(
        route: &str,
        path: &str,
        source: String,
        ast: String,
        roc: String,
        html: Option<String>,
    ) -> Self {
        let html = match html {
            Some(body) if !body.is_empty() => Ok(body),
            _ => Err(HTML_NOT_BUILT.to_string()),
        };
        Self::from_views(
            route,
            path,
            language_for_path(path),
            Ok(source),
            Ok(ast),
            Ok(roc),
            html,
        )
    }

    pub fn from_okf(route: &str, path: &str, source: String, html: Option<String>) -> Self {
        let html = match html {
            Some(body) if !body.is_empty() => Ok(body),
            _ => Err(HTML_NOT_BUILT.to_string()),
        };
        Self::from_views(
            route,
            path,
            language_for_path(path),
            Ok(source),
            Err(AST_UNAVAILABLE_OKF.to_string()),
            Err(ROC_UNAVAILABLE_OKF.to_string()),
            html,
        )
    }

    pub fn capture_html_from_origin(&mut self, origin: &str) {
        if self.capabilities.html.available {
            return;
        }
        let url = inspect_origin_url(origin, &self.route);
        let Ok(response) = ureq::get(&url).timeout(Duration::from_secs(2)).call() else {
            return;
        };
        if response.status() != 200 {
            return;
        }
        let Ok(body) = response.into_string() else {
            return;
        };
        if body.is_empty() {
            return;
        }
        self.html = body;
        self.capabilities.html = ViewCapability::available();
    }

    pub fn body_for(&self, view: InspectView) -> &str {
        match view {
            InspectView::Source => &self.source,
            InspectView::Ast => &self.ast,
            InspectView::Roc => &self.roc,
            InspectView::Html => &self.html,
        }
    }

    pub fn capability_for(&self, view: InspectView) -> &ViewCapability {
        match view {
            InspectView::Source => &self.capabilities.source,
            InspectView::Ast => &self.capabilities.ast,
            InspectView::Roc => &self.capabilities.roc,
            InspectView::Html => &self.capabilities.html,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectSnapshot {
    #[serde(default)]
    pub pages: Vec<InspectPage>,
    #[serde(default)]
    pub profile: ProfileSnapshot,
}

impl From<ProfileSnapshot> for InspectSnapshot {
    fn from(profile: ProfileSnapshot) -> Self {
        Self {
            pages: Vec::new(),
            profile,
        }
    }
}

impl InspectSnapshot {
    pub fn from_profile(profile: ProfileSnapshot) -> Self {
        Self::from(profile)
    }

    pub fn resolve<'a>(&'a self, route: Option<&str>) -> Result<&'a InspectPage, String> {
        let requested = normalize_route(route.unwrap_or(""));
        if let Some(page) = find_page(&self.pages, &requested) {
            return Ok(page);
        }
        Err(requested)
    }

    pub fn inspect_json(&self, route: Option<&str>) -> (u16, String) {
        match self.resolve(route) {
            Ok(page) => (200, serialize_page(page, &self.profile)),
            Err(route) => (404, not_found_json(&route)),
        }
    }

    pub fn with_pages(profile: ProfileSnapshot, pages: Vec<InspectPage>) -> Self {
        Self { pages, profile }
    }

    pub fn capture_html_from_origin(&mut self, origin: &str) {
        for page in &mut self.pages {
            page.capture_html_from_origin(origin);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectQuery {
    pub route: Option<String>,
    pub view: InspectView,
}

pub fn parse_inspect_query(target: &str) -> InspectQuery {
    let query = target.split_once('?').map(|(_, query)| query).unwrap_or("");
    let mut route = None;
    let mut view = None;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match key {
            "route" => route = Some(percent_decode(value)),
            "view" => view = Some(percent_decode(value)),
            _ => {}
        }
    }
    InspectQuery {
        route: route.filter(|value| !value.is_empty()),
        view: InspectView::parse(view.as_deref()),
    }
}

pub fn inspect_json(snapshot: Option<&InspectSnapshot>, target: &str) -> (u16, String) {
    let query = parse_inspect_query(target);
    match snapshot {
        Some(snapshot) => snapshot.inspect_json(query.route.as_deref()),
        None => {
            let route = normalize_route(query.route.as_deref().unwrap_or(""));
            (404, not_found_json(&route))
        }
    }
}

fn split_view(value: Result<String, String>) -> (String, ViewCapability) {
    match value {
        Ok(body) => (body, ViewCapability::available()),
        Err(reason) => (String::new(), ViewCapability::unavailable(reason)),
    }
}

pub fn language_for_path(path: &str) -> &'static str {
    if path.ends_with(".rocci") {
        "rocci"
    } else if path.ends_with(".rocdown") {
        "rocdown"
    } else {
        "markdown"
    }
}

fn html_capability_for_rocci(path: &str, compiled: &CompileOutput) -> Result<String, String> {
    if compiled.has_errors() {
        return Err("Fix template errors before HTML can be rendered.".into());
    }
    let type_name = rocci_template::type_name_from_path(Path::new(path));
    match playground_html::select_html_target(
        &compiled.document,
        &compiled.components,
        &compiled.fixtures,
        &type_name,
    ) {
        Ok(_) => Err(HTML_NOT_CAPTURED.to_string()),
        Err(reason) => Err(reason),
    }
}

fn inspect_origin_url(origin: &str, route: &str) -> String {
    let origin = origin.trim_end_matches('/');
    if route.is_empty() || route == "/" {
        format!("{origin}/")
    } else if route.starts_with('/') {
        format!("{origin}{route}")
    } else {
        format!("{origin}/{route}")
    }
}

fn serialize_page(page: &InspectPage, profile: &ProfileSnapshot) -> String {
    serde_json::to_string(&InspectJson {
        route: &page.route,
        path: &page.path,
        language: &page.language,
        source: &page.source,
        ast: &page.ast,
        roc: &page.roc,
        html: &page.html,
        capabilities: &page.capabilities,
        profile,
    })
    .unwrap_or_else(|_| "{}".to_string())
}

fn not_found_json(route: &str) -> String {
    serde_json::json!({
        "error": "route not found",
        "route": route,
    })
    .to_string()
}

#[derive(Serialize)]
struct InspectJson<'a> {
    route: &'a str,
    path: &'a str,
    language: &'a str,
    source: &'a str,
    ast: &'a str,
    roc: &'a str,
    html: &'a str,
    capabilities: &'a InspectCapabilities,
    profile: &'a ProfileSnapshot,
}

fn normalize_route(route: &str) -> String {
    let trimmed = route.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

fn find_page<'a>(pages: &'a [InspectPage], route: &str) -> Option<&'a InspectPage> {
    pages.iter().find(|page| page.route == route).or_else(|| {
        let alt = if route.len() > 1 && route.ends_with('/') {
            route.trim_end_matches('/').to_string()
        } else if route != "/" {
            format!("{route}/")
        } else {
            return None;
        };
        pages.iter().find(|page| page.route == alt)
    })
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let before = i;
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((high << 4) | low);
                i += 3;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
        if i <= before {
            i = before + 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{ProfileSpan, SpanRecorder};

    fn sample_page() -> InspectPage {
        InspectPage {
            route: "/".into(),
            path: "App.rocci".into(),
            language: "rocci".into(),
            source: "<div class=\"x\">alert(\"hi\") & more</div>".into(),
            ast: "(Document ...)".into(),
            roc: "app [html] {}".into(),
            html: String::new(),
            capabilities: InspectCapabilities {
                source: ViewCapability::available(),
                ast: ViewCapability::available(),
                roc: ViewCapability::available(),
                html: ViewCapability::unavailable("HTML snapshot was not captured for this route."),
            },
        }
    }

    fn sample_snapshot() -> InspectSnapshot {
        let mut recorder = SpanRecorder::new();
        recorder.push("parse", 4, None);
        InspectSnapshot {
            pages: vec![
                sample_page(),
                InspectPage {
                    route: "/guide/".into(),
                    path: "guide.rocdown".into(),
                    language: "rocdown".into(),
                    source: "# Guide".into(),
                    ast: "(Rocdown ...)".into(),
                    roc: "module [] {}".into(),
                    html: "<h1>Guide</h1>".into(),
                    capabilities: InspectCapabilities::all_available(),
                },
            ],
            profile: recorder.finish(),
        }
    }

    #[test]
    fn inspect_json_includes_required_keys_and_capability_reasons() {
        let snapshot = sample_snapshot();
        let (status, body) = snapshot.inspect_json(Some("/"));
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        for key in [
            "route",
            "path",
            "language",
            "source",
            "ast",
            "roc",
            "html",
            "capabilities",
            "profile",
        ] {
            assert!(value.get(key).is_some(), "missing {key} in {body}");
        }
        assert_eq!(value["route"], "/");
        assert_eq!(value["path"], "App.rocci");
        assert_eq!(value["language"], "rocci");
        assert_eq!(value["capabilities"]["html"]["available"], false);
        assert_eq!(
            value["capabilities"]["html"]["reason"],
            "HTML snapshot was not captured for this route."
        );
        assert_eq!(value["html"], "");
        assert_eq!(value["capabilities"]["source"]["available"], true);
        assert_eq!(value["profile"]["total_ms"], 4);
        assert!(
            value["profile"]["spans"]
                .as_array()
                .is_some_and(|spans| spans.iter().any(|span| span["name"] == "parse"))
        );
    }

    #[test]
    fn inspect_json_does_not_html_escape_source() {
        let snapshot = sample_snapshot();
        let (_, body) = snapshot.inspect_json(Some("/"));
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            value["source"].as_str().unwrap(),
            "<div class=\"x\">alert(\"hi\") & more</div>"
        );
        assert!(body.contains("<div class=\\\"x\\\">alert(\\\"hi\\\") & more</div>"));
        assert!(!body.contains("&lt;div"));
        assert!(!body.contains("&amp; more"));
    }

    #[test]
    fn empty_or_missing_route_falls_back_to_entry() {
        let snapshot = sample_snapshot();
        let (status, body) = snapshot.inspect_json(None);
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["route"], "/");
        assert_eq!(value["path"], "App.rocci");

        let (status, body) = inspect_json(Some(&snapshot), "/__rocci/inspect?route=");
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["path"], "App.rocci");
    }

    #[test]
    fn unknown_route_returns_404_json() {
        let snapshot = sample_snapshot();
        let (status, body) = snapshot.inspect_json(Some("/missing"));
        assert_eq!(status, 404);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["error"], "route not found");
        assert_eq!(value["route"], "/missing");
        assert!(value.get("source").is_none());
    }

    #[test]
    fn missing_snapshot_is_404() {
        let (status, body) = inspect_json(None, "/__rocci/inspect?route=/");
        assert_eq!(status, 404);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["route"], "/");
    }

    #[test]
    fn trailing_slash_and_percent_encoded_routes_resolve() {
        let snapshot = sample_snapshot();
        let (status, body) = inspect_json(Some(&snapshot), "/__rocci/inspect?route=/guide");
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["path"], "guide.rocdown");

        let (status, body) = inspect_json(
            Some(&snapshot),
            "/__rocci/inspect?route=%2Fguide%2F&view=ast",
        );
        assert_eq!(status, 200);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["ast"], "(Rocdown ...)");
    }

    #[test]
    fn unknown_view_falls_back_to_source() {
        assert_eq!(
            parse_inspect_query("/__rocci/inspect?view=nope").view,
            InspectView::Source
        );
        assert_eq!(
            parse_inspect_query("/__rocci/inspect?view=roc").view,
            InspectView::Roc
        );
    }

    #[test]
    fn profile_only_snapshot_has_no_pages() {
        let snapshot = InspectSnapshot::from(ProfileSnapshot {
            total_ms: 1,
            spans: vec![ProfileSpan {
                name: "load".into(),
                duration_ms: 1,
                note: None,
            }],
        });
        assert!(snapshot.pages.is_empty());
        let (status, _) = snapshot.inspect_json(Some("/"));
        assert_eq!(status, 404);
    }

    #[test]
    fn counter_inspect_json_fills_source_ast_and_roc() {
        use crate::inspector::InspectorServer;
        use std::io::{Read, Write};
        use std::net::TcpStream;

        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/counter/Counter.rocci");
        let page = InspectPage::from_rocci_file("/", &path).unwrap();
        assert_eq!(page.language, "rocci");
        assert!(page.path.ends_with("Counter.rocci"), "{}", page.path);
        assert!(page.source.contains("@on:get(\"/\")"), "{}", page.source);
        assert!(page.ast.contains("(module"), "{}", page.ast);
        assert!(page.ast.contains("(on"), "{}", page.ast);
        assert!(page.roc.contains("counterPage"), "{}", page.roc);
        assert!(!page.capabilities.html.available);
        assert!(
            page.capabilities
                .html
                .reason
                .contains("HTML preview needs a @fixture")
                || page.capabilities.html.reason == HTML_NOT_CAPTURED,
            "{}",
            page.capabilities.html.reason
        );

        let snapshot = InspectSnapshot::with_pages(ProfileSnapshot::default(), vec![page]);
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
        let body = response.split("\r\n\r\n").nth(1).unwrap_or("");
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(value["language"], "rocci");
        assert_eq!(value["capabilities"]["source"]["available"], true);
        assert_eq!(value["capabilities"]["ast"]["available"], true);
        assert_eq!(value["capabilities"]["roc"]["available"], true);
        assert_eq!(value["capabilities"]["html"]["available"], false);
        assert!(value["source"].as_str().unwrap().contains("@on:get(\"/\")"));
        assert!(value["ast"].as_str().unwrap().contains("(on"));
        assert!(value["roc"].as_str().unwrap().contains("counterPage"));
    }
}
