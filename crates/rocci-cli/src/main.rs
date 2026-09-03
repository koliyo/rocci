use rocci_cli::{browse, bundle, datastar_asset, render_file, rocci_test, run, serve, style, view};

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rocci_core::Config;
use rocci_template::{
    LowerOptions, SourceFile, compile, format_ast, format_diagnostic, inspect_handlers,
};

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
        /// Roc `roc build --target=` for the process binary (see possible values).
        /// Match the Linux container CPU (`arm64musl` on Apple Silicon Docker).
        /// Apply `--host` is separate. macOS `.app` bundles stay host-native.
        #[arg(long, value_enum)]
        target: Option<rocci_cli::native_target::NativeTarget>,
    },
    /// Build a .rocci template to ordinary Roc, or package a Linux server with `--release`.
    Build {
        /// `.rocci` file (template-to-Roc), or with `--release` an app directory / `.rocci` / `rocci.toml`.
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Package a Roc server binary plus assets (not a macOS `.app`).
        #[arg(long)]
        release: bool,
        /// Roc `roc build --target=` for the process binary (see possible values).
        /// Match the Linux container CPU (`arm64musl` on Apple Silicon Docker).
        /// Requires `--release`. macOS `.app` bundling stays on `rocci bundle`.
        #[arg(long, value_enum)]
        target: Option<rocci_cli::native_target::NativeTarget>,
        /// Stream Roc compiler output and show compiler phase timings.
        #[arg(long)]
        verbose: bool,
        /// Roc backend optimization mode (defaults to Roc's speed backend).
        #[arg(long, value_enum)]
        opt: Option<rocci_cli::native_target::RocOpt>,
        /// Experimental WASI HTTP component compiled from the input `.rocci`
        /// entry (sibling `.rocci` / `.roc` in the standalone tree included;
        /// not `--host wasm` apply). Writes a `wasi:http/service` artifact
        /// for `wasmtime serve`; `rocci run` stays native Rocci platform.
        #[arg(long)]
        http_module: bool,
        /// Pin generated apps to the in-tree Rocci platform (`rocci`).
        /// Default is already that pin; `--http-module` still requires 0.16.0.
        /// Also `ROCCI_PLATFORM=rocci`.
        #[arg(long, value_name = "NAME", env = "ROCCI_PLATFORM")]
        platform: Option<String>,
    },
    /// Compile sibling .rocci modules and run a Roc app, or run a standalone .rocci file.
    Run {
        #[command(flatten)]
        serve: serve::ServeOptions,
        /// Pin generated apps to the in-tree Rocci platform (`rocci`).
        /// Default is already that pin. Also `ROCCI_PLATFORM=rocci`.
        #[arg(long, value_name = "NAME", env = "ROCCI_PLATFORM")]
        platform: Option<String>,
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
    /// Run `@test` declarations with `roc test`.
    Test {
        /// `.rocci` file or directory of `.rocci` files.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
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
        Commands::Bundle { config, target } => bundle::bundle(&config, target),
        Commands::Build {
            input,
            output,
            release,
            target,
            verbose,
            opt,
            http_module,
            platform,
        } => {
            if http_module && platform.is_some() {
                bail!("`--http-module` requires basic-webserver 0.16.0; do not pass `--platform`");
            }
            let platform = rocci_cli::resolve_platform_pin(platform.as_deref())?;
            if http_module {
                if release {
                    bail!("`--http-module` is not a musl/process `--release` package");
                }
                if target.is_some() || opt.is_some() {
                    bail!(
                        "`--http-module` does not take `--target` or `--opt` (those are process binaries; `--host wasm` stays apply)"
                    );
                }
                let _ = verbose;
                ensure_rocci_file(&input, "build")?;
                let dest = output.unwrap_or_else(|| PathBuf::from("http-module.wasm"));
                write_http_module(&input, &dest)?;
                println!("{}", style::success_text(&dest.display().to_string()));
                eprintln!(
                    "{}",
                    style::note(&format!(
                        "WASI 0.3 component; wasmtime serve -Sp3 -Scli --dir={}::/assets {}",
                        rocci_cli::http_module::assets_dir(&dest).display(),
                        dest.display()
                    ))
                );
                Ok(())
            } else if release {
                let report = bundle::package_server_with_opt(
                    &input,
                    output.as_deref(),
                    target,
                    verbose,
                    opt,
                    platform.clone(),
                )?;
                println!(
                    "{}",
                    style::success_text(&report.output.display().to_string())
                );
                Ok(())
            } else if let Some(platform) = platform {
                if target.is_some() || opt.is_some() {
                    bail!(
                        "`--target` and `--opt` require `--release` (Linux server packaging, not template-to-Roc)"
                    );
                }
                build_standalone_with_platform(&input, output.as_deref(), platform, verbose)
            } else {
                if target.is_some() || opt.is_some() {
                    bail!(
                        "`--target` and `--opt` require `--release` (Linux server packaging, not template-to-Roc)"
                    );
                }
                if input.is_dir() && input.join("main.roc").is_file() {
                    let dest = output.clone().unwrap_or_else(|| PathBuf::from("server"));
                    rocci_cli::driver::compile_custom_app_dir(&input, &dest, verbose)?;
                    println!("{}", style::success_text(&dest.display().to_string()));
                    Ok(())
                } else {
                    build_module(&input, output.as_deref())
                }
            }
        }
        Commands::Run {
            file,
            args,
            serve,
            platform,
        } => {
            let platform = rocci_cli::resolve_platform_pin(platform.as_deref())?;
            run::run(
                &file,
                &args,
                serve.no_window,
                serve.port,
                serve.live_reload(),
                serve.log_handlers,
                serve.verbose,
                serve.public,
                platform,
            )
        }
        Commands::Inspect { input, ast } => inspect_module(&input, ast),
        Commands::Ast { input } => ast_module(&input),
        Commands::Test { path } => rocci_test::run(&path),
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
        } => view::view(
            &input,
            &component,
            &args,
            serve.no_window,
            serve.port,
            serve.live_reload(),
            serve.verbose,
            serve.public,
        ),
        Commands::Browse { roots, serve } => browse::browse(
            &roots,
            serve.no_window,
            serve.port,
            serve.live_reload(),
            serve.verbose,
            serve.public,
        ),
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

fn write_http_module(input: &Path, dest: &Path) -> Result<()> {
    rocci_cli::http_module::build_http_module(input, dest)
}

fn build_standalone_with_platform(
    input: &Path,
    output: Option<&Path>,
    platform: String,
    verbose: bool,
) -> Result<()> {
    let dest = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("server"));
    rocci_cli::driver::compile_standalone_input(input, &dest, Some(platform), verbose)?;
    println!("{}", style::success_text(&dest.display().to_string()));
    Ok(())
}

