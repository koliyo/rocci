//! Empty WASI 0.3 `wasi:http/service` guest. No Roc.

pub const HELLO_WEB_HTML: &str = "<!doctype html><html><body>hello-web</body></html>";

#[cfg(target_family = "wasm")]
mod service {
    use wasip3::http::types::{ErrorCode, Fields, Request, Response};
    use wasip3::{spawn_local, wit_future, wit_stream};

    use super::HELLO_WEB_HTML;

    struct EmptyService;

    impl wasip3::exports::http::handler::Guest for EmptyService {
        async fn handle(_request: Request) -> Result<Response, ErrorCode> {
            let headers =
                Fields::from_list(&[("content-type".into(), b"text/html; charset=utf-8".to_vec())])
                    .expect("static content-type");
            let (mut body_tx, body_rx) = wit_stream::new();
            let (trailers_tx, trailers_rx) = wit_future::new(|| Ok(None));
            let body = HELLO_WEB_HTML.as_bytes().to_vec();
            // Write after return so wasmtime can poll the stream (write-first deadlocks).
            spawn_local(async move {
                let _ = body_tx.write_all(body).await;
                drop(body_tx);
                let _ = trailers_tx.write(Ok(None)).await;
            });
            let (response, _sent) = Response::new(headers, Some(body_rx), trailers_rx);
            Ok(response)
        }
    }

    wasip3::http::service::export!(EmptyService);
}

#[cfg(test)]
mod tests {
    #[test]
    fn hello_web_html_matches_embedder_fixture() {
        assert_eq!(
            super::HELLO_WEB_HTML,
            "<!doctype html><html><body>hello-web</body></html>"
        );
    }
}
