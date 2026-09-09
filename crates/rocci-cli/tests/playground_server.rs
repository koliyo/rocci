use rocci_cli::playground::{
    start_playground_server, PlaygroundMode, APP_JS, COMPILER_WASM, PLAYGROUND_CSP, STYLES_CSS,
    WORKER_JS,
};
use rocci_cli::serve::free_port;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const IO_TIMEOUT: Duration = Duration::from_secs(10);

fn wait_for_get(port: u16, path: &str) -> (u16, String, Vec<u8>) {
    let mut last = (0, String::new(), Vec::new());
    for _ in 0..50 {
        match exchange(
            port,
            &format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
            &[],
        ) {
            Ok(resp) if resp.0 != 0 => return resp,
            Ok(resp) => last = resp,
            Err(_) => {}
        }
        thread::sleep(Duration::from_millis(10));
    }
    last
}

fn send_raw_get(port: u16, path: &str) -> (u16, String, Vec<u8>) {
    exchange(
        port,
        &format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
        &[],
    )
    .expect("GET response")
}

fn send_raw_post(port: u16, path: &str, body: &[u8]) -> (u16, String, Vec<u8>) {
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    exchange(port, &headers, body).expect("POST response")
}

fn exchange(port: u16, headers: &str, body: &[u8]) -> io::Result<(u16, String, Vec<u8>)> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    stream.write_all(headers.as_bytes())?;
    if !body.is_empty() {
        stream.write_all(body)?;
    }
    stream.flush()?;
    read_http_response(&mut stream)
}

fn read_http_response(stream: &mut TcpStream) -> io::Result<(u16, String, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    let header_end = loop {
        let n = read_some(stream, &mut tmp)?;
        if n == 0 {
            return Ok(split_http(&buf));
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(idx) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break idx;
        }
    };

    let header_part = String::from_utf8_lossy(&buf[..header_end]).into_owned();
    let status_code = header_part
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut body = buf[header_end + 4..].to_vec();
    if let Some(len) = content_length(&header_part) {
        while body.len() < len {
            let n = read_some(stream, &mut tmp)?;
            if n == 0 {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "truncated HTTP body",
                ));
            }
            body.extend_from_slice(&tmp[..n]);
        }
        body.truncate(len);
    } else {
        loop {
            let n = read_some(stream, &mut tmp)?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&tmp[..n]);
        }
    }

    Ok((status_code, header_part, body))
}

fn content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn read_some(stream: &mut TcpStream, tmp: &mut [u8]) -> io::Result<usize> {
    loop {
        match stream.read(tmp) {
            Ok(n) => return Ok(n),
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }
}

fn split_http(resp: &[u8]) -> (u16, String, Vec<u8>) {
    let resp_str = String::from_utf8_lossy(resp);
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
    let mut handle = start_playground_server(
        "Counter.rocci",
        initial_src,
        "rocci",
        port,
        PlaygroundMode::Wasm,
        None,
        &[],
        false,
    )
    .expect("start server");

    let (status, headers, body) = wait_for_get(port, "/");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: text/html; charset=utf-8"));
    assert!(headers.contains(&format!("Content-Security-Policy: {PLAYGROUND_CSP}")));
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("playground-root"));
    assert!(html.contains("src=\"/boot.js\""));
    assert!(!html.contains("import { PlaygroundApp }"));

    let (status, headers, body) = send_raw_get(port, "/boot.js");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/javascript; charset=utf-8"));
    let boot = String::from_utf8_lossy(&body);
    assert!(boot.contains("import { PlaygroundApp } from \"/app.js\""));
    assert!(boot.contains("/api/session"));

    let (status, headers, body) = send_raw_get(port, "/app.js");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(headers.contains("Cache-Control: public, max-age=31536000, immutable"));
    assert_eq!(body, APP_JS);

    let (status, headers, body) = send_raw_get(port, "/compiler-worker.js");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/javascript; charset=utf-8"));
    assert_eq!(body, WORKER_JS);

    let (status, headers, body) = send_raw_get(port, "/styles.css");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: text/css; charset=utf-8"));
    assert_eq!(body, STYLES_CSS);

    let (status, headers, body) = send_raw_get(port, "/compiler.wasm");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/wasm"));
    assert_eq!(body, COMPILER_WASM);

    let (status, headers, body) = send_raw_get(port, "/api/session");
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/json; charset=utf-8"));
    assert!(headers.contains("Cache-Control: no-store, no-cache, must-revalidate"));
    let json_str = String::from_utf8_lossy(&body);
    assert!(json_str.contains("\"filename\":\"Counter.rocci\""));
    assert!(json_str.contains("\"language\":\"rocci\""));
    assert!(json_str.contains("Counter = |{ count }|"));
    assert!(json_str.contains("\"mode\":\"wasm\""));
    assert!(json_str.contains("\"compile_url\":\"\""));
    assert!(json_str.contains("\"native_languages\":[]"));

    let (status, _, _) = send_raw_get(port, "/../Cargo.toml");
    assert_eq!(status, 404);

    let (status, _, _) = send_raw_get(port, "/unknown-path");
    assert_eq!(status, 404);

    let (status, _, _) = send_raw_post(port, "/api/compile", b"{}");
    assert_eq!(status, 404);

    handle.stop();
}

