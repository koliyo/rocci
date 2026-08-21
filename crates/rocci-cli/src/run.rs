use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_desktop::PreviewOptions;
use rocci_template::{Diagnostic, LowerOptions, Segment, SourceFile, compile, format_diagnostic};

use crate::datastar_asset;
use crate::driver::{self, GenericAppPlan, GenericModule, ResolvedEntry};
use crate::error_page::{FailedFile, MappedModule};
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::runtime_assets;
use crate::serve;
use crate::style;

pub fn run(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    log_handlers: bool,
    verbose: bool,
) -> Result<()> {
    if is_standalone_file(file) {
        return run_standalone(
            file,
            args,
            no_window,
            port,
            live_reload,
            log_handlers,
            verbose,
        );
    }
    let resolved = resolve_entry(file)?;
    datastar_asset::ensure_app(&resolved.app_dir, datastar_asset::HintMode::Print)?;
    runtime_assets::stage_into(&resolved.app_dir)?;
    let compiled = compile_rocci_app(&resolved.app_dir)?;
    if !compiled.failures.is_empty() {
        return driver::serve_template_errors(
            &compiled.failures,
            port,
            no_window,
            live_reload,
            &driver::window_title(&resolved),
        );
    }
    driver::execute_resolved_entry(
        &resolved,
        args,
        no_window,
        live_reload,
        port,
        &compiled.maps,
        None,
        compiled.profile,
        compiled.inspect_pages,
        verbose,
    )
}

fn resolve_entry(file: &Path) -> Result<ResolvedEntry> {
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };

    if path.is_dir() {
        let roc_file = path.join("main.roc");
        if !roc_file.is_file() {
            if crate::path_hint::looks_like_okf_bundle(&path) {
                bail!(
                    "no main.roc in {}; preview OKF knowledge bundles with `rocci-okf run {}`",
                    path.display(),
                    file.display()
                );
            }
            let standalone = suggest_standalone(&path)?;
            if standalone.is_empty() {
                bail!("no main.roc in {}", path.display());
            }
            bail!(
                "no main.roc in {}; run a standalone file with `rocci run {}`",
                path.display(),
                standalone[0].display()
            );
        }
        return Ok(ResolvedEntry {
            app_dir: path,
            roc_file: PathBuf::from("main.roc"),
        });
    }

    if !path.is_file() {
        bail!("no such Roc app: {}", path.display());
    }

    let ext = path.extension().and_then(|e| e.to_str());
    if ext != Some("roc") {
        if ext == Some("rocdown") || ext == Some("md") || ext == Some("markdown") {
            if ext != Some("rocdown") && crate::path_hint::looks_like_okf_file(&path) {
                bail!(
                    "unsupported file extension for `rocci run`: {}; preview OKF knowledge records with `rocci-okf run {}`",
                    path.display(),
                    file.display()
                );
            }
            bail!(
                "unsupported file extension for `rocci run`: {}; run Markdown and Rocdown documents with `rocdown run {}`",
                path.display(),
                file.display()
            );
        }
        bail!(
            "unsupported file extension for `rocci run`: {}; expected a .roc or .rocci file",
            path.display()
        );
    }

    let roc_file = path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("main.roc"));
    let app_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));

    if !app_dir.join(&roc_file).is_file() {
        bail!("no such Roc app: {}", path.display());
    }

    Ok(ResolvedEntry { app_dir, roc_file })
}

fn suggest_standalone(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(discover_standalone(dir)?
        .into_iter()
        .map(|path| {
            path.strip_prefix(env::current_dir().unwrap_or_else(|_| dir.to_path_buf()))
                .map(Path::to_path_buf)
                .unwrap_or(path)
        })
        .collect())
}

fn is_standalone_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "rocci")
}

