use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU16, Ordering},
    },
    time::Instant,
};

use anyhow::{Context, Result, bail};
pub use rocci_cli::dev_server::DevServer;
use rocci_cli::dev_server::{StaticDevServerConfig, serve_static_site};
use rocci_cli::driver::RunningApp;
use rocci_cli::logs::{self, LogHub, LogLevel};

use crate::build::{BuildSession, absolute};
use crate::config::{SiteConfig, load_config};
use crate::inspect_snapshot::snapshot_from_loaded;
use crate::site::load_site;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ContentRoots {
    dirs: Vec<PathBuf>,
}

impl ContentRoots {
    pub(crate) fn collect(root: &Path, config: &SiteConfig) -> Self {
        let mut dirs = Vec::new();
        push_existing_dir(&mut dirs, root);
        for mount in &config.mounts {
            push_existing_dir(&mut dirs, &root.join(&mount.source));
        }
        for peer in &config.peers {
            push_existing_dir(&mut dirs, &root.join(&peer.source));
        }
        let theme = config.build.theme.as_deref().unwrap_or("theme");
        let theme_path = root.join(theme);
        if theme_path.is_file() {
            if let Some(parent) = theme_path.parent() {
                push_existing_dir(&mut dirs, parent);
            }
        } else {
            push_existing_dir(&mut dirs, &theme_path);
        }
        push_existing_dir(&mut dirs, &root.join(&config.build.assets));
        for entry in &config.snippets.roots {
            push_existing_dir(&mut dirs, &root.join(entry));
        }
        push_out_of_tree_service_parent(&mut dirs, root, config);
        Self { dirs }
    }

    pub(crate) fn dirs(&self) -> &[PathBuf] {
        &self.dirs
    }

