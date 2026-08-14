mod browse;
mod roc_module;
mod run;
mod serve;
mod view;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
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
    },
    /// Browse components under one or more roots.
    Browse {
        /// Skip the embedded window; print the URL and keep the Roc server.
        #[arg(long)]
        no_window: bool,
        /// Directories (recursive) and/or .rocci files.
        #[arg(default_value = ".")]
        roots: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Validate { config } => validate(&config),
        Commands::Bundle { config } => bundle(&config),
        Commands::Build { input, output } => build_rocci(&input, output.as_deref()),
        Commands::Run {
            file,
            args,
            no_window,
        } => run::run(&file, &args, no_window),
        Commands::Inspect { input, ast } => inspect_rocci(&input, ast),
        Commands::Ast { input } => ast_rocci(&input),
        Commands::View {
            input,
            component,
            args,
            no_window,
        } => view::view(&input, &component, &args, no_window),
        Commands::Browse { roots, no_window } => browse::browse(&roots, no_window),
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

fn bundle(config_path: &Path) -> Result<()> {
    let config = Config::from_file(config_path)?;
    let root = workspace_root(config_path)?;
    let package = config.bundle.package.as_deref().unwrap_or("counter");
    let binary = config.bundle.binary.as_deref().unwrap_or(package);
    let identifier = config
        .bundle
        .identifier
        .as_deref()
        .unwrap_or(&config.app.identifier);

    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(&root)
        .args(["build", "--release", "-p", package])
        .status()
        .context("failed to run cargo build")?;
    if !status.success() {
        bail!("cargo build failed");
    }

    match env::consts::OS {
        "macos" => bundle_macos(&root, &config, package, binary, identifier, config_path)?,
        other => bail!("development bundling is not implemented for {other} yet"),
    }
    Ok(())
}

fn bundle_macos(
    root: &Path,
    config: &Config,
    _package: &str,
    binary: &str,
    _identifier: &str,
    config_path: &Path,
) -> Result<()> {
    let app_name = &config.app.name;
    let bundle_dir = root
        .join("target/release/bundle/macos")
        .join(format!("{app_name}.app"));
    let contents = bundle_dir.join("Contents");
    fs::create_dir_all(contents.join("MacOS"))?;
    fs::create_dir_all(contents.join("Resources"))?;

    let binary_src = root.join("target/release").join(binary);
    fs::copy(&binary_src, contents.join("MacOS").join(binary))
        .with_context(|| format!("failed to copy {}", binary_src.display()))?;

    let plist_src = config
        .bundle
        .macos_plist
        .clone()
        .unwrap_or_else(|| PathBuf::from("macos/Info.plist"));
    let plist_src = root.join(plist_src);
    let plist = fs::read_to_string(&plist_src)
        .with_context(|| format!("failed to read {}", plist_src.display()))?;
    fs::write(contents.join("Info.plist"), plist)?;

    let dest_config = contents.join("Resources/rocci.toml");
    fs::copy(
        root.join(config_path_relative(root, config_path)?),
        &dest_config,
    )
    .or_else(|_| fs::copy(config_path, &dest_config))
    .with_context(|| "failed to copy rocci.toml into the app bundle")?;

    for resource in &config.bundle.resources {
        let from = root.join(&resource.from);
        let to = contents.join("Resources").join(&resource.to);
        copy_tree(&from, &to)
            .with_context(|| format!("failed to copy {} -> {}", from.display(), to.display()))?;
    }

    fs::write(contents.join("PkgInfo"), b"APPL????")?;

    let status = Command::new("codesign")
        .args(["--force", "--deep", "--sign", "-"])
        .arg(&bundle_dir)
        .status()
        .context("failed to run codesign")?;
    if !status.success() {
        bail!("ad-hoc codesign failed");
    }

    println!("{}", bundle_dir.display());
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    if from.is_file() {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(from, to)?;
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        copy_tree(&entry.path(), &dest)?;
    }
    Ok(())
}

fn workspace_root(config_path: &Path) -> Result<PathBuf> {
    let mut dir = env::current_dir()?;
    if config_path.is_absolute() {
        if let Some(parent) = config_path.parent() {
            dir = parent.to_path_buf();
        }
    } else if let Some(parent) = config_path.parent()
        && !parent.as_os_str().is_empty()
    {
        dir = dir.join(parent);
    }
    loop {
        let cargo = dir.join("Cargo.toml");
        if cargo.is_file() {
            let text = fs::read_to_string(&cargo)?;
            if text.contains("[workspace]") {
                return Ok(dir);
            }
        }
        dir = dir
            .parent()
            .map(Path::to_path_buf)
            .context("rocci.toml is not inside a Cargo workspace")?;
    }
}

fn config_path_relative(root: &Path, config_path: &Path) -> Result<PathBuf> {
    if config_path.is_absolute() {
        return Ok(config_path
            .strip_prefix(root)
            .unwrap_or(config_path)
            .to_path_buf());
    }
    Ok(config_path.to_path_buf())
}
