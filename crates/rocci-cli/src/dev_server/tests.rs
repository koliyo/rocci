use std::io::{Read, Write};
use std::time::{Duration, Instant};

use super::http::*;
use super::routing::*;
use super::*;

#[test]
fn test_reload_hub_subscription_and_broadcast() {
    let hub = ReloadHub::new();
    let rx1 = hub.subscribe();
    let rx2 = hub.subscribe();

    hub.broadcast();
    assert_eq!(rx1.recv().unwrap(), 1);
    assert_eq!(rx2.recv().unwrap(), 1);

    hub.broadcast();
    assert_eq!(rx1.recv().unwrap(), 2);
    assert_eq!(rx2.recv().unwrap(), 2);
}

#[test]
fn preview_html_csp_allows_unsafe_eval_for_datastar() {
    assert!(
        PREVIEW_HTML_CSP.contains("'unsafe-eval'"),
        "preview header CSP must allow Datastar Function() eval: {PREVIEW_HTML_CSP}"
    );
}

#[test]
fn test_inject_live_reload_and_relax_csp() {
    let html = "<!doctype html><html><head><meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'none'; connect-src 'none'\"></head><body><h1>Hello</h1></body></html>";
    let injected = inject_live_reload(html);
    assert!(injected.contains("<script src=\"/__rocci/reload.js\" defer></script>"));
    assert!(injected.contains("script-src 'self'"));
    assert!(injected.contains("connect-src 'self'"));
    assert!(!injected.contains("script-src 'none'"));

    let rocdown = concat!(
        "<!doctype html><html><head><meta http-equiv=\"Content-Security-Policy\" content=\"",
        "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; ",
        "font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; ",
        "frame-ancestors 'none'; form-action 'none\"></head><body><h1>Docs</h1></body></html>"
    );
    let relaxed = inject_live_reload(rocdown);
    assert!(relaxed.contains("frame-src 'self'"));
    assert!(relaxed.contains("default-src 'none'; frame-src 'self'"));
    assert!(relaxed.contains("<script src=\"/__rocci/reload.js\" defer></script>"));

    let okf =
        "<!doctype html><html><head><title>OKF</title></head><body><h1>Review</h1></body></html>";
    let okf_injected = inject_live_reload(okf);
    assert!(okf_injected.contains("<script src=\"/__rocci/reload.js\" defer></script>"));
    assert!(!okf_injected.contains("frame-src"));
    assert!(!okf_injected.contains("Content-Security-Policy"));
}

#[test]
fn reload_js_honors_live_reload_storage() {
    assert!(RELOAD_JS.contains("window.__rocciLiveReload"));
    assert!(RELOAD_JS.contains("rocci-live-reload"));
    assert!(RELOAD_JS.contains("sessionStorage.getItem(KEY) !== \"0\""));
    assert!(RELOAD_JS.contains("if (enabled())"));
    assert!(RELOAD_JS.contains("dirty = true"));
    assert!(RELOAD_JS.contains("if (on && dirty)"));
    assert!(RELOAD_JS.contains("if (window.__rocciLiveReload)"));
    assert!(RELOAD_JS.contains("URLSearchParams"));
    assert!(RELOAD_JS.contains("get(\"reload\") === \"0\""));
    assert!(RELOAD_JS.contains("seedFromQuery"));
    assert!(RELOAD_JS.contains("location.reload()"));
}

