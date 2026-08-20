use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

static ROC_LOCK: Mutex<()> = Mutex::new(());

fn skip_without_roc() -> bool {
    let help_ok = Command::new("roc")
        .arg("help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !help_ok {
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
            panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
        }
        eprintln!("skipping: roc not on PATH");
        return true;
    }
    false
}

fn rocdown_bin() -> PathBuf {
    let mut path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rocdown");
    if !path.is_file() {
        path = PathBuf::from(env!("CARGO_BIN_EXE_rocdown"));
    }
    path
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn temp_dir(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("rocdown-islands-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            unsafe extern "C" {
                fn kill(pid: i32, sig: i32) -> i32;
            }
            let _ = unsafe { kill(-(self.0.id() as i32), 9) };
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn http_exchange(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let _ = stream.flush();
    let mut body = Vec::new();
    let _ = stream.read_to_end(&mut body);
    String::from_utf8_lossy(&body).into_owned()
}

fn wait_for_health(port: u16, child: &mut Child) {
    let start = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            panic!("island service exited before /health ({status})");
        }
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let req = format!(
                "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
            );
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut body = Vec::new();
                let _ = stream.read_to_end(&mut body);
                let text = String::from_utf8_lossy(&body);
                if text.contains("ok") || text.contains("200") {
                    return;
                }
            }
        }
        if start.elapsed() > Duration::from_secs(120) {
            panic!("timed out waiting for island service on port {port}");
        }
        thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn hybrid_cdn_html_and_island_post_morph() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = repo_root().join("examples/rocdown/hybrid");
    let output = temp_dir("cdn");
    let bin = rocdown_bin();

    let build = Command::new(&bin)
        .args(["build", root.to_str().unwrap(), "--output"])
        .arg(&output)
        .current_dir(repo_root())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "build failed: {}\n{}",
        String::from_utf8_lossy(&build.stderr),
        String::from_utf8_lossy(&build.stdout)
    );
    let summary = String::from_utf8_lossy(&build.stdout);
    assert!(summary.contains("live"), "{summary}");
    assert!(summary.contains("datastar: yes"), "{summary}");
    assert!(summary.contains("POST /actions/reveal/show"), "{summary}");

    let index = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(index.contains("Prose stays Markdown."), "{index}");
    assert!(index.contains("Show tip"), "{index}");
    assert!(index.contains("reveal-tip"), "{index}");
    assert!(index.contains("<script"), "{index}");
    assert!(index.contains("datastar"), "{index}");

    let about = fs::read_to_string(output.join("about/index.html")).unwrap();
    assert!(about.contains("static CDN HTML"), "{about}");
    assert!(about.contains("<script"), "{about}");
    assert!(about.contains("goto."), "{about}");
    assert!(!about.contains("/assets/datastar"), "{about}");
    assert!(!about.to_ascii_lowercase().contains("datastar"), "{about}");

    let widgets = fs::read_to_string(output.join("widgets/index.html")).unwrap();
    assert!(widgets.contains("3 core ideas"), "{widgets}");
    assert!(
        widgets.contains("script-src 'self'") || widgets.contains("script-src &#39;self&#39;"),
        "{widgets}"
    );
    assert!(widgets.contains("goto."), "{widgets}");
    assert!(!widgets.contains("datastar"), "{widgets}");

    let pair = fs::read_to_string(output.join("pair/index.html")).unwrap();
    assert!(pair.contains("id=\"slot-alpha\""), "{pair}");
    assert!(pair.contains("id=\"slot-beta\""), "{pair}");
    assert!(pair.contains("Open alpha"), "{pair}");
    assert!(pair.contains("Open beta"), "{pair}");
    assert!(pair.contains("datastar"), "{pair}");
    let islands: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("islands.json")).unwrap()).unwrap();
    assert!(
        islands["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["method"] == "POST" && route["path"] == "/actions/reveal/show"),
        "{islands}"
    );
    let _ = fs::remove_dir_all(&output);

    let port = rocci_cli::serve::free_port().unwrap();
    let mut child = Command::new(&bin)
        .args([
            "serve-islands",
            root.to_str().unwrap(),
            "--no-window",
            "--port",
            &port.to_string(),
        ])
        .current_dir(repo_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    wait_for_health(port, &mut child);
    let child = KillOnDrop(child);

    let response = http_exchange(
        port,
        &format!(
            "POST /actions/reveal/show HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        response.contains("Hide tip"),
        "POST should morph the host to the open markup:\n{response}"
    );

    let alpha = http_exchange(
        port,
        &format!(
            "POST /actions/alpha/show HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        alpha.contains("id=\"slot-alpha\"") || alpha.contains("id='slot-alpha'"),
        "alpha patch must target slot-alpha:\n{alpha}"
    );
    assert!(alpha.contains("Alpha open"), "{alpha}");
    assert!(
        !alpha.contains("slot-beta"),
        "alpha patch must not include slot-beta:\n{alpha}"
    );

    let beta = http_exchange(
        port,
        &format!(
            "POST /actions/beta/show HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        beta.contains("id=\"slot-beta\"") || beta.contains("id='slot-beta'"),
        "beta patch must target slot-beta:\n{beta}"
    );
    assert!(beta.contains("Beta open"), "{beta}");
    assert!(
        !beta.contains("slot-alpha"),
        "beta patch must not include slot-alpha:\n{beta}"
    );
    drop(child);
}

#[test]
fn hybrid_run_serves_cdn_and_islands_on_one_origin() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = repo_root().join("examples/rocdown/hybrid");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut child = Command::new(&bin)
        .args([
            "run",
            root.to_str().unwrap(),
            "--no-window",
            "--port",
            &port.to_string(),
        ])
        .current_dir(repo_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    wait_for_preview(port, &mut child, "Show tip");
    let child = KillOnDrop(child);

    let home = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(home.contains("Prose stays Markdown."), "{home}");
    assert!(home.contains("Show tip"), "{home}");
    assert!(home.contains("datastar"), "{home}");

    let widgets = http_exchange(
        port,
        &format!("GET /widgets/ HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(widgets.contains("3 core ideas"), "{widgets}");
    assert!(
        !widgets.contains("/assets/datastar") && !widgets.contains("datastar.js"),
        "hydrate preview must not load Datastar:\n{widgets}"
    );

    let about = http_exchange(
        port,
        &format!("GET /about/ HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(about.contains("static CDN HTML"), "{about}");
    assert!(!about.contains("/assets/datastar"), "{about}");

    let response = http_exchange(
        port,
        &format!(
            "POST /actions/reveal/show HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        response.contains("Hide tip"),
        "same-origin preview must proxy island POST:\n{response}"
    );
    assert!(
        !response.contains("<style"),
        "island patches must not re-embed CSS:\n{response}"
    );

    let alpha = http_exchange(
        port,
        &format!(
            "POST /actions/alpha/show HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        alpha.contains("slot-alpha") && !alpha.contains("slot-beta"),
        "{alpha}"
    );
    drop(child);
}

#[test]
fn docs_run_previews_the_site() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = repo_root().join("docs");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut child = Command::new(&bin)
        .args([
            "run",
            root.to_str().unwrap(),
            "--no-window",
            "--port",
            &port.to_string(),
        ])
        .current_dir(repo_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    wait_for_preview(port, &mut child, "Documentation Portal");
    let child = KillOnDrop(child);

    let home = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(
        !home.contains("no built site yet"),
        "persist-HTML preview must serve index.html:\n{home}"
    );
    assert!(home.contains("Documentation Portal"), "{home}");
    assert!(
        !home.contains("/assets/datastar") && !home.to_ascii_lowercase().contains("datastar.js"),
        "docs must stay static:\n{home}"
    );

    let guide = http_exchange(
        port,
        &format!(
            "GET /guides/docs-components/ HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(guide.contains("Write documentation components"), "{guide}");
    assert!(
        guide.contains("Static pages"),
        "widget forest must still paint :note:\n{guide}"
    );
    drop(child);
}

#[test]
fn counter_run_proxies_actions_on_one_origin() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = repo_root().join("examples/rocdown/counter");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut child = Command::new(&bin)
        .args([
            "run",
            root.to_str().unwrap(),
            "--no-window",
            "--port",
            &port.to_string(),
        ])
        .current_dir(repo_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    wait_for_preview(port, &mut child, "Shared count");
    let child = KillOnDrop(child);

    let home = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(home.contains("Prose stays Markdown."), "{home}");
    assert!(home.contains("Shared count"), "{home}");
    assert!(home.contains("Increment"), "{home}");
    assert!(home.contains("datastar"), "{home}");
    assert!(
        !home.contains("no built site yet"),
        "GET / must stay CDN-owned:\n{home}"
    );

    let about = http_exchange(
        port,
        &format!("GET /about/ HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(about.contains("static CDN HTML"), "{about}");
    assert!(!about.contains("/assets/datastar"), "{about}");

    let increment = http_exchange(
        port,
        &format!(
            "POST /actions/counter/increment HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        increment.contains("counter") && (increment.contains(">1<") || increment.contains(">1</")),
        "same-origin preview must proxy island POST:\n{increment}"
    );
    assert!(
        !increment.contains("<style"),
        "island patches must not re-embed CSS already on the CDN page:\n{increment}"
    );
    drop(child);
}

#[test]
fn all_syntax_run_serves_the_kitchen_sink() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let fixture = repo_root().join("test/AllSyntax.rocdown");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut child = Command::new(&bin)
        .args([
            "run",
            fixture.to_str().unwrap(),
            "--no-window",
            "--port",
            &port.to_string(),
        ])
        .current_dir(repo_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    wait_for_preview_or_roc_error(port, &mut child, "Don't do this.");
    let child = KillOnDrop(child);

    let home = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(home.contains("Don't do this."), "{home}");
    assert!(
        home.contains("Hello, render") || home.contains("Hello, island"),
        "{home}"
    );
    drop(child);
}

fn wait_for_preview_or_roc_error(port: u16, child: &mut Child, needle: &str) {
    let start = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            panic!("preview server exited before ready ({status})");
        }
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let req =
                format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut body = Vec::new();
                let _ = stream.read_to_end(&mut body);
                let text = String::from_utf8_lossy(&body);
                if text.contains(needle) {
                    return;
                }
                if text.contains("does not exist")
                    || text.contains("too many args")
                    || text.contains("ROC CRASHED")
                    || text.contains("runtime error")
                {
                    panic!("AllSyntax failed to compile with Roc:\n{text}");
                }
            }
        }
        if start.elapsed() > Duration::from_secs(180) {
            panic!("timed out waiting for AllSyntax preview on port {port}");
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn wait_for_preview(port: u16, child: &mut Child, needle: &str) {
    let start = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            panic!("preview server exited before ready ({status})");
        }
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let req =
                format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut body = Vec::new();
                let _ = stream.read_to_end(&mut body);
                let text = String::from_utf8_lossy(&body);
                if text.contains(needle) {
                    return;
                }
            }
        }
        if start.elapsed() > Duration::from_secs(180) {
            panic!("timed out waiting for preview on port {port}");
        }
        thread::sleep(Duration::from_millis(200));
    }
}
