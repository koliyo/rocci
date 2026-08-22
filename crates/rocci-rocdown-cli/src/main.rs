use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use rocci_cli::dev_server::{StaticDevServerConfig, preview_static_site};
use rocci_cli::driver::{DriverOptions, GenericAppPlan, GenericModule};
use rocci_cli::path_hint;
use rocci_cli::serve::{PortArg, parse_port_arg};
use rocci_rocdown::{
    PageKind, SourceFile, StandaloneReady, ThemeArgs, document_page_kind, format_diagnostic, parse,
    write_static_document_preview,
};

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
        /// Roc `roc build --target=` for island/app process binaries (see possible values).
        /// Match the Linux container CPU (`arm64musl` on Apple Silicon Docker).
        /// Not passed to `--host native` apply on the build machine.
        #[arg(long, value_enum)]
        target: Option<rocci_cli::native_target::NativeTarget>,
        #[command(flatten)]
        theme: ThemeArgs,
    },
    /// Package a site for hosting: static CDN tree, or hybrid CDN plus island binary.
    Package {
        /// Site root directory.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Override output path.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Execution host runtime for evaluating templates (native, wasm, auto [default]).
        #[arg(long, value_enum, default_value_t = HostArg::Auto)]
        host: HostArg,
        /// Gzip tarball path, relative to the output parent unless absolute.
        #[arg(long, default_value = "site.tgz")]
        archive: PathBuf,
        /// Write `dist/` and `publish.json` without a tarball.
        #[arg(long)]
        no_archive: bool,
        /// Error if the site has `live` pages (static CDN package only).
        #[arg(long)]
        cdn_only: bool,
        /// Roc `roc build --target=` for island process binaries (see possible values).
        /// Match the Linux container CPU (`arm64musl` on Apple Silicon Docker).
        /// Not passed to `--host native` apply on the build machine.
        #[arg(long, value_enum)]
        target: Option<rocci_cli::native_target::NativeTarget>,
    },
    /// Preview a document or documentation site with live reload.
    /// Hybrid sites proxy the island service on the same origin.
    View {
        #[command(flatten)]
        preview: PreviewArgs,
    },
    /// Deprecated alias for `view`.
    Run {
        #[command(flatten)]
        preview: PreviewArgs,
    },
    /// Serve a previously built site tree without rebuilding.
    Serve {
        /// Built `dist/` directory (must contain `index.html`).
        #[arg(default_value = ".")]
        dist: PathBuf,
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
    /// Start the island HTTP service for live pages in a documentation site.
    ServeIslands {
        /// Site root directory.
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Skip the preview window; print the URL and keep serving.
        #[arg(long)]
        no_window: bool,
        /// Log each matched `@view` / `@patch` / `@command` / `@live` handler to stderr (CLI and Dev Console).
        #[arg(long)]
        log_handlers: bool,
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

#[derive(Args, Debug)]
struct PreviewArgs {
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
    /// Open that URL with `?reload=0` to pause automatic page refresh.
    #[arg(long)]
    no_window: bool,
    /// Pause automatic page refresh. Watch and rebuild still run.
    #[arg(long)]
    no_live_reload: bool,
    /// Do not print compile diagnostics on stderr; the error page still serves.
    #[arg(long)]
    quiet: bool,
    /// Log each matched `@view` / `@patch` / `@command` / `@live` handler to stderr (CLI and Dev Console).
    #[arg(long)]
    log_handlers: bool,
    /// Print compile, inspect, and wait phases to stderr.
    #[arg(short, long)]
    verbose: bool,
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

fn warn_deprecated_run_alias(bin: &str) {
    eprintln!(
        "{bin}: `run` is a deprecated alias for `view` and will be removed in a later release"
    );
}

fn preview_docs(preview: PreviewArgs) -> Result<()> {
    let PreviewArgs {
        path,
        output,
        host,
        no_window,
        no_live_reload,
        quiet,
        log_handlers,
        verbose,
        port,
        theme,
        args,
    } = preview;
    let live_reload = !no_live_reload;
    if is_document_file(&path) {
        refuse_okf_input(&path, "view")?;
        if let Some(site_root) = rocci_rocdown::find_site_root(&path) {
            let route = rocci_rocdown::site_preview_route(&site_root, &path);
            run_site_dev(
                &site_root,
                output.as_deref(),
                no_window,
                live_reload,
                log_handlers,
                verbose,
                port,
                host.into(),
                Some(&route),
            )
        } else {
            run_standalone_doc(
                &path,
                &args,
                no_window,
                live_reload,
                log_handlers,
                quiet,
                verbose,
                port,
                &theme,
            )
        }
    } else if path.is_file() {
        bail!(
            "unsupported file extension for `rocdown view`: {}; expected a .rocdown, .md, or .markdown file",
            path.display()
        )
    } else {
        refuse_okf_input(&path, "view")?;
        run_site_dev(
            &path,
            output.as_deref(),
            no_window,
            live_reload,
            log_handlers,
            verbose,
            port,
            host.into(),
            None,
        )
    }
}

fn refuse_okf_input(path: &Path, command_name: &str) -> Result<()> {
    if path.is_dir() {
        if path_hint::looks_like_okf_bundle(path) {
            bail!(
                "`rocdown {command_name}` does not preview OKF knowledge bundles; preview with `rocci-okf view {}`",
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
            "`rocdown {command_name}` does not render OKF knowledge records; preview {} with `rocci-okf view {}`",
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
            target,
            theme,
        } => {
            if is_document_file(&path) {
                refuse_okf_input(&path, "build")?;
                if cdn_only {
                    bail!("`--cdn-only` applies to site builds, not a single .rocdown file");
                }
                if target.is_some() {
                    bail!(
                        "`--target` applies to island/app process binaries, not a single .rocdown file"
                    );
                }
                build_single_doc(&path, output.as_deref(), &theme)
            } else if path.is_file() {
                bail!(
                    "unsupported file extension for `rocdown build`: {}; expected a .rocdown, .md, or .markdown file",
                    path.display()
                );
            } else {
                refuse_okf_input(&path, "build")?;
                let _native_target = target;
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
        Commands::Package {
            path,
            output,
            host,
            archive,
            no_archive,
            cdn_only,
            target,
        } => {
            if is_document_file(&path) {
                bail!("`rocdown package` builds a site directory, not a single .rocdown file");
            }
            refuse_okf_input(&path, "package")?;
            let report = rocci_rocdown::package_configured(
                &path,
                output.as_deref(),
                rocci_rocdown::PackageOptions {
                    host: Some(host.into()),
                    archive: Some(archive),
                    write_archive: !no_archive,
                    cdn_only,
                    native_target: target,
                },
            )?;
            print!("{}", report.render());
            Ok(())
        }
        Commands::View { preview } => preview_docs(preview),
        Commands::Run { preview } => {
            warn_deprecated_run_alias("rocdown");
            preview_docs(preview)
        }
        Commands::Serve {
            dist,
            no_window,
            port,
        } => {
            rocci_rocdown::ensure_built_tree(&dist)?;
            let port = port.resolve()?;
            let title = dist
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("rocdown")
                .to_string();
            rocci_cli::dev_server::preview_published_tree(
                rocci_cli::dev_server::PublishedTreeConfig {
                    title,
                    port,
                    dist,
                    open_path: "/".to_string(),
                    log_prefix: "rocdown".to_string(),
                },
                no_window,
                Some("rocdown".to_string()),
            )
        }
        Commands::ServeIslands {
            root,
            no_window,
            log_handlers,
            port,
        } => {
            refuse_okf_input(&root, "serve-islands")?;
            rocci_rocdown::serve_islands(&root, no_window, port, log_handlers)
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
            let serve = rocci_cli::serve::ServeOptions {
                no_window,
                no_live_reload: false,
                log_handlers: false,
                verbose: false,
                port,
            };
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

#[allow(clippy::too_many_arguments)]
fn run_standalone_doc(
    file: &Path,
    args: &[String],
    no_window: bool,
    live_reload: bool,
    log_handlers: bool,
    quiet: bool,
    verbose: bool,
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
    let progress = rocci_cli::logs::Progress { verbose, quiet };
    let src =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let source_name = path.display().to_string();
    let source = SourceFile::new(&source_name, &src);
    progress.step(format!("rocdown: parsing {}", path.display()));
    let parsed = parse(source, compile_opts.raw_html);
    if parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error())
    {
        let failed = vec![rocci_cli::error_page::FailedFile {
            name: source_name,
            src,
            diagnostics: parsed.diagnostics,
        }];
        if !quiet {
            rocci_cli::error_page::eprint_template_errors(&failed);
        }
        return rocci_cli::driver::serve_template_errors(
            &failed,
            port,
            no_window,
            live_reload,
            &title,
        );
    }
    let kind = document_page_kind(&parsed.document);
    progress.detail(format!("rocdown: {} is {}", path.display(), kind.as_str()));
    if kind == PageKind::Static {
        progress.step("rocdown: static document, rendering HTML without roc");
        return run_static_standalone_preview(
            &path,
            &src_dir,
            &title,
            no_window,
            live_reload,
            quiet,
            verbose,
            port,
            &compile_opts,
        );
    }

    let parse_started = Instant::now();
    let plan =
        match rocci_rocdown::plan_standalone_with_progress(&path, &compile_opts.theme, progress)? {
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
                    live_reload,
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
                lives: module.lives,
                routes: module.routes,
                mapped: module.mapped,
                local_assets: module.local_assets,
            })
            .collect(),
        redirect_trailing_slash: plan.redirect_trailing_slash,
        log_handlers,
    };

    let driver_options = DriverOptions {
        args: args.to_vec(),
        no_window,
        live_reload,
        log_handlers,
        verbose,
        port,
        db_path: None,
        title,
        preview_path: None,
        profile: profile.finish(),
        inspect_pages: plan.inspect_pages,
        state_key: Some("rocdown".to_string()),
    };

    rocci_cli::driver::execute_app_plan(&generic_plan, &src_dir, &driver_options)
}

#[allow(clippy::too_many_arguments)]
fn run_static_standalone_preview(
    path: &Path,
    src_dir: &Path,
    title: &str,
    no_window: bool,
    live_reload: bool,
    quiet: bool,
    verbose: bool,
    port: PortArg,
    options: &rocci_rocdown::CompileOptions,
) -> Result<()> {
    let port = port.resolve()?;
    let progress = rocci_cli::logs::Progress { verbose, quiet };
    let watch_file = path.to_path_buf();
    let assets = src_dir.join("assets");
    let custom_filter = Arc::new(move |candidate: &Path| {
        candidate == watch_file.as_path()
            || candidate.starts_with(&assets)
            || candidate
                .extension()
                .is_some_and(|ext| ext == "rocdown" || ext == "md" || ext == "markdown")
    });
    let input = path.to_path_buf();
    let compile_opts = options.clone();
    preview_static_site(
        StaticDevServerConfig {
            title: title.to_string(),
            port,
            open_path: "/".to_string(),
            output: None,
            watch_paths: vec![src_dir.to_path_buf()],
            custom_filter: Some(custom_filter),
            log_prefix: "rocdown".into(),
            backend_port: None,
            log_handlers: false,
            on_stop: None,
        },
        no_window,
        live_reload,
        Some("rocdown".to_string()),
        move |out_dir, _logs| {
            progress.step(format!("rocdown: rendering {}", input.display()));
            write_static_document_preview(&input, out_dir, &compile_opts)?;
            Ok(None)
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn run_site_dev(
    root: &Path,
    output: Option<&Path>,
    no_window: bool,
    live_reload: bool,
    log_handlers: bool,
    verbose: bool,
    port: PortArg,
    host: rocci_rocdown::HostChoice,
    open_path: Option<&str>,
) -> Result<()> {
    let port = port.resolve()?;
    let open_path = open_path.unwrap_or("/");
    let server = rocci_rocdown::run_with_host_at(
        root,
        output,
        port,
        Some(host),
        open_path,
        log_handlers,
        verbose,
    )?;
    rocci_cli::logs::tee(
        &server.logs,
        rocci_cli::logs::LogLevel::Info,
        format!("rocdown: serving {} at {}", server.title, server.url),
    );
    rocci_cli::serve::note_live_reload_paused(live_reload);
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
        live_reload,
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
    fn build_parses_host_and_process_target() {
        let cli = Cli::try_parse_from([
            "rocdown", "build", "docs", "--host", "wasm", "--target", "x64musl",
        ])
        .unwrap();
        match cli.command {
            Commands::Build { host, target, .. } => {
                assert!(matches!(host, HostArg::Wasm));
                assert_eq!(
                    target,
                    Some(rocci_cli::native_target::NativeTarget::X64Musl)
                );
            }
            _ => panic!("expected build --host wasm --target x64musl"),
        }
    }

    #[test]
    fn package_parses_root_output_and_archive() {
        let cli = Cli::try_parse_from([
            "rocdown",
            "package",
            "docs",
            "--output",
            "dist",
            "--archive",
            "site.tgz",
        ])
        .unwrap();
        match cli.command {
            Commands::Package {
                path,
                output,
                archive,
                no_archive,
                ..
            } => {
                assert_eq!(path, PathBuf::from("docs"));
                assert_eq!(output, Some(PathBuf::from("dist")));
                assert_eq!(archive, PathBuf::from("site.tgz"));
                assert!(!no_archive);
            }
            _ => panic!("expected package"),
        }

        let cli = Cli::try_parse_from(["rocdown", "package", "docs", "--no-archive"]).unwrap();
        match cli.command {
            Commands::Package { no_archive, .. } => assert!(no_archive),
            _ => panic!("expected package --no-archive"),
        }

        let cli = Cli::try_parse_from([
            "rocdown",
            "package",
            "examples/rocdown/counter",
            "--target",
            "x64musl",
        ])
        .unwrap();
        match cli.command {
            Commands::Package {
                cdn_only, target, ..
            } => {
                assert!(!cdn_only);
                assert_eq!(
                    target,
                    Some(rocci_cli::native_target::NativeTarget::X64Musl)
                );
            }
            _ => panic!("expected package --target"),
        }
    }

    #[test]
    fn serve_parses_dist_and_port() {
        let cli = Cli::try_parse_from([
            "rocdown",
            "serve",
            "dist/docs",
            "--no-window",
            "--port",
            "8080",
        ])
        .unwrap();
        match cli.command {
            Commands::Serve {
                dist,
                no_window,
                port,
            } => {
                assert_eq!(dist, PathBuf::from("dist/docs"));
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8080));
            }
            _ => panic!("expected serve"),
        }
    }

    fn preview_args(command: Commands) -> PreviewArgs {
        match command {
            Commands::View { preview } | Commands::Run { preview } => preview,
            _ => panic!("expected view or run"),
        }
    }

    #[test]
    fn view_parses_document_and_site() {
        let cli = Cli::try_parse_from([
            "rocdown",
            "view",
            "examples/rocdown/pages/Guide.rocdown",
            "--no-window",
            "--port",
            "8000",
        ])
        .unwrap();
        let preview = preview_args(cli.command);
        assert_eq!(
            preview.path,
            PathBuf::from("examples/rocdown/pages/Guide.rocdown")
        );
        assert!(preview.no_window);
        assert!(!preview.quiet);
        assert_eq!(preview.port, PortArg::Exact(8000));
        assert!(!preview.no_live_reload);
        assert_eq!(preview.output, None);

        let verbose = Cli::try_parse_from(["rocdown", "view", "docs", "-v"]).unwrap();
        assert!(preview_args(verbose.command).verbose);
        let long = Cli::try_parse_from(["rocdown", "view", "docs", "--verbose"]).unwrap();
        assert!(preview_args(long.command).verbose);

        let kept = Cli::try_parse_from([
            "rocdown",
            "view",
            "examples/rocdown/counter",
            "--no-window",
            "--output",
            "/tmp/rocdown-counter-preview",
        ])
        .unwrap();
        assert_eq!(
            preview_args(kept.command).output,
            Some(PathBuf::from("/tmp/rocdown-counter-preview"))
        );

        let paused = Cli::try_parse_from(["rocdown", "view", "docs", "--no-live-reload"]).unwrap();
        assert!(preview_args(paused.command).no_live_reload);

        let cli = Cli::try_parse_from([
            "rocdown",
            "view",
            "examples/rocdown/errors/parse/Broken.rocdown",
            "--quiet",
        ])
        .unwrap();
        let preview = preview_args(cli.command);
        assert!(preview.quiet);
        assert_eq!(
            preview.path,
            PathBuf::from("examples/rocdown/errors/parse/Broken.rocdown")
        );
    }

    #[test]
    fn run_remains_a_deprecated_alias_for_view() {
        let cli =
            Cli::try_parse_from(["rocdown", "run", "docs", "--no-window", "--verbose"]).unwrap();
        match cli.command {
            Commands::Run { preview } => {
                assert_eq!(preview.path, PathBuf::from("docs"));
                assert!(preview.no_window);
                assert!(preview.verbose);
            }
            _ => panic!("expected run alias"),
        }
    }

    #[test]
    fn serve_islands_parses_root_and_port() {
        let cli = Cli::try_parse_from([
            "rocdown",
            "serve-islands",
            "examples/rocdown/hybrid",
            "--no-window",
            "--port",
            "9001",
        ])
        .unwrap();
        match cli.command {
            Commands::ServeIslands {
                root,
                no_window,
                log_handlers,
                port,
            } => {
                assert_eq!(root, PathBuf::from("examples/rocdown/hybrid"));
                assert!(no_window);
                assert!(!log_handlers);
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
        let err = refuse_okf_input(&concept, "view").unwrap_err().to_string();
        assert!(err.contains("rocci-okf view"));

        fs::write(
            temp_dir.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        let err = refuse_okf_input(&temp_dir, "build")
            .unwrap_err()
            .to_string();
        assert!(err.contains("rocci-okf view"));

        let ordinary = temp_dir.join("notes.md");
        fs::write(&ordinary, "# Notes\n").unwrap();
        assert!(refuse_okf_input(&ordinary, "view").is_ok());
        let _ = fs::remove_dir_all(&temp_dir);
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
