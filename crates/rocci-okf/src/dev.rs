use std::{fs, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
use okf::{Profile, load};
pub use rocci_cli::dev_server::DevServer;
use rocci_cli::dev_server::{StaticDevServerConfig, serve_static_site};
use rocci_cli::profile::ProfileSnapshot;

use crate::presentation::build_review_site_with_host;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfileReportMode {
    #[default]
    Off,
    Terminal,
    Json,
}

pub fn run_knowledge(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    profile: Profile,
    open_path: &str,
    host: Option<rocci_roc_host::HostChoice>,
    profile_report: ProfileReportMode,
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
    };

    let build_root = root.clone();
    serve_static_site(config, move |out_dir| {
        let snapshot = rebuild_site(&build_root, out_dir, profile, host)?;
        if let Some(snapshot) = snapshot.as_ref() {
            emit_profile_report(profile_report, snapshot);
        }
        Ok(snapshot)
    })
}

fn rebuild_site(
    root: &Path,
    output: &Path,
    profile: Profile,
    host: Option<rocci_roc_host::HostChoice>,
) -> Result<Option<rocci_cli::profile::ProfileSnapshot>> {
    let mut rec = rocci_cli::profile::SpanRecorder::new();
    let bundle = rec.span("load", || {
        let bundle = load(root, profile)?;
        if bundle.has_errors() {
            bail!("knowledge bundle has validation errors");
        }
        Ok(bundle)
    })?;
    let mut snapshot = rec.finish();
    let mut built = build_review_site_with_host(&bundle, output, host)?;
    snapshot.spans.append(&mut built.spans);
    snapshot.total_ms += built.total_ms;
    Ok(Some(snapshot))
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
                rocci_cli::profile::ProfileSpan {
                    name: "load".into(),
                    duration_ms: 30,
                    note: None,
                },
                rocci_cli::profile::ProfileSpan {
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
}