    #[cfg(test)]
    fn from_dirs(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

fn push_existing_dir(dirs: &mut Vec<PathBuf>, path: &Path) {
    if !path.is_dir() {
        return;
    }
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !dirs.iter().any(|existing| existing == &canonical) {
        dirs.push(canonical);
    }
}

fn push_out_of_tree_service_parent(dirs: &mut Vec<PathBuf>, root: &Path, config: &SiteConfig) {
    if config.http.service.is_empty() {
        return;
    }
    let service = root.join(&config.http.service);
    if !service.is_file() {
        return;
    }
    let Some(parent) = service.parent() else {
        return;
    };
    let parent_canonical = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    let root_canonical = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    if parent_canonical.starts_with(&root_canonical) {
        return;
    }
    push_existing_dir(dirs, parent);
}

pub fn run(root: &Path, output: Option<&Path>, port: u16) -> Result<DevServer> {
    run_with_host(root, output, port, None)
}

pub fn run_with_host(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    host: Option<rocci_roc_host::HostChoice>,
) -> Result<DevServer> {
    run_with_host_at(root, output, port, host, "/", false, false, false)
}

#[allow(clippy::too_many_arguments)]
pub fn run_with_host_at(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    host: Option<rocci_roc_host::HostChoice>,
    open_path: &str,
    log_handlers: bool,
    verbose: bool,
    public: bool,
) -> Result<DevServer> {
    let root = absolute(root)?;
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let root = fs::canonicalize(&root)
        .with_context(|| format!("failed to resolve root {}", root.display()))?;

    let config = load_config(&root).unwrap_or_default();
    let title = config.site.title.clone();
    let content_roots = ContentRoots::collect(&root, &config);
    let watch_paths = content_roots.dirs().to_vec();

    let host_choice = host.unwrap_or_default();
    let mut session = BuildSession::create_with_host(host_choice)?;

    let custom_filter = Arc::new(move |path: &Path| path_is_relevant(path, &content_roots));

    let backend_port = Arc::new(AtomicU16::new(0));
    let backend = Arc::new(Mutex::new(None::<RunningApp>));
    let backend_slot = backend.clone();

    let config = StaticDevServerConfig {
        title,
        port,
        open_path: open_path.to_string(),
        output: output.map(Path::to_path_buf),
        watch_paths,
        custom_filter: Some(custom_filter),
        log_prefix: "rocdown".into(),
        backend_port: Some(backend_port.clone()),
        log_handlers,
        on_stop: Some(Arc::new(move || {
            *backend_slot.lock().unwrap_or_else(|err| err.into_inner()) = None;
        })),
        public,
        extra_http: None,
    };

    let session_root = root.clone();
    let progress = rocci_cli::logs::Progress {
        verbose,
        quiet: false,
    };
    serve_static_site(config, move |out_dir, logs| {
        progress.step("rocdown: loading site");
        let load_started = Instant::now();
        let loaded = load_site(&session_root)?;
        let load_ms = load_started.elapsed().as_millis();
        progress.detail(format!("rocdown: load {load_ms}ms"));
        progress.step("rocdown: rebuilding site");
        let mut report = session.rebuild_loaded(&loaded, out_dir)?;
        report.load_ms = load_ms;
        progress.detail(format!(
            "rocdown: rebuild parse={}ms generate={}ms compile={}ms render={}ms",
            report.plan_ms, report.generate_ms, report.compile_ms, report.roc_ms
        ));
        sync_island_backend(
            &session_root,
            &backend,
            &backend_port,
            Some(logs.clone()),
            log_handlers,
        )?;
        Ok(Some(snapshot_from_loaded(
            &loaded,
            out_dir,
            profile_from_report(&report),
        )))
    })
}

fn sync_island_backend(
    root: &Path,
    backend: &Mutex<Option<RunningApp>>,
    advertised: &AtomicU16,
    logs: Option<Arc<LogHub>>,
    log_handlers: bool,
) -> Result<()> {
    let app = match crate::service::generated_island_plan(root)? {
        Some(plan) => Some((plan.into_app_plan(), root.to_path_buf())),
        None => crate::service::configured_service_app_plan(root)?
            .map(|configured| (configured.app, configured.source_dir)),
    };
    match app {
        None => {
            *backend.lock().unwrap_or_else(|err| err.into_inner()) = None;
            advertised.store(0, Ordering::Relaxed);
            Ok(())
        }
        Some((app, source_dir)) => {
            let fingerprint = {
                let mut app = app.clone();
                app.log_handlers = log_handlers;
                app.fingerprint()
            };
            let mut slot = backend.lock().unwrap_or_else(|err| err.into_inner());
            if slot
                .as_ref()
                .is_some_and(|running| running.fingerprint == fingerprint)
            {
                return Ok(());
            }
            let port = match slot.as_ref() {
                Some(running) => running.port,
                None => rocci_cli::serve::free_port()?,
            };
            advertised.store(0, Ordering::Relaxed);
            *slot = None;
            let running = rocci_cli::driver::spawn_app_plan(
                &app,
                &source_dir,
                port,
                logs.clone(),
                log_handlers,
            )?;
            advertised.store(running.port, Ordering::Relaxed);
            if let Some(hub) = &logs {
                logs::tee(
                    hub,
                    LogLevel::Info,
                    "rocdown: island actions available on this origin",
                );
            } else {
                eprintln!(
                    "{}",
                    rocci_cli::style::cli_line("rocdown: island actions available on this origin")
                );
            }
            *slot = Some(running);
            Ok(())
        }
    }
}

fn profile_from_report(report: &crate::build::BuildReport) -> rocci_cli::profile::ProfileSnapshot {
    let mut rec = rocci_cli::profile::SpanRecorder::new();
    let compile_note = if report.recompiled {
        None
    } else {
        Some("cached".into())
    };
    push_span(&mut rec, "load", report.load_ms, None);
    push_span(&mut rec, "parse", report.plan_ms, None);
    push_span(&mut rec, "generate", report.generate_ms, None);
    rec.push("compile", report.compile_ms, compile_note);
    push_span(&mut rec, "render", report.roc_ms, None);
    push_span(&mut rec, "write", report.write_ms, None);
    rec.finish()
}

fn push_span(
    rec: &mut rocci_cli::profile::SpanRecorder,
    name: &str,
    duration_ms: u128,
    note: Option<String>,
) {
    if duration_ms == 0 && note.is_none() {
        return;
    }
    rec.push(name, duration_ms, note);
}

pub(crate) fn path_is_relevant(path: &Path, roots: &ContentRoots) -> bool {
    if has_ignored_watch_component(path) {
        return false;
    }
    let candidate = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if has_ignored_watch_component(&candidate) {
        return false;
    }
    for root in &roots.dirs {
        let Ok(rel) = candidate.strip_prefix(root) else {
            continue;
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        if has_hidden_watch_segment(rel) {
            return false;
        }
        return true;
    }
    false
}

fn has_ignored_watch_component(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == ".git" || c.as_os_str() == "target")
}

fn has_hidden_watch_segment(rel: &Path) -> bool {
    rel.iter().any(|comp| {
        let s = comp.to_string_lossy();
        s.starts_with('.') && s != "." && s != ".."
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CONFIG_FILE, load_config};
    use std::path::PathBuf;

    #[test]
    fn path_filter_keeps_content_and_ignores_noise() {
        let roots = ContentRoots::from_dirs(vec![PathBuf::from("/docs")]);
        assert!(path_is_relevant(Path::new("/docs/index.rocdown"), &roots));
        assert!(path_is_relevant(Path::new("/docs/rocdown.toml"), &roots));
        assert!(path_is_relevant(Path::new("/docs/assets/og.png"), &roots));
        assert!(!path_is_relevant(Path::new("/docs/.git/index"), &roots));
    }

    #[test]
    fn profile_from_report_omits_empty_spans_and_notes_cached_compile() {
        let snapshot = profile_from_report(&crate::build::BuildReport {
            generated_roc_bytes: 10,
            load_ms: 2,
            plan_ms: 0,
            generate_ms: 3,
            compile_ms: 0,
            roc_ms: 4,
            write_ms: 1,
            recompiled: false,
            pages: Vec::new(),
            datastar: false,
            service_origin: String::new(),
            service_routes: Vec::new(),
            artifacts: Vec::new(),
        });
        let names: Vec<_> = snapshot
            .spans
            .iter()
            .map(|span| span.name.as_str())
            .collect();
        assert_eq!(names, ["load", "generate", "compile", "render", "write"]);
        assert_eq!(snapshot.spans[2].note.as_deref(), Some("cached"));
        assert_eq!(snapshot.total_ms, 10);
    }

    fn temp_tree(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "rocdown-content-roots-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn site_shaped_mount_is_relevant_and_skips_git() {
        let workspace = temp_tree("site-shaped");
        let site = workspace.join("site");
        let docs = workspace.join("docs");
        fs::create_dir_all(&site).unwrap();
        fs::create_dir_all(docs.join(".git")).unwrap();
        fs::write(
            site.join(CONFIG_FILE),
            r#"
[site]
title = "Site"

[[mount]]
source = "../docs"
prefix = "docs"
"#,
        )
        .unwrap();
        fs::write(site.join("index.rocdown"), "# Site\n").unwrap();
        fs::write(docs.join("index.rocdown"), "# Docs\n").unwrap();
        fs::write(docs.join(".git/index"), "noise").unwrap();

        let config = load_config(&site).unwrap();
        let roots = ContentRoots::collect(&site, &config);
        let docs_canonical = fs::canonicalize(&docs).unwrap();
        assert!(roots.dirs().contains(&docs_canonical), "{:?}", roots.dirs());
        assert!(path_is_relevant(&docs.join("index.rocdown"), &roots));
        assert!(!path_is_relevant(&docs.join(".git/index"), &roots));
        assert!(path_is_relevant(&site.join("index.rocdown"), &roots));
        assert!(path_is_relevant(
            &site.join("../docs/index.rocdown"),
            &roots
        ));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn docs_shaped_peer_is_relevant_and_missing_peer_is_omitted() {
        let workspace = temp_tree("docs-shaped");
        let docs = workspace.join("docs");
        let project = workspace.join("site/project");
        fs::create_dir_all(&docs).unwrap();
        fs::create_dir_all(&project).unwrap();
        fs::write(
            docs.join(CONFIG_FILE),
            r#"
[site]
title = "Docs"

[[peer]]
source = "../site/project"
prefix = "project"

[[peer]]
source = "../site/missing"
prefix = "missing"
"#,
        )
        .unwrap();
        fs::write(docs.join("index.rocdown"), "# Docs\n").unwrap();
        fs::write(project.join("page.rocdown"), "# Project\n").unwrap();

        let config = load_config(&docs).unwrap();
        let roots = ContentRoots::collect(&docs, &config);
        assert!(
            !roots
                .dirs()
                .iter()
                .any(|dir| dir.ends_with("missing") || dir.ends_with("site/missing")),
            "{:?}",
            roots.dirs()
        );
        let project_canonical = fs::canonicalize(&project).unwrap();
        assert!(
            roots.dirs().contains(&project_canonical),
            "{:?}",
            roots.dirs()
        );
        assert!(path_is_relevant(&project.join("page.rocdown"), &roots));
        assert!(path_is_relevant(&docs.join("index.rocdown"), &roots));
        let _ = fs::remove_dir_all(workspace);
    }
}
