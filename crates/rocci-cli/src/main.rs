mod browse;
mod bundle;
mod roc_module;
mod run;
mod serve;
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
    /// Build a .rocci module to ordinary Roc.
    Build {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compile sibling .rocci modules and run a Roc app.
    Run {
        /// Skip the embedded window; print the URL and keep the Roc server.
        #[arg(long)]
        no_window: bool,
        #[command(flatten)]
        port: serve::PortOptions,
        /// Roc app file or directory
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
    /// Render a component in an embedded window.
    View {
        input: PathBuf,
        #[arg(long, default_value = "main")]
        component: String,
        /// Component parameter as name=value (repeatable).
        #[arg(long = "arg", value_name = "NAME=VALUE", action = clap::ArgAction::Append)]
        args: Vec<String>,
        /// Skip the embedded window; print the URL and keep the Roc server.
        #[arg(long)]
        no_window: bool,
        #[command(flatten)]
        port: serve::PortOptions,
    },
    /// Browse components under one or more roots.
    Browse {
        /// Skip the embedded window; print the URL and keep the Roc server.
        #[arg(long)]
        no_window: bool,
        #[command(flatten)]
        port: serve::PortOptions,
        /// Directories (recursive) and/or .rocci files.
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    if env::args().len() <= 1
        && let Some(resources) = bundle::bundled_resources()
    {
        return run::run_bundled(&resources);
    }

    match Cli::parse().command {
        Commands::Validate { config } => validate(&config),
        Commands::Bundle { config } => bundle::bundle(&config),
        Commands::Build { input, output } => build_rocci(&input, output.as_deref()),
        Commands::Run {
            file,
            args,
            no_window,
            port,
        } => run::run(&file, &args, no_window, port.port),
        Commands::Inspect { input, ast } => inspect_rocci(&input, ast),
        Commands::Ast { input } => ast_rocci(&input),
        Commands::View {
            input,
            component,
            args,
            no_window,
            port,
        } => view::view(&input, &component, &args, no_window, port.port),
        Commands::Browse {
            roots,
            no_window,
            port,
        } => browse::browse(&roots, no_window, port.port),
    }
}

fn build_rocci(input: &Path, output: Option<&Path>) -> Result<()> {
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!(
            "{}",
            format_diagnostic(SourceFile::new(&name, &src), diagnostic)
        );
    }
    if compiled.has_errors() {
        bail!("template compilation failed");
    }
    match output {
        Some(path) => fs::write(path, compiled.roc)
            .with_context(|| format!("failed to write {}", path.display()))?,
        None => print!("{}", compiled.roc),
    }
    Ok(())
}

fn ast_rocci(input: &Path) -> Result<()> {
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

fn inspect_rocci(input: &Path, ast: bool) -> Result<()> {
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
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
        println!("\n# ast\n{}", format_ast(&src, &compiled.document));
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
        "ok: {} ({} window{})",
        config.app.identifier,
        config.windows.len(),
        if config.windows.len() == 1 { "" } else { "s" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn port_of(cli: Cli) -> serve::PortArg {
        match cli.command {
            Commands::Run { port, .. }
            | Commands::View { port, .. }
            | Commands::Browse { port, .. } => port.port,
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
                port_of(Cli::try_parse_from(args).unwrap()),
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
                port_of(Cli::try_parse_from(args).unwrap()),
                serve::PortArg::Exact(9001)
            );
        }
    }

    #[test]
    fn run_accepts_port_after_app_path() {
        let cli =
            Cli::try_parse_from(["rocci", "run", "examples/counter", "--port", "auto"]).unwrap();
        assert_eq!(port_of(cli), serve::PortArg::Auto);
    }
}
