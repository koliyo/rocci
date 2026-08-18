use rocci_cli::playground::{PLAYGROUND_CSP, start_playground_server};
use rocci_cli::serve::free_port;
use std::io::{Read, Write};
use std::net::TcpStream;

fn send_raw_get(port: u16, path: &str) -> (u16, String, Vec<u8>) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to server");
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("write request");

    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).expect("read response");

    let resp_str = String::from_utf8_lossy(&resp);
    let mut parts = resp_str.split("\r\n\r\n");
    let header_part = parts.next().unwrap_or("");
    let status_line = header_part.lines().next().unwrap_or("");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let body = if let Some(idx) = resp.windows(4).position(|w| w == b"\r\n\r\n") {
        resp[idx + 4..].to_vec()
    } else {
        Vec::new()
    };

    (status_code, header_part.to_string(), body)
}

#[test]
fn test_playground_loopback_server_routes_and_headers() {
    let port = free_port().expect("allocate free port");
    let initial_src = "@component Counter = |{ count }| { <button>{count}</button> }";
    let mut handle =
        start_playground_server("Counter.rocci", initial_src, "rocci", port).expect("start server");

    // 1. GET /
    let (status, headers, body) = send_raw_get(port, "/");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: text/html; charset=utf-8"));
    assert!(headers.contains(&format!("Content-Security-Policy: {PLAYGROUND_CSP}")));
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("playground-root"));
    assert!(html.contains("/app.js"));

    // 2. GET /app.js
    let (status, headers, body) = send_raw_get(port, "/app.js");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(headers.contains("Cache-Control: public, max-age=31536000, immutable"));
    assert!(!body.is_empty());

    // 3. GET /compiler-worker.js
    let (status, headers, body) = send_raw_get(port, "/compiler-worker.js");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(!body.is_empty());

    // 4. GET /styles.css
    let (status, headers, body) = send_raw_get(port, "/styles.css");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: text/css; charset=utf-8"));
    assert!(!body.is_empty());

    // 5. GET /compiler.wasm
    let (status, headers, body) = send_raw_get(port, "/compiler.wasm");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/wasm"));
    assert!(!body.is_empty());

    // 6. GET /api/session
    let (status, headers, body) = send_raw_get(port, "/api/session");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/json; charset=utf-8"));
    assert!(headers.contains("Cache-Control: no-store, no-cache, must-revalidate"));
    let json_str = String::from_utf8_lossy(&body);
    assert!(json_str.contains("\"filename\":\"Counter.rocci\""));
    assert!(json_str.contains("\"language\":\"rocci\""));
    assert!(json_str.contains("Counter = |{ count }|"));

    // 7. Path Traversal rejection
    let (status, _, _) = send_raw_get(port, "/../Cargo.toml");
    assert_eq!(status, 404);

    // 8. Unknown route 404
    let (status, _, _) = send_raw_get(port, "/unknown-path");
    assert_eq!(status, 404);

    handle.stop();
}
