use crate::{Result, WindowId};

/// A backend factory. Additional languages can implement this without changing
/// the tao/wry shell.
pub trait Backend: Send {
    fn name(&self) -> &str;
    fn start(&self) -> Result<Box<dyn RunningBackend>>;
}

/// A running HTTP backend owned by the desktop process.
pub trait RunningBackend: Send {
    /// Origin the webview should treat as the application server.
    fn origin(&self) -> &str;

    /// Register a window-scoped session and return the URL the webview should load.
    fn attach_window(&self, window: &WindowId, start_url: &str) -> Result<String>;

    /// Drop a window-scoped session. Default is a no-op for backends that share
    /// a single process-wide capability.
    fn detach_window(&self, _window: &WindowId) {}

    fn shutdown(&mut self);
}

/// Points every window at an already-running HTTP origin.
#[derive(Debug)]
pub struct ExternalBackend {
    origin: String,
}

impl ExternalBackend {
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into().trim_end_matches('/').to_owned(),
        }
    }
}

impl RunningBackend for ExternalBackend {
    fn origin(&self) -> &str {
        &self.origin
    }

    fn attach_window(&self, _window: &WindowId, start_url: &str) -> Result<String> {
        Ok(join_origin(&self.origin, start_url))
    }

    fn shutdown(&mut self) {}
}

pub fn join_origin(origin: &str, start_url: &str) -> String {
    if start_url.starts_with("http://") || start_url.starts_with("https://") {
        start_url.to_owned()
    } else if start_url.starts_with('/') {
        format!("{origin}{start_url}")
    } else {
        format!("{origin}/{start_url}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_backend_joins_relative_window_paths() {
        let backend = ExternalBackend::new("http://127.0.0.1:5173/");
        let url = backend
            .attach_window(&WindowId::new("main"), "/htmx")
            .unwrap();
        assert_eq!(url, "http://127.0.0.1:5173/htmx");
    }
}
