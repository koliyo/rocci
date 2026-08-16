use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_template::{Diagnostic, LowerOptions, Segment, SourceFile, compile, format_diagnostic};
use rocci_wry::PreviewOptions;

use crate::datastar_asset;
use crate::dispatch;
use crate::error_page::{self, FailedFile, MappedModule};
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::runtime_assets;
use crate::serve;
use crate::style;
use crate::theme::ThemeArgs;

pub fn run(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    theme: &ThemeArgs,
) -> Result<()> {
    if file
        .extension()
        .is_some_and(|ext| ext == "rocci" || ext == "rocdown")
    {
        return run_standalone(file, args, no_window, port, theme);
    }
    let resolved = resolve_entry(file)?;
    datastar_asset::ensure_app(&resolved.app_dir, datastar_asset::HintMode::Print)?;
    runtime_assets::stage_into(&resolved.app_dir)?;
    let compiled = compile_rocci_app(&resolved.app_dir, theme)?;
    if !compiled.failures.is_empty() {
        return serve_template_errors(
            &compiled.failures,
            port,
            no_window,
            &window_title(&resolved),
        );
    }
    invoke_roc(&resolved, args, no_window, port, &compiled.maps)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedEntry {
    app_dir: PathBuf,
    roc_file: PathBuf,
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
    Ok(discover_rocci(dir)?
        .into_iter()
        .map(|path| {
            path.strip_prefix(env::current_dir().unwrap_or_else(|_| dir.to_path_buf()))
                .map(Path::to_path_buf)
                .unwrap_or(path)
        })
        .collect())
}

fn run_standalone(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    theme: &ThemeArgs,
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
    let plan = match plan_standalone(&path, theme)? {
        StandaloneReady::Failed(files) => {
            return serve_template_errors(&files, port, no_window, &title);
        }
        StandaloneReady::Ready(plan) => plan,
    };
    let type_name = plan.primary_name.clone();
    let workspace = TempDir::create("run")?;
    runtime_assets::stage_into(&workspace.path)?;
    copy_sibling_roc(&src_dir, &workspace.path, &type_name)?;
    let sibling_assets = src_dir.join("assets");
    let workspace_assets = workspace.path.join("assets");
    if sibling_assets.is_dir() {
        copy_tree(&sibling_assets, &workspace_assets)?;
    }
    let stage_version = datastar_asset::stage_version_for_dir(&src_dir);
    if let Some(version) = &stage_version {
        datastar_asset::stage_into(&workspace_assets, version)?;
        datastar_asset::print_hint(version);
    } else {
        fs::create_dir_all(&workspace_assets)?;
    }

    for module in &plan.modules {
        fs::write(
            workspace.path.join(format!("{}.roc", module.type_name)),
            wrap_type_module(&module.roc, &module.type_name),
        )
        .with_context(|| format!("failed to write {}.roc", module.type_name))?;
    }
    fs::write(workspace.path.join("main.roc"), plan.main_roc())
        .context("failed to write generated main.roc")?;

    let db_path = src_dir.join(format!("{}.db", type_name.to_ascii_lowercase()));
    let resolved = ResolvedEntry {
        app_dir: workspace.path.clone(),
        roc_file: PathBuf::from("main.roc"),
    };
    invoke_standalone(
        &resolved,
        args,
        no_window,
        port,
        &db_path,
        &title,
        preview_path(&plan.modules[0].routes),
        &plan.maps(),
    )
}

struct StandaloneModule {
    type_name: String,
    roc: String,
    state_type: Option<String>,
    init: Option<rocci_template::InitInfo>,
    routes: Vec<rocci_template::RouteInfo>,
    mapped: MappedModule,
}

enum StandaloneReady {
    Ready(StandalonePlan),
    Failed(Vec<FailedFile>),
}

struct StandalonePlan {
    primary_name: String,
    modules: Vec<StandaloneModule>,
}

impl StandalonePlan {
    fn maps(&self) -> Vec<MappedModule> {
        self.modules
            .iter()
            .map(|module| module.mapped.clone())
            .collect()
    }

    fn main_roc(&self) -> String {
        let primary = &self.modules[0];
        let siblings: Vec<dispatch::DispatchSource<'_>> = self.modules[1..]
            .iter()
            .map(|module| dispatch::DispatchSource {
                type_name: &module.type_name,
                routes: &module.routes,
            })
            .collect();
        let bound = dispatch::merge_standalone_routes(
            dispatch::DispatchSource {
                type_name: &primary.type_name,
                routes: &primary.routes,
            },
            &siblings,
        );
        dispatch::generate_bound_main_roc(
            &primary.type_name,
            primary.state_type.as_deref(),
            primary.init.as_ref(),
            &bound,
        )
    }
}

fn plan_standalone(primary: &Path, theme: &ThemeArgs) -> Result<StandaloneReady> {
    let mut modules = Vec::new();
    let mut failures = Vec::new();
    for input in linked_standalone_inputs(primary)? {
        let src = fs::read_to_string(&input)
            .with_context(|| format!("failed to read {}", input.display()))?;
        let name = input.display().to_string();
        let compiled = compile_source(&input, &name, &src, theme)?;
        if compiled.failed {
            failures.push(FailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }
        let type_name = type_name_from_path(&input);
        modules.push(StandaloneModule {
            type_name: type_name.clone(),
            roc: compiled.roc.clone(),
            state_type: compiled.state_type,
            init: compiled.init,
            routes: compiled.routes,
            mapped: MappedModule {
                type_name,
                generated: compiled.roc,
                source_name: name,
                source_src: src,
                segments: compiled.segments,
            },
        });
    }
    if !failures.is_empty() {
        return Ok(StandaloneReady::Failed(failures));
    }
    Ok(StandaloneReady::Ready(StandalonePlan {
        primary_name: type_name_from_path(primary),
        modules,
    }))
}

fn linked_standalone_inputs(primary: &Path) -> Result<Vec<PathBuf>> {
    let primary = primary
        .canonicalize()
        .unwrap_or_else(|_| primary.to_path_buf());
    if !primary.extension().is_some_and(|ext| ext == "rocdown") {
        return Ok(vec![primary]);
    }
    let Some(dir) = primary.parent() else {
        return Ok(vec![primary]);
    };
    let mut files: Vec<PathBuf> = discover_rocci(dir)?
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "rocdown"))
        .map(|path| path.canonicalize().unwrap_or(path))
        .collect();
    files.sort();
    files.dedup();
    if let Some(index) = files.iter().position(|path| path == &primary) {
        files.swap(0, index);
    } else {
        files.insert(0, primary);
    }
    Ok(files)
}

