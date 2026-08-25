use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_desktop::PreviewOptions;
use rocci_template::{Diagnostic, LowerOptions, Segment, SourceFile, compile, format_diagnostic};

use crate::datastar_asset;
use crate::driver::{self, GenericAppPlan, GenericModule, ResolvedEntry};
use crate::error_page::{FailedFile, MappedModule};
use crate::logs::{self, Progress};
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::runtime_assets;
use crate::serve;
use crate::style;

#[allow(clippy::too_many_arguments)]
pub fn run(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    log_handlers: bool,
    verbose: bool,
    public: bool,
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
            public,
        );
    }
    let resolved = resolve_entry(file)?;
    datastar_asset::ensure_app(&resolved.app_dir, datastar_asset::HintMode::Print)?;
    runtime_assets::stage_into(&resolved.app_dir)?;
    let compiled = compile_rocci_app(&resolved.app_dir, Progress::from_verbose(verbose))?;
    if !compiled.failures.is_empty() {
        return driver::serve_template_errors(
            &compiled.failures,
            port,
            no_window,
            live_reload,
            &driver::window_title(&resolved),
            public,
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
        public,
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
                    "no main.roc in {}; preview OKF knowledge bundles with `rocci-okf view {}`",
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
                    "unsupported file extension for `rocci run`: {}; preview OKF knowledge records with `rocci-okf view {}`",
                    path.display(),
                    file.display()
                );
            }
            bail!(
                "unsupported file extension for `rocci run`: {}; preview Markdown and Rocdown documents with `rocdown view {}`",
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

fn standalone_app_root(entry: &Path) -> PathBuf {
    let start = entry
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut dir = start.clone();
    loop {
        if dir.join("rocci.toml").is_file() && !is_project_boundary(&dir) {
            return dir;
        }
        if is_project_boundary(&dir) {
            return start;
        }
        match dir.parent() {
            Some(parent) if parent != dir => dir = parent.to_path_buf(),
            _ => return start,
        }
    }
}

fn is_project_boundary(dir: &Path) -> bool {
    dir.join(".git").exists() || cargo_workspace_root(dir)
}

fn cargo_workspace_root(dir: &Path) -> bool {
    let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) else {
        return false;
    };
    text.lines().any(|line| line.trim() == "[workspace]")
}

fn skip_standalone_dir(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "generated" | "target" | "node_modules")
}

fn discover_standalone_tree(app_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stems = HashMap::new();
    walk_standalone(app_root, &mut files, &mut stems)?;
    files.sort();
    Ok(files)
}

fn walk_standalone(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    stems: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if skip_standalone_dir(&name) {
                continue;
            }
            walk_standalone(&path, files, stems)?;
            continue;
        }
        if !is_standalone_file(&path) {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
            .to_string();
        if let Some(previous) = stems.insert(stem.clone(), path.clone()) {
            bail!(
                "duplicate standalone module `{stem}`: {} and {}",
                previous.display(),
                path.display()
            );
        }
        files.push(path);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_standalone(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    log_handlers: bool,
    verbose: bool,
    public: bool,
) -> Result<()> {
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };
    if !path.is_file() {
        bail!("no such Rocci file: {}", path.display());
    }
    let src_dir = standalone_app_root(&path);

    let title = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("rocci")
        .to_string();
    let plan = match plan_standalone(
        &path,
        &LowerOptions::default(),
        Progress::from_verbose(verbose),
    )? {
        (StandaloneReady::Failed(files), _, _) => {
            return driver::serve_template_errors(
                &files,
                port,
                no_window,
                live_reload,
                &title,
                public,
            );
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
        public,
    };
    driver::execute_app_plan(&plan, &src_dir, &options)
}

enum StandaloneReady {
    Ready(GenericAppPlan),
    Failed(Vec<FailedFile>),
}

pub fn standalone_island_lower_options() -> LowerOptions {
    LowerOptions {
        embed_css: false,
        ..LowerOptions::default()
    }
}

pub fn standalone_island_app_plan(primary: &Path) -> Result<GenericAppPlan> {
    match plan_standalone(
        primary,
        &standalone_island_lower_options(),
        Progress::default(),
    )? {
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

fn plan_standalone(
    primary: &Path,
    lower: &LowerOptions,
    progress: Progress,
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
    let app_root = standalone_app_root(&primary);
    for path in discover_standalone_tree(&app_root)? {
        let path = path.canonicalize().unwrap_or(path);
        if path != primary {
            inputs.push(path);
        }
    }
    let templates_started = Instant::now();
    progress.step(logs::run_phase_start(
        "templates",
        &format!("modules={}", inputs.len()),
    ));
    for input in inputs {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src, lower)?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        progress.detail(logs::run_module_detail(
            &name,
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            compiled.timings.lower_ms,
        ));
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
            lives: compiled.lives,
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
    progress.step(logs::run_phase_done(
        "templates",
        templates_started.elapsed().as_millis(),
        "",
    ));
    if !failures.is_empty() {
        return Ok((StandaloneReady::Failed(failures), profile, inspect_pages));
    }
    // Primary first so dispatch / @init / @get:live come from the entry file.
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
            redirect_trailing_slash: redirect_trailing_slash_for(&app_root),
            log_handlers: false,
        }),
        profile,
        inspect_pages,
    ))
}

