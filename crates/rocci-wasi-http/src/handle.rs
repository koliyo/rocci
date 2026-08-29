//! WASI 0.3-shaped `handle`: buffer the body, call Roc, map ordinary or SSE.

use std::time::Duration;

use anyhow::Result;

use crate::abi::{
    IncomingRequest, OutcomeToHost, OutgoingResponse, SseStepToHost, map_ordinary, map_request,
};
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
        match outcome {
            OutcomeToHost::Ordinary(ordinary) => Ok(map_ordinary(ordinary)),
            OutcomeToHost::Stream { source } => self.stream_sse(source).await,
        }
    }

    async fn stream_sse(&mut self, source: u64) -> Result<OutgoingResponse> {
        let mut body = Vec::new();
        let mut wake = 0u64;
        loop {
            let step = self.guest.sse_advance(source, wake);
            match step {
                SseStepToHost::EmitToHost { item, wait_millis } => {
                    body.extend_from_slice(&item);
                    if wait_millis == 0 {
                        continue;
                    }
                    tokio::time::sleep(Duration::from_millis(wait_millis)).await;
                    wake = wake.wrapping_add(1);
                }
                SseStepToHost::WaitToHost { wait_millis } => {
                    if wait_millis > 0 {
                        tokio::time::sleep(Duration::from_millis(wait_millis)).await;
                    }
                    wake = wake.wrapping_add(1);
                }
                SseStepToHost::EndToHost => {
                    self.guest.sse_drop_source(source);
                    break;
                }
            }
        }
        Ok(OutgoingResponse {
            status: 200,
            headers: vec![("content-type".into(), "text/event-stream".into())],
            body,
            streamed: true,
        })
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
    use crate::guest::{EmptySseGuest, StubGuest, WaitEmitGuest};
    use std::time::Instant;

    fn get_root() -> IncomingRequest {
        IncomingRequest {
            method: "GET".into(),
            path: "/".into(),
            headers: vec![],
            body: Vec::new(),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stub_get_root_returns_html() {
        let mut adapter = Adapter::new(StubGuest::new());
        let response = adapter.handle(get_root()).await.unwrap();
        assert_eq!(response.status, 200);
        assert!(!response.streamed);
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
            adapter.handle(get_root()).await.unwrap();
        }
        assert_eq!(adapter.guest().init_count, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn empty_sse_ends_immediately() {
        let mut adapter = Adapter::new(EmptySseGuest);
        let response = adapter.handle(get_root()).await.unwrap();
        assert!(response.streamed);
        assert!(response.body.is_empty());
        assert_eq!(response.status, 200);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_then_emit_keepalive() {
        let mut adapter = Adapter::new(WaitEmitGuest::new(Duration::from_millis(20)));
        let response = adapter.handle(get_root()).await.unwrap();
        assert!(response.streamed);
        assert_eq!(response.body, b"data: keepalive\n\n");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn overlapping_sse_waits_do_not_serialize() {
        let wait = Duration::from_millis(40);
        let start = Instant::now();
        let mut a = Adapter::new(WaitEmitGuest::new(wait));
        let mut b = Adapter::new(WaitEmitGuest::new(wait));
        let (ra, rb) = tokio::join!(a.handle(get_root()), b.handle(get_root()));
        ra.unwrap();
        rb.unwrap();
        let wall = start.elapsed();
        assert!(
            wall < wait + wait / 2,
            "SSE Wait should overlap like adapter-await: {wall:?}"
        );
    }
}