#[test]
fn test_playground_local_mode_compile_hook() {
    let port = free_port().expect("allocate free port");
    let hook = Arc::new(|body: &[u8]| {
        let incoming: serde_json::Value = serde_json::from_slice(body).unwrap();
        serde_json::to_vec(&serde_json::json!({
            "protocol_version": 1,
            "revision": incoming["revision"],
            "language": "rocci",
            "roc": "hello = |{}| {}",
            "ast": "(component Hello)",
            "html": "<p>hi</p>",
            "diagnostics": [],
            "highlights": { "source": [], "roc": [], "ast": [] },
            "capabilities": {
                "roc": { "available": true },
                "ast": { "available": true },
                "html": { "available": true, "reason": "" }
            },
            "has_errors": false
        }))
        .unwrap()
    });
    let mut handle = start_playground_server(
        "Hello.rocci",
        "@component Hello = |{}| { <p>hi</p> }",
        "rocci",
        port,
        PlaygroundMode::Local,
        Some(hook),
        &["rocci"],
        false,
    )
    .expect("start server");

    let (status, _, body) = wait_for_get(port, "/api/session");
    assert_eq!(status, 200);
    let json_str = String::from_utf8_lossy(&body);
    assert!(json_str.contains("\"mode\":\"local\""));
    assert!(json_str.contains("\"compile_url\":\"/api/compile\""));
    assert!(json_str.contains("\"native_languages\":[\"rocci\"]"));
    assert!(json_str.contains("\"available\":true"));

    let req = serde_json::json!({
        "protocol_version": 1,
        "revision": 7,
        "filename": "Hello.rocci",
        "language": "rocci",
        "source": "@component Hello = |{}| { <p>hi</p> }"
    });
    let req_bytes = serde_json::to_vec(&req).unwrap();
    let (status, headers, body) = send_raw_post(port, "/api/compile", &req_bytes);
    assert_eq!(status, 200);
    assert!(headers.contains("Content-Type: application/json; charset=utf-8"));
    let json_str = String::from_utf8_lossy(&body);
    assert!(json_str.contains("\"html\":\"<p>hi</p>\""));
    assert!(json_str.contains("\"revision\":7"));

    let (status, _, _) = send_raw_get(port, "/api/compile");
    assert_eq!(status, 405);

    let (status, _, _) = exchange(
        port,
        &format!(
            "POST /api/compile HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            1_048_576 + 1
        ),
        &[],
    )
    .expect("oversized POST response");
    assert_eq!(status, 413);

    handle.stop();
}