pub fn standalone_app_plan(primary: &Path) -> Result<GenericAppPlan> {
    match plan_standalone(primary, &LowerOptions::default(), Progress::default())? {
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
    let compiled = compile_rocci_app(app_dir, Progress::default())?;
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

fn compile_rocci_app(app_dir: &Path, progress: Progress) -> Result<CompiledApp> {
    let mut rec = crate::profile::SpanRecorder::new();
    let mut failures = Vec::new();
    let mut maps = Vec::new();
    let mut inspect_pages = Vec::new();
    let inputs = discover_rocci(app_dir)?;
    let templates_started = Instant::now();
    progress.step(logs::run_phase_start(
        "templates",
        &format!("modules={}", inputs.len()),
    ));
    for input in inputs {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src, &LowerOptions::default())?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        progress.detail(logs::run_module_detail(
            &name,
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            compiled.timings.lower_ms,
        ));
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
    progress.step(logs::run_phase_done(
        "templates",
        templates_started.elapsed().as_millis(),
        "",
    ));
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
    lives: Vec<rocci_template::LiveInfo>,
    routes: Vec<rocci_template::RouteInfo>,
    failed: bool,
    diagnostics: Vec<Diagnostic>,
    segments: Vec<Segment>,
    local_assets: Vec<String>,
    timings: rocci_template::CompileTimings,
    inspect: crate::inspect::InspectPage,
}

fn compile_source(name: &str, src: &str, lower: &LowerOptions) -> Result<CompiledSource> {
    let source = SourceFile::new(name, src);
    let compiled = compile(source, lower);
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
        lives: compiled.lives,
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
    crate::serve::emit_preview_ready(&url);
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

    #[test]
    fn work_queue_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _workspace =
            roc_build_staged_standalone("examples/rocci/standalone/work-queue/WorkQueue.rocci");
    }

    #[test]
    fn multi_page_streams_generated_app_roc_builds() {
        if skip_without_roc() {
            return;
        }
        let _workspace = roc_build_staged_standalone(
            "examples/rocci/standalone/multi-page-streams/Dashboard.rocci",
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

        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn work_queue_http_smoke() {
        if skip_without_roc() {
            return;
        }
        let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let workspace = roc_build_staged_standalone_locked(
            "examples/rocci/standalone/work-queue/WorkQueue.rocci",
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
            .expect("spawn work-queue server");
        if let Err(err) =
            crate::serve::wait_for_server(&mut child, port, crate::logs::Progress::default())
        {
            let _ = child.kill();
            let _ = child.wait();
            panic!("work-queue server did not listen: {err:#}");
        }

        let document = http_exchange(
            port,
            &format!("GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
        );
        assert!(document.contains("200"), "{document}");
        assert!(document.contains("id=\"queue\""), "{document}");
        assert!(document.contains("id=\"inspect\""), "{document}");

        let inspect_body = "{\"job_id\":\"1\"}";
        let inspect = http_exchange(
            port,
            &format!(
                "POST /actions/inspect HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{inspect_body}",
                inspect_body.len()
            ),
        );
        assert!(
            inspect.contains("datastar-patch-elements") && inspect.contains("id=\"inspect\""),
            "{inspect}"
        );

        let enqueue_body = "{\"title\":\"From curl\"}";
        let enqueue = http_exchange(
            port,
            &format!(
                "POST /actions/enqueue HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{enqueue_body}",
                enqueue_body.len()
            ),
        );
        assert!(enqueue.contains("204"), "{enqueue}");
        assert!(!enqueue.contains("application/json"), "{enqueue}");
        assert!(!enqueue.contains("datastar-patch-elements"), "{enqueue}");

        let datastar_cmd = http_exchange(
            port,
            &format!(
                "POST /actions/enqueue HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nDatastar-Request: true\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{enqueue_body}",
                enqueue_body.len()
            ),
        );
        assert!(datastar_cmd.contains("200"), "{datastar_cmd}");
        assert!(datastar_cmd.contains("text/event-stream"), "{datastar_cmd}");
        assert!(
            !datastar_cmd.contains("datastar-patch-elements"),
            "{datastar_cmd}"
        );

        let live = http_stream_sample(port, "/sse", "");
        assert!(live.contains("text/event-stream"), "{live}");

        let _ = child.kill();
        let _ = child.wait();
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
        let mut child = Command::new(&server)
            .current_dir(&workspace.path)
            .env("ROC_BASIC_WEBSERVER_HOST", "127.0.0.1")
            .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn multi-page server");
        if let Err(err) =
            crate::serve::wait_for_server(&mut child, port, crate::logs::Progress::default())
        {
            let _ = child.kill();
            let _ = child.wait();
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
            assert!(sample.matches("data:").count() >= 2, "{sample}");
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

        let _ = child.kill();
        let _ = child.wait();
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
        assert!(err.contains("rocci-okf view"));
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
        assert!(err.contains("rocci-okf view"));
        cleanup(&dir);
    }
}
