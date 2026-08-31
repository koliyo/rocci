use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_template::LowerOptions;

use crate::datastar_asset;
use crate::driver::{self, GenericAppPlan, ResolvedEntry};
use crate::error_page::FailedFile;
use crate::logs::Progress;
use crate::runtime_assets;
use crate::serve;

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
        return run_standalone(StandaloneRun {
            file,
            args,
            no_window,
            port,
            live_reload,
            log_handlers,
            verbose,
            public,
        });
    }
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };
    if path.is_dir() && !path.join("main.roc").is_file() {
        if crate::path_hint::looks_like_okf_bundle(&path) {
            bail!(
                "no main.roc in {}; preview OKF knowledge bundles with `okmate view {}`",
                path.display(),
                file.display()
            );
        }
        let entry = resolve_standalone_entry(&path)?;
        return run_standalone(StandaloneRun {
            file: &entry,
            args,
            no_window,
            port,
            live_reload,
            log_handlers,
            verbose,
            public,
        });
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
                    "no main.roc in {}; preview OKF knowledge bundles with `okmate view {}`",
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
                    "unsupported file extension for `rocci run`: {}; preview OKF knowledge records with `okmate view {}`",
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

pub(crate) fn standalone_app_root(entry: &Path) -> PathBuf {
    let start = entry
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    app_root_from(&start)
}

fn app_root_from(start_dir: &Path) -> PathBuf {
    let start = start_dir.to_path_buf();
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

pub(crate) fn resolve_standalone_entry(start: &Path) -> Result<PathBuf> {
    let start = if start.is_absolute() {
        start.to_path_buf()
    } else {
        env::current_dir()?.join(start)
    };
    if !start.is_dir() {
        bail!("no such standalone app directory: {}", start.display());
    }
    let app_root = app_root_from(&start);
    let files = discover_standalone_tree(&app_root)?;
    if files.is_empty() {
        bail!(
            "no .rocci modules in {}; pass a standalone file or a directory with a unique entry",
            start.display()
        );
    }

    let mut inits = Vec::new();
    let mut root_views = Vec::new();
    let mut routed = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let compiled = compile_source(&path.display().to_string(), &src, &LowerOptions::default())?;
        let has_init = compiled.init.is_some() || compiled.state_type.is_some();
        let has_root_view = compiled
            .routes
            .iter()
            .any(|route| route.method == "GET" && route.path == "/");
        if has_init {
            inits.push(path.clone());
        }
        if has_root_view {
            root_views.push(path.clone());
        }
        if !compiled.routes.is_empty() && !has_init && !has_root_view {
            routed.push(path.clone());
        }
    }

    if inits.len() > 1 {
        bail!(
            "multiple process `@init` / `@context` modules in one app: {}",
            format_module_list(&app_root, &inits)
        );
    }
    if let Some(entry) = configured_app_entry(&app_root)? {
        return Ok(entry);
    }
    if let Some(entry) = inits.into_iter().next() {
        return Ok(entry);
    }
    if root_views.len() == 1 {
        return Ok(root_views.remove(0));
    }
    if files.len() == 1 {
        return Ok(files.into_iter().next().expect("one module"));
    }

    let mut lines = vec![format!(
        "ambiguous standalone app in {}: entry is not unique",
        start.display()
    )];
    if !root_views.is_empty() {
        lines.push(format!(
            "view(\"/\"): {}",
            format_module_list(&app_root, &root_views)
        ));
    }
    if !routed.is_empty() {
        lines.push(format!(
            "other routes: {}",
            format_module_list(&app_root, &routed)
        ));
    }
    let others: Vec<_> = files
        .iter()
        .filter(|path| !root_views.contains(path) && !routed.contains(path))
        .cloned()
        .collect();
    if !others.is_empty() {
        lines.push(format!(
            "other modules: {}",
            format_module_list(&app_root, &others)
        ));
    }
    bail!("{}", lines.join("\n"))
}

fn configured_app_entry(app_root: &Path) -> Result<Option<PathBuf>> {
    let config_path = app_root.join("rocci.toml");
    if !config_path.is_file() {
        return Ok(None);
    }
    let config = Config::from_file(&config_path)?;
    let Some(entry) = config.app.entry else {
        return Ok(None);
    };
    let resolved = app_root.join(&entry);
    if !resolved.is_file() {
        bail!(
            "app.entry `{entry}` is not a file under {}",
            app_root.display()
        );
    }
    if resolved.extension().and_then(|ext| ext.to_str()) != Some("rocci") {
        bail!("app.entry `{entry}` must be a .rocci file");
    }
    let root = app_root
        .canonicalize()
        .unwrap_or_else(|_| app_root.to_path_buf());
    let canonical = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
    if !canonical.starts_with(&root) {
        bail!("app.entry `{entry}` must stay under the app root");
    }
    Ok(Some(resolved))
}

fn format_module_list(app_root: &Path, paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| {
            format!(
                "`{}`",
                path.strip_prefix(app_root).unwrap_or(path).display()
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
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

pub(crate) fn discover_standalone_tree(app_root: &Path) -> Result<Vec<PathBuf>> {
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

struct StandaloneRun<'a> {
    file: &'a Path,
    args: &'a [String],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    log_handlers: bool,
    verbose: bool,
    public: bool,
}

fn run_standalone(req: StandaloneRun<'_>) -> Result<()> {
    let file = req.file;
    let args = req.args;
    let no_window = req.no_window;
    let port = req.port;
    let live_reload = req.live_reload;
    let log_handlers = req.log_handlers;
    let verbose = req.verbose;
    let public = req.public;
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
    ensure_unique_process_init(&plan)?;
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

pub(crate) enum StandaloneReady {
    Ready(GenericAppPlan),
    Failed(Vec<FailedFile>),
}

mod standalone;
use standalone::{compile_rocci_app, compile_source, ensure_unique_process_init, plan_standalone};
pub use standalone::{
    compile_rocci_modules, run_bundled, standalone_app_plan, standalone_http_module_app_plan,
    standalone_http_module_lower_options, standalone_island_app_plan,
    standalone_island_lower_options,
};
#[cfg(test)]
pub(crate) use standalone::{discover_rocci, generated_module_path};

#[cfg(test)]
mod tests;
