use rocci_template::{Diagnostic, DiagnosticFrame, SourceFile, format_diagnostic};

pub use rocci_template::{MappedModule, remap_roc_output};

const METHOD_PLACEHOLDER: &str = "ROCCI_DEV_METHOD";
const PATH_PLACEHOLDER: &str = "ROCCI_DEV_PATH";
const HANDLER_PLACEHOLDER: &str = "ROCCI_DEV_HANDLER";
const ERROR_PLACEHOLDER: &str = "ROCCI_DEV_ERROR";
const HINT_PLACEHOLDER: &str = "ROCCI_DEV_HINT";

const DOCUMENT_HTML: &str = include_str!("../templates/error/document.html");
const DOCUMENT_CSS: &str = include_str!("../templates/error/document.css");
const BUILD_DIALOG_HTML: &str = include_str!("../templates/error/build-dialog.html");
const HANDLER_OVERLAY_HTML: &str = include_str!("../templates/error/handler-overlay.html");
const HANDLER_ERROR_BODY: &str = include_str!("../templates/error/handler-error-body.html");
const NOT_FOUND_BODY: &str = include_str!("../templates/error/not-found-body.html");
const FRAME_HTML: &str = include_str!("../templates/error/frame.html");
const TEMPLATE_ERROR_BODY: &str = include_str!("../templates/error/template-error-body.html");
const ROC_COMPILE_BODY: &str = include_str!("../templates/error/roc-compile-body.html");
const BUILD_SHELL_BODY: &str = include_str!("../templates/error/build-shell-body.html");
const HINT_HTML: &str = include_str!("../templates/error/hint.html");
const ROUTES_TABLE: &str = include_str!("../templates/error/routes-table.html");

pub const ERROR_OVERLAY_CSS: &str = include_str!("../templates/error/overlay.css");

pub fn fill_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
    out
}

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
    let hint = match suggest_path(path, routes) {
        Some(target) => fill_template(HINT_HTML, &[("target", &html_escape(&target))]),
        None => String::new(),
    };
    not_found_document(method, path, &hint, routes)
}

fn not_found_document(method: &str, path: &str, hint: &str, routes: &[ListedRoute]) -> String {
    document(
        "404",
        "Not Found",
        "This route is not registered.",
        &not_found_body(method, path, hint, routes),
        "404",
    )
}

fn not_found_body(method: &str, path: &str, hint: &str, routes: &[ListedRoute]) -> String {
    fill_template(
        NOT_FOUND_BODY,
        &[
            ("method", &html_escape(method)),
            ("path", &html_escape(path)),
            ("hint", hint),
            ("routes", &routes_html(routes)),
        ],
    )
}

fn routes_html(routes: &[ListedRoute]) -> String {
    if routes.is_empty() {
        return "<p class=\"muted\">No routes are registered.</p>".to_string();
    }
    let mut rows = String::new();
    for route in routes {
        rows.push_str("<tr><td><code>");
        rows.push_str(&html_escape(&route.method));
        rows.push_str("</code></td><td>");
        if route.method == "GET" {
            rows.push_str("<a href=\"");
            rows.push_str(&html_escape(&route.path));
            rows.push_str("\"><code>");
            rows.push_str(&html_escape(&route.path));
            rows.push_str("</code></a>");
        } else {
            rows.push_str("<code>");
            rows.push_str(&html_escape(&route.path));
            rows.push_str("</code>");
        }
        rows.push_str("</td><td><code>");
        rows.push_str(&html_escape(&route.handler));
        rows.push_str("</code></td></tr>");
    }
    fill_template(ROUTES_TABLE, &[("rows", &rows)])
}

