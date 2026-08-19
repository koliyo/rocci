use rocci_template::{Diagnostic, DiagnosticFrame, SourceFile, format_diagnostic};

pub use rocci_template::{MappedModule, remap_roc_output};

const METHOD_PLACEHOLDER: &str = "ROCCI_DEV_METHOD";
const PATH_PLACEHOLDER: &str = "ROCCI_DEV_PATH";
const HANDLER_PLACEHOLDER: &str = "ROCCI_DEV_HANDLER";
const ERROR_PLACEHOLDER: &str = "ROCCI_DEV_ERROR";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedRoute {
    pub method: String,
    pub path: String,
    pub handler: String,
}

impl ListedRoute {
    pub fn new(
        method: impl Into<String>,
        path: impl Into<String>,
        handler: impl Into<String>,
    ) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            handler: handler.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FailedFile {
    pub name: String,
    pub src: String,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            ch => out.push(ch),
        }
    }
    out
}

pub fn suggest_path(requested: &str, routes: &[ListedRoute]) -> Option<String> {
    slash_alternates(routes)
        .into_iter()
        .find(|(from, _)| from == requested)
        .map(|(_, to)| to)
}

/// Alternate GET path that should 308 (or hint) to the registered form.
///
/// `/page` and `/page/` are distinct URLs: relative links resolve differently.
/// When only one form is registered, the other is this alternate. Both stay
/// distinct when each is registered. `/` has no alternate.
pub fn slash_alternates(routes: &[ListedRoute]) -> Vec<(String, String)> {
    let gets: Vec<&str> = routes
        .iter()
        .filter(|route| route.method == "GET")
        .map(|route| route.path.as_str())
        .collect();
    let mut arms = Vec::new();
    for path in &gets {
        if *path == "/" {
            continue;
        }
        let from = if let Some(stripped) = path.strip_suffix('/') {
            if stripped.is_empty() {
                "/".to_string()
            } else {
                stripped.to_string()
            }
        } else {
            format!("{path}/")
        };
        if !gets.contains(&from.as_str()) {
            arms.push((from, (*path).to_string()));
        }
    }
    arms
}

pub fn render_not_found(method: &str, path: &str, routes: &[ListedRoute]) -> String {
    let hint = suggest_path(path, routes);
    let mut body = String::new();
    body.push_str("<p class=\"lead\">No handler for this request.</p>");
    body.push_str("<p class=\"request\"><code>");
    body.push_str(&html_escape(method));
    body.push(' ');
    body.push_str(&html_escape(path));
    body.push_str("</code></p>");
    if let Some(hint) = &hint {
        body.push_str("<p class=\"hint\">Did you mean <a href=\"");
        body.push_str(&html_escape(hint));
        body.push_str("\"><code>");
        body.push_str(&html_escape(hint));
        body.push_str("</code></a>?</p>");
    }
    if routes.is_empty() {
        body.push_str("<p class=\"muted\">No routes are registered.</p>");
    } else {
        body.push_str("<h2>Registered routes</h2><table><thead><tr><th>Method</th><th>Path</th><th>Handler</th></tr></thead><tbody>");
        for route in routes {
            body.push_str("<tr><td><code>");
            body.push_str(&html_escape(&route.method));
            body.push_str("</code></td><td>");
            if route.method == "GET" {
                body.push_str("<a href=\"");
                body.push_str(&html_escape(&route.path));
                body.push_str("\"><code>");
                body.push_str(&html_escape(&route.path));
                body.push_str("</code></a>");
            } else {
                body.push_str("<code>");
                body.push_str(&html_escape(&route.path));
                body.push_str("</code>");
            }
            body.push_str("</td><td><code>");
            body.push_str(&html_escape(&route.handler));
            body.push_str("</code></td></tr>");
        }
        body.push_str("</tbody></table>");
    }
    document(
        "404",
        "Not Found",
        "This route is not registered.",
        &body,
        "404",
    )
}

pub fn render_handler_error(method: &str, path: &str, handler: &str, error: &str) -> String {
    let body = format!(
        "<p class=\"lead\">The route handler returned an error.</p>\
         <dl class=\"meta\">\
         <div><dt>Request</dt><dd><code>{method} {path}</code></dd></div>\
         <div><dt>Handler</dt><dd><code>{handler}</code></dd></div>\
         </dl>\
         <h2>Error</h2><pre>{error}</pre>",
        method = html_escape(method),
        path = html_escape(path),
        handler = html_escape(handler),
        error = html_escape(error),
    );
    document(
        "500",
        "Handler failed",
        "The route handler returned an error.",
        &body,
        "500",
    )
}

