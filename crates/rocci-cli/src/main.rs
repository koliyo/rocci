use rocci_cli::{browse, bundle, datastar_asset, render_file, run, serve, style, view};

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rocci_core::Config;
use rocci_template::{LowerOptions, SourceFile, compile, format_ast, format_diagnostic};

#[derive(Parser)]
#[command(name = "rocci", about = "Rocci desktop runtime and template tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a rocci.toml configuration file.
    Validate {
        #[arg(default_value = "rocci.toml")]
        config: PathBuf,
    },
    /// Build and package an unsigned (ad-hoc signed) development app bundle.
    Bundle {
        #[arg(long, default_value = "rocci.toml")]
        config: PathBuf,
    },
    /// Build a .rocci template to ordinary Roc.
    Build {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compile sibling .rocci modules and run a Roc app, or run a standalone .rocci file.
    Run {
        #[command(flatten)]
        serve: serve::ServeOptions,
        /// Roc app file, directory, or standalone .rocci file
        #[arg(default_value = "main.roc")]
        file: PathBuf,
        /// Extra arguments forwarded to `roc` after `--`.
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Show generated Roc, components, source-map segments, and optional AST.
    Inspect {
        input: PathBuf,
        /// Also print the parse tree as an S-expression.
        #[arg(long)]
        ast: bool,
    },
    /// Print a .rocci parse tree as a LISPy S-expression.
    Ast { input: PathBuf },
    /// Snapshot a .rocci component to HTML via Html.render.
    Render {
        input: PathBuf,
        /// Emit the component fragment without wrapping `<html><body>`.
        #[arg(long)]
        fragment: bool,
        /// Write HTML to this path instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Render a component in a preview window.
    View {
        input: PathBuf,
        #[arg(long, default_value = "main")]
        component: String,
        /// Component parameter as name=value (repeatable).
        #[arg(long = "arg", value_name = "NAME=VALUE", action = clap::ArgAction::Append)]
        args: Vec<String>,
        #[command(flatten)]
        serve: serve::ServeOptions,
    },
    /// Browse components under one or more roots.
    Browse {
        #[command(flatten)]
        serve: serve::ServeOptions,
        /// Directories (recursive) and/or .rocci files.
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
    },
    /// Open a playground to live edit a `.rocci` template or `.rocdown` document.
    Playground {
        #[command(flatten)]
        serve: serve::ServeOptions,
        /// Compiler host: `wasm` runs in the browser worker; `local` compiles natively and can snapshot HTML.
        #[arg(long, value_enum, default_value_t = PlaygroundModeArg::Wasm)]
        mode: PlaygroundModeArg,
        /// Source file to open (`.rocci`, `.rocdown`, `.md`, or `.markdown`).
        input: PathBuf,
    },
    /// Manage the Datastar JS runtime for an app.
    Datastar {
        #[command(subcommand)]
        command: DatastarCmd,
    },
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

#[derive(Subcommand)]
enum DatastarCmd {
    /// Upgrade this app to the latest Datastar release.
    Update {
        /// App directory, rocci.toml, or main.roc
        #[arg(long, default_value = ".")]
        app: PathBuf,
    },
    /// Pin this app to an exact Datastar version.
    Pin {
        version: String,
        /// App directory, rocci.toml, or main.roc
        #[arg(long, default_value = ".")]
        app: PathBuf,
    },
}

fn main() {
    if let Err(err) = try_main() {
        style::print_anyhow(&err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    if env::args().len() <= 1
        && let Some(resources) = bundle::bundled_resources()
    {
        return run::run_bundled(&resources);
    }

    match Cli::parse().command {
        Commands::Validate { config } => validate(&config),
        Commands::Bundle { config } => bundle::bundle(&config),
        Commands::Build { input, output } => build_module(&input, output.as_deref()),
        Commands::Run { file, args, serve } => run::run(&file, &args, serve.no_window, serve.port),
        Commands::Inspect { input, ast } => inspect_module(&input, ast),
        Commands::Ast { input } => ast_module(&input),
        Commands::Render {
            input,
            fragment,
            output,
        } => render_file(&input, fragment, output.as_deref()),
        Commands::View {
            input,
            component,
            args,
            serve,
        } => view::view(&input, &component, &args, serve.no_window, serve.port),
        Commands::Browse { roots, serve } => browse::browse(&roots, serve.no_window, serve.port),
        Commands::Playground { input, serve, mode } => {
            let hook = match mode {
                PlaygroundModeArg::Local => {
                    let src_dir = input
                        .parent()
                        .filter(|parent| !parent.as_os_str().is_empty())
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| env::current_dir().expect("current directory"));
                    Some(rocci_cli::playground::rocci_local_compile_hook(src_dir))
                }
                PlaygroundModeArg::Wasm => None,
            };
            rocci_cli::playground::run_playground_cli(&input, serve, "rocci", mode.into(), hook)
        }
        Commands::Datastar { command } => match command {
            DatastarCmd::Update { app } => datastar_asset::update_app(&app),
            DatastarCmd::Pin { version, app } => datastar_asset::pin_app(&app, &version),
        },
    }
}

fn ensure_rocci_file(input: &Path, command: &str) -> Result<()> {
    if !input.is_file() {
        bail!("no such file: {}", input.display());
    }
    if input.extension().and_then(|ext| ext.to_str()) != Some("rocci") {
        bail!(
            "unsupported file extension for `rocci {command}`: {}; expected a .rocci file",
            input.display()
        );
    }
    Ok(())
}

fn build_module(input: &Path, output: Option<&Path>) -> Result<()> {
    ensure_rocci_file(input, "build")?;
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let compiled = compile(source, &LowerOptions::default());
    let failed = compiled.has_errors();
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    if failed {
        bail!("template compilation failed");
    }
    match output {
        Some(path) => fs::write(path, &compiled.roc)
            .with_context(|| format!("failed to write {}", path.display()))?,
        None => print!("{}", compiled.roc),
    }
    Ok(())
}

fn ast_module(input: &Path) -> Result<()> {
    ensure_rocci_file(input, "ast")?;
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    print!("{}", format_ast(&src, &compiled.document));
    if compiled.has_errors() {
        bail!("template compilation failed");
    }
    Ok(())
}

fn inspect_module(input: &Path, ast: bool) -> Result<()> {
    ensure_rocci_file(input, "inspect")?;
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    inspect_rocci(input, ast, &src, source)
}

fn inspect_rocci(input: &Path, ast: bool, src: &str, source: SourceFile<'_>) -> Result<()> {
    let name = input.display().to_string();
    let compiled = compile(source, &LowerOptions::default());
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
    if ast {
        println!("\n# ast\n{}", format_ast(src, &compiled.document));
    }
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
        bail!("template compilation failed");
    }
    Ok(())
}

fn validate(path: &Path) -> Result<()> {
    let config = Config::from_file(path)?;
    println!(
        "{}",
        style::ok(&format!(
            "{} ({} window{})",
            config.app.identifier,
            config.windows.len(),
            if config.windows.len() == 1 { "" } else { "s" }
        ))
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn port_of(cli: &Cli) -> serve::PortArg {
        match &cli.command {
            Commands::Run { serve, .. }
            | Commands::View { serve, .. }
            | Commands::Browse { serve, .. } => serve.port,
            _ => panic!("expected a hosting command"),
        }
    }

    fn no_window_of(cli: &Cli) -> bool {
        match &cli.command {
            Commands::Run { serve, .. }
            | Commands::View { serve, .. }
            | Commands::Browse { serve, .. } => serve.no_window,
            _ => panic!("expected a hosting command"),
        }
    }

    #[test]
    fn hosting_commands_accept_port_auto() {
        for args in [
            ["rocci", "run", "--port", "auto"].as_slice(),
            ["rocci", "view", "Foo.rocci", "--port", "auto"].as_slice(),
            ["rocci", "browse", "--port", "auto"].as_slice(),
        ] {
            assert_eq!(
                port_of(&Cli::try_parse_from(args).unwrap()),
                serve::PortArg::Auto
            );
        }
    }

    #[test]
    fn hosting_commands_accept_numeric_port() {
        for args in [
            ["rocci", "run", "--port", "9001"].as_slice(),
            ["rocci", "view", "Foo.rocci", "--port", "9001"].as_slice(),
            ["rocci", "browse", "--port", "9001"].as_slice(),
        ] {
            assert_eq!(
                port_of(&Cli::try_parse_from(args).unwrap()),
                serve::PortArg::Exact(9001)
            );
        }
    }

    #[test]
    fn run_accepts_port_after_app_path() {
        let cli = Cli::try_parse_from([
            "rocci",
            "run",
            "examples/counter/Counter.rocci",
            "--port",
            "auto",
        ])
        .unwrap();
        assert_eq!(port_of(&cli), serve::PortArg::Auto);
    }

    #[test]
    fn windowed_hosting_defaults_to_auto_port() {
        if std::env::var_os("ROC_BASIC_WEBSERVER_PORT").is_some() {
            return;
        }
        for args in [
            ["rocci", "run"].as_slice(),
            ["rocci", "view", "Foo.rocci"].as_slice(),
            ["rocci", "browse"].as_slice(),
        ] {
            let cli = Cli::try_parse_from(args).unwrap();
            assert!(!no_window_of(&cli));
            assert_eq!(port_of(&cli), serve::PortArg::Auto);
        }
    }

    #[test]
    fn no_window_defaults_to_port_8000() {
        if std::env::var_os("ROC_BASIC_WEBSERVER_PORT").is_some() {
            return;
        }
        for args in [
            ["rocci", "run", "--no-window"].as_slice(),
            ["rocci", "view", "Foo.rocci", "--no-window"].as_slice(),
            ["rocci", "browse", "--no-window"].as_slice(),
        ] {
            let cli = Cli::try_parse_from(args).unwrap();
            assert!(no_window_of(&cli));
            assert_eq!(port_of(&cli), serve::PortArg::Exact(8000));
        }
    }

    #[test]
    fn no_window_still_accepts_explicit_port() {
        let cli = Cli::try_parse_from(["rocci", "run", "--no-window", "--port", "auto"]).unwrap();
        assert_eq!(port_of(&cli), serve::PortArg::Auto);
    }

    #[test]
    fn render_parses_fragment_and_output() {
        let cli = Cli::try_parse_from([
            "rocci",
            "render",
            "Card.rocci",
            "--fragment",
            "-o",
            "out.html",
        ])
        .unwrap();
        match cli.command {
            Commands::Render {
                input,
                fragment,
                output,
            } => {
                assert_eq!(input, PathBuf::from("Card.rocci"));
                assert!(fragment);
                assert_eq!(output.as_deref(), Some(Path::new("out.html")));
            }
            _ => panic!("expected render"),
        }
    }

    #[test]
    fn playground_mode_flag_defaults_to_wasm() {
        let cli = Cli::try_parse_from(["rocci", "playground", "Foo.rocci"]).unwrap();
        match cli.command {
            Commands::Playground { mode, input, .. } => {
                assert!(matches!(mode, PlaygroundModeArg::Wasm));
                assert_eq!(input, PathBuf::from("Foo.rocci"));
            }
            _ => panic!("expected playground"),
        }

        let cli = Cli::try_parse_from([
            "rocci",
            "playground",
            "--mode",
            "local",
            "examples/rocdown/Guide.rocdown",
        ])
        .unwrap();
        match cli.command {
            Commands::Playground { mode, input, .. } => {
                assert!(matches!(mode, PlaygroundModeArg::Local));
                assert_eq!(input, PathBuf::from("examples/rocdown/Guide.rocdown"));
            }
            _ => panic!("expected playground"),
        }
    }

    #[test]
    fn datastar_pin_and_update_parse() {
        let cli = Cli::try_parse_from([
            "rocci",
            "datastar",
            "pin",
            "1.0.2",
            "--app",
            "examples/datastar",
        ])
        .unwrap();
        match cli.command {
            Commands::Datastar {
                command: DatastarCmd::Pin { version, app },
            } => {
                assert_eq!(version, "1.0.2");
                assert_eq!(app, PathBuf::from("examples/datastar"));
            }
            _ => panic!("unexpected command"),
        }

        let cli = Cli::try_parse_from(["rocci", "datastar", "update"]).unwrap();
        match cli.command {
            Commands::Datastar {
                command: DatastarCmd::Update { app },
            } => assert_eq!(app, PathBuf::from(".")),
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn ensure_rocci_file_rejects_unsupported_extensions() {
        let temp_dir = std::env::temp_dir().join(format!("rocci-test-main-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let md_file = temp_dir.join("test.md");
        fs::write(&md_file, "# Hello").unwrap();
        let err = ensure_rocci_file(&md_file, "build")
            .unwrap_err()
            .to_string();
        assert!(err.contains("unsupported file extension for `rocci build`"));
        assert!(err.contains("expected a .rocci file"));

        let rocci_file = temp_dir.join("test.rocci");
        fs::write(&rocci_file, "Hello := []").unwrap();
        assert!(ensure_rocci_file(&rocci_file, "build").is_ok());
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
