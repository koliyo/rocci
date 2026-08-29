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

/// Records the last `ServerRequest` and echoes mapped fields (root GET stays stub HTML).
pub struct EchoGuest {
    pub init_count: u32,
    pub last: Option<ServerRequest>,
}

impl EchoGuest {
    pub fn new() -> Self {
        Self {
            init_count: 0,
            last: None,
        }
    }

    pub fn echo_body(request: &ServerRequest) -> String {
        let accept = request
            .headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("accept"))
            .map(|h| h.value.as_str())
            .unwrap_or("");
        format!(
            "method={}\npath={}\nquery={}\nquery_present={}\nauthority={}\nauthority_port={}\nheader.accept={}\ncontent_length={}\nbody={}\n",
            request.method,
            request.target_path,
            request.target_query,
            request.target_query_present,
            request.authority_host,
            request.authority_port,
            accept,
            request.content_length,
            String::from_utf8_lossy(&request.body),
        )
    }
}

impl Default for EchoGuest {
    fn default() -> Self {
        Self::new()
    }
}

impl RocGuest for EchoGuest {
    fn init(&mut self) {
        self.init_count += 1;
    }

    fn respond(&mut self, request: &ServerRequest) -> OutcomeToHost {
        self.last = Some(request.clone());
        if request.method == ServerRequest::METHOD_GET && request.target_path == "/" {
            return StubGuest::new().respond(request);
        }
        OutcomeToHost::Ordinary(crate::abi::OrdinaryResponse {
            exit_code: 0,
            body: Self::echo_body(request).into_bytes(),
            headers: vec![ServerHeader {
                name: "content-type".into(),
                value: "text/plain; charset=utf-8".into(),
            }],
            status: 200,
            stop: false,
        })
    }

    fn shutdown(&mut self) {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::{IncomingRequest, map_request};

    #[test]
    fn echo_matches_get_path_query_and_headers() {
        let request = map_request(IncomingRequest {
            method: "GET".into(),
            path: "/hello?x=1".into(),
            headers: vec![
                ("host".into(), "example.test:8080".into()),
                ("accept".into(), "text/html".into()),
            ],
            body: Vec::new(),
        });
        let mut guest = EchoGuest::new();
        let OutcomeToHost::Ordinary(out) = guest.respond(&request) else {
            panic!("ordinary");
        };
        let last = guest.last.expect("recorded");
        assert_eq!(last.method, ServerRequest::METHOD_GET);
        assert_eq!(last.target_path, "/hello");
        assert_eq!(last.target_query, "x=1");
        assert!(last.target_query_present);
        assert_eq!(last.authority_host, "example.test");
        assert_eq!(last.authority_port, 8080);
        assert_eq!(last.headers[1].name, "accept");
        assert_eq!(last.content_length, 0);
        let body = String::from_utf8(out.body).unwrap();
        assert!(body.contains("path=/hello"), "{body}");
        assert!(body.contains("query=x=1"), "{body}");
        assert!(body.contains("header.accept=text/html"), "{body}");
        assert_eq!(out.status, 200);
    }

    #[test]
    fn echo_matches_buffered_post_body() {
        let request = map_request(IncomingRequest {
            method: "POST".into(),
            path: "/".into(),
            headers: vec![],
            body: b"abc".to_vec(),
        });
        let mut guest = EchoGuest::new();
        let OutcomeToHost::Ordinary(out) = guest.respond(&request) else {
            panic!("ordinary");
        };
        let last = guest.last.expect("recorded");
        assert_eq!(last.method, ServerRequest::METHOD_POST);
        assert_eq!(last.body, b"abc");
        assert_eq!(last.content_length, 3);
        let body = String::from_utf8(out.body).unwrap();
        assert!(body.contains("method=7"), "{body}");
        assert!(body.contains("body=abc"), "{body}");
        assert!(body.contains("content_length=3"), "{body}");
    }
}