pub fn render_handler_overlay(method: &str, path: &str, handler: &str, error: &str) -> String {
    format!(
        "<div id=\"rocci-dev-error\" style=\"position:fixed;inset:auto 1rem 1rem 1rem;z-index:2147483647;max-height:45vh;overflow:auto;padding:1rem 1.1rem;border:1px solid #5c2a32;border-radius:12px;background:#1b1416;color:#f4e8ea;font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,monospace;box-shadow:0 18px 50px rgba(0,0,0,.45)\">\
         <div style=\"font:700 11px/1.2 ui-sans-serif,system-ui,sans-serif;letter-spacing:.08em;text-transform:uppercase;color:#ff7b8a;margin-bottom:.4rem\">Handler failed</div>\
         <div style=\"margin-bottom:.55rem\"><code>{method} {path}</code> · <code>{handler}</code></div>\
         <pre style=\"margin:0;white-space:pre-wrap\">{error}</pre></div>",
        method = html_escape(method),
        path = html_escape(path),
        handler = html_escape(handler),
        error = html_escape(error),
    )
}

pub fn format_template_errors(files: &[FailedFile]) -> String {
    let mut out = String::new();
    for file in files {
        let source = SourceFile::new(&file.name, &file.src);
        for diagnostic in &file.diagnostics {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format_diagnostic(source, diagnostic));
            out.push('\n');
        }
    }
    out
}

pub fn eprint_template_errors(files: &[FailedFile]) {
    let text = format_template_errors(files);
    if !text.is_empty() {
        eprint!("{text}");
    }
}

pub fn render_template_errors(files: &[FailedFile]) -> String {
    let mut body = String::from(
        "<p class=\"lead\">Fix the diagnostics below and rerun <code>rocci run</code>.</p>",
    );
    let mut count = 0usize;
    for file in files {
        let source = SourceFile::new(&file.name, &file.src);
        for diagnostic in &file.diagnostics {
            count += 1;
            body.push_str(&frame_html(&DiagnosticFrame::from_source(
                source, diagnostic,
            )));
        }
    }
    if count == 0 {
        body.push_str("<p class=\"muted\">Compilation failed without diagnostics.</p>");
    }
    document(
        "Compile",
        "Template error",
        "Rocci could not compile this module.",
        &body,
        "compile",
    )
}

pub fn render_roc_compile_error(output: &str, modules: &[MappedModule]) -> String {
    let mut body = String::from(
        "<p class=\"lead\">Roc rejected the generated program. The compiler output is below, with source locations remapped when possible.</p>",
    );
    let mapped = remap_roc_output(output, modules);
    if !mapped.is_empty() {
        body.push_str("<h2>Source</h2>");
        for frame in &mapped {
            body.push_str(&frame_html(frame));
        }
    }
    body.push_str("<h2>Compiler output</h2><pre>");
    body.push_str(&html_escape(output.trim()));
    body.push_str("</pre>");
    document(
        "Compile",
        "Roc compile error",
        "Roc rejected the generated program.",
        &body,
        "roc",
    )
}

pub fn roc_runtime_helpers(routes: &[ListedRoute]) -> String {
    let mut out = String::new();
    out.push_str(
        r#"
html_status = |status, body|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers([{ name: "Content-Type", value: "text/html; charset=utf-8" }])
            .with_body(Str.to_utf8(body)),
        ),
    )

html_join = |parts, sep|
    List.fold(
        List.drop_first(parts, 1),
        List.get(parts, 0) ?? "",
        |acc, part| "${acc}${sep}${part}",
    )

html_escape = |text| {
    amp = html_join(Str.split_on(text, "&"), "&amp;")
    lt = html_join(Str.split_on(amp, "<"), "&lt;")
    gt = html_join(Str.split_on(lt, ">"), "&gt;")
    quot = html_join(Str.split_on(gt, "\""), "&quot;")
    html_join(Str.split_on(quot, "'"), "&#39;")
}

