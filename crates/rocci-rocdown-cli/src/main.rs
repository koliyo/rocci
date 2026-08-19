use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rocci_cli::driver::{DriverOptions, GenericAppPlan, GenericModule};
use rocci_cli::path_hint;
use rocci_cli::serve::{PortArg, parse_port_arg};
use rocci_rocdown::{SourceFile, StandaloneReady, ThemeArgs, format_diagnostic};

#[derive(Parser)]
#[command(
    name = "rocdown",
    about = "Rocdown documentation compiler, static site generator, and interactive document runtime"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a static documentation site or a single .rocdown document.
    Build {
        /// Site root directory or single .rocdown file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Override output path (directory for site, file for single document).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Execution host runtime for evaluating templates (native, wasm, auto [default]).
        #[arg(long, value_enum, default_value_t = HostArg::Auto)]
        host: HostArg,
        /// Error if the site has `live` pages (CDN-only publish with no island service).
        #[arg(long)]
        cdn_only: bool,
        #[command(flatten)]
        theme: ThemeArgs,
    },
    /// Run an interactive document or serve a documentation site with live reload.
    /// Hybrid sites proxy the island service on the same origin.
    Run {
        /// Site root directory, or a .rocdown file. A file inside a site
        /// (ancestor `rocdown.toml`) previews that site at the page route.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Write preview output here instead of a temp directory (for site dev server).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Execution host runtime for evaluating templates (native, wasm, auto [default]).
        #[arg(long, value_enum, default_value_t = HostArg::Auto)]
        host: HostArg,
        /// Skip the preview window; print the URL and keep serving.
        #[arg(long)]
        no_window: bool,
        /// Do not print compile diagnostics on stderr; the error page still serves.
        #[arg(long)]
        quiet: bool,
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
        #[command(flatten)]
        theme: ThemeArgs,
        /// Extra arguments forwarded to `roc` after `--` (for interactive documents).
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Start the island HTTP service for live pages in a documentation site.
    ServeIslands {
        /// Site root directory.
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Skip the preview window; print the URL and keep serving.
        #[arg(long)]
        no_window: bool,
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
    },
    /// Validate the documentation catalog or document without writing output.
    Check {
        #[arg(default_value = ".")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Run declared `:example` commands. Never part of `rocdown build`.
    Test {
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Rewrite golden files from captured stdout.
        #[arg(long)]
        update: bool,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Print resolved catalog, document AST, or generated Roc.
    Inspect {
        #[command(subcommand)]
        target: InspectTarget,
    },
    /// Open a playground to live edit a `.rocdown` document or `.rocci` template.
    Playground {
        /// Compiler host: `wasm` runs in the browser worker; `local` compiles natively.
        #[arg(long, value_enum, default_value_t = PlaygroundModeArg::Wasm)]
        mode: PlaygroundModeArg,
        /// Source file to open (`.rocdown`, `.md`, `.markdown`, or `.rocci`).
        #[arg(default_value = "Guide.rocdown")]
        input: PathBuf,
        /// Skip the preview window; print the URL and keep serving.
        #[arg(long)]
        no_window: bool,
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
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFormatArg {
    Terminal,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum HostArg {
    /// Pick host automatically (native on dev, wasm when requested or embedded).
    #[default]
    Auto,
    /// Compile and run native host executable (requires roc on PATH).
    Native,
    /// In-process Wasmtime host.
    Wasm,
}

impl From<HostArg> for rocci_rocdown::HostChoice {
    fn from(arg: HostArg) -> Self {
        match arg {
            HostArg::Auto => rocci_rocdown::HostChoice::Auto,
            HostArg::Native => rocci_rocdown::HostChoice::Native,
            HostArg::Wasm => rocci_rocdown::HostChoice::Wasm,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
enum PlaygroundModeArg {
    #[default]
    Wasm,
    Local,
}

impl From<PlaygroundModeArg> for rocci_cli::playground::PlaygroundMode {
    fn from(mode: PlaygroundModeArg) -> Self {
        match mode {
            PlaygroundModeArg::Wasm => Self::Wasm,
            PlaygroundModeArg::Local => Self::Local,
        }
    }
}

fn rocdown_local_compile_hook() -> rocci_cli::playground::PlaygroundCompileHook {
    Arc::new(
        |body| match serde_json::from_slice::<rocci_playground::CompileRequest>(body) {
            Ok(request) => {
                let mut resp = rocci_playground::compile(&request);
                resp.capabilities.html = rocci_playground::HtmlCapability {
                    available: false,
                    reason: rocci_cli::playground::ROCDOWN_LOCAL_HTML_REASON.to_string(),
                };
                serde_json::to_vec(&resp).unwrap_or_else(|err| {
                    serde_json::to_vec(&serde_json::json!({
                        "protocol_version": 1,
                        "revision": request.revision,
                        "error": format!("serialization error: {err}"),
                        "has_errors": true,
                    }))
                    .unwrap_or_default()
                })
            }
            Err(err) => serde_json::to_vec(&serde_json::json!({
                "protocol_version": 1,
                "error": format!("invalid JSON request: {err}"),
                "has_errors": true,
            }))
            .unwrap_or_default(),
        },
    )
}

#[derive(Subcommand)]
enum InspectTarget {
    Config {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Catalog {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Page {
        page: String,
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Graph {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Nav {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Artifacts {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    /// Print a .rocdown parse tree as a LISPy S-expression.
    Ast {
        input: PathBuf,
        #[command(flatten)]
        theme: ThemeArgs,
    },
    /// Print generated Roc and source map segments for a single .rocdown file.
    Roc {
        input: PathBuf,
        #[command(flatten)]
        theme: ThemeArgs,
    },
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn is_document_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .is_some_and(|ext| ext == "rocdown" || ext == "md" || ext == "markdown")
}

fn refuse_okf_input(path: &Path, command_name: &str) -> Result<()> {
    if path.is_dir() {
        if path_hint::looks_like_okf_bundle(path) {
            bail!(
                "`rocdown {command_name}` does not preview OKF knowledge bundles; run `rocci-okf run {}`",
                path.display()
            );
        }
        return Ok(());
    }
    let is_markdown = path
        .extension()
        .is_some_and(|ext| ext == "md" || ext == "markdown");
    if is_markdown && path_hint::looks_like_okf_file(path) {
        bail!(
            "`rocdown {command_name}` does not render OKF knowledge records; preview {} with `rocci-okf run {}`",
            path.display(),
            path.display()
        );
    }
    Ok(())
}

fn try_main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            path,
            output,
            host,
            cdn_only,
            theme,
        } => {
            if is_document_file(&path) {
                refuse_okf_input(&path, "build")?;
                if cdn_only {
                    bail!("`--cdn-only` applies to site builds, not a single .rocdown file");
                }
                build_single_doc(&path, output.as_deref(), &theme)
            } else if path.is_file() {
                bail!(
                    "unsupported file extension for `rocdown build`: {}; expected a .rocdown, .md, or .markdown file",
                    path.display()
                );
            } else {
                refuse_okf_input(&path, "build")?;
                let report = rocci_rocdown::build_configured_with_options(
                    &path,
                    output.as_deref(),
                    rocci_rocdown::BuildOptions {
                        host: Some(host.into()),
                        cdn_only,
                    },
                )?;
                print!("{}", report.render_publish());
                Ok(())
            }
        }
        Commands::Run {
            path,
            output,
            host,
            no_window,
            quiet,
            port,
            theme,
            args,
        } => {
            if is_document_file(&path) {
                refuse_okf_input(&path, "run")?;
                if let Some(site_root) = rocci_rocdown::find_site_root(&path) {
                    let route = rocci_rocdown::site_preview_route(&site_root, &path);
                    run_site_dev(
                        &site_root,
                        output.as_deref(),
                        no_window,
                        port,
                        host.into(),
                        Some(&route),
                    )
                } else {
                    run_standalone_doc(&path, &args, no_window, quiet, port, &theme)
                }
            } else if path.is_file() {
                bail!(
                    "unsupported file extension for `rocdown run`: {}; expected a .rocdown, .md, or .markdown file",
                    path.display()
                );
            } else {
                refuse_okf_input(&path, "run")?;
                run_site_dev(&path, output.as_deref(), no_window, port, host.into(), None)
            }
        }
        Commands::ServeIslands {
            root,
            no_window,
            port,
        } => {
            refuse_okf_input(&root, "serve-islands")?;
            rocci_rocdown::serve_islands(&root, no_window, port)
        }
        Commands::Check { root, format } => {
            let report = rocci_rocdown::check(&root)?;
            let rendered = report.render(match format {
                CheckFormatArg::Terminal => rocci_rocdown::CheckFormat::Terminal,
                CheckFormatArg::Json => rocci_rocdown::CheckFormat::Json,
            })?;
            if !rendered.is_empty() {
                println!("{rendered}");
            }
            if report.has_errors() {
                bail!("documentation catalog has errors");
            }
            Ok(())
        }
        Commands::Test {
            root,
            update,
            format,
        } => {
            let report = rocci_rocdown::test_examples(&root, update)?;
            let rendered = report.render(match format {
                CheckFormatArg::Terminal => rocci_rocdown::CheckFormat::Terminal,
                CheckFormatArg::Json => rocci_rocdown::CheckFormat::Json,
            })?;
            if !rendered.is_empty() {
                println!("{rendered}");
            }
            if report.has_errors() {
                bail!("documentation examples failed");
            }
            Ok(())
        }
        Commands::Inspect { target } => match target {
            InspectTarget::Config { root } => {
                println!(
                    "{}",
                    rocci_rocdown::inspect(&root, rocci_rocdown::InspectKind::Config, None)?
                );
                Ok(())
            }
            InspectTarget::Catalog { root } => {
                println!(
                    "{}",
                    rocci_rocdown::inspect(&root, rocci_rocdown::InspectKind::Catalog, None)?
                );
                Ok(())
            }
            InspectTarget::Page { page, root } => {
                println!(
                    "{}",
                    rocci_rocdown::inspect(
                        &root,
                        rocci_rocdown::InspectKind::Page,
                        Some(page.as_str())
                    )?
                );
                Ok(())
            }
            InspectTarget::Graph { root } => {
                println!(
                    "{}",
                    rocci_rocdown::inspect(&root, rocci_rocdown::InspectKind::Graph, None)?
                );
                Ok(())
            }
            InspectTarget::Nav { root } => {
                println!(
                    "{}",
                    rocci_rocdown::inspect(&root, rocci_rocdown::InspectKind::Nav, None)?
                );
                Ok(())
            }
            InspectTarget::Artifacts { root } => {
                println!(
                    "{}",
                    rocci_rocdown::inspect(&root, rocci_rocdown::InspectKind::Artifacts, None)?
                );
                Ok(())
            }
            InspectTarget::Ast { input, theme } => inspect_ast(&input, &theme),
            InspectTarget::Roc { input, theme } => inspect_roc(&input, &theme),
        },
        Commands::Playground {
            input,
            no_window,
            port,
            mode,
        } => {
            let serve = rocci_cli::serve::ServeOptions { no_window, port };
            let hook = match mode {
                PlaygroundModeArg::Local => Some(rocdown_local_compile_hook()),
                PlaygroundModeArg::Wasm => None,
            };
            rocci_cli::playground::run_playground_cli(&input, serve, "rocdown", mode.into(), hook)
        }
    }
}

fn ensure_document_file(input: &Path, command_name: &str) -> Result<()> {
    if !input.is_file() {
        bail!("no such file: {}", input.display());
    }
    if !is_document_file(input) {
        bail!(
            "unsupported file extension for `rocdown {command_name}`: {}; expected a .rocdown, .md, or .markdown file",
            input.display()
        );
    }
    Ok(())
}

fn build_single_doc(input: &Path, output: Option<&Path>, theme: &ThemeArgs) -> Result<()> {
    ensure_document_file(input, "build")?;
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let options = theme.compile_options(Some(input));
    let compiled = rocci_rocdown::compile(source, &options);
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    if compiled.has_errors() {
        bail!("document compilation failed");
    }
    match output {
        Some(path) => {
            fs::write(path, &compiled.roc)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
        None => print!("{}", compiled.roc),
    }
    Ok(())
}

fn run_standalone_doc(
    file: &Path,
    args: &[String],
    no_window: bool,
    quiet: bool,
    port: PortArg,
    theme: &ThemeArgs,
) -> Result<()> {
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };
    let src_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));
    let title = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("rocdown")
        .to_string();

    let compile_opts = theme.compile_options(Some(&path));
    let parse_started = Instant::now();
    let plan = match rocci_rocdown::plan_standalone(&path, &compile_opts.theme)? {
        StandaloneReady::Failed(files) => {
            let failed_files: Vec<rocci_cli::error_page::FailedFile> = files
                .into_iter()
                .map(|f| rocci_cli::error_page::FailedFile {
                    name: f.name,
                    src: f.src,
                    diagnostics: f.diagnostics,
                })
                .collect();
            if !quiet {
                rocci_cli::error_page::eprint_template_errors(&failed_files);
            }
            return rocci_cli::driver::serve_template_errors(
                &failed_files,
                port,
                no_window,
                &title,
            );
        }
        StandaloneReady::Ready(plan) => plan,
    };
    let parse_ms = parse_started.elapsed().as_millis();
    let mut profile = rocci_cli::profile::SpanRecorder::new();
    profile.push("parse", parse_ms, None);

    let generic_plan = GenericAppPlan {
        primary_name: plan.primary_name,
        modules: plan
            .modules
            .into_iter()
            .map(|module| GenericModule {
                type_name: module.type_name,
                roc: module.roc,
                state_type: module.state_type,
                init: module.init,
                routes: module.routes,
                mapped: module.mapped,
                local_assets: module.local_assets,
            })
            .collect(),
        redirect_trailing_slash: plan.redirect_trailing_slash,
    };

    let driver_options = DriverOptions {
        args: args.to_vec(),
        no_window,
        port,
        db_path: None,
        title,
        preview_path: None,
        profile: profile.finish(),
        state_key: Some("rocdown".to_string()),
    };

    rocci_cli::driver::execute_app_plan(&generic_plan, &src_dir, &driver_options)
}

fn run_site_dev(
    root: &Path,
    output: Option<&Path>,
    no_window: bool,
    port: PortArg,
    host: rocci_rocdown::HostChoice,
    open_path: Option<&str>,
) -> Result<()> {
    let port = port.resolve()?;
    let open_path = open_path.unwrap_or("/");
    let server = rocci_rocdown::run_with_host_at(root, output, port, Some(host), open_path)?;
    eprintln!("rocdown: serving {} at {}", server.title, server.url);
    if no_window {
        server.wait();
        return Ok(());
    }
    let source_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let result = rocci_desktop::preview(rocci_desktop::PreviewOptions {
        url: server.url.clone(),
        title: server.title.clone(),
        state_key: Some("rocdown".to_string()),
        inspector_url: Some(server.inspector_url.clone()),
        source_root: Some(source_root),
        ..rocci_desktop::PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

fn inspect_ast(input: &Path, theme: &ThemeArgs) -> Result<()> {
    ensure_document_file(input, "inspect ast")?;
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let options = theme.compile_options(Some(input));
    let compiled = rocci_rocdown::compile(source, &options);
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    print!("{}", rocci_rocdown::format_ast(&src, &compiled.document));
    if compiled.has_errors() {
        bail!("document compilation failed");
    }
    Ok(())
}

fn inspect_roc(input: &Path, theme: &ThemeArgs) -> Result<()> {
    ensure_document_file(input, "inspect roc")?;
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let options = theme.compile_options(Some(input));
    let compiled = rocci_rocdown::compile(source, &options);
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    println!("# components ({})", compiled.components.len());
    for component in &compiled.components {
        println!(
            "- {} ({})",
            component.name,
            component.param_names.join(", ")
        );
    }
    println!("# fixtures ({})", compiled.fixtures.len());
    for fixture in &compiled.fixtures {
        println!("- {} -> {}", fixture.name, fixture.target);
    }
    println!(
        "# page route={} draft={} layout={}",
        compiled.page_meta.route.as_deref().unwrap_or("/"),
        compiled.page_meta.draft,
        compiled.page_meta.layout.as_deref().unwrap_or("-")
    );
    println!(
        "# theme id={} color_scheme={}",
        compiled
            .theme
            .as_ref()
            .map(|theme| theme.id.as_str())
            .unwrap_or("none"),
        compiled
            .theme
            .as_ref()
            .map(|theme| theme.policy.as_str())
            .unwrap_or("-")
    );
    println!("\n# generated roc\n{}", compiled.roc);
    println!("# segments ({})", compiled.segments.len());
    for segment in &compiled.segments {
        let (line, col) = source.line_col(segment.source.start);
        println!(
            "- generated {}..{} <- {name}:{line}:{col} {}",
            segment.generated.start, segment.generated.end, segment.origin
        );
    }
    if compiled.has_errors() {
        bail!("document compilation failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_parses_path_and_output() {
        let cli = Cli::try_parse_from(["rocdown", "build", "docs", "--output", "dist"]).unwrap();
        match cli.command {
            Commands::Build {
                path,
                output,
                cdn_only,
                ..
            } => {
                assert_eq!(path, PathBuf::from("docs"));
                assert_eq!(output, Some(PathBuf::from("dist")));
                assert!(!cdn_only);
            }
            _ => panic!("expected build"),
        }

        let cli = Cli::try_parse_from(["rocdown", "build", "docs", "--cdn-only"]).unwrap();
        match cli.command {
            Commands::Build { cdn_only, .. } => assert!(cdn_only),
            _ => panic!("expected build --cdn-only"),
        }
    }

    #[test]
    fn run_parses_document_and_site() {
        let cli = Cli::try_parse_from([
            "rocdown",
            "run",
            "examples/rocdown/Guide.rocdown",
            "--no-window",
            "--port",
            "8000",
        ])
        .unwrap();
        match cli.command {
            Commands::Run {
                path,
                no_window,
                quiet,
                port,
                ..
            } => {
                assert_eq!(path, PathBuf::from("examples/rocdown/Guide.rocdown"));
                assert!(no_window);
                assert!(!quiet);
                assert_eq!(port, PortArg::Exact(8000));
            }
            _ => panic!("expected run"),
        }

        let cli = Cli::try_parse_from([
            "rocdown",
            "run",
            "examples/errors/parse/Broken.rocdown",
            "--quiet",
        ])
        .unwrap();
        match cli.command {
            Commands::Run { quiet, path, .. } => {
                assert!(quiet);
                assert_eq!(path, PathBuf::from("examples/errors/parse/Broken.rocdown"));
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn serve_islands_parses_root_and_port() {
        let cli = Cli::try_parse_from([
            "rocdown",
            "serve-islands",
            "examples/rocdown-hybrid",
            "--no-window",
            "--port",
            "9001",
        ])
        .unwrap();
        match cli.command {
            Commands::ServeIslands {
                root,
                no_window,
                port,
            } => {
                assert_eq!(root, PathBuf::from("examples/rocdown-hybrid"));
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(9001));
            }
            _ => panic!("expected serve-islands"),
        }
    }

    #[test]
    fn playground_mode_flag_defaults_to_wasm() {
        let cli = Cli::try_parse_from(["rocdown", "playground", "Guide.rocdown"]).unwrap();
        match cli.command {
            Commands::Playground { mode, input, .. } => {
                assert!(matches!(mode, PlaygroundModeArg::Wasm));
                assert_eq!(input, PathBuf::from("Guide.rocdown"));
            }
            _ => panic!("expected playground"),
        }

        let cli =
            Cli::try_parse_from(["rocdown", "playground", "--mode", "local", "Guide.rocdown"])
                .unwrap();
        match cli.command {
            Commands::Playground { mode, .. } => {
                assert!(matches!(mode, PlaygroundModeArg::Local));
            }
            _ => panic!("expected playground"),
        }

        let cli = Cli::try_parse_from(["rocdown", "playground", "Foo.rocci"]).unwrap();
        match cli.command {
            Commands::Playground { input, .. } => {
                assert_eq!(input, PathBuf::from("Foo.rocci"));
            }
            _ => panic!("expected playground"),
        }
    }

    #[test]
    fn check_and_test_parse() {
        let cli = Cli::try_parse_from(["rocdown", "check", "docs", "--format", "json"]).unwrap();
        match cli.command {
            Commands::Check { root, format } => {
                assert_eq!(root, PathBuf::from("docs"));
                assert!(matches!(format, CheckFormatArg::Json));
            }
            _ => panic!("expected check"),
        }

        let cli = Cli::try_parse_from(["rocdown", "test", "docs", "--update"]).unwrap();
        match cli.command {
            Commands::Test { root, update, .. } => {
                assert_eq!(root, PathBuf::from("docs"));
                assert!(update);
            }
            _ => panic!("expected test"),
        }
    }

    #[test]
    fn inspect_ast_and_roc_parse() {
        let cli =
            Cli::try_parse_from(["rocdown", "inspect", "ast", "test/AllSyntax.rocdown"]).unwrap();
        match cli.command {
            Commands::Inspect {
                target: InspectTarget::Ast { input, .. },
            } => {
                assert_eq!(input, PathBuf::from("test/AllSyntax.rocdown"));
            }
            _ => panic!("expected inspect ast"),
        }

        let cli =
            Cli::try_parse_from(["rocdown", "inspect", "roc", "test/AllSyntax.rocdown"]).unwrap();
        match cli.command {
            Commands::Inspect {
                target: InspectTarget::Roc { input, .. },
            } => {
                assert_eq!(input, PathBuf::from("test/AllSyntax.rocdown"));
            }
            _ => panic!("expected inspect roc"),
        }
    }

    #[test]
    fn ensure_document_file_rejects_unsupported_extensions() {
        let temp_dir =
            std::env::temp_dir().join(format!("rocdown-test-main-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let rocci_file = temp_dir.join("test.rocci");
        fs::write(&rocci_file, "Hello := []").unwrap();
        let err = ensure_document_file(&rocci_file, "build")
            .unwrap_err()
            .to_string();
        assert!(err.contains("unsupported file extension for `rocdown build`"));
        assert!(err.contains("expected a .rocdown, .md, or .markdown file"));

        let md_file = temp_dir.join("test.md");
        fs::write(&md_file, "# Doc").unwrap();
        assert!(ensure_document_file(&md_file, "build").is_ok());
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn refuse_okf_input_rejects_knowledge_records_and_bundles() {
        let temp_dir =
            std::env::temp_dir().join(format!("rocdown-test-okf-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let concept = temp_dir.join("plan.md");
        fs::write(
            &concept,
            "---\ntype: Implementation Plan\ntitle: Plan\nauthority: exploratory\n---\n\n# Plan\n",
        )
        .unwrap();
        let err = refuse_okf_input(&concept, "run").unwrap_err().to_string();
        assert!(err.contains("rocci-okf run"));

        fs::write(
            temp_dir.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        let err = refuse_okf_input(&temp_dir, "build")
            .unwrap_err()
            .to_string();
        assert!(err.contains("rocci-okf run"));

        let ordinary = temp_dir.join("notes.md");
        fs::write(&ordinary, "# Notes\n").unwrap();
        assert!(refuse_okf_input(&ordinary, "run").is_ok());
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