#[test]
fn test_resolve_request_routing() {
    let temp = std::env::temp_dir().join(format!("rocci-dev-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(temp.join("guide")).unwrap();
    fs::write(temp.join("index.html"), "<h1>Home</h1>").unwrap();
    fs::write(temp.join("guide").join("index.html"), "<h1>Guide</h1>").unwrap();
    fs::write(temp.join("about.html"), "<h1>About</h1>").unwrap();

    assert_eq!(
        resolve_request(&temp, "/__rocci/events"),
        ServeTarget::Events
    );
    assert_eq!(
        resolve_request(&temp, "/__rocdown/events"),
        ServeTarget::Events
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci_okf/events"),
        ServeTarget::Events
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci/profile"),
        ServeTarget::Profile
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci_okf/profile"),
        ServeTarget::Profile
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci/inspect"),
        ServeTarget::Inspect
    );
    assert_eq!(
        resolve_request(&temp, "/__rocdown/inspect?route=/"),
        ServeTarget::Inspect
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci_okf/inspect"),
        ServeTarget::Inspect
    );
    assert_eq!(resolve_request(&temp, "/__rocci/dev"), ServeTarget::Dev);
    assert_eq!(resolve_request(&temp, "/__rocdown/dev"), ServeTarget::Dev);
    assert_eq!(resolve_request(&temp, "/__rocci/logs"), ServeTarget::Logs);
    assert_eq!(
        resolve_request(&temp, "/__rocdown/logs/events"),
        ServeTarget::LogEvents
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci_okf/logs/clear"),
        ServeTarget::LogClear
    );
    assert_eq!(
        resolve_request(&temp, "/__rocci/reload.js"),
        ServeTarget::ReloadJs
    );
    assert_eq!(
        resolve_request(&temp, "/"),
        ServeTarget::File {
            relative: "index.html".into()
        }
    );
    assert_eq!(
        resolve_request(&temp, "/guide"),
        ServeTarget::Redirect("/guide/".into())
    );
    assert_eq!(
        resolve_request(&temp, "/guide/"),
        ServeTarget::File {
            relative: "guide/index.html".into()
        }
    );
    assert_eq!(
        resolve_request(&temp, "/about"),
        ServeTarget::File {
            relative: "about.html".into()
        }
    );
    assert_eq!(
        resolve_request(&temp, "/nonexistent"),
        ServeTarget::NotFound
    );
    assert_eq!(resolve_request(&temp, "/../outside"), ServeTarget::NotFound);

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn published_request_serves_files_without_preview_routes() {
    let temp = std::env::temp_dir().join(format!("rocci-published-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(temp.join("guide")).unwrap();
    fs::write(temp.join("index.html"), "<h1>Home</h1>").unwrap();
    fs::write(temp.join("guide").join("index.html"), "<h1>Guide</h1>").unwrap();

    assert_eq!(
        resolve_published_request(&temp, "/"),
        ServeTarget::File {
            relative: "index.html".into()
        }
    );
    assert_eq!(
        resolve_published_request(&temp, "/__rocci/reload.js"),
        ServeTarget::NotFound
    );
    assert_eq!(
        resolve_published_request(&temp, "/guide"),
        ServeTarget::Redirect("/guide/".into())
    );
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn missing_page_message_depends_on_build_state() {
    assert_eq!(missing_page_message(false), "no built site yet");
    assert_eq!(missing_page_message(true), "page not found");
}

#[test]
fn html_files_are_document_routes() {
    assert!(is_html_file("index.html"));
    assert!(is_html_file("about/index.html"));
    assert!(!is_html_file("theme.css"));
    assert!(!is_html_file("assets/datastar.js"));
}

#[test]
fn client_abort_covers_broken_pipe_reset_and_eof() {
    assert!(is_client_abort(&io::Error::new(
        io::ErrorKind::BrokenPipe,
        "write failed"
    )));
    assert!(is_client_abort(&io::Error::new(
        io::ErrorKind::ConnectionReset,
        "reset"
    )));
    assert!(is_client_abort(&io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "eof"
    )));
    assert!(is_client_abort(&io::Error::other(
        "Broken pipe (os error 32)"
    )));
    assert!(!is_client_abort(&io::Error::new(
        io::ErrorKind::TimedOut,
        "timed out"
    )));
}

#[test]
fn should_proxy_posts_and_missing_gets_to_backend() {
    let temp = std::env::temp_dir().join(format!(
        "rocci-proxy-policy-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&temp).unwrap();
    fs::write(temp.join("404.html"), "<!DOCTYPE html><title>404</title>").unwrap();
    fs::write(
        temp.join("islands.json"),
        r#"{"routes":[{"method":"GET","path":"/health"},{"method":"GET","path":"/sse"}]}"#,
    )
    .unwrap();

    assert!(!should_proxy("GET", "/", &ServeTarget::NotFound, 0, &temp));
    assert!(!should_proxy(
        "GET",
        "/",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(!should_proxy(
        "HEAD",
        "/",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(!should_proxy(
        "GET",
        "/index.html",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(should_proxy(
        "GET",
        "/health",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(should_proxy(
        "GET",
        "/sse",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(should_proxy(
        "GET",
        "/sse?datastar=%7B%22tz%22%3A%22Europe%2FOslo%22%7D",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(!should_proxy(
        "GET",
        "/docs/getting-started/quickstart/",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(should_proxy(
        "POST",
        "/actions/reveal/show",
        &ServeTarget::NotFound,
        9000,
        &temp
    ));
    assert!(should_proxy(
        "POST",
        "/",
        &ServeTarget::File {
            relative: "index.html".into()
        },
        9000,
        &temp
    ));
    assert!(!should_proxy(
        "GET",
        "/",
        &ServeTarget::File {
            relative: "index.html".into()
        },
        9000,
        &temp
    ));
    assert!(!should_proxy(
        "GET",
        "/__rocci/events",
        &ServeTarget::Events,
        9000,
        &temp
    ));
    assert!(!should_proxy(
        "POST",
        "/__rocci/events",
        &ServeTarget::Events,
        9000,
        &temp
    ));
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn remaining_body_uses_content_length() {
    let request = b"POST /actions/x HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 7\r\n\r\nhello";
    assert_eq!(remaining_body(request), 2);
    let complete = b"POST /actions/x HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n";
    assert_eq!(remaining_body(complete), 0);
}

#[test]
fn force_connection_close_replaces_keep_alive() {
    let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: keep-alive\r\nKeep-Alive: timeout=5\r\n";
    let rewritten = force_connection_close(headers);
    assert!(rewritten.contains("Connection: close\r\n"), "{rewritten}");
    assert!(!rewritten.to_ascii_lowercase().contains("keep-alive"));
    assert_eq!(
        rewritten
            .lines()
            .filter(|line| line.to_ascii_lowercase().starts_with("connection:"))
            .count(),
        1
    );
}

#[test]
fn static_server_proxies_unmatched_posts() {
    let output = std::env::temp_dir().join(format!(
        "rocci-proxy-out-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("index.html"), "<h1>cdn</h1>").unwrap();

    let backend_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let backend_port = backend_listener.local_addr().unwrap().port();
    let backend = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = backend_listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            assert!(request.starts_with("POST /actions/x"), "{request}");
            assert!(
                request.to_ascii_lowercase().contains("connection: close"),
                "proxy must forward Connection: close:\n{request}"
            );
            let payload = b"event: datastar-patch-elements\ndata: elements ok\n\n";
            let body = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: keep-alive\r\nKeep-Alive: timeout=5\r\n\r\n",
                payload.len()
            );
            stream.write_all(body.as_bytes()).unwrap();
            stream.write_all(payload).unwrap();
        }
    });

    let port = crate::serve::free_port().unwrap();
    let advertised = Arc::new(AtomicU16::new(backend_port));
    let server = serve_static_site(
        StaticDevServerConfig {
            title: "proxy".into(),
            port,
            open_path: "/".into(),
            output: Some(output.clone()),
            watch_paths: Vec::new(),
            custom_filter: None,
            log_prefix: "test".into(),
            backend_port: Some(advertised),
            log_handlers: false,
            on_stop: None,
            public: false,
            extra_http: None,
        },
        |_, _| Ok(None),
    )
    .unwrap();

    let logged = server.logs.snapshot();
    assert!(
        logged.iter().any(|line| {
            line.text.contains("preview files at")
                && line.text.contains(&output.display().to_string())
        }),
        "{logged:?}"
    );

    for _ in 0..2 {
        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client
            .write_all(b"POST /actions/x HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        assert!(response.contains("datastar-patch-elements"), "{response}");
        assert!(
            response.to_ascii_lowercase().contains("connection: close"),
            "proxied response must advertise close, not keep-alive:\n{response}"
        );
        assert!(
            !response.to_ascii_lowercase().contains("keep-alive"),
            "{response}"
        );
    }

    let mut home = TcpStream::connect(("127.0.0.1", port)).unwrap();
    home.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut html = String::new();
    home.read_to_string(&mut html).unwrap();
    assert!(html.contains("<h1>cdn</h1>"), "{html}");

    drop(server);
    let _ = backend.join();
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn proxy_flushes_sse_events_before_backend_closes() {
    let output = std::env::temp_dir().join(format!(
        "rocci-proxy-sse-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("index.html"), "<h1>cdn</h1>").unwrap();

    let backend_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let backend_port = backend_listener.local_addr().unwrap().port();
    let (continue_tx, continue_rx) = mpsc::channel::<()>();
    let backend = thread::spawn(move || {
        let (mut stream, _) = backend_listener.accept().unwrap();
        let mut buf = [0u8; 2048];
        let n = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        assert!(request.starts_with("GET /sse"), "{request}");
        stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
                )
                .unwrap();
        stream
            .write_all(b"event: datastar-patch-elements\ndata: elements first\n\n")
            .unwrap();
        stream.flush().unwrap();
        continue_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        stream
            .write_all(b"event: datastar-patch-elements\ndata: elements second\n\n")
            .unwrap();
        stream.flush().unwrap();
    });

    let port = crate::serve::free_port().unwrap();
    let advertised = Arc::new(AtomicU16::new(backend_port));
    let server = serve_static_site(
        StaticDevServerConfig {
            title: "proxy-sse".into(),
            port,
            open_path: "/".into(),
            output: Some(output.clone()),
            watch_paths: Vec::new(),
            custom_filter: None,
            log_prefix: "test".into(),
            backend_port: Some(advertised),
            log_handlers: false,
            on_stop: None,
            public: false,
            extra_http: None,
        },
        |_, _| Ok(None),
    )
    .unwrap();

    let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client
        .write_all(b"GET /sse HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .unwrap();
    let mut received = Vec::new();
    let mut buf = [0u8; 512];
    let started = Instant::now();
    while !received
        .windows(b"elements first".len())
        .any(|w| w == b"elements first")
    {
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "first SSE event was buffered until the backend closed:\n{}",
            String::from_utf8_lossy(&received)
        );
        let n = client.read(&mut buf).unwrap();
        assert!(n > 0, "proxy closed before the first SSE event");
        received.extend_from_slice(&buf[..n]);
    }
    continue_tx.send(()).unwrap();
    while !received
        .windows(b"elements second".len())
        .any(|w| w == b"elements second")
    {
        let n = client.read(&mut buf).unwrap();
        assert!(n > 0, "proxy closed before the second SSE event");
        received.extend_from_slice(&buf[..n]);
    }

    drop(server);
    let _ = backend.join();
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn proxy_logs_client_closed_when_sse_client_drops() {
    let output = std::env::temp_dir().join(format!(
        "rocci-proxy-sse-abort-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("index.html"), "<h1>cdn</h1>").unwrap();

    let backend_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let backend_port = backend_listener.local_addr().unwrap().port();
    let (continue_tx, continue_rx) = mpsc::channel::<()>();
    let backend = thread::spawn(move || {
        let (mut stream, _) = backend_listener.accept().unwrap();
        let mut buf = [0u8; 2048];
        let n = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        assert!(request.starts_with("GET /sse"), "{request}");
        stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
                )
                .unwrap();
        stream
            .write_all(b"event: datastar-patch-elements\ndata: elements first\n\n")
            .unwrap();
        stream.flush().unwrap();
        continue_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let chunk = vec![b'x'; 64 * 1024];
        loop {
            let mut event = b"event: datastar-patch-elements\ndata: ".to_vec();
            event.extend_from_slice(&chunk);
            event.extend_from_slice(b"\n\n");
            if stream.write_all(&event).is_err() {
                break;
            }
        }
    });

    let port = crate::serve::free_port().unwrap();
    let advertised = Arc::new(AtomicU16::new(backend_port));
    let server = serve_static_site(
        StaticDevServerConfig {
            title: "proxy-sse-abort".into(),
            port,
            open_path: "/".into(),
            output: Some(output.clone()),
            watch_paths: Vec::new(),
            custom_filter: None,
            log_prefix: "test".into(),
            backend_port: Some(advertised),
            log_handlers: true,
            on_stop: None,
            public: false,
            extra_http: None,
        },
        |_, _| Ok(None),
    )
    .unwrap();

    let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client
        .write_all(b"GET /sse HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .unwrap();
    let mut received = Vec::new();
    let mut buf = [0u8; 512];
    let started = Instant::now();
    while !received
        .windows(b"elements first".len())
        .any(|w| w == b"elements first")
    {
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "first SSE event was buffered until the backend closed:\n{}",
            String::from_utf8_lossy(&received)
        );
        let n = client.read(&mut buf).unwrap();
        assert!(n > 0, "proxy closed before the first SSE event");
        received.extend_from_slice(&buf[..n]);
    }
    drop(client);
    continue_tx.send(()).unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    let logged = loop {
        let snapshot = server.logs.snapshot();
        if snapshot
            .iter()
            .any(|line| line.text.contains("client closed"))
        {
            break snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "proxy did not log client closed:\n{snapshot:?}"
        );
        thread::sleep(Duration::from_millis(20));
    };
    assert!(
        logged.iter().all(|line| !line.text.contains("proxy error")),
        "{logged:?}"
    );

    drop(server);
    let _ = backend.join();
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn static_server_serves_rebuild_error_over_stale_html() {
    let output = std::env::temp_dir().join(format!(
        "rocci-rebuild-err-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&output).unwrap();
    let port = crate::serve::free_port().unwrap();
    let server = serve_static_site(
        StaticDevServerConfig {
            title: "rebuild-err".into(),
            port,
            open_path: "/".into(),
            output: Some(output.clone()),
            watch_paths: Vec::new(),
            custom_filter: None,
            log_prefix: "test".into(),
            backend_port: None,
            log_handlers: false,
            on_stop: None,
            public: false,
            extra_http: None,
        },
        |out, _| {
            fs::write(out.join("index.html"), "<h1>cdn</h1>").unwrap();
            fs::write(out.join("theme.css"), "body{color:red}").unwrap();
            anyhow::bail!("name not in scope: read_count!")
        },
    )
    .unwrap();

    let mut home = TcpStream::connect(("127.0.0.1", port)).unwrap();
    home.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut html = String::new();
    home.read_to_string(&mut html).unwrap();
    assert!(html.contains("HTTP/1.1 200"), "{html}");
    assert!(html.contains("<h1>cdn</h1>"), "{html}");
    assert!(html.contains("rocci-build-error"), "{html}");
    assert!(html.contains("Build error"), "{html}");
    assert!(html.contains("read_count!"), "{html}");

    let mut css = TcpStream::connect(("127.0.0.1", port)).unwrap();
    css.write_all(b"GET /theme.css HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut assets = String::new();
    css.read_to_string(&mut assets).unwrap();
    assert!(assets.contains("body{color:red}"), "{assets}");
    assert!(!assets.contains("Build error"), "{assets}");

    drop(server);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn static_server_serves_build_error_shell_when_no_html_exists() {
    let output = std::env::temp_dir().join(format!(
        "rocci-rebuild-shell-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&output).unwrap();
    let port = crate::serve::free_port().unwrap();
    let server = serve_static_site(
        StaticDevServerConfig {
            title: "rebuild-shell".into(),
            port,
            open_path: "/".into(),
            output: Some(output.clone()),
            watch_paths: Vec::new(),
            custom_filter: None,
            log_prefix: "test".into(),
            backend_port: None,
            log_handlers: false,
            on_stop: None,
            public: false,
            extra_http: None,
        },
        |_, _| anyhow::bail!("catalog resolve failed"),
    )
    .unwrap();

    let mut home = TcpStream::connect(("127.0.0.1", port)).unwrap();
    home.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut html = String::new();
    home.read_to_string(&mut html).unwrap();
    assert!(html.contains("HTTP/1.1 200"), "{html}");
    assert!(html.contains("rocci-build-error"), "{html}");
    assert!(html.contains("catalog resolve failed"), "{html}");

    drop(server);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn static_server_serves_inspect_json_after_rebuild() {
    use crate::inspect::{InspectCapabilities, InspectPage, InspectSnapshot, ViewCapability};
    use crate::profile::ProfileSnapshot;
    use std::process::Command;

    let output = std::env::temp_dir().join(format!(
        "rocci-inspect-out-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&output).unwrap();
    let port = crate::serve::free_port().unwrap();
    let server = serve_static_site(
        StaticDevServerConfig {
            title: "inspect".into(),
            port,
            open_path: "/".into(),
            output: Some(output.clone()),
            watch_paths: Vec::new(),
            custom_filter: None,
            log_prefix: "test".into(),
            backend_port: None,
            log_handlers: false,
            on_stop: None,
            public: false,
            extra_http: None,
        },
        |out, _| {
            fs::write(out.join("index.html"), "<h1>home</h1>").unwrap();
            Ok(Some(InspectSnapshot {
                pages: vec![InspectPage {
                    route: "/".into(),
                    path: "index.rocdown".into(),
                    language: "rocdown".into(),
                    source: "<p>source & \"quotes\"</p>".into(),
                    ast: "(Document)".into(),
                    roc: "module [] {}".into(),
                    html: "<h1>home</h1>".into(),
                    source_highlighted: String::new(),
                    capabilities: InspectCapabilities {
                        source: ViewCapability::available(),
                        ast: ViewCapability::available(),
                        roc: ViewCapability::available(),
                        html: ViewCapability::available(),
                    },
                }],
                profile: ProfileSnapshot {
                    total_ms: 2,
                    spans: Vec::new(),
                },
            }))
        },
    )
    .unwrap();

    let url = format!("http://127.0.0.1:{port}/__rocci/inspect?route=/");
    let curl = Command::new("curl")
        .args(["-sS", "-w", "\n%{http_code}", &url])
        .output()
        .expect("curl");
    assert!(curl.status.success(), "curl failed: {curl:?}");
    let stdout = String::from_utf8_lossy(&curl.stdout);
    let (body, status) = stdout.rsplit_once('\n').unwrap_or((&stdout, ""));
    assert_eq!(status.trim(), "200", "{stdout}");
    let value: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(value["path"], "index.rocdown");
    assert_eq!(value["source"], "<p>source & \"quotes\"</p>");
    assert_eq!(value["html"], "<h1>home</h1>");
    assert_eq!(value["profile"]["total_ms"], 2);

    let dev_url = format!("http://127.0.0.1:{port}/__rocci/dev?tab=console");
    let dev = Command::new("curl")
        .args(["-sS", "-w", "\n%{http_code}", &dev_url])
        .output()
        .expect("curl");
    let dev_out = String::from_utf8_lossy(&dev.stdout);
    let (dev_body, dev_status) = dev_out.rsplit_once('\n').unwrap_or((&dev_out, ""));
    assert_eq!(dev_status.trim(), "200", "{dev_out}");
    assert!(dev_body.contains("role=\"tablist\""));
    assert!(!dev_body.contains("/__rocci/reload.js"));

    let logs_url = format!("http://127.0.0.1:{port}/__rocci/logs");
    let logs = Command::new("curl")
        .args(["-sS", "-w", "\n%{http_code}", &logs_url])
        .output()
        .expect("curl");
    let logs_out = String::from_utf8_lossy(&logs.stdout);
    let (logs_body, logs_status) = logs_out.rsplit_once('\n').unwrap_or((&logs_out, ""));
    assert_eq!(logs_status.trim(), "200", "{logs_out}");
    let logs_json: serde_json::Value = serde_json::from_str(logs_body).unwrap();
    assert!(logs_json.is_array());

    drop(server);
    let _ = fs::remove_dir_all(&output);
}