"#,
    );
    out.push_str(&roc_suggest_path(routes));
    out.push_str(&roc_not_found_fn(routes));
    out.push_str(&roc_interpolated_fn(
        "handler_error_html",
        "|method, path, handler, err|",
        &render_handler_error(
            METHOD_PLACEHOLDER,
            PATH_PLACEHOLDER,
            HANDLER_PLACEHOLDER,
            ERROR_PLACEHOLDER,
        ),
        &[
            (METHOD_PLACEHOLDER, "${html_escape(method)}"),
            (PATH_PLACEHOLDER, "${html_escape(path)}"),
            (HANDLER_PLACEHOLDER, "${html_escape(handler)}"),
            (ERROR_PLACEHOLDER, "${html_escape(err)}"),
        ],
    ));
    out.push_str(&roc_interpolated_fn(
        "handler_error_overlay_str",
        "|method, path, handler, err|",
        &render_handler_overlay(
            METHOD_PLACEHOLDER,
            PATH_PLACEHOLDER,
            HANDLER_PLACEHOLDER,
            ERROR_PLACEHOLDER,
        ),
        &[
            (METHOD_PLACEHOLDER, "${html_escape(method)}"),
            (PATH_PLACEHOLDER, "${html_escape(path)}"),
            (HANDLER_PLACEHOLDER, "${html_escape(handler)}"),
            (ERROR_PLACEHOLDER, "${html_escape(err)}"),
        ],
    ));
    out.push_str(
        r#"
error_overlay_html = |method, path, handler, err|
    Html.dangerously_include_unescaped_html(handler_error_overlay_str(method, path, handler, err))
"#,
    );
    out
}

pub fn roc_not_found_arm() -> &'static str {
    r#"        _ =>
            html_status(404, not_found_html(Method.to_str(request.method()), path))
"#
}

fn roc_not_found_fn(routes: &[ListedRoute]) -> String {
    let mut html = render_not_found(METHOD_PLACEHOLDER, PATH_PLACEHOLDER, routes);
    let insert_at = html
        .find("<h2>Registered routes</h2>")
        .or_else(|| html.find("<p class=\"muted\">No routes are registered.</p>"))
        .unwrap_or(html.len());
    html.insert_str(insert_at, "ROCCI_DEV_HINT");
    let mut contents = roc_escape_contents(&html.replace("${", "&#36;{"));
    contents = contents.replace(METHOD_PLACEHOLDER, "${html_escape(method)}");
    contents = contents.replace(PATH_PLACEHOLDER, "${html_escape(path)}");
    contents = contents.replace("ROCCI_DEV_HINT", "${hint}");
    format!(
        "not_found_html = |method, path| {{\n    hint =\n        match suggest_path(path) {{\n            Ok(target) => \"<p class=\\\"hint\\\">Did you mean <a href=\\\"${{html_escape(target)}}\\\"><code>${{html_escape(target)}}</code></a>?</p>\"\n            Err(_) => \"\"\n        }}\n    \"{contents}\"\n}}\n\n"
    )
}

fn roc_suggest_path(routes: &[ListedRoute]) -> String {
    let arms = slash_alternates(routes);
    if arms.is_empty() {
        return "suggest_path = |_| Err({})\n\n".to_string();
    }
    let mut out = String::from("suggest_path = |path|\n    match path {\n");
    for (from, to) in arms {
        out.push_str("        \"");
        out.push_str(&roc_escape_contents(&from));
        out.push_str("\" => Ok(\"");
        out.push_str(&roc_escape_contents(&to));
        out.push_str("\")\n");
    }
    out.push_str("        _ => Err({})\n    }\n\n");
    out
}

pub fn roc_slash_redirect_arms(routes: &[ListedRoute]) -> String {
    let mut out = String::new();
    for (from, to) in slash_alternates(routes) {
        out.push_str("        (\"GET\", \"");
        out.push_str(&roc_escape_contents(&from));
        out.push_str("\") =>\n            redirect_slash(\"");
        out.push_str(&roc_escape_contents(&to));
        out.push_str("\")\n");
    }
    out
}

pub fn roc_redirect_slash_binding() -> &'static str {
    r#"
    redirect_slash = |target| {
        location =
            match request.target() {
                Resource({ raw_query: Present(q), .. }) =>
                    match q {
                        "" => target
                        _ => "${target}?${q}"
                    }
                _ => target
            }
        Ok(
            Server.respond(
                Response.from_status(308)
                .with_headers([{ name: "Location", value: location }])
                .with_body([]),
            ),
        )
    }

"#
}

fn roc_interpolated_fn(
    name: &str,
    params: &str,
    html: &str,
    replacements: &[(&str, &str)],
) -> String {
    let mut contents = roc_escape_contents(&html.replace("${", "&#36;{"));
    for (placeholder, expr) in replacements {
        contents = contents.replace(placeholder, expr);
    }
    format!("{name} = {params}\n    \"{contents}\"\n\n")
}

fn roc_escape_contents(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out
}

