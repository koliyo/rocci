use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use okf::{LoadOptions, LoadTimings, Profile, Severity};
pub use rocci_cli::dev_server::DevServer;
use rocci_cli::dev_server::{StaticDevServerConfig, serve_static_site};
use rocci_cli::inspect::InspectSnapshot;
use rocci_cli::logs::{self, LogHub, LogLevel};
use rocci_cli::profile::{ProfileSnapshot, ProfileSpan};

use crate::inspect;
use crate::presentation::{ApplySession, build_review_site_with_session};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfileReportMode {
    #[default]
    Off,
    Terminal,
    Json,
}

#[allow(clippy::too_many_arguments)]
pub fn run_knowledge(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    profile: Profile,
    provenance: bool,
    open_path: &str,
    host: Option<rocci_roc_host::HostChoice>,
    profile_report: ProfileReportMode,
    public: bool,
    extra_http: Option<rocci_cli::dev_server::ExtraHttpHandler>,
) -> Result<DevServer> {
    let root = okf::absolute(root)?;
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let root = fs::canonicalize(&root)
        .with_context(|| format!("failed to resolve knowledge root {}", root.display()))?;

    let filter_root = root.clone();
    let custom_filter = Arc::new(move |path: &Path| knowledge_path_is_relevant(path, &filter_root));

    let config = StaticDevServerConfig {
        title: "Knowledge".into(),
        port,
        open_path: open_path.to_string(),
        output: output.map(Path::to_path_buf),
        watch_paths: vec![root.clone()],
        custom_filter: Some(custom_filter),
        log_prefix: "rocci-okf".into(),
        backend_port: None,
        log_handlers: false,
        on_stop: Some(start_git_poll_thread()),
        public,
        extra_http,
    };

    let build_root = root.clone();
    let cache_dir = parse_cache_dir(&root);
    let mut cache = okf::ParseCache::load_dir(&cache_dir, profile);
    let mut apply_session: Option<ApplySession> = None;
    serve_static_site(config, move |out_dir, logs| {
        let snapshot = rebuild_site(
            &build_root,
            out_dir,
            profile,
            provenance,
            host,
            &mut cache,
            &mut apply_session,
            &logs,
        )?;
        if let Err(error) = cache.save_dir(&cache_dir) {
            eprintln!("rocci-okf: failed to save parse cache: {error:#}");
        }
        if let Some(snapshot) = snapshot.as_ref() {
            emit_profile_report(profile_report, &snapshot.profile);
        }
        Ok(snapshot)
    })
}

fn start_git_poll_thread() -> Arc<dyn Fn() + Send + Sync> {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let _ = thread::Builder::new()
        .name("okf-git-poll".into())
        .spawn(move || git_poll_loop(thread_stop));
    Arc::new(move || {
        stop.store(true, Ordering::Relaxed);
    })
}