fn discover_standalone(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.is_file() && is_standalone_file(&path) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn run_standalone(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    log_handlers: bool,
    verbose: bool,
) -> Result<()> {
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };
    if !path.is_file() {
        bail!("no such Rocci file: {}", path.display());
    }
    let src_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));

    let title = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("rocci")
        .to_string();
    let plan = match plan_standalone(&path)? {
        (StandaloneReady::Failed(files), _, _) => {
            return driver::serve_template_errors(&files, port, no_window, live_reload, &title);
        }
        (StandaloneReady::Ready(plan), profile, inspect_pages) => (plan, profile, inspect_pages),
    };
    let (plan, profile, inspect_pages) = plan;
    let options = driver::DriverOptions {
        args: args.to_vec(),
        no_window,
        live_reload,
        log_handlers,
        verbose,
        port,
        db_path: None,
        title,
        preview_path: None,
        profile,
        inspect_pages,
        state_key: None,
    };
    driver::execute_app_plan(&plan, &src_dir, &options)
}

enum StandaloneReady {
    Ready(GenericAppPlan),
    Failed(Vec<FailedFile>),
}

fn plan_standalone(
    primary: &Path,
) -> Result<(
    StandaloneReady,
    crate::profile::ProfileSnapshot,
    Vec<crate::inspect::InspectPage>,
)> {
    let mut rec = crate::profile::SpanRecorder::new();
    let mut modules = Vec::new();
    let mut failures = Vec::new();
    let mut inspect_pages = Vec::new();
    let primary = primary
        .canonicalize()
        .unwrap_or_else(|_| primary.to_path_buf());
    let mut inputs = vec![primary.clone()];
    if let Some(dir) = primary.parent() {
        for path in discover_standalone(dir)? {
            let path = path.canonicalize().unwrap_or(path);
            if path != primary {
                inputs.push(path);
            }
        }
    }
    for input in inputs {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src)?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        if compiled.failed {
            failures.push(FailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }
        inspect_pages.push(compiled.inspect);
        let type_name = type_name_from_path(&input);
        modules.push(GenericModule {
            type_name: type_name.clone(),
            roc: compiled.roc.clone(),
            state_type: compiled.state_type,
            init: compiled.init,
            live: compiled.live,
            routes: compiled.routes,
            mapped: MappedModule {
                type_name,
                generated: compiled.roc,
                source_name: name,
                source_src: src,
                segments: compiled.segments,
            },
            local_assets: compiled.local_assets,
        });
    }
    let profile = rec.finish();
    if !failures.is_empty() {
        return Ok((StandaloneReady::Failed(failures), profile, inspect_pages));
    }
    // Primary first so dispatch / @init / @live come from the entry file.
    if let Some(index) = modules
        .iter()
        .position(|module| module.type_name == type_name_from_path(&primary))
    {
        modules.swap(0, index);
    }
    Ok((
        StandaloneReady::Ready(GenericAppPlan {
            primary_name: type_name_from_path(&primary),
            modules,
            redirect_trailing_slash: redirect_trailing_slash_for(
                primary.parent().unwrap_or_else(|| Path::new(".")),
            ),
            log_handlers: false,
        }),
        profile,
        inspect_pages,
    ))
}

pub fn standalone_app_plan(primary: &Path) -> Result<GenericAppPlan> {
    match plan_standalone(primary)? {
        (StandaloneReady::Ready(plan), _, _) => Ok(plan),
        (StandaloneReady::Failed(files), _, _) => {
            let name = files
                .first()
                .map(|file| file.name.as_str())
                .unwrap_or("template");
            bail!("template compilation failed for {name}")
        }
    }
}

fn redirect_trailing_slash_for(dir: &Path) -> bool {
    let path = dir.join("rocci.toml");
    if !path.is_file() {
        return true;
    }
    Config::from_file(path)
        .map(|config| config.http.redirect_trailing_slash)
        .unwrap_or(true)
}