pub fn render_handler_error(method: &str, path: &str, handler: &str, error: &str) -> String {
    let body = fill_template(
        HANDLER_ERROR_BODY,
        &[
            ("method", &html_escape(method)),
            ("path", &html_escape(path)),
            ("handler", &html_escape(handler)),
            ("error", &html_escape(error)),
        ],
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
    fill_template(
        HANDLER_OVERLAY_HTML,
        &[
            ("method", &html_escape(method)),
            ("path", &html_escape(path)),
            ("handler", &html_escape(handler)),
            ("error", &html_escape(error)),
        ],
    )
}

pub fn render_build_error_dialog(message: &str) -> String {
    fill_template(BUILD_DIALOG_HTML, &[("message", &html_escape(message))])
}

pub fn inject_build_error_dialog(html: &str, message: &str) -> String {
    let overlay = render_build_error_dialog(message);
    if let Some(idx) = html.to_ascii_lowercase().rfind("</body>") {
        let mut out = String::with_capacity(html.len() + overlay.len());
        out.push_str(&html[..idx]);
        out.push_str(&overlay);
        out.push_str(&html[idx..]);
        out
    } else {
        format!("{html}{overlay}")
    }
}

pub fn render_build_error_shell(message: &str) -> String {
    document(
        "Build",
        "Build error",
        "The documentation preview could not finish a clean rebuild.",
        &fill_template(BUILD_SHELL_BODY, &[("message", &html_escape(message))]),
        "compile",
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
    let mut frames = String::new();
    let mut count = 0usize;
    for file in files {
        let source = SourceFile::new(&file.name, &file.src);
        for diagnostic in &file.diagnostics {
            count += 1;
            frames.push_str(&frame_html(&DiagnosticFrame::from_source(
                source, diagnostic,
            )));
        }
    }
    if count == 0 {
        frames.push_str("<p class=\"muted\">Compilation failed without diagnostics.</p>");
    }
    document(
        "Compile",
        "Template error",
        "Rocci could not compile this module.",
        &fill_template(TEMPLATE_ERROR_BODY, &[("frames", &frames)]),
        "compile",
    )
}

pub fn render_roc_compile_error(output: &str, modules: &[MappedModule]) -> String {
    let mapped = remap_roc_output(output, modules);
    let mut mapped_html = String::new();
    if !mapped.is_empty() {
        mapped_html.push_str("<h2>Source</h2>");
        for frame in &mapped {
            mapped_html.push_str(&frame_html(frame));
        }
    }
    document(
        "Compile",
        "Roc compile error",
        "Roc rejected the generated program.",
        &fill_template(
            ROC_COMPILE_BODY,
            &[
                ("mapped", &mapped_html),
                ("output", &html_escape(output.trim())),
            ],
        ),
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
    let html = not_found_document(
        METHOD_PLACEHOLDER,
        PATH_PLACEHOLDER,
        HINT_PLACEHOLDER,
        routes,
    );
    let mut contents = roc_escape_contents(&html.replace("${", "&#36;{"));
    contents = contents.replace(METHOD_PLACEHOLDER, "${html_escape(method)}");
    contents = contents.replace(PATH_PLACEHOLDER, "${html_escape(path)}");
    contents = contents.replace(HINT_PLACEHOLDER, "${hint}");
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
    fill_template(
        FRAME_HTML,
        &[
            ("kind", &html_escape(frame.severity_label())),
            ("label", &html_escape(&frame.kind_label())),
            ("message", &html_escape(&frame.message)),
            ("file", &html_escape(&frame.file)),
            ("line", &frame.line.to_string()),
            ("column", &frame.column.to_string()),
            ("source", &html_escape(&frame.source_line)),
            ("caret", &html_escape(&frame.caret_line())),
        ],
    )
}

fn document(code: &str, title: &str, summary: &str, body: &str, kind: &str) -> String {
    fill_template(
        DOCUMENT_HTML,
        &[
            ("title", &html_escape(title)),
            ("css", DOCUMENT_CSS),
            ("kind", &html_escape(kind)),
            ("code", &html_escape(code)),
            ("summary", &html_escape(summary)),
            ("body", body),
        ],
    )
}

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
    fn coded_template_error_appears_in_html_and_text() {
        let src = "@init { {} }\n";
        let diagnostic = Diagnostic::error_code(
            rocci_template::codes::RC2003,
            Span::new(0, 5),
            "`@init` requires `@context` to declare the app state type",
        );
        let files = [FailedFile {
            name: "App.rocci".into(),
            src: src.into(),
            diagnostics: vec![diagnostic],
        }];
        let html = render_template_errors(&files);
        assert!(html.contains("error[RC2003]"), "{html}");
        let text = format_template_errors(&files);
        assert!(text.contains("error[RC2003]:"), "{text}");
    }

    #[test]
    fn build_error_dialog_does_not_emit_line_continuation_slashes() {
        let html = render_build_error_dialog("RD2201 boom");
        assert!(html.contains("rocci-build-error"));
        assert!(html.contains("<div id=\"rocci-build-error\""));
        assert!(!html.contains("<dialog"));
        assert!(html.contains("/__rocci/error.css"));
        assert!(html.contains("Build error"));
        assert!(ERROR_OVERLAY_CSS.contains("#ff7b8a"));
        assert!(html.contains("RD2201 boom"));
        assert!(
            !html.contains('\\'),
            "raw-string `\\` line continuations must not appear in the overlay: {html}"
        );
    }

    #[test]
    fn build_error_shell_uses_shared_error_document() {
        let html = render_build_error_shell("RD2101 broken internal link");
        assert!(html.contains("class=\"brand\""));
        assert!(html.contains("rocci"));
        assert!(html.contains("Build error"));
        assert!(html.contains("RD2101 broken internal link"));
        assert!(html.contains("--accent"));
        assert!(!html.contains("<dialog"));
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
