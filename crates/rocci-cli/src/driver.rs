use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use anyhow::{Context, Result};
use rocci_template::{InitInfo, RouteInfo};

use crate::datastar_asset;
use crate::dispatch::{self, DispatchOptions, DispatchSource};
use crate::error_page::{self, FailedFile, MappedModule};
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
    pub routes: Vec<RouteInfo>,
    pub mapped: MappedModule,
    pub local_assets: Vec<String>,
}

pub struct GenericAppPlan {
    pub primary_name: String,
    pub modules: Vec<GenericModule>,
    pub redirect_trailing_slash: bool,
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
        let bound = dispatch::merge_standalone_routes(
            DispatchSource {
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
            DispatchOptions {
                redirect_trailing_slash: self.redirect_trailing_slash,
                media_dirs: dispatch::media_dirs_from_urls(
                    self.modules
                        .iter()
                        .flat_map(|module| module.local_assets.iter()),
                ),
            },
        )
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
    pub port: serve::PortArg,
    pub db_path: Option<PathBuf>,
    pub title: String,
    pub preview_path: Option<String>,
    pub profile: crate::profile::ProfileSnapshot,
}

pub fn execute_app_plan(
    plan: &GenericAppPlan,
    src_dir: &Path,
    options: &DriverOptions,
) -> Result<()> {
    let write_started = Instant::now();
    let type_name = plan.primary_name.clone();
    let workspace = TempDir::create("run")?;
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

    let default_db_path = src_dir.join(format!("{}.db", type_name.to_ascii_lowercase()));
    let db_path = options.db_path.as_deref().unwrap_or(&default_db_path);
    let resolved = ResolvedEntry {
        app_dir: workspace.path.clone(),
        roc_file: PathBuf::from("main.roc"),
    };
    let write_ms = write_started.elapsed().as_millis();
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
        options.port,
        db_path,
        &options.title,
        options
            .preview_path
            .clone()
            .unwrap_or_else(|| preview_path(&plan.modules[0].routes)),
        &plan.maps(),
        profile,
    )
}

pub fn execute_resolved_entry(
    resolved: &ResolvedEntry,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
    maps: &[MappedModule],
    title: Option<&str>,
    mut profile: ProfileSnapshot,
) -> Result<()> {
    let default_title = window_title(resolved);
    let title = title.unwrap_or(&default_title);
    let invocation = roc_invocation(resolved, args);
    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}/");
    let cmd = roc_command(&invocation, port);
    let roc_started = Instant::now();
    let (mut child, mut tee) = serve::spawn_roc(cmd)?;
    match serve::wait_for_roc(&mut child, &mut tee, port, "/")? {
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
            println!(
                "{}",
                style::serving(&invocation.app_dir.display().to_string(), &url)
            );
            serve::with_window_and_inspector(&mut child, &url, title, no_window, Some(profile))
        }
        serve::RocStart::Failed(output) => serve_roc_failure(&output, maps, port, no_window, title),
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

pub fn serve_template_errors(
    files: &[FailedFile],
    port: serve::PortArg,
    no_window: bool,
    title: &str,
) -> Result<()> {
    let html = error_page::render_template_errors(files);
    let port = port.resolve()?;
    serve::serve_html(port, 500, &html, title, no_window)
}

pub fn serve_roc_failure(
    output: &str,
    maps: &[MappedModule],
    port: u16,
    no_window: bool,
    title: &str,
) -> Result<()> {
    let html = error_page::render_roc_compile_error(output, maps);
    serve::serve_html(port, 500, &html, title, no_window)
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
    port: serve::PortArg,
    db_path: &Path,
    title: &str,
    path: String,
    maps: &[MappedModule],
    mut profile: ProfileSnapshot,
) -> Result<()> {
    let invocation = roc_invocation(resolved, args);
    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut cmd = roc_command(&invocation, port);
    if env::var_os("DB_PATH").is_none() {
        cmd.env("DB_PATH", db_path);
    }
    let roc_started = Instant::now();
    let (mut child, mut tee) = serve::spawn_roc(cmd)?;
    match serve::wait_for_roc(&mut child, &mut tee, port, &path)? {
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
            println!("{}", style::serving(title, &url));
            serve::with_window_and_inspector(&mut child, &url, title, no_window, Some(profile))
        }
        serve::RocStart::Failed(output) => serve_roc_failure(&output, maps, port, no_window, title),
    }
}

pub fn copy_sibling_roc(src_dir: &Path, dest: &Path, type_name: &str) -> Result<()> {
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
