use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rocci_cli::driver::{DriverOptions, GenericAppPlan, GenericModule};
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
        #[command(flatten)]
        theme: ThemeArgs,
    },
    /// Run an interactive document or serve a documentation site with live reload.
    Run {
        /// Site root directory or single .rocdown file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Write preview output here instead of a temp directory (for site dev server).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Skip the embedded window; print the URL and keep serving.
        #[arg(long)]
        no_window: bool,
        /// TCP port to listen on. Defaults to a free port with the embedded window,
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
    /// Validate the documentation catalog or document without writing output.
    Check {
        #[arg(default_value = ".")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Run declared `@docs example` commands. Never part of `rocdown build`.
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
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFormatArg {
    Terminal,
    Json,
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

fn try_main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build {
            path,
            output,
            theme,
        } => {
            if is_document_file(&path) {
                build_single_doc(&path, output.as_deref(), &theme)
            } else {
                rocci_rocdown::build_configured(&path, output.as_deref())?;
                Ok(())
            }
        }
        Commands::Run {
            path,
            output,
            no_window,
            port,
            theme,
            args,
        } => {
            if is_document_file(&path) {
                run_standalone_doc(&path, &args, no_window, port, &theme)
            } else {
                run_site_dev(&path, output.as_deref(), no_window, port)
            }
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
    }
}

fn build_single_doc(input: &Path, output: Option<&Path>, theme: &ThemeArgs) -> Result<()> {
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
            return rocci_cli::driver::serve_template_errors(
                &failed_files,
                port,
                no_window,
                &title,
            );
        }
        StandaloneReady::Ready(plan) => plan,
    };

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
    };

    rocci_cli::driver::execute_app_plan(&generic_plan, &src_dir, &driver_options)
}

fn run_site_dev(root: &Path, output: Option<&Path>, no_window: bool, port: PortArg) -> Result<()> {
    let port = port.resolve()?;
    let server = rocci_rocdown::run(root, output, port)?;
    eprintln!("rocdown: serving {} at {}", server.title, server.url);
    if no_window {
        server.wait();
        return Ok(());
    }
    let result = rocci_desktop::preview(rocci_desktop::PreviewOptions {
        url: server.url.clone(),
        title: server.title.clone(),
        state_key: Some("rocdown".to_string()),
        ..rocci_desktop::PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

fn inspect_ast(input: &Path, theme: &ThemeArgs) -> Result<()> {
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
            Commands::Build { path, output, .. } => {
                assert_eq!(path, PathBuf::from("docs"));
                assert_eq!(output, Some(PathBuf::from("dist")));
            }
            _ => panic!("expected build"),
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
                port,
                ..
            } => {
                assert_eq!(path, PathBuf::from("examples/rocdown/Guide.rocdown"));
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8000));
            }
            _ => panic!("expected run"),
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
}
