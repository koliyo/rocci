//! WASI 0.3 `wasi:http/service` guest. Maps onto `Adapter` + `StubGuest`. No Roc.

pub const HELLO_WEB_HTML: &str = rocci_wasi_http::StubGuest::HTML;

#[cfg(target_family = "wasm")]
mod service {
    use std::sync::{Mutex, OnceLock};

    use rocci_wasi_http::{Adapter, EchoGuest, IncomingRequest, OutgoingResponse};
    use wasip3::http::types::{ErrorCode, Fields, Method, Request, Response};
    use wasip3::{spawn_local, wit_future, wit_stream};

    fn adapter() -> &'static Mutex<Adapter<EchoGuest>> {
        static ADAPTER: OnceLock<Mutex<Adapter<EchoGuest>>> = OnceLock::new();
        ADAPTER.get_or_init(|| Mutex::new(Adapter::new(EchoGuest::new())))
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
            let outgoing = adapter()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .handle(incoming)
                .await
                .map_err(|err| ErrorCode::InternalError(Some(err.to_string())))?;
            outgoing_to_wasi(outgoing)
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
}