fn git_poll_loop(stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        match crate::config::load() {
            Ok(config) => crate::resolve::tick_git_roots(&config, &crate::resolve::okf_cache_dir()),
            Err(error) => eprintln!("rocci-okf: failed to load OKF config: {error:#}"),
        }
        for _ in 0..10 {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn parse_cache_dir(root: &Path) -> std::path::PathBuf {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(root.to_string_lossy().as_bytes());
    let hash = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    rocci_roc_host::TwoTierCache::default_dir()
        .join("okf-parse")
        .join(format!("v{}", okf::PARSE_CACHE_VERSION))
        .join(hash)
}

#[allow(clippy::too_many_arguments)]
fn rebuild_site(
    root: &Path,
    output: &Path,
    profile: Profile,
    provenance: bool,
    host: Option<rocci_roc_host::HostChoice>,
    cache: &mut okf::ParseCache,
    apply_session: &mut Option<ApplySession>,
    logs: &LogHub,
) -> Result<Option<InspectSnapshot>> {
    let load_started = Instant::now();
    let loaded = okf::load_with_cache(
        root,
        LoadOptions::new(profile).with_provenance(provenance),
        Some(cache),
    )?;
    let load_ms = load_started.elapsed().as_millis();
    log_bundle_diagnostics(logs, &loaded.bundle);
    let mut snapshot = load_profile_snapshot(load_ms, &loaded.timings);
    let (mut built, next_session) =
        build_review_site_with_session(&loaded.bundle, output, host, apply_session.as_ref())?;
    *apply_session = next_session;
    snapshot.spans.append(&mut built.spans);
    snapshot.total_ms += built.total_ms;
    Ok(Some(inspect::from_bundle(
        root,
        &loaded.bundle,
        output,
        snapshot,
    )))
}

fn log_bundle_diagnostics(logs: &LogHub, bundle: &okf::Bundle) {
    for diagnostic in &bundle.diagnostics {
        let level = match diagnostic.severity {
            Severity::Error => LogLevel::Error,
            Severity::Warning => LogLevel::Warn,
        };
        logs::tee(logs, level, format!("rocci-okf: {diagnostic}"));
    }
}

fn load_profile_snapshot(load_ms: u128, timings: &LoadTimings) -> ProfileSnapshot {
    let mut spans = vec![
        ProfileSpan {
            name: "load".into(),
            duration_ms: load_ms,
            note: None,
        },
        ProfileSpan {
            name: "discover".into(),
            duration_ms: timings.discover.as_millis(),
            note: None,
        },
        ProfileSpan {
            name: "parse".into(),
            duration_ms: timings.parse.as_millis(),
            note: parse_cache_note(timings),
        },
        ProfileSpan {
            name: "graph".into(),
            duration_ms: timings.graph.as_millis(),
            note: None,
        },
    ];
    if let Some(provenance) = timings.provenance {
        spans.push(ProfileSpan {
            name: "provenance".into(),
            duration_ms: provenance.as_millis(),
            note: None,
        });
    }
    ProfileSnapshot {
        total_ms: load_ms,
        spans,
    }
}

fn parse_cache_note(timings: &LoadTimings) -> Option<String> {
    if timings.parse_cache_hits == 0 && timings.parse_cache_misses == 0 {
        None
    } else {
        Some(format!(
            "cache_hit={} miss={}",
            timings.parse_cache_hits, timings.parse_cache_misses
        ))
    }
}

fn emit_profile_report(mode: ProfileReportMode, snapshot: &ProfileSnapshot) {
    match mode {
        ProfileReportMode::Off => {}
        ProfileReportMode::Terminal => eprintln!("{}", format_profile_report(snapshot)),
        ProfileReportMode::Json => {
            eprintln!("rocci-okf profile {}", snapshot.to_json());
        }
    }
}

fn format_profile_report(snapshot: &ProfileSnapshot) -> String {
    let mut out = format!("rocci-okf profile total={}ms", snapshot.total_ms);
    for span in &snapshot.spans {
        out.push_str("\n  - ");
        out.push_str(&span.name);
        out.push_str(": ");
        out.push_str(&span.duration_ms.to_string());
        out.push_str("ms");
        if let Some(note) = span.note.as_deref() {
            out.push_str(" (");
            out.push_str(note);
            out.push(')');
        }
    }
    out
}

fn knowledge_path_is_relevant(path: &Path, root: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    if relative.as_os_str().is_empty() {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_profile_report_lists_total_and_spans() {
        let snapshot = ProfileSnapshot {
            total_ms: 42,
            spans: vec![
                ProfileSpan {
                    name: "load".into(),
                    duration_ms: 30,
                    note: None,
                },
                ProfileSpan {
                    name: "compile".into(),
                    duration_ms: 12,
                    note: Some("cached".into()),
                },
            ],
        };
        let report = format_profile_report(&snapshot);
        assert!(report.contains("rocci-okf profile total=42ms"));
        assert!(report.contains("load: 30ms"));
        assert!(report.contains("compile: 12ms (cached)"));
    }

    #[test]
    fn load_profile_snapshot_lists_load_and_named_subspans() {
        let timings = LoadTimings {
            discover: std::time::Duration::from_millis(1),
            parse: std::time::Duration::from_millis(20),
            graph: std::time::Duration::from_millis(2),
            provenance: Some(std::time::Duration::from_millis(7)),
            ..LoadTimings::default()
        };
        let snapshot = load_profile_snapshot(30, &timings);
        assert_eq!(snapshot.total_ms, 30);
        let names: Vec<_> = snapshot
            .spans
            .iter()
            .map(|span| span.name.as_str())
            .collect();
        assert_eq!(names, ["load", "discover", "parse", "graph", "provenance"]);
        assert_eq!(snapshot.spans[0].duration_ms, 30);
        assert_eq!(snapshot.spans[2].duration_ms, 20);
    }

    #[test]
    fn load_profile_snapshot_omits_provenance_when_absent() {
        let timings = LoadTimings {
            discover: std::time::Duration::from_millis(1),
            parse: std::time::Duration::from_millis(4),
            graph: std::time::Duration::from_millis(1),
            provenance: None,
            ..LoadTimings::default()
        };
        let snapshot = load_profile_snapshot(6, &timings);
        assert!(!snapshot.spans.iter().any(|span| span.name == "provenance"));
    }
}