fn discover_rocci(app_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(app_dir).with_context(|| format!("failed to read {}", app_dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rocci") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn generated_module_path(rocci: &Path) -> PathBuf {
    rocci.with_extension("roc")
}

pub fn compile_rocci_modules(app_dir: &Path) -> Result<()> {
    let compiled = compile_rocci_app(app_dir)?;
    if !compiled.failures.is_empty() {
        bail!("template compilation failed");
    }
    Ok(())
}

struct CompiledApp {
    failures: Vec<FailedFile>,
    maps: Vec<MappedModule>,
    profile: crate::profile::ProfileSnapshot,
    inspect_pages: Vec<crate::inspect::InspectPage>,
}

fn compile_rocci_app(app_dir: &Path) -> Result<CompiledApp> {
    let mut rec = crate::profile::SpanRecorder::new();
    let mut failures = Vec::new();
    let mut maps = Vec::new();
    let mut inspect_pages = Vec::new();
    for input in discover_rocci(app_dir)? {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src)?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        if compiled.failed {
            failures.push(FailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }
        if inspect_pages.is_empty() {
            inspect_pages.push(compiled.inspect);
        }
        let type_name = type_name_from_path(&input);
        let output = generated_module_path(&input);
        rec.span("write", || {
            fs::write(&output, wrap_type_module(&compiled.roc, &type_name))
                .with_context(|| format!("failed to write {}", output.display()))
        })?;
        maps.push(MappedModule {
            type_name,
            generated: compiled.roc,
            source_name: name,
            source_src: src,
            segments: compiled.segments,
        });
    }
    Ok(CompiledApp {
        failures,
        maps,
        profile: rec.finish(),
        inspect_pages,
    })
}

struct CompiledSource {
    roc: String,
    state_type: Option<String>,
    init: Option<rocci_template::InitInfo>,
    live: Option<rocci_template::LiveInfo>,
    routes: Vec<rocci_template::RouteInfo>,
    failed: bool,
    diagnostics: Vec<Diagnostic>,
    segments: Vec<Segment>,
    local_assets: Vec<String>,
    timings: rocci_template::CompileTimings,
    inspect: crate::inspect::InspectPage,
}

fn compile_source(name: &str, src: &str) -> Result<CompiledSource> {
    let source = SourceFile::new(name, src);
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    let failed = compiled.has_errors();
    let route = crate::driver::preview_path(&compiled.routes);
    let inspect = crate::inspect::InspectPage::from_rocci_compile(&route, name, src, &compiled);
    Ok(CompiledSource {
        roc: compiled.roc,
        state_type: compiled.state_type,
        init: compiled.init,
        live: compiled.live,
        routes: compiled.routes,
        failed,
        diagnostics: compiled.diagnostics,
        segments: compiled.segments,
        local_assets: Vec::new(),
        timings: compiled.timings,
        inspect,
    })
}

pub fn run_bundled(resources: &Path) -> Result<()> {
    let config = Config::from_file(resources.join("rocci.toml"))?;
    let app_dir = resources.join("app");
    let server = app_dir.join("server");
    if !server.is_file() {
        bail!("bundled app is missing {}", server.display());
    }

    let port = if config.http.port == 0 {
        serve::PortArg::Auto.resolve()?
    } else {
        serve::PortArg::Exact(config.http.port).resolve()?
    };
    let window = config.windows.first();
    let title = window
        .map(|window| window.title.as_str())
        .unwrap_or(&config.app.name)
        .to_string();
    let width = window.map(|window| window.width).unwrap_or(1040.0);
    let height = window.map(|window| window.height).unwrap_or(760.0);
    let path = window.map(|window| window.url.as_str()).unwrap_or("/");
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let url = format!("http://127.0.0.1:{port}{path}");

    let mut cmd = Command::new(&server);
    cmd.current_dir(&app_dir)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to start {}", server.display()))?;
    if let Err(err) = serve::wait_for_server(&mut child, port, crate::logs::Progress::default()) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }

    println!("{}", style::serving(&app_dir.display().to_string(), &url));
    let state_key = format!("rocci:{}", config.app.identifier);
    let preview_result = rocci_desktop::preview(PreviewOptions {
        url,
        title,
        width,
        height,
        devtools: config.development.devtools,
        state_key: Some(state_key),
        inspector_url: None,
        source_root: None,
        live_reload: true,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    let _ = child.kill();
    let _ = child.wait();
    preview_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{roc_command, roc_invocation, stage_app_workspace, window_title};
    use std::sync::Mutex;

    static ROC_LOCK: Mutex<()> = Mutex::new(());

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

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn roc_build_staged_standalone(relative: &str) -> crate::driver::TempDir {
        let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        roc_build_staged_standalone_locked(relative)
    }

    fn roc_build_staged_standalone_locked(relative: &str) -> crate::driver::TempDir {
        let primary = repo_root().join(relative);
        let src_dir = primary.parent().unwrap();
        let plan = standalone_app_plan(&primary).expect("plan standalone app");
        let workspace =
            stage_app_workspace(&plan, src_dir, "roc-build").expect("stage generated app");
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

    #[test]
    fn live_counter_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _workspace =
            roc_build_staged_standalone("examples/rocci/standalone/live-counter/LiveCounter.rocci");
    }

    #[test]
    fn counter_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _workspace =
            roc_build_staged_standalone("examples/rocci/standalone/counter/Counter.rocci");
    }

    #[test]
    fn handler_matrix_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _workspace = roc_build_staged_standalone(
            "examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci",
        );
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
        let mut child = Command::new(&server)
            .current_dir(&workspace.path)
            .env("ROC_BASIC_WEBSERVER_HOST", "127.0.0.1")
            .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn handler-matrix server");
        if let Err(err) =
            crate::serve::wait_for_server(&mut child, port, crate::logs::Progress::default())
        {
            let _ = child.kill();
            let _ = child.wait();
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

        let post_json = http_exchange(
            port,
            &format!(
                "POST /actions/post-cmd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(post_json.contains("200"), "{post_json}");
        assert!(
            post_json.contains("application/json") && post_json.contains("\"n\":"),
            "{post_json}"
        );
        assert!(
            !post_json.contains("datastar-patch-elements"),
            "{post_json}"
        );

        let put_json = http_exchange(
            port,
            &format!(
                "PUT /actions/put-cmd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(put_json.contains("\"n\":"), "{put_json}");

        let patch_json = http_exchange(
            port,
            &format!(
                "PATCH /actions/patch-cmd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(patch_json.contains("\"items\""), "{patch_json}");

        let delete_json = http_exchange(
            port,
            &format!(
                "DELETE /actions/delete-cmd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(delete_json.contains("\"deleted\":"), "{delete_json}");

        let datastar_cmd = http_exchange(
            port,
            &format!(
                "POST /actions/post-cmd HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
        );
        assert!(
            datastar_cmd.contains("200") && datastar_cmd.contains("text/event-stream"),
            "{datastar_cmd}"
        );
        assert!(!datastar_cmd.contains("204"), "{datastar_cmd}");
        assert!(
            !datastar_cmd.contains("datastar-patch-elements"),
            "{datastar_cmd}"
        );

        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn command_without_json_encoder_fails_roc_build() {
        if skip_without_roc() {
            return;
        }
        let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let dir = temp_app("command-no-encoder");
        fs::write(
            dir.join("NoEncoder.rocci"),
            r#"
import Html

@command("/x") {
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
            .expect_err("command returning Html must fail JSON encoding");
        let message = format!("{err:#}");
        assert!(
            message.contains("encoder")
                || message.contains("Json")
                || message.contains("Encoding")
                || message.contains("to_str_try"),
            "failure should mention JSON encoding, got {message}"
        );
        cleanup(&dir);
    }

    #[test]
    fn command_record_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let dir = temp_app("command-record");
        fs::write(
            dir.join("Cmd.rocci"),
            r#"
import Html

@command("/x") = |_state| {
    { count: 0.I64 }
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
            .unwrap_or_else(|err| panic!("command record roc build failed: {err:#}"));
        cleanup(&dir);
    }

    #[test]
    fn command_str_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let dir = temp_app("command-str");
        fs::write(
            dir.join("Cmd.rocci"),
            r#"
import Html

@command("/x") = |_state| {
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
            .unwrap_or_else(|err| panic!("command str roc build failed: {err:#}"));
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

    #[test]
    fn resolve_entry_directory_suggests_standalone_rocci() {
        let dir = temp_app("standalone-hint");
        fs::write(dir.join("Counter.rocci"), "").unwrap();
        let err = resolve_entry(&dir).unwrap_err().to_string();
        assert!(err.contains("no main.roc"));
        assert!(err.contains("rocci run"));
        assert!(err.contains("Counter.rocci"));
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

        let cmd = roc_command(&invocation, 9001);
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
        assert!(err.contains("rocdown run"));
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
        assert!(err.contains("rocci-okf run"));
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
        assert!(err.contains("rocci-okf run"));
        cleanup(&dir);
    }
}
