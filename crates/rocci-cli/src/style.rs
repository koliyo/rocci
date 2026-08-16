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
}