fn frame_html(frame: &DiagnosticFrame) -> String {
    format!(
        "<section class=\"frame {kind}\">\
         <div class=\"frame-head\"><span class=\"pill\">{kind}</span> {message}</div>\
         <div class=\"loc\">{file}:{line}:{column}</div>\
         <pre class=\"code\"><span class=\"gutter\">{line}</span><span class=\"src\">{source}</span>\n<span class=\"gutter\"></span><span class=\"caret\">{caret}</span></pre>\
         </section>",
        kind = html_escape(frame.severity_label()),
        message = html_escape(&frame.message),
        file = html_escape(&frame.file),
        line = frame.line,
        column = frame.column,
        source = html_escape(&frame.source_line),
        caret = html_escape(&frame.caret_line()),
    )
}

fn document(code: &str, title: &str, summary: &str, body: &str, kind: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title} · rocci</title>\n<style>{css}</style>\n</head>\n<body class=\"{kind}\">\n<main>\n<p class=\"brand\">rocci</p>\n<p class=\"code-mark\">{code}</p>\n<h1>{title}</h1>\n<p class=\"summary\">{summary}</p>\n{body}\n</main>\n</body>\n</html>\n",
        title = html_escape(title),
        css = ERROR_CSS,
        kind = html_escape(kind),
        code = html_escape(code),
        summary = html_escape(summary),
        body = body,
    )
}

