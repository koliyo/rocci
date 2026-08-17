mod browse;
mod bundle;
mod datastar_asset;
mod dispatch;
mod error_page;
mod roc_module;
mod run;
mod runtime_assets;
mod serve;
mod style;
mod theme;
mod view;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
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
    /// Build a .rocci or .rocdown module to ordinary Roc.
    Build {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[command(flatten)]
        theme: theme::ThemeArgs,
    },
    /// Compile sibling .rocci/.rocdown modules and run a Roc app, or run a standalone file.
    Run {
        #[command(flatten)]
        serve: serve::ServeOptions,
        #[command(flatten)]
        theme: theme::ThemeArgs,
        /// Roc app file, directory, or standalone .rocci/.rocdown file
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
        #[command(flatten)]
        theme: theme::ThemeArgs,
    },
    /// Print a .rocci or .rocdown parse tree as a LISPy S-expression.
    Ast { input: PathBuf },
    /// Render a component in an embedded window.
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
    /// Manage the Datastar JS runtime for an app.
    Datastar {
        #[command(subcommand)]
        command: DatastarCmd,
    },
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
        Commands::Build {
            input,
            output,
            theme,
        } => build_module(&input, output.as_deref(), &theme),
        Commands::Run {
            file,
            args,
            serve,
            theme,
        } => run::run(&file, &args, serve.no_window, serve.port, &theme),
        Commands::Inspect { input, ast, theme } => inspect_module(&input, ast, &theme),
        Commands::Ast { input } => ast_module(&input),
        Commands::View {
            input,
            component,
            args,
            serve,
        } => view::view(&input, &component, &args, serve.no_window, serve.port),
        Commands::Browse { roots, serve } => browse::browse(&roots, serve.no_window, serve.port),
        Commands::Datastar { command } => match command {
            DatastarCmd::Update { app } => datastar_asset::update_app(&app),
            DatastarCmd::Pin { version, app } => datastar_asset::pin_app(&app, &version),
        },
    }
}

fn is_rocdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "rocdown" || ext == "md" || ext == "markdown")
}

fn build_module(input: &Path, output: Option<&Path>, theme: &theme::ThemeArgs) -> Result<()> {
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let (roc, diagnostics, failed) = if is_rocdown(input) {
        let compiled = rocci_rocdown::compile(
            SourceFile::new(&name, &src),
            &theme.compile_options(Some(input)),
        );
        let failed = compiled.has_errors();
        (compiled.roc, compiled.diagnostics, failed)
    } else {
        let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
        let failed = compiled.has_errors();
        (compiled.roc, compiled.diagnostics, failed)
    };
    for diagnostic in &diagnostics {
        eprintln!(
            "{}",
            format_diagnostic(SourceFile::new(&name, &src), diagnostic)
        );
    }
    if failed {
        bail!("template compilation failed");
    }
    match output {
        Some(path) => {
            fs::write(path, roc).with_context(|| format!("failed to write {}", path.display()))?
        }
        None => print!("{roc}"),
    }
    Ok(())
}

fn ast_module(input: &Path) -> Result<()> {
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    if is_rocdown(input) {
        let compiled = rocci_rocdown::compile(
            source,
            &theme::ThemeArgs::from_env().compile_options(Some(input)),
        );
        for diagnostic in &compiled.diagnostics {
            eprintln!("{}", format_diagnostic(source, diagnostic));
        }
        print!("{}", rocci_rocdown::format_ast(&src, &compiled.document));
        if compiled.has_errors() {
            bail!("template compilation failed");
        }
        return Ok(());
    }
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

fn inspect_module(input: &Path, ast: bool, theme: &theme::ThemeArgs) -> Result<()> {
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    if is_rocdown(input) {
        let compiled = rocci_rocdown::compile(source, &theme.compile_options(Some(input)));
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
        if ast {
            println!(
                "\n# ast\n{}",
                rocci_rocdown::format_ast(&src, &compiled.document)
            );
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
        return Ok(());
    }
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
    fn run_theme_flag_parses() {
        let cli = Cli::try_parse_from([
            "rocci",
            "run",
            "--theme",
            "paper",
            "--color-scheme",
            "dark",
            "foo.rocdown",
        ])
        .unwrap();
        match cli.command {
            Commands::Run { theme, file, .. } => {
                assert_eq!(theme.theme.as_deref(), Some("paper"));
                assert_eq!(theme.color_scheme.as_deref(), Some("dark"));
                assert_eq!(file, PathBuf::from("foo.rocdown"));
            }
            _ => panic!("unexpected command"),
        }
    }
}
