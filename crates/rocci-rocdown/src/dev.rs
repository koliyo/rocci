use std::{collections::BTreeSet, fs, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
pub use rocci_cli::dev_server::DevServer;
use rocci_cli::dev_server::{StaticDevServerConfig, serve_static_site};

use crate::build::{BuildSession, absolute};
use crate::config::load_config;

pub fn run(root: &Path, output: Option<&Path>, port: u16) -> Result<DevServer> {
    run_with_host(root, output, port, None)
}

pub fn run_with_host(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    host: Option<rocci_roc_host::HostChoice>,
) -> Result<DevServer> {
    let root = absolute(root)?;
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let root = fs::canonicalize(&root)
        .with_context(|| format!("failed to resolve root {}", root.display()))?;

    let title = load_config(&root)
        .map(|config| config.site.title)
        .unwrap_or_else(|_| "Documentation".into());
    let assets = load_config(&root)
        .map(|config| config.build.assets)
        .unwrap_or_else(|_| "assets".into());

    let host_choice = host.unwrap_or_default();
    let mut session = BuildSession::create_with_host(host_choice)?;

    let mut watch_paths = vec![root.clone()];
    if let Ok(config) = load_config(&root) {
        for mount in &config.mounts {
            let mount_dir = root.join(&mount.source);
            if mount_dir.is_dir() {
                let canonical = fs::canonicalize(&mount_dir).unwrap_or(mount_dir);
                if !canonical.starts_with(&root) {
                    watch_paths.push(canonical);
                }
            }
        }
        for entry in &config.snippets.roots {
            let path = root.join(entry);
            if path.is_dir() && !path.starts_with(&root) {
                watch_paths.push(path);
            }
        }
    }

    let filter_root = root.clone();
    let filter_assets = assets;
    let snippet_paths = session.snippet_paths.clone();
    let custom_filter = Arc::new(move |path: &Path| {
        path_is_relevant(path, &filter_root, &filter_assets, &snippet_paths)
    });

    let config = StaticDevServerConfig {
        title,
        port,
        open_path: "/".into(),
        output: output.map(Path::to_path_buf),
        watch_paths,
        custom_filter: Some(custom_filter),
        log_prefix: "rocdown".into(),
    };

    let session_root = root.clone();
    serve_static_site(config, move |out_dir| {
        let report = session.rebuild(&session_root, out_dir)?;
        Ok(Some(profile_from_report(&report)))
    })
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

pub(crate) fn path_is_relevant(
    path: &Path,
    root: &Path,
    assets: &str,
    snippet_paths: &BTreeSet<String>,
) -> bool {
    let components: Vec<_> = path.components().collect();
    if components
        .iter()
        .any(|c| c.as_os_str() == ".git" || c.as_os_str() == "target")
    {
        return false;
    }
    if let Some(s) = path.to_str()
        && snippet_paths.contains(s)
    {
        return true;
    }
    if let Ok(rel) = path.strip_prefix(root) {
        if rel.as_os_str().is_empty() {
            return false;
        }
        if rel.iter().any(|comp| {
            let s = comp.to_string_lossy();
            s.starts_with('.') && s != "." && s != ".."
        }) {
            return false;
        }
        if rel == Path::new("rocdown.toml") {
            return true;
        }
        if rel.starts_with(assets) {
            return true;
        }
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            return matches!(
                ext,
                "rocdown" | "md" | "markdown" | "rocci" | "roc" | "css" | "png" | "jpg" | "svg"
            );
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn path_filter_keeps_content_and_ignores_noise() {
        let root = PathBuf::from("/docs");
        let none = std::collections::BTreeSet::new();
        assert!(path_is_relevant(
            Path::new("/docs/index.rocdown"),
            &root,
            "assets",
            &none
        ));
        assert!(path_is_relevant(
            Path::new("/docs/rocdown.toml"),
            &root,
            "assets",
            &none
        ));
        assert!(path_is_relevant(
            Path::new("/docs/assets/og.png"),
            &root,
            "assets",
            &none
        ));
        assert!(!path_is_relevant(
            Path::new("/docs/.git/index"),
            &root,
            "assets",
            &none
        ));
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
}
