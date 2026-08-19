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
    let root = repo_root().join("examples/rocdown-hybrid");
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

    let index = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(index.contains("Prose stays Markdown."), "{index}");
    assert!(index.contains("Show tip"), "{index}");
    assert!(index.contains("reveal-tip"), "{index}");
    assert!(index.contains("<script"), "{index}");
    assert!(index.contains("datastar"), "{index}");

    let about = fs::read_to_string(output.join("about/index.html")).unwrap();
    assert!(about.contains("static CDN HTML"), "{about}");
    assert!(!about.to_ascii_lowercase().contains("<script"), "{about}");
    assert!(!about.contains("/assets/datastar"), "{about}");
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
    drop(child);
}