fn discover_rocci(app_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(app_dir).with_context(|| format!("failed to read {}", app_dir.display()))?
    {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "rocci" || ext == "rocdown")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn generated_module_path(rocci: &Path) -> PathBuf {
    rocci.with_extension("roc")
}

pub fn compile_rocci_modules(app_dir: &Path, theme: &ThemeArgs) -> Result<()> {
    let compiled = compile_rocci_app(app_dir, theme)?;
    if !compiled.failures.is_empty() {
        bail!("template compilation failed");
    }
    Ok(())
}

struct CompiledApp {
    failures: Vec<FailedFile>,
    maps: Vec<MappedModule>,
}

fn compile_rocci_app(app_dir: &Path, theme: &ThemeArgs) -> Result<CompiledApp> {
    let mut failures = Vec::new();
    let mut maps = Vec::new();
    for input in discover_rocci(app_dir)? {
        let src = fs::read_to_string(&input)
            .with_context(|| format!("failed to read {}", input.display()))?;
        let name = input.display().to_string();
        let compiled = compile_source(&input, &name, &src, theme)?;
        if compiled.failed {
            failures.push(FailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }
        let type_name = type_name_from_path(&input);
        let output = generated_module_path(&input);
        fs::write(&output, wrap_type_module(&compiled.roc, &type_name))
            .with_context(|| format!("failed to write {}", output.display()))?;
        maps.push(MappedModule {
            type_name,
            generated: compiled.roc,
            source_name: name,
            source_src: src,
            segments: compiled.segments,
        });
    }
    Ok(CompiledApp { failures, maps })
}

struct CompiledSource {
    roc: String,
    state_type: Option<String>,
    init: Option<rocci_template::InitInfo>,
    routes: Vec<rocci_template::RouteInfo>,
    failed: bool,
    diagnostics: Vec<Diagnostic>,
    segments: Vec<Segment>,
}

fn compile_source(
    input: &Path,
    name: &str,
    src: &str,
    theme: &ThemeArgs,
) -> Result<CompiledSource> {
    let source = SourceFile::new(name, src);
    if input.extension().is_some_and(|ext| ext == "rocdown") {
        let compiled = rocci_rocdown::compile(source, &theme.compile_options(Some(input)));
        for diagnostic in &compiled.diagnostics {
            eprintln!("{}", format_diagnostic(source, diagnostic));
        }
        let failed = compiled.has_errors();
        return Ok(CompiledSource {
            roc: compiled.roc,
            state_type: compiled.state_type,
            init: compiled.init,
            routes: compiled.routes,
            failed,
            diagnostics: compiled.diagnostics,
            segments: compiled.segments,
        });
    }
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    let failed = compiled.has_errors();
    Ok(CompiledSource {
        roc: compiled.roc,
        state_type: compiled.state_type,
        init: compiled.init,
        routes: compiled.routes,
        failed,
        diagnostics: compiled.diagnostics,
        segments: compiled.segments,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RocInvocation {
    program: &'static str,
    app_dir: PathBuf,
    roc_file: PathBuf,
    args: Vec<String>,
}

fn roc_invocation(resolved: &ResolvedEntry, args: &[String]) -> RocInvocation {
    RocInvocation {
        program: "roc",
        app_dir: resolved.app_dir.clone(),
        roc_file: resolved.roc_file.clone(),
        args: args.to_vec(),
    }
}

fn roc_command(invocation: &RocInvocation, port: u16) -> Command {
    let mut cmd = Command::new(invocation.program);
    cmd.arg(&invocation.roc_file)
        .args(&invocation.args)
        .current_dir(&invocation.app_dir)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string());
    cmd
}

fn preview_path(routes: &[rocci_template::RouteInfo]) -> String {
    routes
        .iter()
        .find(|route| route.method == "GET" && route.path != "/health")
        .map(|route| {
            if route.path.starts_with('/') {
                route.path.clone()
            } else {
                format!("/{}", route.path)
            }
        })
        .unwrap_or_else(|| "/".to_string())
}

fn invoke_standalone(
    resolved: &ResolvedEntry,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    db_path: &Path,
    title: &str,
    path: String,
    maps: &[MappedModule],
) -> Result<()> {
    let invocation = roc_invocation(resolved, args);
    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut cmd = roc_command(&invocation, port);
    if env::var_os("DB_PATH").is_none() {
        cmd.env("DB_PATH", db_path);
    }
    let (mut child, mut tee) = serve::spawn_roc(cmd)?;
    match serve::wait_for_listen(&mut child, port)? {
        serve::ListenWait::Ready => {
            println!("{}", style::serving(title, &url));
            serve::with_window(&mut child, &url, title, no_window)
        }
        serve::ListenWait::Exited(_) => {
            let output = tee.finish();
            serve_roc_failure(&output, maps, port, no_window, title)
        }
    }
}

fn copy_sibling_roc(src_dir: &Path, dest: &Path, type_name: &str) -> Result<()> {
    let skip = format!("{type_name}.roc");
    if !src_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("roc") {
            continue;
        }
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name == "main.roc" || name == skip {
            continue;
        }
        fs::copy(&path, dest.join(&file_name))
            .with_context(|| format!("failed to copy {}", path.display()))?;
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    if from.is_file() {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(from, to)?;
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        copy_tree(&entry.path(), &to.join(entry.file_name()))?;
    }
    Ok(())
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn create(kind: &str) -> Result<Self> {
        let path = env::temp_dir().join(format!("rocci-{kind}-{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("failed to clear {}", path.display()))?;
        }
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create {}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn invoke_roc(
    resolved: &ResolvedEntry,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    maps: &[MappedModule],
) -> Result<()> {
    let invocation = roc_invocation(resolved, args);
    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}/");
    let title = window_title(resolved);
    let cmd = roc_command(&invocation, port);
    let (mut child, mut tee) = serve::spawn_roc(cmd)?;
    match serve::wait_for_listen(&mut child, port)? {
        serve::ListenWait::Ready => {
            println!(
                "{}",
                style::serving(&invocation.app_dir.display().to_string(), &url)
            );
            serve::with_window(&mut child, &url, &title, no_window)
        }
        serve::ListenWait::Exited(_) => {
            let output = tee.finish();
            serve_roc_failure(&output, maps, port, no_window, &title)
        }
    }
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
    let url = format!("http://127.0.0.1:{port}/");
    let window = config.windows.first();
    let title = window
        .map(|window| window.title.as_str())
        .unwrap_or(&config.app.name)
        .to_string();
    let width = window.map(|window| window.width).unwrap_or(1040.0);
    let height = window.map(|window| window.height).unwrap_or(760.0);

    let mut cmd = Command::new(&server);
    cmd.current_dir(&app_dir)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to start {}", server.display()))?;
    if let Err(err) = serve::wait_for_server(&mut child, port) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }

    println!("{}", style::serving(&app_dir.display().to_string(), &url));
    let preview_result = rocci_wry::preview(PreviewOptions {
        url,
        title,
        width,
        height,
        devtools: config.development.devtools,
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    let _ = child.kill();
    let _ = child.wait();
    preview_result
}

fn serve_template_errors(
    files: &[FailedFile],
    port: serve::PortArg,
    no_window: bool,
    title: &str,
) -> Result<()> {
    let html = error_page::render_template_errors(files);
    let port = port.resolve()?;
    serve::serve_html(port, 500, &html, title, no_window)
}

fn serve_roc_failure(
    output: &str,
    maps: &[MappedModule],
    port: u16,
    no_window: bool,
    title: &str,
) -> Result<()> {
    let html = error_page::render_roc_compile_error(output, maps);
    serve::serve_html(port, 500, &html, title, no_window)
}

fn window_title(resolved: &ResolvedEntry) -> String {
    resolved
        .app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rocci")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_app(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("rocci-run-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn plan_ready(path: &Path) -> StandalonePlan {
        match plan_standalone(path, &crate::theme::ThemeArgs::default()).unwrap() {
            StandaloneReady::Ready(plan) => plan,
            StandaloneReady::Failed(files) => {
                panic!(
                    "expected successful compile, got {} failed file(s)",
                    files.len()
                )
            }
        }
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
        let input = Path::new("examples/snake/Snake.rocci");
        assert_eq!(
            generated_module_path(input),
            PathBuf::from("examples/snake/Snake.roc")
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
        compile_rocci_modules(&dir, &crate::theme::ThemeArgs::default()).unwrap();
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
    fn linked_standalone_inputs_puts_primary_first() {
        let dir = temp_app("linked-inputs");
        let home = dir.join("Home.rocdown");
        let about = dir.join("About.rocdown");
        fs::write(&home, "").unwrap();
        fs::write(&about, "").unwrap();
        let inputs = linked_standalone_inputs(&home).unwrap();
        assert_eq!(inputs[0], home.canonicalize().unwrap());
        assert!(inputs.contains(&about.canonicalize().unwrap()));
        cleanup(&dir);
    }

    #[test]
    fn standalone_rocdown_serves_sibling_page_routes() {
        let dir = temp_app("linked-pages");
        fs::write(
            dir.join("Home.rocdown"),
            r#"
@page { route: "/home/" }

# Home

See [[About]]
"#,
        )
        .unwrap();
        fs::write(
            dir.join("About.rocdown"),
            r#"
@page { route: "/about/" }

@on:get("/") = |_| {
    rocci_page({})
}

# About
"#,
        )
        .unwrap();
        let plan = plan_ready(&dir.join("Home.rocdown"));
        assert_eq!(plan.primary_name, "Home");
        assert!(
            plan.modules
                .iter()
                .any(|module| module.type_name == "About")
        );
        let main = plan.main_roc();
        assert!(main.contains("import Home"));
        assert!(main.contains("import About"));
        assert!(main.contains("(\"GET\", \"/about/\")"));
        assert!(main.contains("About.on_get_about!"));
        assert_eq!(
            dispatch_handler(&main, "GET", "/"),
            "Home.on_get_home!(context)"
        );
        assert_eq!(
            dispatch_handler(&main, "GET", "/about/"),
            "About.on_get_about!(context)"
        );
        assert!(main.contains("html_status(404, not_found_html("));
        assert!(!main.contains("Not found"));
        assert!(!main.contains("About.on_get_root!"));
        cleanup(&dir);
    }

    #[test]
    fn guide_example_serves_interactive_route() {
        let guide = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/rocdown/Guide.rocdown")
            .canonicalize()
            .unwrap();
        let plan = plan_ready(&guide);
        let main = plan.main_roc();
        assert!(main.contains("import Interactive"));
        assert_eq!(
            dispatch_handler(&main, "GET", "/guides/rocdown-interactive/"),
            "Interactive.on_get_guides_rocdown_interactive!(context)"
        );
        assert_eq!(
            dispatch_handler(&main, "GET", "/"),
            "Guide.on_get_guides_rocdown!(context)"
        );
        assert_eq!(
            dispatch_handler(&main, "POST", "/actions/reveal/show"),
            "Interactive.on_post_actions_reveal_show!(context)"
        );
        assert!(!main.contains("Interactive.on_get_root!"));
    }

    #[test]
    fn errors_example_lists_dx_route_on_404() {
        let dx = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/errors/Dx.rocdown")
            .canonicalize()
            .unwrap();
        let plan = plan_ready(&dx);
        let main = plan.main_roc();
        assert_eq!(
            dispatch_handler(&main, "GET", "/dx/"),
            "Dx.on_get_dx!(context)"
        );
        assert!(main.contains("html_status(404, not_found_html("));
        assert!(main.contains("/dx/"));
        assert!(main.contains("\"/dx\" => Ok(\"/dx/\")"));
    }

    #[test]
    fn errors_parse_example_builds_error_page() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/errors/parse/Broken.rocdown")
            .canonicalize()
            .unwrap();
        let StandaloneReady::Failed(files) =
            plan_standalone(&path, &crate::theme::ThemeArgs::default()).unwrap()
        else {
            panic!("expected template failure");
        };
        let html = error_page::render_template_errors(&files);
        assert!(html.contains("Broken.rocdown"));
        assert!(html.contains("@page"));
        assert!(html.contains("error"));
    }

    #[test]
    fn standalone_compile_failure_builds_error_page() {
        let dir = temp_app("compile-fail");
        let path = dir.join("Broken.rocdown");
        fs::write(&path, "@page {\n").unwrap();
        let StandaloneReady::Failed(files) =
            plan_standalone(&path, &crate::theme::ThemeArgs::default()).unwrap()
        else {
            cleanup(&dir);
            panic!("expected template failure");
        };
        let html = error_page::render_template_errors(&files);
        assert!(html.contains("Broken.rocdown") || html.contains("@page"));
        assert!(html.contains("error"));
        cleanup(&dir);
    }

    fn dispatch_handler<'a>(main: &'a str, method: &str, path: &str) -> &'a str {
        let needle = format!("(\"{method}\", \"{path}\") =>");
        let start = main
            .find(&needle)
            .unwrap_or_else(|| panic!("missing route {needle} in {main}"));
        let after = &main[start + needle.len()..];
        let match_at = after
            .find("match ")
            .unwrap_or_else(|| panic!("missing handler for {needle}"));
        after[match_at + "match ".len()..]
            .lines()
            .next()
            .unwrap()
            .trim()
            .trim_end_matches('{')
            .trim()
    }
}
