//! WASI 0.3-shaped `handle`: buffer the body, call Roc, map an ordinary response.

use anyhow::Result;

use crate::abi::{IncomingRequest, OutgoingResponse, map_ordinary, map_request};
use crate::guest::RocGuest;

pub struct Adapter<G> {
    guest: G,
    initialized: bool,
}

impl<G: RocGuest> Adapter<G> {
    pub fn new(guest: G) -> Self {
        Self {
            guest,
            initialized: false,
        }
    }

    pub fn guest(&self) -> &G {
        &self.guest
    }

    pub fn guest_mut(&mut self) -> &mut G {
        &mut self.guest
    }

    /// First request runs `init` (listen host/port ignored). `_initialize` is the same path.
    pub fn initialize(&mut self) {
        if !self.initialized {
            self.guest.init();
            self.initialized = true;
        }
    }

    pub async fn handle(&mut self, incoming: IncomingRequest) -> Result<OutgoingResponse> {
        self.initialize();
        let request = map_request(incoming);
        let outcome = self.guest.respond(&request);
        Ok(map_ordinary(outcome))
    }

    pub fn shutdown(&mut self) {
        if self.initialized {
            self.guest.shutdown();
            self.initialized = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::IncomingRequest;
    use crate::guest::StubGuest;

    #[tokio::test(flavor = "current_thread")]
    async fn stub_get_root_returns_html() {
        let mut adapter = Adapter::new(StubGuest::new());
        let response = adapter
            .handle(IncomingRequest {
                method: "GET".into(),
                path: "/".into(),
                headers: vec![],
                body: Vec::new(),
            })
            .await
            .unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers,
            vec![("content-type".into(), "text/html; charset=utf-8".into())]
        );
        assert_eq!(response.body, StubGuest::HTML.as_bytes());
        assert_eq!(adapter.guest().init_count, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn init_runs_once_per_instance() {
        let mut adapter = Adapter::new(StubGuest::new());
        for _ in 0..2 {
            adapter
                .handle(IncomingRequest {
                    method: "GET".into(),
                    path: "/".into(),
                    headers: vec![],
                    body: Vec::new(),
                })
                .await
                .unwrap();
        }
        assert_eq!(adapter.guest().init_count, 1);
    }
}
