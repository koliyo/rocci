//! WASI 0.3 `wasi:http/service` guest. Maps onto `Adapter` + `StubGuest`. No Roc.

pub const HELLO_WEB_HTML: &str = rocci_wasi_http::StubGuest::HTML;

#[cfg(target_family = "wasm")]
mod service {
    use std::sync::{Mutex, OnceLock};

    use std::time::Duration;

    use rocci_wasi_http::{
        Adapter, EmptySseGuest, IncomingRequest, LinkedHelloWebGuest, OutcomeToHost,
        OutgoingResponse, RocGuest, ServerRequest, SseStepToHost, WaitEmitGuest,
        abi::{map_ordinary, map_request},
    };
    use wasip3::http::types::{ErrorCode, Fields, Method, Request, Response};
    use wasip3::{spawn_local, wit_future, wit_stream};

    struct RoutedGuest {
        linked: LinkedHelloWebGuest,
    }

    impl Default for RoutedGuest {
        fn default() -> Self {
            Self {
                linked: LinkedHelloWebGuest::new(),
            }
        }
    }

    impl RocGuest for RoutedGuest {
        fn init(&mut self) {
            self.linked.init();
        }

        fn respond(&mut self, request: &ServerRequest) -> rocci_wasi_http::OutcomeToHost {
            if request.method == ServerRequest::METHOD_GET && request.target_path == "/hello.txt" {
                return rocci_wasi_http::OutcomeToHost::File {
                    rel_path: "hello.txt".into(),
                };
            }
            self.linked.respond(request)
        }

        fn shutdown(&mut self) {
            self.linked.shutdown();
        }

        fn sse_advance(&mut self, source: u64, wake_generation: u64) -> SseStepToHost {
            self.linked.sse_advance(source, wake_generation)
        }

        fn sse_drop_source(&mut self, source: u64) {
            self.linked.sse_drop_source(source);
        }
    }