const ERROR_CSS: &str = r#"
:root {
  color-scheme: dark light;
  --bg: #141218;
  --fg: #f4eef2;
  --muted: #b7a8b0;
  --card: #1d1820;
  --line: #3a3036;
  --accent: #ff7b8a;
  --warn: #f5c36e;
  --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  --sans: ui-sans-serif, system-ui, sans-serif;
}
@media (prefers-color-scheme: light) {
  :root {
    --bg: #f7f3f4;
    --fg: #1b1418;
    --muted: #6b5c63;
    --card: #fff;
    --line: #e4d9de;
    --accent: #c42b45;
    --warn: #8a5a00;
  }
}
html, body { margin: 0; background: var(--bg); color: var(--fg); font: 16px/1.5 var(--sans); }
main { max-width: 52rem; margin: 0 auto; padding: 2.5rem 1.4rem 4rem; }
.brand { margin: 0; letter-spacing: .16em; text-transform: uppercase; font-size: .72rem; color: var(--muted); }
.code-mark { margin: .4rem 0 0; font: 700 3.2rem/1 var(--sans); letter-spacing: -.04em; }
body.compile .code-mark, body.roc .code-mark, body.500 .code-mark { color: var(--accent); }
body.404 .code-mark { color: var(--warn); }
h1 { margin: .2rem 0 .4rem; font-size: 1.65rem; }
.summary, .lead, .muted, .hint { color: var(--muted); }
.request code, .meta code, table code { font-family: var(--mono); font-size: .92em; }
.meta { display: grid; gap: .6rem; margin: 1.2rem 0; }
.meta div { background: var(--card); border: 1px solid var(--line); border-radius: 10px; padding: .7rem .9rem; }
dt { font-size: .75rem; text-transform: uppercase; letter-spacing: .08em; color: var(--muted); }
dd { margin: .15rem 0 0; }
h2 { margin: 1.8rem 0 .7rem; font-size: .82rem; text-transform: uppercase; letter-spacing: .08em; color: var(--muted); }
table { width: 100%; border-collapse: collapse; background: var(--card); border: 1px solid var(--line); border-radius: 12px; overflow: hidden; }
th, td { text-align: left; padding: .55rem .8rem; border-bottom: 1px solid var(--line); }
th { font-size: .72rem; text-transform: uppercase; letter-spacing: .06em; color: var(--muted); }
a { color: inherit; }
pre { background: var(--card); border: 1px solid var(--line); border-radius: 12px; padding: .9rem 1rem; overflow: auto; font: 13px/1.45 var(--mono); white-space: pre-wrap; }
.frame { margin: 1rem 0 1.3rem; }
.frame-head { font-weight: 650; }
.pill { display: inline-block; margin-right: .35rem; padding: .1rem .4rem; border-radius: 999px; background: color-mix(in srgb, var(--accent) 18%, transparent); color: var(--accent); font-size: .72rem; text-transform: uppercase; letter-spacing: .06em; }
.loc { color: var(--muted); font: 12px/1.4 var(--mono); margin: .25rem 0 .45rem; }
pre.code { display: grid; grid-template-columns: auto 1fr; column-gap: .75rem; }
.gutter { color: var(--muted); text-align: right; user-select: none; }
.caret { color: var(--accent); }
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_template::{Diagnostic, Span};

    fn route(method: &str, path: &str, handler: &str) -> ListedRoute {
        ListedRoute::new(method, path, handler)
    }

    #[test]
    fn not_found_lists_routes_and_escapes() {
        let html = render_not_found(
            "GET",
            "/<script>",
            &[
                route("GET", "/", "Home.on_get_root!"),
                route("GET", "/about/", "About.on_get_about!"),
            ],
        );
        assert!(html.contains("404"));
        assert!(html.contains("/about/"));
        assert!(html.contains("Home.on_get_root!"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("text/html") || html.contains("Not Found"));
    }

    #[test]
    fn not_found_hints_trailing_slash() {
        let routes = [route("GET", "/about/", "About.on_get_about!")];
        assert_eq!(suggest_path("/about", &routes).as_deref(), Some("/about/"));
        assert_eq!(suggest_path("/about/", &routes), None);
        let html = render_not_found("GET", "/about", &routes);
        assert!(html.contains("Did you mean"));
        assert!(html.contains("/about/"));
    }

    #[test]
    fn slash_alternates_follow_the_registered_form() {
        let with_slash = [route("GET", "/dx/", "Dx.on_get_dx!")];
        assert_eq!(
            slash_alternates(&with_slash),
            vec![("/dx".into(), "/dx/".into())]
        );
        let without = [route("GET", "/dx", "Dx.on_get_dx!")];
        assert_eq!(
            slash_alternates(&without),
            vec![("/dx/".into(), "/dx".into())]
        );
        let both = [
            route("GET", "/dx", "Dx.on_get_dx!"),
            route("GET", "/dx/", "Other.on_get_dx_slash!"),
        ];
        assert!(slash_alternates(&both).is_empty());
        assert!(slash_alternates(&[route("GET", "/", "Home.on_get_root!")]).is_empty());
    }

    #[test]
    fn slash_redirect_arms_are_get_308() {
        let routes = [route("GET", "/dx/", "Dx.on_get_dx!")];
        let arms = roc_slash_redirect_arms(&routes);
        assert!(arms.contains("(\"GET\", \"/dx\") =>"));
        assert!(arms.contains("redirect_slash(\"/dx/\")"));
        assert!(!arms.contains("POST"));
    }

    #[test]
    fn template_error_page_contains_source_and_message() {
        let src = "@component Broken\n";
        let diagnostic = Diagnostic::error(Span::new(0, 10), "expected `=` after component name");
        let html = render_template_errors(&[FailedFile {
            name: "Page.rocci".into(),
            src: src.into(),
            diagnostics: vec![diagnostic],
        }]);
        assert!(html.contains("expected `=` after component name"));
        assert!(html.contains("Page.rocci"));
        assert!(html.contains("@component"));
        assert!(html.contains("^^^^^^^^^^"));
    }

    #[test]
    fn template_errors_format_rustc_style_frames() {
        let src = "@component Broken\n";
        let diagnostic = Diagnostic::error(Span::new(0, 10), "expected `=` after component name");
        let text = format_template_errors(&[FailedFile {
            name: "Page.rocci".into(),
            src: src.into(),
            diagnostics: vec![diagnostic],
        }]);
        assert!(text.contains("error: expected `=` after component name"));
        assert!(text.contains(" --> Page.rocci:1:1"));
        assert!(text.contains("@component"));
        assert!(text.contains("^^^^^^^^^^"));
    }

    #[test]
    fn handler_error_escapes_inspect_output() {
        let html = render_handler_error("POST", "/x", "Home.go!", "<boom>");
        assert!(html.contains("&lt;boom&gt;"));
        assert!(html.contains("Home.go!"));
        assert!(!html.contains("<boom>"));
    }

    #[test]
    fn roc_helpers_interpolate_request_fields() {
        let roc = roc_runtime_helpers(&[route("GET", "/about/", "About.on_get_about!")]);
        assert!(roc.contains("not_found_html = |method, path|"));
        assert!(roc.contains("${html_escape(method)}"));
        assert!(roc.contains("${html_escape(path)}"));
        assert!(roc.contains("/about/"));
        assert!(roc.contains("\"/about\" => Ok(\"/about/\")"));
        assert!(roc.contains("handler_error_html = |method, path, handler, err|"));
        assert!(roc.contains("List.drop_first(parts, 1)"));
        assert!(!roc.contains("Bool.false"));
        assert!(!roc.contains("ROCCI_DEV_METHOD"));
    }
}
