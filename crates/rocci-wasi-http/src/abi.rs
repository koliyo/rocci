//! 0.16 host request/response shapes (`request_to_roc` field names).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerHeader {
    pub name: String,
    pub value: String,
}

/// Same fields as basic-webserver 0.16 `ServerRequest` after `request_to_roc`.
/// The WASI adapter buffers the body instead of passing a streaming handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerRequest {
    pub authority_host: String,
    pub authority_port: u16,
    pub authority_port_present: bool,
    pub authority_present: bool,
    pub body: Vec<u8>,
    pub body_limit_bytes: u64,
    pub content_length: u64,
    pub content_length_known: bool,
    pub headers: Vec<ServerHeader>,
    pub method: u8,
    pub method_ext: String,
    pub target_authority_host: String,
    pub target_authority_port: u16,
    pub target_authority_port_present: bool,
    pub target_path: String,
    pub target_query: String,
    pub target_query_present: bool,
    pub target_tag: u8,
}

impl ServerRequest {
    pub const TARGET_RESOURCE: u8 = 0;
    pub const TARGET_AUTHORITY: u8 = 1;
    pub const TARGET_ASTERISK: u8 = 2;
    pub const METHOD_CONNECT: u8 = 0;
    pub const METHOD_DELETE: u8 = 1;
    pub const METHOD_EXT: u8 = 2;
    pub const METHOD_GET: u8 = 3;
    pub const METHOD_HEAD: u8 = 4;
    pub const METHOD_OPTIONS: u8 = 5;
    pub const METHOD_PATCH: u8 = 6;
    pub const METHOD_POST: u8 = 7;
    pub const METHOD_PUT: u8 = 8;
    pub const METHOD_TRACE: u8 = 9;
    pub const METHOD_QUERY: u8 = 10;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryResponse {
    pub exit_code: i64,
    pub body: Vec<u8>,
    pub headers: Vec<ServerHeader>,
    pub status: u16,
    pub stop: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SseStepToHost {
    EmitToHost { item: Vec<u8>, wait_millis: u64 },
    WaitToHost { wait_millis: u64 },
    EndToHost,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutcomeToHost {
    Ordinary(OrdinaryResponse),
    Stream { source: u64 },
    File { rel_path: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncomingRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutgoingResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub streamed: bool,
}

pub fn method_to_tag(method: &str) -> (u8, String) {
    match method.to_ascii_uppercase().as_str() {
        "CONNECT" => (ServerRequest::METHOD_CONNECT, String::new()),
        "DELETE" => (ServerRequest::METHOD_DELETE, String::new()),
        "GET" => (ServerRequest::METHOD_GET, String::new()),
        "HEAD" => (ServerRequest::METHOD_HEAD, String::new()),
        "OPTIONS" => (ServerRequest::METHOD_OPTIONS, String::new()),
        "PATCH" => (ServerRequest::METHOD_PATCH, String::new()),
        "POST" => (ServerRequest::METHOD_POST, String::new()),
        "PUT" => (ServerRequest::METHOD_PUT, String::new()),
        "TRACE" => (ServerRequest::METHOD_TRACE, String::new()),
        "QUERY" => (ServerRequest::METHOD_QUERY, String::new()),
        other => (ServerRequest::METHOD_EXT, other.to_string()),
    }
}

pub fn map_request(incoming: IncomingRequest) -> ServerRequest {
    let (method, method_ext) = method_to_tag(&incoming.method);
    let headers: Vec<ServerHeader> = incoming
        .headers
        .iter()
        .map(|(name, value)| ServerHeader {
            name: name.clone(),
            value: value.clone(),
        })
        .collect();
    let (target_path, target_query, target_query_present) = split_target(&incoming.path);
    let (authority_host, authority_port, authority_port_present, authority_present) =
        authority_from_headers(&incoming.headers);
    let content_length = incoming.body.len() as u64;
    ServerRequest {
        authority_host,
        authority_port,
        authority_port_present,
        authority_present,
        body: incoming.body,
        body_limit_bytes: content_length.max(4096),
        content_length,
        content_length_known: true,
        headers,
        method,
        method_ext,
        target_authority_host: String::new(),
        target_authority_port: 0,
        target_authority_port_present: false,
        target_path,
        target_query,
        target_query_present,
        target_tag: ServerRequest::TARGET_RESOURCE,
    }
}

fn split_target(path: &str) -> (String, String, bool) {
    match path.split_once('?') {
        Some((path, query)) => (normalize_path(path), query.to_string(), true),
        None => (normalize_path(path), String::new(), false),
    }
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        "/".into()
    } else {
        path.to_string()
    }
}

fn authority_from_headers(headers: &[(String, String)]) -> (String, u16, bool, bool) {
    let Some((_, value)) = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("host"))
    else {
        return (String::new(), 0, false, false);
    };
    if let Some((host, port)) = value.rsplit_once(':')
        && !host.starts_with('[')
        && let Ok(port) = port.parse::<u16>()
    {
        return (host.to_string(), port, true, true);
    }
    (value.clone(), 0, false, true)
}

pub fn map_ordinary(ordinary: OrdinaryResponse) -> OutgoingResponse {
    OutgoingResponse {
        status: ordinary.status,
        headers: ordinary
            .headers
            .into_iter()
            .map(|h| (h.name, h.value))
            .collect(),
        body: ordinary.body,
        streamed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_get_path_query_and_headers() {
        let request = map_request(IncomingRequest {
            method: "GET".into(),
            path: "/hello?x=1".into(),
            headers: vec![
                ("host".into(), "example.test:8080".into()),
                ("accept".into(), "text/html".into()),
            ],
            body: Vec::new(),
        });
        assert_eq!(request.method, ServerRequest::METHOD_GET);
        assert_eq!(request.method_ext, "");
        assert_eq!(request.target_path, "/hello");
        assert_eq!(request.target_query, "x=1");
        assert!(request.target_query_present);
        assert_eq!(request.target_tag, ServerRequest::TARGET_RESOURCE);
        assert_eq!(request.authority_host, "example.test");
        assert_eq!(request.authority_port, 8080);
        assert!(request.authority_present);
        assert_eq!(request.headers[1].name, "accept");
        assert_eq!(request.content_length, 0);
        assert!(request.content_length_known);
    }

    #[test]
    fn buffers_post_body() {
        let request = map_request(IncomingRequest {
            method: "POST".into(),
            path: "/".into(),
            headers: vec![],
            body: b"abc".to_vec(),
        });
        assert_eq!(request.method, ServerRequest::METHOD_POST);
        assert_eq!(request.body, b"abc");
        assert_eq!(request.content_length, 3);
    }
}
