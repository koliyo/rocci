//! Roc guest surface: 0.16 `init` / `respond` / `shutdown` without C layout yet.

use crate::abi::{OutcomeToHost, ServerHeader, ServerRequest};

pub trait RocGuest: Send {
    fn init(&mut self);
    fn respond(&mut self, request: &ServerRequest) -> OutcomeToHost;
    fn shutdown(&mut self);
}

/// Stub `roc_respond_for_host`: 200 `text/html` ordinary body.
pub struct StubGuest {
    pub init_count: u32,
    pub shutdown_count: u32,
}

impl StubGuest {
    pub const HTML: &'static str = "<!doctype html><html><body>hello-web</body></html>";

    pub fn new() -> Self {
        Self {
            init_count: 0,
            shutdown_count: 0,
        }
    }
}

impl Default for StubGuest {
    fn default() -> Self {
        Self::new()
    }
}

impl RocGuest for StubGuest {
    fn init(&mut self) {
        self.init_count += 1;
    }

    fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
        OutcomeToHost::Ordinary(crate::abi::OrdinaryResponse {
            exit_code: 0,
            body: Self::HTML.as_bytes().to_vec(),
            headers: vec![ServerHeader {
                name: "content-type".into(),
                value: "text/html; charset=utf-8".into(),
            }],
            status: 200,
            stop: false,
        })
    }

    fn shutdown(&mut self) {
        self.shutdown_count += 1;
    }
}