#[cfg(test)]
fn is_wasi_http_component(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes.starts_with(b"\0asm") && bytes[4] == 0x0d
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
    println!("# tests ({})", compiled.tests.len());
    for test in &compiled.tests {
        match &test.fixture {
            Some(fixture) => println!("- {} fixture:{}", test.name, fixture),
            None => println!("- {}", test.name),
        }
    }
    let handlers = inspect_handlers(&compiled.document);
    println!("# handlers ({})", handlers.len());
    for handler in &handlers {
        println!("- {}", handler.line());
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
    use clap::{CommandFactory, Parser};

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

    fn no_live_reload_of(cli: &Cli) -> bool {
        match &cli.command {
            Commands::Run { serve, .. }
            | Commands::View { serve, .. }
            | Commands::Browse { serve, .. } => serve.no_live_reload,
            _ => panic!("expected a hosting command"),
        }
    }

    #[test]
    fn bundle_parses_target() {
        let cli = Cli::try_parse_from([
            "rocci",
            "bundle",
            "--config",
            "rocci.toml",
            "--target",
            "x64musl",
        ])
        .unwrap();
        match cli.command {
            Commands::Bundle { target, .. } => {
                assert_eq!(
                    target,
                    Some(rocci_cli::native_target::NativeTarget::X64Musl)
                );
            }
            _ => panic!("expected bundle --target"),
        }
    }

    #[test]
    fn build_release_parses_target_and_keeps_template_mode() {
        let cli = Cli::try_parse_from([
            "rocci",
            "build",
            "--release",
            "examples/rocci/custom/datastar",
            "--target",
            "x64musl",
            "--verbose",
            "--opt",
            "dev",
            "-o",
            "target/release/rocci-server",
        ])
        .unwrap();
        match cli.command {
            Commands::Build {
                input,
                output,
                release,
                target,
                verbose,
                opt,
                http_module,
                platform,
            } => {
                assert!(!http_module);
                assert!(platform.is_none());
                assert_eq!(input, PathBuf::from("examples/rocci/custom/datastar"));
                assert_eq!(output, Some(PathBuf::from("target/release/rocci-server")));
                assert!(release);
                assert_eq!(
                    target,
                    Some(rocci_cli::native_target::NativeTarget::X64Musl)
                );
                assert!(verbose);
                assert_eq!(opt, Some(rocci_cli::native_target::RocOpt::Dev));
            }
            _ => panic!("expected build --release"),
        }

        let cli = Cli::try_parse_from([
            "rocci",
            "build",
            "examples/rocci/standalone/counter/Counter.rocci",
        ])
        .unwrap();
        match cli.command {
            Commands::Build {
                release, target, ..
            } => {
                assert!(!release);
                assert!(target.is_none());
            }
            _ => panic!("expected build without --release"),
        }
    }

    #[test]
    fn build_and_run_parse_platform_rocci() {
        let cli = Cli::try_parse_from([
            "rocci",
            "build",
            "examples/rocci/standalone/counter",
            "--platform",
            "rocci",
        ])
        .unwrap();
        match cli.command {
            Commands::Build { platform, .. } => {
                assert_eq!(platform.as_deref(), Some("rocci"));
            }
            _ => panic!("expected build --platform rocci"),
        }

        let cli =
            Cli::try_parse_from(["rocci", "run", "--platform", "rocci", "Counter.rocci"]).unwrap();
        match cli.command {
            Commands::Run { platform, .. } => {
                assert_eq!(platform.as_deref(), Some("rocci"));
            }
            _ => panic!("expected run --platform rocci"),
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
            "examples/rocci/standalone/counter/Counter.rocci",
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
    fn hosting_commands_accept_no_live_reload() {
        for args in [
            ["rocci", "run", "--no-live-reload"].as_slice(),
            ["rocci", "view", "Foo.rocci", "--no-live-reload"].as_slice(),
            ["rocci", "browse", "--no-live-reload"].as_slice(),
        ] {
            assert!(no_live_reload_of(&Cli::try_parse_from(args).unwrap()));
        }
        assert!(!no_live_reload_of(
            &Cli::try_parse_from(["rocci", "run"]).unwrap()
        ));
    }

    fn verbose_of(cli: &Cli) -> bool {
        match &cli.command {
            Commands::Run { serve, .. }
            | Commands::View { serve, .. }
            | Commands::Browse { serve, .. } => serve.verbose,
            _ => panic!("expected a hosting command"),
        }
    }

    #[test]
    fn hosting_commands_accept_verbose() {
        for args in [
            ["rocci", "run", "--verbose"].as_slice(),
            ["rocci", "run", "-v"].as_slice(),
            ["rocci", "view", "Foo.rocci", "--verbose"].as_slice(),
            ["rocci", "browse", "--verbose"].as_slice(),
        ] {
            assert!(verbose_of(&Cli::try_parse_from(args).unwrap()));
        }
        assert!(!verbose_of(&Cli::try_parse_from(["rocci", "run"]).unwrap()));
    }

    fn public_of(cli: &Cli) -> bool {
        match &cli.command {
            Commands::Run { serve, .. }
            | Commands::View { serve, .. }
            | Commands::Browse { serve, .. } => serve.public,
            _ => panic!("expected a hosting command"),
        }
    }

    #[test]
    fn hosting_commands_accept_public() {
        for args in [
            ["rocci", "run", "--public"].as_slice(),
            ["rocci", "view", "Foo.rocci", "--public"].as_slice(),
            ["rocci", "browse", "--public"].as_slice(),
        ] {
            assert!(public_of(&Cli::try_parse_from(args).unwrap()));
        }
        assert!(!public_of(&Cli::try_parse_from(["rocci", "run"]).unwrap()));
    }

    #[test]
    fn test_parses_path_and_defaults_to_cwd() {
        let cli = Cli::try_parse_from(["rocci", "test", "Hello.rocci"]).unwrap();
        match cli.command {
            Commands::Test { path } => assert_eq!(path, PathBuf::from("Hello.rocci")),
            _ => panic!("expected test"),
        }
        let cli = Cli::try_parse_from(["rocci", "test"]).unwrap();
        match cli.command {
            Commands::Test { path } => assert_eq!(path, PathBuf::from(".")),
            _ => panic!("expected test"),
        }
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
            "examples/rocdown/pages/Guide.rocdown",
        ])
        .unwrap();
        match cli.command {
            Commands::Playground { mode, input, .. } => {
                assert!(matches!(mode, PlaygroundModeArg::Local));
                assert_eq!(input, PathBuf::from("examples/rocdown/pages/Guide.rocdown"));
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
            "examples/rocci/custom/datastar",
        ])
        .unwrap();
        match cli.command {
            Commands::Datastar {
                command: DatastarCmd::Pin { version, app },
            } => {
                assert_eq!(version, "1.0.2");
                assert_eq!(app, PathBuf::from("examples/rocci/custom/datastar"));
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

    #[test]
    fn build_http_module_is_distinct_from_host_wasm() {
        let cli = Cli::try_parse_from([
            "rocci",
            "build",
            "--http-module",
            "App.rocci",
            "-o",
            "out.wasm",
        ])
        .unwrap();
        match cli.command {
            Commands::Build {
                http_module,
                output,
                ..
            } => {
                assert!(http_module);
                assert_eq!(output, Some(PathBuf::from("out.wasm")));
            }
            _ => panic!("expected build --http-module"),
        }
        let help = Cli::command()
            .find_subcommand("build")
            .expect("build subcommand")
            .clone()
            .render_long_help()
            .to_string();
        assert!(help.contains("--http-module"), "{help}");
        assert!(help.contains("not `--host wasm`"), "{help}");
        assert!(help.contains("compiled from the input"), "{help}");
        assert!(
            help.contains("wasmtime serve") || help.contains("wasi:http/service"),
            "{help}"
        );
        assert!(!help.contains("not a WASI component"), "{help}");
    }

    #[test]
    #[ignore = "ROCCI_REQUIRE_ROC=1: compile two .rocci inputs to different GET / bodies"]
    fn http_module_two_rocci_inputs_differ() {
        if std::env::var("ROCCI_REQUIRE_ROC").as_deref() != Ok("1") {
            panic!("set ROCCI_REQUIRE_ROC=1 to run this ignored test");
        }
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let dir = std::env::temp_dir().join(format!(
            "rocci-http-module-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let alpha = dir.join("alpha.wasm");
        let beta = dir.join("beta.wasm");
        write_http_module(&fixtures.join("http-alpha/HttpAlpha.rocci"), &alpha).unwrap();
        write_http_module(&fixtures.join("http-beta/HttpBeta.rocci"), &beta).unwrap();
        let alpha_bytes = fs::read(&alpha).unwrap();
        let beta_bytes = fs::read(&beta).unwrap();
        assert!(is_wasi_http_component(&alpha_bytes), "alpha component");
        assert!(is_wasi_http_component(&beta_bytes), "beta component");
        assert_ne!(alpha_bytes, beta_bytes);
        assert!(
            contains_bytes(&alpha_bytes, b"http-alpha"),
            "alpha GET / body"
        );
        assert!(contains_bytes(&beta_bytes, b"http-beta"), "beta GET / body");
        let _ = fs::remove_dir_all(&dir);
    }

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }
}
