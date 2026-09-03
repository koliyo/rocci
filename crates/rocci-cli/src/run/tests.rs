use super::*;
use crate::driver::{
    EXTRACTED_STYLESHEET_HREF, roc_command, roc_invocation, stage_app_workspace, window_title,
};
use crate::roc_module::type_name_from_path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

static ROC_LOCK: Mutex<()> = Mutex::new(());

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn temp_app(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("rocci-run-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn live_counter_http_module_plan_links_extracted_css() {
    let path = repo_root().join("examples/rocci/standalone/live-counter/LiveCounter.rocci");
    let plan = standalone_http_module_app_plan(&path).expect("plan live-counter");
    let css = plan.extracted_css();
    assert!(css.contains("counter-card"), "{css}");
    assert!(css.contains("min-height: 100vh"), "{css}");
    assert!(
        plan.modules[0].roc.contains(EXTRACTED_STYLESHEET_HREF),
        "{}",
        plan.modules[0].roc
    );
    let ui = plan
        .modules
        .iter()
        .find(|module| module.type_name == "LiveCounterUi")
        .expect("LiveCounterUi");
    assert!(
        !ui.roc.contains("\"style\""),
        "live fragments must stay style-free\n{}",
        ui.roc
    );
}

fn roc_build_staged_standalone(relative: &str) -> crate::driver::TempDir {
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    roc_build_staged_standalone_locked(relative)
}

fn roc_build_staged_standalone_locked(relative: &str) -> crate::driver::TempDir {
    let primary = repo_root().join(relative);
    let src_dir = primary.parent().unwrap();
    let plan = standalone_app_plan(&primary).expect("plan standalone app");
    let workspace = stage_app_workspace(&plan, src_dir, "roc-build").expect("stage generated app");
    let output = workspace.path.join("server");
    crate::native_target::build_roc_server(&workspace.path, &output, None)
        .unwrap_or_else(|err| panic!("roc build failed for {relative}: {err:#}"));
    assert!(
        output.is_file(),
        "roc build did not write {}",
        output.display()
    );
    workspace
}

fn smoke_server(workspace: &crate::driver::TempDir, label: &str) -> (KillOnDrop, u16) {
    let server = workspace.path.join("server");
    let port = crate::serve::free_port().expect("free port");
    let mut child = KillOnDrop(
        Command::new(&server)
            .current_dir(&workspace.path)
            .env("ROC_BASIC_WEBSERVER_HOST", "127.0.0.1")
            .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap_or_else(|err| panic!("spawn {label} server: {err}")),
    );
    if let Err(err) =
        crate::serve::wait_for_server(&mut child.0, port, crate::logs::Progress::default())
    {
        panic!("{label} server did not listen: {err:#}");
    }
    (child, port)
}

#[test]
fn live_counter_generated_app_roc_builds() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let workspace = roc_build_staged_standalone_locked(
        "examples/rocci/standalone/live-counter/LiveCounter.rocci",
    );
    let main = fs::read_to_string(workspace.path.join("main.roc")).expect("read generated main");
    assert!(
        !main.contains("basic-webserver/releases/download/0.16.0"),
        "{main}"
    );
    assert!(
        main.contains("crates/rocci-platform/platform/main.roc")
            || main.contains("rocci-platform/platform/main.roc"),
        "{main}"
    );
    let (_child, port) = smoke_server(&workspace, "live-counter");
    let document = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(document.contains("200"), "{document}");
    assert!(document.contains("Live counter"), "{document}");
    let increment = http_exchange(
        port,
        &format!(
            "POST /actions/counter/increment HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        increment.contains("204") || increment.contains("200"),
        "{increment}"
    );
}

#[test]
fn counter_generated_app_roc_builds() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let workspace =
        roc_build_staged_standalone_locked("examples/rocci/standalone/counter/Counter.rocci");
    let main = fs::read_to_string(workspace.path.join("main.roc")).expect("read generated main");
    assert!(
        !main.contains("basic-webserver/releases/download/0.16.0"),
        "{main}"
    );
    let (_child, port) = smoke_server(&workspace, "counter");
    let document = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(document.contains("200"), "{document}");
    assert!(document.contains("Welcome to Rocci"), "{document}");
    assert!(document.contains("id=\"counter\""), "{document}");
    let increment = http_exchange(
        port,
        &format!(
            "POST /actions/counter/increment HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        increment.contains("datastar-patch-elements") && increment.contains("counter"),
        "{increment}"
    );
}

#[test]
fn handler_matrix_generated_app_roc_builds() {
    if skip_without_roc() {
        return;
    }
    let _workspace =
        roc_build_staged_standalone("examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci");
}

#[test]
fn multi_page_streams_generated_app_roc_builds() {
    if skip_without_roc() {
        return;
    }
    let _workspace =
        roc_build_staged_standalone("examples/rocci/standalone/multi-page-streams/Dashboard.rocci");
}

fn http_exchange(port: u16, request: &str) -> String {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

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

fn http_stream_sample(port: u16, path: &str, extra_headers: &str) -> String {
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpStream;
    use std::time::{Duration, Instant};

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect stream");
    stream
        .set_read_timeout(Some(Duration::from_millis(150)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{extra_headers}Connection: close\r\n\r\n"
    )
    .unwrap();
    stream.flush().unwrap();

    let deadline = Instant::now() + Duration::from_millis(650);
    let mut body = Vec::new();
    let mut buf = [0u8; 8192];
    while Instant::now() < deadline {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => body.extend_from_slice(&buf[..n]),
            Err(err) if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(err) => panic!("read stream {path}: {err}"),
        }
    }
    String::from_utf8_lossy(&body).into_owned()
}

#[test]
fn handler_matrix_http_smoke() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let workspace = roc_build_staged_standalone_locked(
        "examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci",
    );
    let server = workspace.path.join("server");
    let port = crate::serve::free_port().expect("free port");
    let mut child = KillOnDrop(
        Command::new(&server)
            .current_dir(&workspace.path)
            .env("ROC_BASIC_WEBSERVER_HOST", "127.0.0.1")
            .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn handler-matrix server"),
    );
    if let Err(err) =
        crate::serve::wait_for_server(&mut child.0, port, crate::logs::Progress::default())
    {
        panic!("handler-matrix server did not listen: {err:#}");
    }

    let document = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(document.contains("200"), "{document}");
    assert!(
        document.contains("text/html") || document.contains("<html"),
        "{document}"
    );
    assert!(document.contains("id=\"frag-post\""), "{document}");
    assert!(document.contains("id=\"live-tick\""), "{document}");

    for (method, path, marker) in [
        ("GET", "/fragments/get", "frag-get"),
        ("POST", "/actions/post-frag", "frag-post"),
        ("PUT", "/actions/put-frag", "frag-put"),
        ("PATCH", "/actions/patch-frag", "frag-patch"),
        ("DELETE", "/actions/delete-frag", "frag-delete"),
    ] {
        let response = http_exchange(
            port,
            &format!(
                "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(
            response.contains("datastar-patch-elements") && response.contains(marker),
            "{method} {path}: {response}"
        );
    }

    for (method, path) in [
        ("POST", "/actions/post-cmd"),
        ("PUT", "/actions/put-cmd"),
        ("PATCH", "/actions/patch-cmd"),
        ("DELETE", "/actions/delete-cmd"),
    ] {
        let response = http_exchange(
            port,
            &format!(
                "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(response.contains("204"), "{method} {path}: {response}");
        assert!(!response.contains("application/json"), "{response}");
        assert!(!response.contains("datastar-patch-elements"), "{response}");
    }

    let datastar_cmd = http_exchange(
        port,
        &format!(
            "POST /actions/post-cmd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(datastar_cmd.contains("200"), "{datastar_cmd}");
    assert!(datastar_cmd.contains("text/event-stream"), "{datastar_cmd}");
    assert!(
        !datastar_cmd.contains("datastar-patch-elements"),
        "{datastar_cmd}"
    );
}

#[test]
fn multi_page_streams_http_smoke() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let workspace = roc_build_staged_standalone_locked(
        "examples/rocci/standalone/multi-page-streams/Dashboard.rocci",
    );
    let server = workspace.path.join("server");
    let port = crate::serve::free_port().expect("free port");
    let mut child = KillOnDrop(
        Command::new(&server)
            .current_dir(&workspace.path)
            .env("ROC_BASIC_WEBSERVER_HOST", "127.0.0.1")
            .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn multi-page server"),
    );
    if let Err(err) =
        crate::serve::wait_for_server(&mut child.0, port, crate::logs::Progress::default())
    {
        panic!("multi-page server did not listen: {err:#}");
    }

    let dashboard = http_exchange(
        port,
        &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(dashboard.contains("/streams/dashboard"), "{dashboard}");
    assert!(dashboard.contains("/streams/notifications"), "{dashboard}");
    assert!(!dashboard.contains("/streams/admin"), "{dashboard}");

    let admin = http_exchange(
        port,
        &format!("GET /admin HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    );
    assert!(admin.contains("/streams/admin"), "{admin}");
    assert!(admin.contains("/streams/notifications"), "{admin}");
    assert!(!admin.contains("/streams/dashboard"), "{admin}");

    let dashboard_thread =
        std::thread::spawn(move || http_stream_sample(port, "/streams/dashboard", ""));
    let notifications_thread =
        std::thread::spawn(move || http_stream_sample(port, "/streams/notifications", ""));
    let dashboard_stream = dashboard_thread.join().unwrap();
    let notifications_stream = notifications_thread.join().unwrap();
    for (sample, marker) in [
        (&dashboard_stream, "dashboard-summary"),
        (&notifications_stream, "notifications"),
    ] {
        assert!(sample.contains("text/event-stream"), "{sample}");
        assert!(sample.contains("datastar-patch-elements"), "{sample}");
        assert!(sample.contains(marker), "{sample}");
        assert!(sample.contains("data:"), "{sample}");
    }
    assert!(dashboard_stream.contains("dashboard-activity"));

    let unauthorized = http_stream_sample(port, "/streams/admin", "");
    assert!(unauthorized.contains("text/event-stream"), "{unauthorized}");
    assert!(
        !unauthorized.contains("Authorized admin summary"),
        "{unauthorized}"
    );
    let authorized = http_stream_sample(port, "/streams/admin", "X-Rocci-Admin: demo\r\n");
    assert!(
        authorized.contains("Authorized admin summary"),
        "{authorized}"
    );

    let unknown = http_exchange(
        port,
        &format!(
            "GET /streams/unknown HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(unknown.contains("404"), "{unknown}");
    assert!(!unknown.contains("dashboard-summary"), "{unknown}");
    assert!(!unknown.contains("Authorized admin summary"), "{unknown}");
}

#[test]
fn command_returning_html_fails_unit_result_constraint() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = temp_app("command-no-encoder");
    fs::write(
        dir.join("NoEncoder.rocci"),
        r#"
import Html

@post:command("/x") {
    Html.text("nope")
}

@component Unused = |{}| {
    <p>x</p>
}
"#,
    )
    .unwrap();
    let plan = standalone_app_plan(&dir.join("NoEncoder.rocci")).expect("plan app");
    let workspace = stage_app_workspace(&plan, &dir, "roc-build").expect("stage generated app");
    let output = workspace.path.join("server");
    let err = crate::native_target::build_roc_server(&workspace.path, &output, None)
        .expect_err("command returning Html must fail the unit success constraint");
    let message = format!("{err:#}");
    assert!(
        message.contains("type") || message.contains("{}") || message.contains("Record"),
        "failure should describe the command result type, got {message}"
    );
    cleanup(&dir);
}

#[test]
fn command_unit_generated_app_roc_builds() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = temp_app("command-record");
    fs::write(
        dir.join("Cmd.rocci"),
        r#"
import Html

@post:command("/x") = |_state| {
    {}
}

@component Unused = |{}| {
    <p>x</p>
}
"#,
    )
    .unwrap();
    let plan = standalone_app_plan(&dir.join("Cmd.rocci")).expect("plan app");
    let workspace = stage_app_workspace(&plan, &dir, "roc-build").expect("stage generated app");
    let output = workspace.path.join("server");
    crate::native_target::build_roc_server(&workspace.path, &output, None)
        .unwrap_or_else(|err| panic!("command unit roc build failed: {err:#}"));
    cleanup(&dir);
}

#[test]
fn command_string_fails_unit_result_constraint() {
    if skip_without_roc() {
        return;
    }
    let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let dir = temp_app("command-str");
    fs::write(
        dir.join("Cmd.rocci"),
        r#"
import Html

@post:command("/x") = |_state| {
    "ok"
}

@component Unused = |{}| {
    <p>x</p>
}
"#,
    )
    .unwrap();
    let plan = standalone_app_plan(&dir.join("Cmd.rocci")).expect("plan app");
    let workspace = stage_app_workspace(&plan, &dir, "roc-build").expect("stage generated app");
    let output = workspace.path.join("server");
    crate::native_target::build_roc_server(&workspace.path, &output, None)
        .expect_err("command returning a string must fail the unit success constraint");
    cleanup(&dir);
}

#[test]
fn resolve_entry_uses_file_name_and_parent_dir() {
    let dir = temp_app("file");
    let main = dir.join("main.roc");
    fs::write(&main, "app").unwrap();
    let resolved = resolve_entry(&main).unwrap();
    assert_eq!(resolved.app_dir, dir);
    assert_eq!(resolved.roc_file, PathBuf::from("main.roc"));
    cleanup(&dir);
}

#[test]
fn resolve_entry_directory_uses_main_roc() {
    let dir = temp_app("dir");
    fs::write(dir.join("main.roc"), "app").unwrap();
    let resolved = resolve_entry(&dir).unwrap();
    assert_eq!(resolved.app_dir, dir);
    assert_eq!(resolved.roc_file, PathBuf::from("main.roc"));
    cleanup(&dir);
}

#[test]
fn resolve_entry_rejects_missing_app() {
    let dir = temp_app("missing");
    let err = resolve_entry(&dir.join("main.roc"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("no such Roc app"));
    let err = resolve_entry(&dir).unwrap_err().to_string();
    assert!(err.contains("no main.roc"));
    cleanup(&dir);
}

fn standalone_example(name: &str) -> PathBuf {
    repo_root().join("examples/rocci/standalone").join(name)
}

#[test]
fn resolve_standalone_picks_unique_example_entries() {
    let cases = [
        ("counter", "Counter.rocci"),
        ("styling", "Styling.rocci"),
        ("live-counter", "LiveCounter.rocci"),
        ("blocks", "backend/Blocks.rocci"),
        ("multi-page-streams", "Dashboard.rocci"),
    ];
    for (dir, entry) in cases {
        let path = standalone_example(dir);
        if !path.is_dir() {
            continue;
        }
        let resolved = resolve_standalone_entry(&path).expect(dir);
        assert_eq!(
            resolved,
            path.join(entry),
            "{dir} -> {}",
            resolved.display()
        );
    }
}

#[test]
fn resolve_standalone_walks_up_from_blocks_backend() {
    let backend = standalone_example("blocks").join("backend");
    if !backend.is_dir() {
        return;
    }
    let resolved = resolve_standalone_entry(&backend).expect("blocks/backend");
    assert_eq!(
        resolved,
        standalone_example("blocks").join("backend/Blocks.rocci")
    );
}

#[test]
fn resolve_standalone_parent_fails_as_multiple_inits() {
    let parent = repo_root().join("examples/rocci/standalone");
    if !parent.is_dir() {
        return;
    }
    let err = resolve_standalone_entry(&parent).unwrap_err().to_string();
    assert!(err.contains("multiple process `@init`"), "{err}");
    assert!(!err.contains("main.roc"), "{err}");
}

#[test]
fn resolve_standalone_empty_directory_lists_no_main_hint() {
    let dir = temp_app("empty-standalone");
    let err = resolve_standalone_entry(&dir).unwrap_err().to_string();
    assert!(err.contains("no .rocci modules"), "{err}");
    assert!(!err.contains("main.roc"), "{err}");
    cleanup(&dir);
}

#[test]
fn resolve_standalone_prefers_configured_entry() {
    let dir = temp_app("entry-prefers");
    fs::write(
        dir.join("rocci.toml"),
        "[app]\nidentifier = \"dev.rocci.entry\"\nentry = \"Beta.rocci\"\n",
    )
    .unwrap();
    fs::write(dir.join("Alpha.rocci"), view_only_src()).unwrap();
    fs::write(dir.join("Beta.rocci"), view_only_src()).unwrap();
    let resolved = resolve_standalone_entry(&dir).expect("configured entry");
    assert_eq!(resolved, dir.join("Beta.rocci"));
    cleanup(&dir);
}

#[test]
fn resolve_standalone_ambiguous_root_views_list_candidates() {
    let dir = temp_app("two-roots");
    fs::write(dir.join("Alpha.rocci"), view_only_src()).unwrap();
    fs::write(dir.join("Beta.rocci"), view_only_src()).unwrap();
    let err = resolve_standalone_entry(&dir).unwrap_err().to_string();
    assert!(err.contains("ambiguous standalone app"), "{err}");
    assert!(err.contains("Alpha.rocci"), "{err}");
    assert!(err.contains("Beta.rocci"), "{err}");
    assert!(!err.contains("main.roc"), "{err}");
    cleanup(&dir);
}

#[test]
fn discover_rocci_is_non_recursive_and_ignores_other_extensions() {
    let dir = temp_app("discover");
    fs::write(dir.join("Snake.rocci"), "").unwrap();
    fs::write(dir.join("Game.roc"), "").unwrap();
    fs::write(dir.join("notes.txt"), "").unwrap();
    let nested = dir.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("Other.rocci"), "").unwrap();

    let found = discover_rocci(&dir).unwrap();
    assert_eq!(found, vec![dir.join("Snake.rocci")]);
    cleanup(&dir);
}

#[test]
fn standalone_app_root_stops_at_git_workspace_rocci_toml() {
    let root = temp_app("boundary-root");
    fs::write(root.join(".git"), "").unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    fs::write(root.join("rocci.toml"), "[app]\nname = \"root\"\n").unwrap();
    let app = root.join("examples").join("app");
    fs::create_dir_all(&app).unwrap();
    let entry = app.join("Live.rocci");
    fs::write(&entry, "").unwrap();
    fs::write(app.join("Ui.rocci"), "").unwrap();
    fs::create_dir_all(root.join("examples").join("other")).unwrap();
    fs::write(root.join("examples").join("other").join("Skip.rocci"), "").unwrap();

    assert_eq!(standalone_app_root(&entry), app);
    let found = discover_standalone_tree(&app).unwrap();
    assert_eq!(found, vec![app.join("Live.rocci"), app.join("Ui.rocci")]);
    cleanup(&root);
}

#[test]
fn nested_standalone_discovers_backend_and_ui() {
    let app = temp_app("nested-app");
    fs::write(
        app.join("rocci.toml"),
        "[app]\nname = \"blocks\"\nidentifier = \"dev.rocci.blocks\"\n",
    )
    .unwrap();
    let backend = app.join("backend");
    let ui = app.join("ui");
    fs::create_dir_all(&backend).unwrap();
    fs::create_dir_all(&ui).unwrap();
    fs::create_dir_all(app.join("generated")).unwrap();
    fs::create_dir_all(app.join(".hidden")).unwrap();
    fs::write(backend.join("Blocks.rocci"), "").unwrap();
    fs::write(ui.join("BlocksUi.rocci"), "").unwrap();
    fs::write(app.join("generated").join("Skip.rocci"), "").unwrap();
    fs::write(app.join(".hidden").join("Nope.rocci"), "").unwrap();

    let entry = backend.join("Blocks.rocci");
    assert_eq!(standalone_app_root(&entry), app);
    let found = discover_standalone_tree(&app).unwrap();
    assert_eq!(
        found,
        vec![backend.join("Blocks.rocci"), ui.join("BlocksUi.rocci")]
    );
    cleanup(&app);
}

#[test]
fn nested_standalone_rejects_duplicate_stems() {
    let app = temp_app("dup-stem");
    fs::write(app.join("rocci.toml"), "[app]\nname = \"x\"\n").unwrap();
    fs::create_dir_all(app.join("backend")).unwrap();
    fs::create_dir_all(app.join("ui")).unwrap();
    fs::write(app.join("backend").join("Foo.rocci"), "").unwrap();
    fs::write(app.join("ui").join("Foo.rocci"), "").unwrap();
    let err = discover_standalone_tree(&app).unwrap_err().to_string();
    assert!(err.contains("duplicate standalone module `Foo`"));
    cleanup(&app);
}

fn process_init_src() -> &'static str {
    r#"
import Html

@context { n : I64 }

@init {
    { n: 0 }
}

@get:view("/") = |_| {
    page({})
}

@component Page = |{}|
    <html><body><p>ok</p></body></html>
"#
}

fn view_only_src() -> &'static str {
    r#"
import Html

@get:view("/") = |_| {
    page({})
}

@component Page = |{}|
    <html><body><p>ok</p></body></html>
"#
}

#[test]
fn two_process_init_modules_fail_to_plan() {
    let dir = temp_app("two-init");
    fs::write(dir.join("App.rocci"), process_init_src()).unwrap();
    fs::write(dir.join("Other.rocci"), process_init_src()).unwrap();
    let err = standalone_app_plan(&dir.join("App.rocci"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("multiple process `@init`"), "{err}");
    assert!(err.contains("App.rocci"), "{err}");
    assert!(err.contains("Other.rocci"), "{err}");
    cleanup(&dir);
}

#[test]
fn named_file_fails_when_sibling_owns_process_init() {
    let dir = temp_app("ui-not-init");
    fs::write(dir.join("App.rocci"), process_init_src()).unwrap();
    fs::write(dir.join("Ui.rocci"), view_only_src()).unwrap();
    let err = standalone_app_plan(&dir.join("Ui.rocci"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("process `@init` is in `App.rocci`"), "{err}");
    cleanup(&dir);
}

#[test]
fn view_only_sibling_plans_with_init_primary() {
    let dir = temp_app("init-plus-ui");
    fs::write(dir.join("App.rocci"), process_init_src()).unwrap();
    fs::write(dir.join("Ui.rocci"), view_only_src()).unwrap();
    let plan = standalone_app_plan(&dir.join("App.rocci")).expect("plan with unique init");
    assert_eq!(plan.primary_name, "App");
    let mut names: Vec<_> = plan.modules.iter().map(|m| m.type_name.as_str()).collect();
    names.sort();
    assert_eq!(names, vec!["App", "Ui"]);
    cleanup(&dir);
}

#[test]
fn nested_standalone_plan_includes_ui_module() {
    let app = temp_app("nested-plan");
    fs::write(
        app.join("rocci.toml"),
        "[app]\nname = \"blocks\"\nidentifier = \"dev.rocci.blocks\"\n",
    )
    .unwrap();
    let backend = app.join("backend");
    let ui = app.join("ui");
    fs::create_dir_all(&backend).unwrap();
    fs::create_dir_all(&ui).unwrap();
    fs::write(
        backend.join("Blocks.rocci"),
        r#"
import Html

@get:view("/") = |_| {
    page({})
}

@component Page = |{}|
    <html><body><p>ok</p></body></html>
"#,
    )
    .unwrap();
    fs::write(
        ui.join("BlocksUi.rocci"),
        r#"
import Html

@component Board = |{}|
    <div id="board"></div>
"#,
    )
    .unwrap();
    let plan = standalone_app_plan(&backend.join("Blocks.rocci")).expect("plan nested app");
    assert_eq!(plan.primary_name, "Blocks");
    let mut names: Vec<_> = plan.modules.iter().map(|m| m.type_name.as_str()).collect();
    names.sort();
    assert_eq!(names, vec!["Blocks", "BlocksUi"]);
    cleanup(&app);
}

#[test]
fn live_counter_stays_flat_and_does_not_absorb_sibling_apps() {
    let live = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/rocci/standalone/live-counter/LiveCounter.rocci");
    if !live.is_file() {
        return;
    }
    let root = standalone_app_root(&live);
    assert_eq!(
        root.file_name().and_then(|n| n.to_str()),
        Some("live-counter")
    );
    let found = discover_standalone_tree(&root).unwrap();
    let names: Vec<_> = found
        .iter()
        .filter_map(|path| path.file_name()?.to_str())
        .collect();
    assert!(names.contains(&"LiveCounter.rocci"));
    assert!(names.contains(&"LiveCounterUi.rocci"));
    assert!(!names.contains(&"Counter.rocci"));
    assert!(!names.contains(&"HandlerMatrix.rocci"));
}

#[test]
fn generated_module_uses_stem() {
    let input = Path::new("examples/rocci/custom/snake/Snake.rocci");
    assert_eq!(
        generated_module_path(input),
        PathBuf::from("examples/rocci/custom/snake/Snake.roc")
    );
    assert_eq!(type_name_from_path(input), "Snake");
}

#[test]
fn compile_writes_wrapped_type_module() {
    let dir = temp_app("compile");
    fs::write(
        dir.join("Hello.rocci"),
        "import Html\n\n@component Hello = |{ name }| {\n    <p>{name}</p>\n}\n",
    )
    .unwrap();
    compile_rocci_modules(&dir).unwrap();
    let generated = fs::read_to_string(dir.join("Hello.roc")).unwrap();
    assert!(generated.starts_with("import Html\n\nHello := [].{\n"));
    assert!(generated.contains("    hello = |{ name }| {"));
    cleanup(&dir);
}

#[test]
fn roc_invocation_forwards_args_and_runs_from_app_dir() {
    let resolved = ResolvedEntry {
        app_dir: PathBuf::from("/tmp/app"),
        roc_file: PathBuf::from("main.roc"),
    };
    let invocation = roc_invocation(&resolved, &["--".into(), "arg1".into()]);
    assert_eq!(invocation.program, "roc");
    assert_eq!(invocation.app_dir, PathBuf::from("/tmp/app"));
    assert_eq!(invocation.roc_file, PathBuf::from("main.roc"));
    assert_eq!(invocation.args, vec!["--".to_string(), "arg1".to_string()]);

    let cmd = roc_command(&invocation, 9001, false);
    assert_eq!(cmd.get_program(), "roc");
    let args: Vec<_> = cmd.get_args().collect();
    assert_eq!(args, ["main.roc", "--", "arg1"]);
    assert_eq!(cmd.get_current_dir(), Some(Path::new("/tmp/app")));
    let port = cmd
        .get_envs()
        .find(|(key, _)| *key == "ROC_BASIC_WEBSERVER_PORT")
        .and_then(|(_, value)| value)
        .unwrap();
    assert_eq!(port, "9001");
}

#[test]
fn window_title_uses_app_directory_name() {
    let resolved = ResolvedEntry {
        app_dir: PathBuf::from("/tmp/snake"),
        roc_file: PathBuf::from("main.roc"),
    };
    assert_eq!(window_title(&resolved), "snake");
}

#[test]
fn resolve_entry_rejects_unsupported_file_extensions() {
    let dir = temp_app("unsupported-ext");
    let txt_file = dir.join("notes.txt");
    fs::write(&txt_file, "hello").unwrap();
    let err = resolve_entry(&txt_file).unwrap_err().to_string();
    assert!(err.contains("unsupported file extension"));
    assert!(err.contains("expected a .roc or .rocci file"));
    cleanup(&dir);
}

#[test]
fn resolve_entry_suggests_rocdown_for_markdown_documents() {
    let dir = temp_app("markdown-hint");
    let md_file = dir.join("PLAN.md");
    fs::write(&md_file, "# Plan").unwrap();
    let err = resolve_entry(&md_file).unwrap_err().to_string();
    assert!(err.contains("unsupported file extension"));
    assert!(err.contains("rocdown view"));
    cleanup(&dir);
}

#[test]
fn resolve_entry_suggests_okf_for_knowledge_records() {
    let dir = temp_app("okf-hint");
    let md_file = dir.join("plan.md");
    fs::write(
        &md_file,
        "---\ntype: Implementation Plan\ntitle: Plan\nauthority: exploratory\n---\n\n# Plan\n",
    )
    .unwrap();
    let err = resolve_entry(&md_file).unwrap_err().to_string();
    assert!(err.contains("unsupported file extension"));
    assert!(err.contains("okmate view"));
    cleanup(&dir);
}

#[test]
fn resolve_entry_suggests_okf_for_knowledge_bundle() {
    let dir = temp_app("okf-bundle-hint");
    fs::write(
        dir.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
    )
    .unwrap();
    let err = resolve_entry(&dir).unwrap_err().to_string();
    assert!(err.contains("okmate view"));
    cleanup(&dir);
}
