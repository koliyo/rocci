use std::{fs, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
use okf::{Profile, load};
pub use rocci_cli::dev_server::DevServer;
use rocci_cli::dev_server::{StaticDevServerConfig, serve_static_site};

use crate::presentation::build_review_site_with_host;

pub fn run_knowledge(
    root: &Path,
    output: Option<&Path>,
    port: u16,
    profile: Profile,
    open_path: &str,
    host: Option<rocci_roc_host::HostChoice>,
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
        rebuild_site(&build_root, out_dir, profile, host)
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

fn knowledge_path_is_relevant(path: &Path, root: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    if relative.as_os_str().is_empty() {
        return false;
    }
    true
}
