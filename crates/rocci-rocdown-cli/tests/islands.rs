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
    if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() != Some("1") {
        eprintln!("skipping: ROCCI_REQUIRE_ROC is not 1");
        return true;
    }
    let help_ok = Command::new("roc")
        .arg("help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !help_ok {
        panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
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

fn stage_example_docs() {
    let root = repo_root();
    rocci_docs::generate(
        &root.join("examples/rocci/apps.toml"),
        &root.join("dist/example-docs"),
    )
    .expect("stage dist/example-docs for docs [[peer]] routes");
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

struct Spawned {
    child: KillOnDrop,
    roc_log: std::sync::Arc<Mutex<String>>,
}

fn spawn_kill_on_drop(mut command: Command) -> Spawned {
    command.stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut stderr = child.stderr.take().expect("piped stderr");
    let roc_log = std::sync::Arc::new(Mutex::new(String::new()));
    let captured = roc_log.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match stderr.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    eprint!("{chunk}");
                    captured
                        .lock()
                        .unwrap_or_else(|err| err.into_inner())
                        .push_str(&chunk);
                }
                Err(_) => break,
            }
        }
    });
    Spawned {
        child: KillOnDrop(child),
        roc_log,
    }
}

fn assert_no_roc_errors(log: &Mutex<String>) {
    thread::sleep(Duration::from_millis(50));
    let text = log.lock().unwrap_or_else(|err| err.into_inner()).clone();
    if text.contains("── ✗") {
        panic!("Roc reported an error while the preview stayed up:\n{text}");
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

fn http_get(port: u16, path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).ok()?;
    let mut body = Vec::new();
    let _ = stream.read_to_end(&mut body);
    Some(String::from_utf8_lossy(&body).into_owned())
}

fn panic_if_preview_failed(body: &str, context: &str) {
    if body.contains("rocci-build-error")
        || body.contains("island service failed")
        || body.contains("absolute platform path")
        || body.contains("ROC CRASHED")
        || body.contains("<title>Build error</title>")
    {
        panic!("{context}:\n{body}");
    }
}

fn wait_for_health(port: u16, child: &mut Child) {
    let start = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            panic!("island service exited before /health ({status})");
        }
        if let Some(home) = http_get(port, "/") {
            panic_if_preview_failed(
                &home,
                "preview served a build error while waiting for /health",
            );
        }
        if let Some(text) = http_get(port, "/health") {
            panic_if_preview_failed(&text, "GET /health served a build error");
            if text.contains("ok") && !text.contains("<html") {
                return;
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
    let _lock = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
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
    let mut spawned = spawn_kill_on_drop({
        let mut command = Command::new(&bin);
        command
            .args([
                "serve-islands",
                root.to_str().unwrap(),
                "--no-window",
                "--port",
                &port.to_string(),
            ])
            .current_dir(repo_root())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
    });
    wait_for_health(port, &mut spawned.child.0);
    assert_no_roc_errors(&spawned.roc_log);

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
    drop(spawned.child);
}

#[test]
fn hybrid_run_serves_cdn_and_islands_on_one_origin() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let root = repo_root().join("examples/rocdown/hybrid");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut spawned = spawn_kill_on_drop({
        let mut command = Command::new(&bin);
        command
            .args([
                "view",
                root.to_str().unwrap(),
                "--no-window",
                "--port",
                &port.to_string(),
            ])
            .current_dir(repo_root())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
    });
    wait_for_preview(port, &mut spawned.child.0, "Show tip");
    assert_no_roc_errors(&spawned.roc_log);

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
    drop(spawned.child);
}

#[test]
fn docs_run_previews_the_site() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    stage_example_docs();
    let root = repo_root().join("docs");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut spawned = spawn_kill_on_drop({
        let mut command = Command::new(&bin);
        command
            .args([
                "view",
                root.to_str().unwrap(),
                "--no-window",
                "--port",
                &port.to_string(),
            ])
            .current_dir(repo_root())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
    });
    wait_for_preview(port, &mut spawned.child.0, "Overview");
    assert_no_roc_errors(&spawned.roc_log);

    let home = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(
        !home.contains("no built site yet"),
        "persist-HTML preview must serve index.html:\n{home}"
    );
    assert!(home.contains("Overview"), "{home}");
    assert!(
        !home.contains("/assets/datastar") && !home.to_ascii_lowercase().contains("datastar.js"),
        "docs must stay static:\n{home}"
    );
    drop(spawned.child);
}

#[test]
fn counter_run_proxies_actions_on_one_origin() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let root = repo_root().join("examples/rocdown/counter");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut spawned = spawn_kill_on_drop({
        let mut command = Command::new(&bin);
        command
            .args([
                "view",
                root.to_str().unwrap(),
                "--no-window",
                "--port",
                &port.to_string(),
            ])
            .current_dir(repo_root())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
    });
    wait_for_preview(port, &mut spawned.child.0, "Shared count");
    assert_no_roc_errors(&spawned.roc_log);

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
            "POST /actions/counter/increment HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(increment.contains("204"), "{increment}");
    assert!(!increment.contains("application/json"), "{increment}");
    assert!(
        !increment.contains("datastar-patch-elements"),
        "{increment}"
    );
    let datastar_increment = http_exchange(
        port,
        &format!(
            "POST /actions/counter/increment HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        datastar_increment.contains("HTTP/1.1 200 OK")
            && datastar_increment.contains("content-type: text/event-stream")
            && !datastar_increment.contains("datastar-patch-elements"),
        "representation-free command from Datastar returns an empty SSE response:\n{datastar_increment}"
    );
    drop(spawned.child);
}

#[test]
fn all_syntax_run_serves_the_kitchen_sink() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let fixture = repo_root().join("test/AllSyntax.rocdown");
    let bin = rocdown_bin();
    let port = rocci_cli::serve::free_port().unwrap();
    let mut spawned = spawn_kill_on_drop({
        let mut command = Command::new(&bin);
        command
            .args([
                "view",
                fixture.to_str().unwrap(),
                "--no-window",
                "--port",
                &port.to_string(),
            ])
            .current_dir(repo_root())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
    });
    wait_for_preview(port, &mut spawned.child.0, "Don't do this.");
    assert_no_roc_errors(&spawned.roc_log);

    let home = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(home.contains("Don't do this."), "{home}");
    assert!(
        home.contains("Hello, render") || home.contains("Hello, island"),
        "{home}"
    );
    drop(spawned.child);
}

fn wait_for_preview(port: u16, child: &mut Child, needle: &str) {
    let start = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            panic!("preview server exited before ready ({status})");
        }
        if let Some(text) = http_get(port, "/") {
            panic_if_preview_failed(&text, "preview served a build error");
            if text.contains(needle) {
                return;
            }
            if text.contains("</html>") {
                panic!("preview HTML did not contain {needle:?}:\n{text}");
            }
        }
        if start.elapsed() > Duration::from_secs(180) {
            panic!("timed out waiting for preview on port {port}");
        }
        thread::sleep(Duration::from_millis(200));
    }
}
