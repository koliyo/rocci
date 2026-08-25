use std::sync::OnceLock;

static VERBOSE: OnceLock<bool> = OnceLock::new();

pub fn verbose_enabled() -> bool {
    *VERBOSE.get_or_init(|| match std::env::var("ROCCI_LSP_VERBOSE") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    })
}

pub fn always(message: impl std::fmt::Display) {
    eprintln!("[rocci-lsp] {message}");
}

pub fn verbose(message: impl std::fmt::Display) {
    if verbose_enabled() {
        always(message);
    }
}
