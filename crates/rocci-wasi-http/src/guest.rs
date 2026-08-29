//! Roc guest surface: 0.16 `init` / `respond` / `shutdown` / SSE advance.

use crate::abi::{OutcomeToHost, ServerHeader, ServerRequest, SseStepToHost};

pub trait RocGuest: Send {
    fn init(&mut self);
    fn respond(&mut self, request: &ServerRequest) -> OutcomeToHost;
    fn shutdown(&mut self);
    fn sse_advance(&mut self, source: u64, wake_generation: u64) -> SseStepToHost {
        let _ = (source, wake_generation);
        SseStepToHost::EndToHost
    }
    fn sse_drop_source(&mut self, source: u64) {
        let _ = source;
    }
    fn sse_drop_step(&mut self, source: u64) {
        let _ = source;
    }
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

/// `empty_sse!`: respond with a stream that Ends immediately.
pub struct EmptySseGuest;

impl RocGuest for EmptySseGuest {
    fn init(&mut self) {}
    fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
        OutcomeToHost::Stream { source: 1 }
    }
    fn shutdown(&mut self) {}
    fn sse_advance(&mut self, _source: u64, _wake: u64) -> SseStepToHost {
        SseStepToHost::EndToHost
    }
}

/// Wait then one Emit (keepalive / fake Datastar frame).
pub struct WaitEmitGuest {
    step: u8,
    wait: std::time::Duration,
}

impl WaitEmitGuest {
    pub fn new(wait: std::time::Duration) -> Self {
        Self { step: 0, wait }
    }
}

impl RocGuest for WaitEmitGuest {
    fn init(&mut self) {}
    fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
        self.step = 0;
        OutcomeToHost::Stream { source: 1 }
    }
    fn shutdown(&mut self) {}
    fn sse_advance(&mut self, _source: u64, _wake: u64) -> SseStepToHost {
        match self.step {
            0 => {
                self.step = 1;
                SseStepToHost::WaitToHost {
                    wait_millis: u64::try_from(self.wait.as_millis()).unwrap_or(0),
                }
            }
            1 => {
                self.step = 2;
                SseStepToHost::EmitToHost {
                    item: b"data: keepalive\n\n".to_vec(),
                    wait_millis: 0,
                }
            }
            _ => SseStepToHost::EndToHost,
        }
    }
}
