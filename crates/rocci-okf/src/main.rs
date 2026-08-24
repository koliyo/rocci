use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use okf::InspectKind;
use rocci_cli::serve::{PortArg, parse_port_arg};

mod dev;
mod inspect;
mod presentation;
mod runtime;
mod session;

#[derive(Parser)]
#[command(
    name = "rocci-okf",
    about = "Rocci Open Knowledge Format (OKF) review and query application"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Preview an OKF bundle or a concept file with live reload.
    View {
        #[command(flatten)]
        preview: PreviewArgs,
    },
    /// Deprecated alias for `view`.
    Run {
        #[command(flatten)]
        preview: PreviewArgs,
    },
    /// Validate an OKF bundle without writing output.
    Check {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Print normalized concepts or the bundle graph as JSON.
    Inspect {
        #[command(subcommand)]
        target: KnowledgeInspectTarget,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
    },
    /// Search metadata and heading chunks as JSON.
    Search {
        query: String,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        #[command(flatten)]
        filters: KnowledgeFiltersArg,
    },
    /// Run a retrieval benchmark TOML file against a knowledge bundle.
    Benchmark {
        benchmark: PathBuf,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
    },
    /// Emit derived bundle artifacts and the minimal static review site.
    Build {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(short, long, default_value = "dist/knowledge")]
        output: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        /// URL prefix for emitted HTML, indexes, and assets. Empty keeps local viewer routes.
        #[arg(long, default_value = "")]
        base_path: String,
        /// Omit live-reload and desktop session scripts from the static tree.
        #[arg(long)]
        public: bool,
        /// Execution host runtime for evaluating templates (native, wasm, auto [default]).
        #[arg(long, value_enum, default_value_t = HostArg::Auto)]
        host: HostArg,
    },
}

