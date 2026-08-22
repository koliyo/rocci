use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_template::{InitInfo, LiveInfo, RouteInfo};
use sha2::{Digest, Sha256};

use crate::datastar_asset;
use crate::dispatch::{self, DispatchOptions, DispatchSource, LiveSource};
use crate::error_page::{self, FailedFile, MappedModule};
use crate::logs::{self, LogHub, LogLevel};
use crate::profile::{ProfileSnapshot, ProfileSpan};
use crate::roc_module::wrap_type_module;
use crate::runtime_assets;
use crate::serve;
use crate::style;

#[derive(Debug, Clone)]
pub struct GenericModule {
    pub type_name: String,
    pub roc: String,
    pub state_type: Option<String>,
    pub init: Option<InitInfo>,
    pub lives: Vec<LiveInfo>,
    pub routes: Vec<RouteInfo>,
    pub mapped: MappedModule,
    pub local_assets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GenericAppPlan {
    pub primary_name: String,
    pub modules: Vec<GenericModule>,
    pub redirect_trailing_slash: bool,
    pub log_handlers: bool,
}

impl GenericAppPlan {
    pub fn maps(&self) -> Vec<MappedModule> {
        self.modules
            .iter()
            .map(|module| module.mapped.clone())
            .collect()
    }

    pub fn main_roc(&self) -> String {
        let primary = &self.modules[0];
        let siblings: Vec<DispatchSource<'_>> = self.modules[1..]
            .iter()
            .map(|module| DispatchSource {
                type_name: &module.type_name,
                routes: &module.routes,
            })
            .collect();
        let primary_source = DispatchSource {
            type_name: &primary.type_name,
            routes: &primary.routes,
        };
        let bound = dispatch::merge_standalone_routes(primary_source, &siblings);
        let live_siblings = self.modules[1..]
            .iter()
            .map(|module| LiveSource {
                type_name: &module.type_name,
                lives: &module.lives,
            })
            .collect::<Vec<_>>();
        let bound_lives = dispatch::merge_standalone_lives(
            LiveSource {
                type_name: &primary.type_name,
                lives: &primary.lives,
            },
            &live_siblings,
        );
        dispatch::generate_bound_main_roc(
            &primary.type_name,
            primary.state_type.as_deref(),
            primary.init.as_ref(),
            &bound_lives,
            &bound,
            DispatchOptions {
                redirect_trailing_slash: self.redirect_trailing_slash,
                media_dirs: dispatch::media_dirs_from_urls(
                    self.modules
                        .iter()
                        .flat_map(|module| module.local_assets.iter()),
                ),
                log_handlers: self.log_handlers,
                log_handlers_color: self.log_handlers && style::stderr_color(),
            },
        )
    }

