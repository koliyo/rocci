use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use rocci_browser::{Host, OpenRequest, Paths};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn python() -> PathBuf {
    ["python3", "python"]
        .into_iter()
        .map(PathBuf::from)
        .find(|bin| {
            Command::new(bin)
                .arg("-c")
                .arg("import sys")
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        })
        .unwrap_or_else(|| PathBuf::from("python3"))
}

fn fixture_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adapter.py")
}

fn temp_paths() -> (Paths, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "rocci-browser-host-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let browser_dir = root.join("browser");
    let project = root.join("fixture");
    fs::create_dir_all(browser_dir.join("plugins")).unwrap();
    fs::create_dir_all(&project).unwrap();
    let paths = Paths::new(browser_dir, root);
    (paths, project)
}

fn write_plugin(paths: &Paths, id: &str, adapter_id: &str, label: &str) {
    let wrapper = paths.browser_dir.join(format!("{id}.sh"));
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nexport ROCCI_BROWSER_FIXTURE_ID={adapter_id}\nexport ROCCI_BROWSER_FIXTURE_LABEL={label}\nexec {} -u {} \"$@\"\n",
            python().display(),
            fixture_script().display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&wrapper).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wrapper, perms).unwrap();
    }
    fs::write(
        paths.plugins_dir().join(format!("{id}.toml")),
        format!(
            "id = \"{id}\"\nbin = \"{}\"\nargv = []\n",
            wrapper.display()
        ),
    )
    .unwrap();
}

fn http_get(url: &str) -> String {
    let without = url
        .trim_end_matches('/')
        .strip_prefix("http://")
        .expect("http url");
    let (host, path) = without.split_once('/').unwrap_or((without, ""));
    let path = if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{path}")
    };
    let mut stream = TcpStream::connect(host).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut body = String::new();
    BufReader::new(stream).read_to_string(&mut body).unwrap();
    body
}

#[test]
fn protocol_round_trip_against_fixture() {
    let (paths, project) = temp_paths();
    write_plugin(&paths, "fixture", "fixture", "Fixture");
    let mut host = Host::connect(paths.clone()).unwrap();
    host.add_project("fixture".into(), project.display().to_string())
        .unwrap();
    let targets = host.probe_targets().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].label, "Fixture");
    let documents = host.list_documents("fixture", &targets[0].path).unwrap();
    assert_eq!(documents.len(), 2);
    let opened = host
        .open(OpenRequest {
            query: "fixture",
            document: None,
        })
        .unwrap();
    assert_eq!(opened.title, "Hello");
    assert!(http_get(&opened.url).contains("hello"));
    let about = host
        .open(OpenRequest {
            query: "fixture",
            document: Some("about"),
        })
        .unwrap();
    assert_eq!(about.title, "About");
    assert!(http_get(&about.url).contains("about"));
}

#[test]
fn two_adapters_claiming_one_path_are_two_targets() {
    let (paths, project) = temp_paths();
    write_plugin(&paths, "alpha", "alpha", "Alpha");
    write_plugin(&paths, "beta", "beta", "Beta");
    let mut host = Host::connect(paths).unwrap();
    host.add_project("shared".into(), project.display().to_string())
        .unwrap();
    let mut targets = host.probe_targets().unwrap();
    targets.sort_by(|a, b| a.adapter_id.cmp(&b.adapter_id));
    assert_eq!(targets.len(), 2, "{targets:?}");
    assert_eq!(targets[0].adapter_id, "alpha");
    assert_eq!(targets[0].label, "Alpha");
    assert_eq!(targets[1].adapter_id, "beta");
    assert_eq!(targets[1].label, "Beta");
    assert_eq!(targets[0].path, targets[1].path);
}

#[test]
fn registry_file_lives_under_browser_dir() {
    let (paths, project) = temp_paths();
    write_plugin(&paths, "fixture", "fixture", "Fixture");
    let host = Host::connect(paths.clone()).unwrap();
    host.add_project("fixture".into(), project.display().to_string())
        .unwrap();
    let raw = fs::read_to_string(paths.projects_path()).unwrap();
    assert!(raw.contains("fixture"));
    assert!(raw.contains(&project.display().to_string()));
}

#[test]
fn cli_open_json_prints_fixture_url() {
    let (paths, project) = temp_paths();
    write_plugin(&paths, "fixture", "fixture", "Fixture");
    let host = Host::connect(paths.clone()).unwrap();
    host.add_project("fixture".into(), project.display().to_string())
        .unwrap();
    drop(host);

    let bin = env!("CARGO_BIN_EXE_rocci-browser");
    let mut child = Command::new(bin)
        .env("ROCCI_BROWSER_DIR", &paths.browser_dir)
        .args(["open", "fixture", "--no-window", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).unwrap();
    let value: serde_json::Value = serde_json::from_str(line.trim()).expect(line.trim());
    let url = value["url"].as_str().unwrap();
    assert!(url.starts_with("http://127.0.0.1:"), "{value}");
    assert_eq!(value["title"], "Hello");
    assert!(http_get(url).contains("hello"));
    drop(child.stdin.take());
    let _ = child.wait();

    let mut child = Command::new(bin)
        .env("ROCCI_BROWSER_DIR", &paths.browser_dir)
        .args([
            "open",
            "fixture",
            "--document",
            "about",
            "--no-window",
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).unwrap();
    let value: serde_json::Value = serde_json::from_str(line.trim()).expect(line.trim());
    assert_eq!(value["title"], "About");
    assert!(http_get(value["url"].as_str().unwrap()).contains("about"));
    drop(child.stdin.take());
    let _ = child.wait();
}

#[test]
fn crate_src_does_not_name_other_products() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    visit_rs(&src, &mut files);
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        if file.file_name().is_some_and(|name| name == "package.rs") {
            continue;
        }
        for needle in [".rocdown", "okf_version", "rocci-okf"] {
            assert!(
                !text.contains(needle),
                "{} mentions {needle}",
                file.display()
            );
        }
    }
}

#[test]
#[ignore = "opens a display; session switching is covered by SessionTable unit tests"]
fn graphical_two_opens_require_a_display() {}

fn visit_rs(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            visit_rs(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