#[derive(Args, Debug)]
struct PreviewArgs {
    /// Knowledge bundle directory or a Markdown file inside one.
    path: Option<PathBuf>,
    /// Write preview output here instead of a temp directory.
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
    profile: KnowledgeProfileArg,
    /// Execution host runtime for evaluating templates (native, wasm, auto [default]).
    #[arg(long, value_enum, default_value_t = HostArg::Auto)]
    host: HostArg,
    /// Skip the preview window; print the URL and keep serving.
    /// Open that URL with `?reload=0` to pause automatic page refresh.
    #[arg(long)]
    no_window: bool,
    /// Pause automatic page refresh. Watch and rebuild still run.
    #[arg(long)]
    no_live_reload: bool,
    /// Bind every interface (`0.0.0.0`). Default is localhost only.
    #[arg(long)]
    public: bool,
    /// Run git provenance checks (OKF4006/4007/4008) during preview.
    /// Off by default; `check --profile rocci` still runs them.
    #[arg(long)]
    provenance: bool,
    /// Emit rebuild profiling to stderr in terminal or JSON form.
    #[arg(long, value_enum, default_value_t = ProfileReportArg::Off)]
    profile_report: ProfileReportArg,
    /// TCP port to listen on. Defaults to a free port with the preview window,
    /// or 8000 with `--no-window`. Pass `auto` to pick a free port.
    #[arg(
        long,
        default_value = "auto",
        default_value_if("no_window", "true", "8000"),
        value_name = "PORT",
        value_parser = parse_port_arg,
        env = "ROC_BASIC_WEBSERVER_PORT"
    )]
    port: PortArg,
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFormatArg {
    Terminal,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum ProfileReportArg {
    #[default]
    Off,
    Terminal,
    Json,
}

impl From<ProfileReportArg> for dev::ProfileReportMode {
    fn from(value: ProfileReportArg) -> Self {
        match value {
            ProfileReportArg::Off => dev::ProfileReportMode::Off,
            ProfileReportArg::Terminal => dev::ProfileReportMode::Terminal,
            ProfileReportArg::Json => dev::ProfileReportMode::Json,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum HostArg {
    /// Use Roc when it is on PATH; otherwise the Rust knowledge shell.
    #[default]
    Auto,
    /// Require the native Roc applicator (`roc` must be on PATH).
    Native,
    /// In-process Wasmtime host.
    Wasm,
}

fn preview_knowledge(preview: PreviewArgs) -> Result<()> {
    let PreviewArgs {
        path,
        output,
        profile,
        host,
        no_window,
        no_live_reload,
        public,
        provenance,
        profile_report,
        port,
    } = preview;
    let live_reload = !no_live_reload;
    let (target, persist_bundle) = resolve_launch_target(path)?;
    let port = port.resolve()?;
    let session_state = std::sync::Arc::new(std::sync::Mutex::new({
        let mut stored = session::load();
        stored.bundle = Some(target.root.clone());
        stored
    }));
    let extra_http = Some(session_http_handler(session_state.clone()));
    let server = dev::run_knowledge(
        &target.root,
        output.as_deref(),
        port,
        profile.into(),
        provenance,
        &target.open_path,
        preview_host(host),
        profile_report.into(),
        public,
        extra_http,
    )?;
    seed_session_file(&server.output, &session_state.lock().unwrap());
    rocci_cli::logs::tee(
        &server.logs,
        rocci_cli::logs::LogLevel::Info,
        format!("rocci-okf: serving {} at {}", server.title, server.url),
    );
    rocci_cli::serve::note_live_reload_paused(live_reload);
    if no_window {
        server.wait();
        return Ok(());
    }
    let home_url = server
        .url
        .split_once("://")
        .and_then(|(scheme, rest)| {
            rest.find('/')
                .map(|index| format!("{scheme}://{rest}", rest = &rest[..index + 1]))
        })
        .unwrap_or_else(|| format!("{}/", server.url.trim_end_matches('/')));
    let persist_root = persist_bundle.clone();
    let persist_session = session_state.clone();
    let result = rocci_desktop::preview(rocci_desktop::PreviewOptions {
        url: server.url.clone(),
        title: format!("{} — Rocci Knowledge", server.title),
        state_key: Some("rocci:knowledge".to_string()),
        width: 1200.0,
        height: 800.0,
        inspector_url: Some(server.inspector_url.clone()),
        live_reload,
        source_root: Some(persist_root.clone()),
        home_url: Some(home_url),
        on_navigate: Some(std::sync::Arc::new(move |url| {
            let route = session::route_from_url(url);
            let mut stored = persist_session.lock().unwrap();
            stored.bundle = Some(persist_root.clone());
            session::record_visit(&mut stored, &route, title_from_route(&route));
            session::save(&stored);
        })),
        ..rocci_desktop::PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

fn resolve_launch_target(path: Option<PathBuf>) -> Result<(okf::PreviewTarget, PathBuf)> {
    if let Some(path) = path {
        let target = okf::resolve_preview_path(&path)?;
        let mut stored = session::load();
        stored.bundle = Some(target.root.clone());
        stored.open_path = target.open_path.clone();
        session::save(&stored);
        let root = target.root.clone();
        return Ok((target, root));
    }
    let stored = session::load();
    if let Some(bundle) = stored.bundle.as_ref()
        && bundle.is_dir()
    {
        let open_path = session::resolve_saved_open_path(bundle, &stored.open_path);
        return Ok((
            okf::PreviewTarget {
                root: bundle.clone(),
                open_path,
            },
            bundle.clone(),
        ));
    }
    let fallback = PathBuf::from("knowledge");
    if fallback.is_dir() {
        let target = okf::resolve_preview_path(&fallback)?;
        let root = target.root.clone();
        return Ok((target, root));
    }
    if session::launched_as_app() {
        let picked = session::pick_bundle_folder()?;
        let target = okf::resolve_preview_path(&picked)?;
        let mut stored = session::load();
        stored.bundle = Some(target.root.clone());
        stored.open_path = "/".into();
        session::save(&stored);
        let root = target.root.clone();
        return Ok((target, root));
    }
    bail!(
        "no knowledge bundle to open; pass a bundle path or run from a repository with ./knowledge"
    );
}

fn title_from_route(route: &str) -> &str {
    route
        .trim_matches('/')
        .rsplit('/')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or("Knowledge")
}

fn seed_session_file(output: &std::path::Path, session: &session::OkfSession) {
    let dest = output.join("__rocci_okf").join("session.json");
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(dest, json);
    }
}

fn session_http_handler(
    state: std::sync::Arc<std::sync::Mutex<session::OkfSession>>,
) -> rocci_cli::dev_server::ExtraHttpHandler {
    std::sync::Arc::new(move |method, path, raw| {
        if path != "/__rocci_okf/session" {
            return None;
        }
        if method == "GET" {
            let json = serde_json::to_vec_pretty(&*state.lock().unwrap())
                .unwrap_or_else(|_| b"{}".to_vec());
            return Some((200, "application/json; charset=utf-8", json));
        }
        if method == "POST" {
            let body = raw
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|index| &raw[index + 4..])
                .unwrap_or(b"");
            if let Ok(visit) = serde_json::from_slice::<serde_json::Value>(body) {
                let mut stored = state.lock().unwrap();
                let route = visit
                    .get("route")
                    .and_then(|value| value.as_str())
                    .unwrap_or("/");
                let title = visit
                    .get("title")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                session::record_visit(&mut stored, route, title);
                session::save(&stored);
                if let Ok(json) = serde_json::to_vec_pretty(&*stored) {
                    return Some((200, "application/json; charset=utf-8", json));
                }
            }
            return Some((400, "application/json; charset=utf-8", b"{}".to_vec()));
        }
        Some((
            405,
            "text/plain; charset=utf-8",
            b"method not allowed".to_vec(),
        ))
    })
}

fn preview_host(host: HostArg) -> Option<rocci_roc_host::HostChoice> {
    match host {
        HostArg::Auto => None,
        HostArg::Native => Some(rocci_roc_host::HostChoice::Native),
        HostArg::Wasm => Some(rocci_roc_host::HostChoice::Wasm),
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum KnowledgeProfileArg {
    Base,
    Rocci,
}

impl From<KnowledgeProfileArg> for okf::Profile {
    fn from(value: KnowledgeProfileArg) -> Self {
        match value {
            KnowledgeProfileArg::Base => Self::Base,
            KnowledgeProfileArg::Rocci => Self::Rocci,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum TrustTierArg {
    HumanReviewed,
    Generated,
    Unverified,
}

impl From<TrustTierArg> for okf::TrustTier {
    fn from(value: TrustTierArg) -> Self {
        match value {
            TrustTierArg::HumanReviewed => Self::HumanReviewed,
            TrustTierArg::Generated => Self::Generated,
            TrustTierArg::Unverified => Self::Unverified,
        }
    }
}

#[derive(Args, Default)]
struct KnowledgeFiltersArg {
    /// Match any of these concept types. Repeat to add alternatives.
    #[arg(long = "type")]
    types: Vec<String>,
    /// Require this tag. Repeat to require multiple tags.
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Match any of these lifecycle statuses. Repeat to add alternatives.
    #[arg(long = "status")]
    statuses: Vec<String>,
    /// Match any of these authority levels. Repeat to add alternatives.
    #[arg(long = "authority")]
    authorities: Vec<String>,
    /// Match any of these derived trust tiers. Repeat to add alternatives.
    #[arg(long = "trust-tier", value_enum)]
    trust_tiers: Vec<TrustTierArg>,
    /// Match stale (`true`) or current (`false`) records.
    #[arg(long)]
    stale: Option<bool>,
}

impl From<&KnowledgeFiltersArg> for okf::KnowledgeFilter {
    fn from(value: &KnowledgeFiltersArg) -> Self {
        Self {
            types: value.types.clone(),
            tags: value.tags.clone(),
            statuses: value.statuses.clone(),
            authorities: value.authorities.clone(),
            trust_tiers: value.trust_tiers.iter().copied().map(Into::into).collect(),
            stale: value.stale,
        }
    }
}

#[derive(Subcommand)]
enum KnowledgeInspectTarget {
    Catalog {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[command(flatten)]
        filters: KnowledgeFiltersArg,
    },
    Concept {
        concept: String,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
    },
    Graph {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse_from(session::filter_launch_args(std::env::args()));
    match cli.command.unwrap_or_else(|| Commands::View {
        preview: PreviewArgs {
            path: None,
            output: None,
            profile: KnowledgeProfileArg::Rocci,
            host: HostArg::Auto,
            no_window: false,
            no_live_reload: false,
            public: false,
            provenance: false,
            profile_report: ProfileReportArg::Off,
            port: PortArg::Auto,
        },
    }) {
        Commands::Check {
            root,
            profile,
            format,
        } => {
            let report = okf::check(&root, profile.into())?;
            match format {
                CheckFormatArg::Terminal => {
                    let formatted = report.terminal();
                    if !formatted.is_empty() {
                        println!("{formatted}");
                    }
                }
                CheckFormatArg::Json => {
                    println!("{}", report.json()?);
                }
            }
            if report.has_errors() {
                bail!("knowledge check failed with errors");
            }
            Ok(())
        }
        Commands::Inspect { target, profile } => {
            let json = match target {
                KnowledgeInspectTarget::Catalog { root, filters } => okf::inspect_filtered(
                    &root,
                    InspectKind::Catalog,
                    None,
                    profile.into(),
                    &(&filters).into(),
                )?,
                KnowledgeInspectTarget::Concept { concept, root } => {
                    okf::inspect(&root, InspectKind::Concept, Some(&concept), profile.into())?
                }
                KnowledgeInspectTarget::Graph { root } => {
                    okf::inspect(&root, InspectKind::Graph, None, profile.into())?
                }
            };
            println!("{json}");
            Ok(())
        }
        Commands::Search {
            query,
            root,
            profile,
            filters,
        } => {
            let json = okf::search(&root, &query, profile.into(), &(&filters).into())?;
            println!("{json}");
            Ok(())
        }
        Commands::Benchmark {
            benchmark,
            root,
            profile,
        } => {
            let report = okf::benchmark_retrieval(&root, &benchmark, profile.into())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.threshold_met {
                bail!(
                    "retrieval benchmark failed: hit rate {:.2}% was below required minimum {:.2}%",
                    report.hit_rate * 100.0,
                    report.minimum_hit_rate * 100.0
                );
            }
            Ok(())
        }
        Commands::Build {
            root,
            output,
            profile,
            host,
            base_path,
            public,
        } => {
            let options = presentation::SiteOptions::from_args(&base_path, public)?;
            let bundle = okf::load(&root, profile.into())?;
            if bundle.has_errors() {
                bail!("knowledge bundle has validation errors");
            }
            let summary = okf::build_artifacts(&bundle, &output)?;
            let _profile =
                presentation::build_review_site_with_host(&bundle, &output, preview_host(host))?;
            presentation::finalize_site(&output, &options)?;
            eprintln!(
                "rocci-okf: built {} concepts and {} indexes into {}",
                summary.concepts, summary.indexes, summary.output
            );
            Ok(())
        }
        Commands::View { preview } => preview_knowledge(preview),
        Commands::Run { preview } => {
            eprintln!(
                "rocci-okf: `run` is a deprecated alias for `view` and will be removed in a later release"
            );
            preview_knowledge(preview)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn auto_preview_host_does_not_force_roc() {
        assert!(preview_host(HostArg::Auto).is_none());
        assert_eq!(
            preview_host(HostArg::Native),
            Some(rocci_roc_host::HostChoice::Native)
        );
        assert_eq!(
            preview_host(HostArg::Wasm),
            Some(rocci_roc_host::HostChoice::Wasm)
        );
    }

    fn preview_args(command: Commands) -> PreviewArgs {
        match command {
            Commands::View { preview } | Commands::Run { preview } => preview,
            _ => panic!("expected view or run"),
        }
    }

    #[test]
    fn no_args_defaults_to_restored_view() {
        let cli = Cli::try_parse_from(["rocci-okf"]).unwrap();
        assert!(cli.command.is_none());
        let cli = Cli::try_parse_from(["rocci-okf", "view"]).unwrap();
        assert!(preview_args(cli.command.unwrap()).path.is_none());
    }

    #[test]
    fn view_accepts_no_live_reload() {
        let cli =
            Cli::try_parse_from(["rocci-okf", "view", "knowledge", "--no-live-reload"]).unwrap();
        assert!(preview_args(cli.command.unwrap()).no_live_reload);
        let cli = Cli::try_parse_from(["rocci-okf", "view", "knowledge"]).unwrap();
        assert!(!preview_args(cli.command.unwrap()).no_live_reload);
    }

    #[test]
    fn view_accepts_public() {
        let cli = Cli::try_parse_from(["rocci-okf", "view", "knowledge", "--public"]).unwrap();
        assert!(preview_args(cli.command.unwrap()).public);
    }

    #[test]
    fn run_remains_a_deprecated_alias_for_view() {
        let cli =
            Cli::try_parse_from(["rocci-okf", "run", "knowledge", "--no-live-reload"]).unwrap();
        match cli.command.unwrap() {
            Commands::Run { preview } => assert!(preview.no_live_reload),
            _ => panic!("expected run alias"),
        }
    }

    #[test]
    fn no_window_help_mentions_reload_query() {
        use clap::CommandFactory;
        let cmd = Cli::command();
        let view = cmd.find_subcommand("view").expect("view");
        let arg = view
            .get_arguments()
            .find(|arg| arg.get_long() == Some("no-window"))
            .expect("no-window");
        let help = format!(
            "{}{}",
            arg.get_help().map(|h| h.to_string()).unwrap_or_default(),
            arg.get_long_help()
                .map(|h| h.to_string())
                .unwrap_or_default()
        );
        assert!(help.contains("?reload=0"), "{help}");
    }
}
