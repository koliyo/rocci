use std::io::IsTerminal;

use rocci_template::supports_ansi;

pub fn stdout_color() -> bool {
    supports_ansi(std::io::stdout().is_terminal())
}

pub fn stderr_color() -> bool {
    supports_ansi(std::io::stderr().is_terminal())
}

fn paint(color: bool, codes: &str, text: &str) -> String {
    if color {
        format!("\x1b[{codes}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn serving(subject: &str, url: &str) -> String {
    status("Serving", subject, url)
}

pub fn viewing(subject: &str, url: &str) -> String {
    status("Viewing", subject, url)
}

pub fn browsing(subject: &str, url: &str) -> String {
    status("Browsing", subject, url)
}

fn status(verb: &str, subject: &str, url: &str) -> String {
    let color = stdout_color();
    format!(
        "{} {subject} at {}",
        paint(color, "1;32", verb),
        paint(color, "1;36", url)
    )
}

pub fn success_text(text: &str) -> String {
    paint(stdout_color(), "1;32", text)
}

pub fn ok(message: &str) -> String {
    labeled(stdout_color(), "1;32", "ok:", message)
}

pub fn pinned(message: &str) -> String {
    labeled(stdout_color(), "1;32", "pinned", message)
}

pub fn warning(message: &str) -> String {
    labeled(stderr_color(), "1;33", "warning:", message)
}

pub fn note(message: &str) -> String {
    labeled(stderr_color(), "1;33", "note:", message)
}

fn labeled(color: bool, codes: &str, label: &str, message: &str) -> String {
    format!("{} {message}", paint(color, codes, label))
}

pub fn handler(method: &str, path: &str, status: &str) -> String {
    format_handler(stderr_color(), method, path, status)
}

pub fn handler_proxied(method: &str, path: &str, ms: u128) -> String {
    format_handler_detail(
        stderr_color(),
        method,
        path,
        &format!("proxied ({ms}ms)"),
        "1;32",
    )
}

pub fn handler_unavailable(method: &str, path: &str) -> String {
    format_handler_detail(stderr_color(), method, path, "island unavailable", "1;33")
}

pub fn handler_proxy_error(method: &str, path: &str, err: impl std::fmt::Display) -> String {
    format_handler_detail(
        stderr_color(),
        method,
        path,
        &format!("proxy error: {err}"),
        "1;31",
    )
}

fn format_handler(color: bool, method: &str, path: &str, status: &str) -> String {
    let status_code = if status == "ok" { "1;32" } else { "1;31" };
    format_handler_detail(color, method, path, status, status_code)
}

fn format_handler_detail(
    color: bool,
    method: &str,
    path: &str,
    detail: &str,
    detail_code: &str,
) -> String {
    format!(
        "{} {path} -> {}",
        paint(color, method_code(method), method),
        paint(color, detail_code, detail)
    )
}

fn method_code(method: &str) -> &'static str {
    match method {
        "GET" => "1;32",
        "HEAD" => "32",
        "POST" => "1;33",
        "PUT" | "PATCH" => "1;35",
        "DELETE" => "1;31",
        _ => "1;36",
    }
}

pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

pub fn print_anyhow(err: &anyhow::Error) {
    let color = stderr_color();
    let mut chain = err.chain();
    if let Some(first) = chain.next() {
        eprintln!("{} {first}", paint(color, "1;31", "error:"));
    }
    for cause in chain {
        eprintln!("  {} {cause}", paint(color, "1;34", "-->"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_keeps_the_plain_shape() {
        let line = serving("Guide", "http://127.0.0.1:8000/guide/");
        assert!(line.contains("Serving Guide at "));
        assert!(line.contains("http://127.0.0.1:8000/guide/"));
    }

    #[test]
    fn labels_keep_the_plain_shape() {
        assert!(ok("app (1 window)").contains("ok: app (1 window)"));
        assert!(warning("skipping Broken.rocdown").contains("warning: skipping Broken.rocdown"));
        assert!(note("Datastar 1.0.3 is available").contains("note: Datastar 1.0.3 is available"));
        assert!(pinned("Datastar 1.0.2 for app").contains("pinned Datastar 1.0.2 for app"));
        assert!(success_text("/tmp/App.app").contains("/tmp/App.app"));
    }

    #[test]
    fn handler_lines_keep_the_plain_shape() {
        assert_eq!(
            format_handler(false, "POST", "/actions/counter/increment", "ok"),
            "POST /actions/counter/increment -> ok"
        );
        let colored = format_handler(true, "POST", "/actions/counter/increment", "ok");
        assert!(colored.contains("\x1b["), "{colored}");
        assert_eq!(
            strip_ansi(&colored),
            "POST /actions/counter/increment -> ok"
        );
        assert_eq!(
            strip_ansi(&format_handler(true, "GET", "/health", "err")),
            "GET /health -> err"
        );
        assert_eq!(
            strip_ansi(&format_handler_detail(
                true,
                "POST",
                "/actions/x",
                "proxied (4ms)",
                "1;32"
            )),
            "POST /actions/x -> proxied (4ms)"
        );
    }
}