    pub fn validate_dispatch(&self) -> Result<()> {
        let primary = &self.modules[0];
        let primary_source = DispatchSource {
            type_name: &primary.type_name,
            routes: &primary.routes,
        };
        let siblings = self.modules[1..]
            .iter()
            .map(|module| DispatchSource {
                type_name: &module.type_name,
                routes: &module.routes,
            })
            .collect::<Vec<_>>();
        let sibling_lives = self.modules[1..]
            .iter()
            .map(|module| LiveSource {
                type_name: &module.type_name,
                lives: &module.lives,
            })
            .collect::<Vec<_>>();
        dispatch::validate_standalone_dispatch(
            primary_source,
            &siblings,
            LiveSource {
                type_name: &primary.type_name,
                lives: &primary.lives,
            },
            &sibling_lives,
        )
        .map_err(anyhow::Error::msg)
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.primary_name.as_bytes());
        hasher.update(self.main_roc().as_bytes());
        for module in &self.modules {
            hasher.update(module.type_name.as_bytes());
            hasher.update(module.roc.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

pub struct RunningApp {
    pub port: u16,
    pub fingerprint: String,
    child: Child,
    _workspace: TempDir,
}

impl Drop for RunningApp {
    fn drop(&mut self) {
        serve::stop_child(&mut self.child);
        serve::wait_port_free(self.port, Duration::from_secs(2));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEntry {
    pub app_dir: PathBuf,
    pub roc_file: PathBuf,
}

pub struct DriverOptions {
    pub args: Vec<String>,
    pub no_window: bool,
    pub live_reload: bool,
    pub log_handlers: bool,
    pub verbose: bool,
    pub port: serve::PortArg,
    pub db_path: Option<PathBuf>,
    pub title: String,
    pub preview_path: Option<String>,
    pub profile: crate::profile::ProfileSnapshot,
    pub inspect_pages: Vec<crate::inspect::InspectPage>,
    pub state_key: Option<String>,
}

pub fn execute_app_plan(
    plan: &GenericAppPlan,
    src_dir: &Path,
    options: &DriverOptions,
) -> Result<()> {
    let mut plan = plan.clone();
    plan.log_handlers = options.log_handlers;
    plan.validate_dispatch()?;
    let write_started = Instant::now();
    let type_name = plan.primary_name.clone();
    let workspace = stage_app_workspace(&plan, src_dir, "run")?;
    let default_db_path = src_dir.join(format!("{}.db", type_name.to_ascii_lowercase()));
    let db_path = options.db_path.as_deref().unwrap_or(&default_db_path);
    let resolved = ResolvedEntry {
        app_dir: workspace.path.clone(),
        roc_file: PathBuf::from("main.roc"),
    };
    let write_ms = write_started.elapsed().as_millis();
    let generated = plan
        .modules
        .iter()
        .map(|module| module.roc.len())
        .sum::<usize>()
        + plan.main_roc().len();
    logs::Progress::from_verbose(options.verbose).detail(format!(
        "starting roc ({} bytes of generated Roc)",
        generated
    ));
    let mut profile = options.profile.clone();
    profile.merge(ProfileSnapshot {
        total_ms: write_ms,
        spans: vec![ProfileSpan {
            name: "write".into(),
            duration_ms: write_ms,
            note: None,
        }],
    });
    invoke_standalone(
        &resolved,
        &options.args,
        options.no_window,
        options.live_reload,
        options.port,
        db_path,
        &options.title,
        options
            .preview_path
            .clone()
            .unwrap_or_else(|| preview_path(&plan.modules[0].routes)),
        &plan.maps(),
        profile,
        options.inspect_pages.clone(),
        options.state_key.clone(),
        options.verbose,
    )
}

pub fn spawn_app_plan(
    plan: &GenericAppPlan,
    src_dir: &Path,
    port: u16,
    logs: Option<Arc<LogHub>>,
    log_handlers: bool,
) -> Result<RunningApp> {
    let mut plan = plan.clone();
    plan.log_handlers = log_handlers;
    let fingerprint = plan.fingerprint();
    let workspace = stage_app_workspace(&plan, src_dir, "islands-dev")?;
    let db_path = workspace.path.join("islands.db");
    let resolved = ResolvedEntry {
        app_dir: workspace.path.clone(),
        roc_file: PathBuf::from("main.roc"),
    };
    let invocation = roc_invocation(&resolved, &[]);
    let mut cmd = roc_command(&invocation, port);
    cmd.env("DB_PATH", &db_path);
    let (mut child, mut tee) = serve::spawn_roc_with_logs(cmd, logs)?;
    match serve::wait_for_roc(
        &mut child,
        &mut tee,
        port,
        "/health",
        logs::Progress::default(),
    )? {
        serve::RocStart::Ready => Ok(RunningApp {
            port,
            fingerprint,
            child,
            _workspace: workspace,
        }),
        serve::RocStart::Failed(output) => {
            serve::stop_child(&mut child);
            bail!("island service failed to start: {output}")
        }
    }
}

pub(crate) fn stage_app_workspace(
    plan: &GenericAppPlan,
    src_dir: &Path,
    kind: &str,
) -> Result<TempDir> {
    plan.validate_dispatch()?;
    let type_name = plan.primary_name.clone();
    let workspace = TempDir::create(kind)?;
    runtime_assets::stage_into(&workspace.path)?;
    copy_sibling_roc(src_dir, &workspace.path, &type_name)?;
    let sibling_assets = src_dir.join("assets");
    let workspace_assets = workspace.path.join("assets");
    if sibling_assets.is_dir() {
        copy_tree(&sibling_assets, &workspace_assets)?;
    }
    for module in &plan.modules {
        for url in &module.local_assets {
            let relative = url
                .strip_prefix("./")
                .or_else(|| url.strip_prefix('/'))
                .unwrap_or(url);
            let relative = relative
                .split('#')
                .next()
                .unwrap_or(relative)
                .split('?')
                .next()
                .unwrap_or(relative);
            if relative.is_empty() {
                continue;
            }
            let from = src_dir.join(relative);
            if !from.is_file() {
                continue;
            }
            let to = workspace.path.join("media").join(relative);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to).with_context(|| format!("failed to copy {}", from.display()))?;
        }
    }
    let stage_version = datastar_asset::stage_version_for_dir(src_dir);
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
    Ok(workspace)
}

pub fn compile_app_plan(
    plan: &GenericAppPlan,
    src_dir: &Path,
    output: &Path,
    target: Option<crate::native_target::NativeTarget>,
) -> Result<()> {
    compile_app_plan_with_options(plan, src_dir, output, target, false)
}

pub fn compile_app_plan_with_options(
    plan: &GenericAppPlan,
    src_dir: &Path,
    output: &Path,
    target: Option<crate::native_target::NativeTarget>,
    verbose: bool,
) -> Result<()> {
    compile_app_plan_with_opt(plan, src_dir, output, target, verbose, None)
}

pub fn compile_app_plan_with_opt(
    plan: &GenericAppPlan,
    src_dir: &Path,
    output: &Path,
    target: Option<crate::native_target::NativeTarget>,
    verbose: bool,
    opt: Option<crate::native_target::RocOpt>,
) -> Result<()> {
    let workspace = stage_app_workspace(plan, src_dir, "islands-build")?;
    crate::native_target::build_roc_server_with_opt(&workspace.path, output, target, verbose, opt)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_resolved_entry(
    resolved: &ResolvedEntry,
    args: &[String],
    no_window: bool,
    live_reload: bool,
    port: serve::PortArg,
    maps: &[MappedModule],
    title: Option<&str>,
    mut profile: ProfileSnapshot,
    inspect_pages: Vec<crate::inspect::InspectPage>,
    verbose: bool,
) -> Result<()> {
    let default_title = window_title(resolved);
    let title = title.unwrap_or(&default_title);
    let invocation = roc_invocation(resolved, args);
    let port = port.resolve()?;
    let start_path = app_start_path(&resolved.app_dir);
    let url = format!("http://127.0.0.1:{port}{start_path}");
    let cmd = roc_command(&invocation, port);
    let roc_started = Instant::now();
    let logs = Arc::new(LogHub::new());
    let (mut child, mut tee) = serve::spawn_roc_with_logs(cmd, Some(logs.clone()))?;
    match serve::wait_for_roc(
        &mut child,
        &mut tee,
        port,
        &start_path,
        logs::Progress::from_verbose(verbose),
    )? {
        serve::RocStart::Ready => {
            let compile_ms = roc_started.elapsed().as_millis();
            profile.merge(ProfileSnapshot {
                total_ms: compile_ms,
                spans: vec![ProfileSpan {
                    name: "compile".into(),
                    duration_ms: compile_ms,
                    note: None,
                }],
            });
            tee.flush_to_hub();
            logs::tee(
                &logs,
                LogLevel::Info,
                style::serving(&invocation.app_dir.display().to_string(), &url),
            );
            let mut inspect = crate::inspect::InspectSnapshot::with_pages(profile, inspect_pages);
            inspect.capture_html_from_origin(&url);
            serve::with_window_and_inspector(
                &mut child,
                &url,
                title,
                no_window,
                live_reload,
                Some(inspect),
                None,
                Some(logs),
            )
        }
        serve::RocStart::Failed(output) => {
            serve_roc_failure(&output, maps, port, no_window, live_reload, title)
        }
    }
}

pub fn preview_path(routes: &[RouteInfo]) -> String {
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

pub(crate) fn app_start_path(app_dir: &Path) -> String {
    let from_toml = Config::from_file(app_dir.join("rocci.toml"))
        .ok()
        .and_then(|config| config.windows.into_iter().next())
        .map(|window| window.url)
        .filter(|url| !url.is_empty());
    match from_toml {
        Some(url) if url.starts_with('/') => url,
        Some(url) => format!("/{url}"),
        None => "/".into(),
    }
}

pub fn serve_template_errors(
    files: &[FailedFile],
    port: serve::PortArg,
    no_window: bool,
    live_reload: bool,
    title: &str,
) -> Result<()> {
    let html = error_page::render_template_errors(files);
    let port = port.resolve()?;
    serve::serve_html(port, 500, &html, title, no_window, live_reload)
}

pub fn serve_roc_failure(
    output: &str,
    maps: &[MappedModule],
    port: u16,
    no_window: bool,
    live_reload: bool,
    title: &str,
) -> Result<()> {
    let html = error_page::render_roc_compile_error(output, maps);
    serve::serve_html(port, 500, &html, title, no_window, live_reload)
}

pub fn window_title(resolved: &ResolvedEntry) -> String {
    resolved
        .app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rocci")
        .to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RocInvocation {
    pub program: &'static str,
    pub app_dir: PathBuf,
    pub roc_file: PathBuf,
    pub args: Vec<String>,
}

pub fn roc_invocation(resolved: &ResolvedEntry, args: &[String]) -> RocInvocation {
    RocInvocation {
        program: "roc",
        app_dir: resolved.app_dir.clone(),
        roc_file: resolved.roc_file.clone(),
        args: args.to_vec(),
    }
}

pub fn roc_command(invocation: &RocInvocation, port: u16) -> Command {
    let mut cmd = Command::new(invocation.program);
    cmd.arg(&invocation.roc_file)
        .args(&invocation.args)
        .current_dir(&invocation.app_dir)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string());
    cmd
}

#[allow(clippy::too_many_arguments)]
pub fn invoke_standalone(
    resolved: &ResolvedEntry,
    args: &[String],
    no_window: bool,
    live_reload: bool,
    port: serve::PortArg,
    db_path: &Path,
    title: &str,
    path: String,
    maps: &[MappedModule],
    mut profile: ProfileSnapshot,
    inspect_pages: Vec<crate::inspect::InspectPage>,
    state_key: Option<String>,
    verbose: bool,
) -> Result<()> {
    let invocation = roc_invocation(resolved, args);
    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut cmd = roc_command(&invocation, port);
    if env::var_os("DB_PATH").is_none() {
        cmd.env("DB_PATH", db_path);
    }
    let roc_started = Instant::now();
    let logs = Arc::new(LogHub::new());
    let (mut child, mut tee) = serve::spawn_roc_with_logs(cmd, Some(logs.clone()))?;
    match serve::wait_for_roc(
        &mut child,
        &mut tee,
        port,
        &path,
        logs::Progress::from_verbose(verbose),
    )? {
        serve::RocStart::Ready => {
            let compile_ms = roc_started.elapsed().as_millis();
            profile.merge(ProfileSnapshot {
                total_ms: compile_ms,
                spans: vec![ProfileSpan {
                    name: "compile".into(),
                    duration_ms: compile_ms,
                    note: None,
                }],
            });
            tee.flush_to_hub();
            logs::tee(&logs, LogLevel::Info, style::serving(title, &url));
            let mut inspect = crate::inspect::InspectSnapshot::with_pages(profile, inspect_pages);
            inspect.capture_html_from_origin(&format!("http://127.0.0.1:{port}"));
            serve::with_window_and_inspector(
                &mut child,
                &url,
                title,
                no_window,
                live_reload,
                Some(inspect),
                state_key,
                Some(logs),
            )
        }
        serve::RocStart::Failed(output) => {
            serve_roc_failure(&output, maps, port, no_window, live_reload, title)
        }
    }
}

pub fn copy_sibling_roc(src_dir: &Path, dest: &Path, type_name: &str) -> Result<()> {
    let skip = format!("{type_name}.roc");
    let mut seen = HashMap::new();
    copy_authored_roc(src_dir, dest, &skip, &mut seen)
}

fn skip_staging_dir(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "generated" | "target" | "node_modules")
}

fn copy_authored_roc(
    dir: &Path,
    dest: &Path,
    skip: &str,
    seen: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if path.is_dir() {
            if skip_staging_dir(&name) {
                continue;
            }
            copy_authored_roc(&path, dest, skip, seen)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("roc") {
            continue;
        }
        if name == "main.roc" || name == skip {
            continue;
        }
        let key = name.into_owned();
        if let Some(previous) = seen.insert(key.clone(), path.clone()) {
            bail!(
                "duplicate Roc module `{key}`: {} and {}",
                previous.display(),
                path.display()
            );
        }
        fs::copy(&path, dest.join(&key))
            .with_context(|| format!("failed to copy {}", path.display()))?;
    }
    Ok(())
}

pub fn copy_tree(from: &Path, to: &Path) -> Result<()> {
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

pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn create(kind: &str) -> Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_start_path_reads_window_url() {
        let dir = env::temp_dir().join(format!("rocci-start-path-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("rocci.toml"),
            "[app]\nname = \"t\"\nidentifier = \"dev.rocci.t\"\n\n[[windows]]\nlabel = \"main\"\nurl = \"/play/blocks/\"\n",
        )
        .unwrap();
        assert_eq!(app_start_path(&dir), "/play/blocks/");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn copy_sibling_roc_flattens_nested_helpers() {
        let src = env::temp_dir().join(format!("rocci-copy-src-{}", std::process::id()));
        let dest = env::temp_dir().join(format!("rocci-copy-dest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
        fs::create_dir_all(src.join("backend")).unwrap();
        fs::create_dir_all(src.join("generated")).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(src.join("backend").join("Game.roc"), "module").unwrap();
        fs::write(src.join("backend").join("Blocks.roc"), "skip-primary").unwrap();
        fs::write(src.join("generated").join("Stale.roc"), "skip-generated").unwrap();
        copy_sibling_roc(&src, &dest, "Blocks").unwrap();
        assert_eq!(fs::read_to_string(dest.join("Game.roc")).unwrap(), "module");
        assert!(!dest.join("Blocks.roc").exists());
        assert!(!dest.join("Stale.roc").exists());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn copy_sibling_roc_rejects_duplicate_stems() {
        let src = env::temp_dir().join(format!("rocci-copy-dup-{}", std::process::id()));
        let dest = env::temp_dir().join(format!("rocci-copy-dup-dest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
        fs::create_dir_all(src.join("backend")).unwrap();
        fs::create_dir_all(src.join("ui")).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(src.join("backend").join("Game.roc"), "a").unwrap();
        fs::write(src.join("ui").join("Game.roc"), "b").unwrap();
        let err = copy_sibling_roc(&src, &dest, "Blocks")
            .unwrap_err()
            .to_string();
        assert!(err.contains("duplicate Roc module `Game.roc`"));
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }
}