    fn adapter() -> &'static Mutex<Adapter<RoutedGuest>> {
        static ADAPTER: OnceLock<Mutex<Adapter<RoutedGuest>>> = OnceLock::new();
        ADAPTER.get_or_init(|| {
            Mutex::new(
                Adapter::new(RoutedGuest::default()).with_file_root(std::path::PathBuf::from("/")),
            )
        })
    }

    fn method_name(method: Method) -> String {
        match method {
            Method::Get => "GET".into(),
            Method::Head => "HEAD".into(),
            Method::Post => "POST".into(),
            Method::Put => "PUT".into(),
            Method::Delete => "DELETE".into(),
            Method::Connect => "CONNECT".into(),
            Method::Options => "OPTIONS".into(),
            Method::Trace => "TRACE".into(),
            Method::Patch => "PATCH".into(),
            Method::Other(name) => name,
        }
    }

    async fn buffer_wasi_request(request: Request) -> Result<IncomingRequest, ErrorCode> {
        let method = method_name(request.get_method());
        let path = request.get_path_with_query().unwrap_or_else(|| "/".into());
        let mut headers: Vec<(String, String)> = request
            .get_headers()
            .copy_all()
            .into_iter()
            .map(|(name, value)| (name, String::from_utf8_lossy(&value).into_owned()))
            .collect();
        if !headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("host"))
            && let Some(authority) = request.get_authority()
        {
            headers.push(("host".into(), authority));
        }
        let (body_done_tx, body_done_rx) = wit_future::new(|| Ok(()));
        let (mut body_rx, _trailers) = Request::consume_body(request, body_done_rx);
        drop(body_done_tx);
        let mut body = Vec::new();
        while let Some(byte) = body_rx.next().await {
            body.push(byte);
        }
        Ok(IncomingRequest {
            method,
            path,
            headers,
            body,
        })
    }

    async fn clocks_wait(wait_millis: u64) {
        if wait_millis == 0 {
            return;
        }
        wasip3::clocks::monotonic_clock::wait_for(wait_millis.saturating_mul(1_000_000)).await;
    }

    async fn stream_linked_sse(mut source: u64) -> Result<Response, ErrorCode> {
        let headers = Fields::from_list(&[("content-type".into(), b"text/event-stream".to_vec())])
            .map_err(|_| ErrorCode::InternalError(None))?;
        let (mut body_tx, body_rx) = wit_stream::new();
        let (trailers_tx, trailers_rx) = wit_future::new(|| Ok(None));
        spawn_local(async move {
            let mut wake = 0u64;
            loop {
                let step = {
                    let mut adapter = adapter()
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    adapter.guest_mut().sse_advance(source, wake)
                };
                match step {
                    SseStepToHost::EmitToHost {
                        item,
                        wait_millis,
                        source: next,
                    } => {
                        let _ = body_tx.write_all(item).await;
                        if next != 0 {
                            source = next;
                        }
                        clocks_wait(wait_millis).await;
                        if wait_millis > 0 {
                            wake = wake.wrapping_add(1);
                        }
                    }
                    SseStepToHost::WaitToHost {
                        wait_millis,
                        source: next,
                    } => {
                        if next != 0 {
                            source = next;
                        }
                        clocks_wait(wait_millis).await;
                        wake = wake.wrapping_add(1);
                    }
                    SseStepToHost::EndToHost => {
                        let mut adapter = adapter()
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        adapter.guest_mut().sse_drop_source(source);
                        break;
                    }
                }
            }
            drop(body_tx);
            let _ = trailers_tx.write(Ok(None)).await;
        });
        let (response, _sent) = Response::new(headers, Some(body_rx), trailers_rx);
        let _ = response.set_status_code(200);
        Ok(response)
    }

    async fn stream_sse_wasi<G: RocGuest + 'static>(
        mut guest: G,
        incoming: IncomingRequest,
    ) -> Result<Response, ErrorCode> {
        guest.init();
        let rocci_wasi_http::OutcomeToHost::Stream { source } =
            guest.respond(&map_request(incoming))
        else {
            return Err(ErrorCode::InternalError(Some(
                "expected SSE stream outcome".into(),
            )));
        };
        let headers = Fields::from_list(&[("content-type".into(), b"text/event-stream".to_vec())])
            .map_err(|_| ErrorCode::InternalError(None))?;
        let (mut body_tx, body_rx) = wit_stream::new();
        let (trailers_tx, trailers_rx) = wit_future::new(|| Ok(None));
        spawn_local(async move {
            let mut wake = 0u64;
            let mut source = source;
            loop {
                let step = guest.sse_advance(source, wake);
                match step {
                    SseStepToHost::EmitToHost {
                        item,
                        wait_millis,
                        source: next,
                    } => {
                        let _ = body_tx.write_all(item).await;
                        if next != 0 {
                            source = next;
                        }
                        clocks_wait(wait_millis).await;
                        if wait_millis > 0 {
                            wake = wake.wrapping_add(1);
                        }
                    }
                    SseStepToHost::WaitToHost {
                        wait_millis,
                        source: next,
                    } => {
                        if next != 0 {
                            source = next;
                        }
                        clocks_wait(wait_millis).await;
                        wake = wake.wrapping_add(1);
                    }
                    SseStepToHost::EndToHost => {
                        guest.sse_drop_source(source);
                        break;
                    }
                }
            }
            drop(body_tx);
            let _ = trailers_tx.write(Ok(None)).await;
        });
        let (response, _sent) = Response::new(headers, Some(body_rx), trailers_rx);
        let _ = response.set_status_code(200);
        Ok(response)
    }

    fn outgoing_to_wasi(outgoing: OutgoingResponse) -> Result<Response, ErrorCode> {
        let entries: Vec<(String, Vec<u8>)> = outgoing
            .headers
            .iter()
            .map(|(name, value)| (name.clone(), value.as_bytes().to_vec()))
            .collect();
        let headers = Fields::from_list(&entries).map_err(|_| ErrorCode::InternalError(None))?;
        let (mut body_tx, body_rx) = wit_stream::new();
        let (trailers_tx, trailers_rx) = wit_future::new(|| Ok(None));
        let body = outgoing.body;
        spawn_local(async move {
            let _ = body_tx.write_all(body).await;
            drop(body_tx);
            let _ = trailers_tx.write(Ok(None)).await;
        });
        let (response, _sent) = Response::new(headers, Some(body_rx), trailers_rx);
        let _ = response.set_status_code(outgoing.status);
        Ok(response)
    }

    struct MapService;

    impl wasip3::exports::http::handler::Guest for MapService {
        async fn handle(request: Request) -> Result<Response, ErrorCode> {
            let incoming = buffer_wasi_request(request).await?;
            let path = incoming.path.split('?').next().unwrap_or("/");
            if path == "/sse-empty" {
                return stream_sse_wasi(EmptySseGuest, incoming).await;
            }
            if path == "/sse-wait" {
                return stream_sse_wasi(WaitEmitGuest::new(Duration::from_millis(200)), incoming)
                    .await;
            }
            let outcome = {
                let mut adapter = adapter()
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                adapter.begin(incoming)
            };
            match outcome {
                OutcomeToHost::Ordinary(ordinary) => outgoing_to_wasi(map_ordinary(ordinary)),
                OutcomeToHost::File { rel_path } => {
                    let outgoing = adapter()
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .serve_file(&rel_path)
                        .map_err(|err| ErrorCode::InternalError(Some(err.to_string())))?;
                    outgoing_to_wasi(outgoing)
                }
                OutcomeToHost::Stream { source } => stream_linked_sse(source).await,
            }
        }
    }

    wasip3::http::service::export!(MapService);
}

#[cfg(test)]
mod tests {
    #[test]
    fn stub_html_matches_map_crate() {
        assert_eq!(super::HELLO_WEB_HTML, rocci_wasi_http::StubGuest::HTML);
        assert_eq!(
            super::HELLO_WEB_HTML,
            "<!doctype html><html><body>hello-web</body></html>"
        );
    }

    #[test]
    fn roc_hello_web_body_is_not_the_rust_constant() {
        assert_ne!(super::HELLO_WEB_HTML, "<b>Hello from server</b><br>");
    }
}
