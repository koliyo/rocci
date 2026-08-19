use std::{
    fs,
    io::{IsTerminal, Read},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rocci_browser::{Host, OpenRequest, Opened, Paths, Registry};
use serde_json::json;

mod tui;
mod window;

#[derive(Parser)]
#[command(
    name = "rocci-browser",
    about = "Product-blind project browser: register directories, fuzzy-pick a target, open via adapters"
)]
struct Cli {
    /// Directory that contains `.rocci/browser.toml` (defaults to the current directory).
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a project directory.
    Add {
        path: PathBuf,
        #[arg(long)]
        id: Option<String>,
    },
    /// Remove a registered project by id or path.
    Remove { query: String },
    /// List registered projects and probe labels.
    List,
    /// Fuzzy-open a target without a TUI.
    Open {
        query: String,
        #[arg(long)]
        document: Option<String>,
        /// Skip a preview window; print the origin and keep serving.
        #[arg(long)]
        no_window: bool,
        #[arg(long)]
        json: bool,
    },
    /// Interactive two-stage picker.
    Tui {
        #[arg(long)]
        no_window: bool,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = paths_from_cli(cli.root)?;
    match cli.command {
        None => window::run(paths)?,
        Some(Commands::Add { path, id }) => {
            let canonical = fs::canonicalize(&path)
                .with_context(|| format!("cannot resolve {}", path.display()))?;
            let id = id.unwrap_or_else(|| {
                canonical
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("target")
                    .to_string()
            });
            let mut registry = Registry::load_user(&paths)?;
            registry.add(id.clone(), canonical.display().to_string());
            registry.save(&paths)?;
            println!("added {id} -> {}", canonical.display());
        }
        Some(Commands::Remove { query }) => {
            let mut registry = Registry::load_user(&paths)?;
            if !registry.remove(&query) {
                bail!("no project matched {query}");
            }
            registry.save(&paths)?;
            println!("removed {query}");
        }
        Some(Commands::List) => {
            let mut host = Host::connect(paths)?;
            print_warnings(&host.warnings);
            let registry = host.registry()?;
            if registry.projects.is_empty() {
                println!("(no projects)");
            }
            let targets = host.probe_targets()?;
            print_warnings(&host.warnings);
            for project in registry.projects {
                let claims: Vec<_> = targets
                    .iter()
                    .filter(|target| target.id == project.id)
                    .collect();
                if claims.is_empty() {
                    println!("{}\t{}\t(unclaimed)", project.id, project.path);
                } else {
                    for target in claims {
                        println!(
                            "{}\t{}\t[{}] {}",
                            project.id, project.path, target.adapter_id, target.label
                        );
                    }
                }
            }
        }
        Some(Commands::Open {
            query,
            document,
            no_window,
            json,
        }) => {
            let mut host = Host::connect(paths)?;
            print_warnings(&host.warnings);
            let opened = host.open(OpenRequest {
                query: &query,
                document: document.as_deref(),
            })?;
            emit_open(&opened, no_window, json)?;
            keep_serving(&mut host, no_window)?;
        }
        Some(Commands::Tui { no_window, json }) => {
            let mut host = Host::connect(paths)?;
            print_warnings(&host.warnings);
            if tui::run(&mut host, no_window, json)?.is_some() {
                keep_serving(&mut host, no_window)?;
            }
        }
    }
    Ok(())
}

fn paths_from_cli(root: Option<PathBuf>) -> Result<Paths> {
    let mut paths = Paths::from_env()?;
    if let Some(root) = root {
        paths.cwd = fs::canonicalize(&root)
            .with_context(|| format!("cannot resolve --root {}", root.display()))?;
    }
    Ok(paths)
}

fn print_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("warning: {warning}");
    }
}

fn emit_open(opened: &Opened, _no_window: bool, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            json!({
                "url": opened.url,
                "title": opened.title,
            })
        );
    } else {
        println!("{}  {}", opened.title, opened.url);
    }
    Ok(())
}

fn keep_serving(host: &mut Host, no_window: bool) -> Result<()> {
    let _ = host;
    if !no_window {
        eprintln!(
            "pass --no-window to print a URL; rocci-browser with no args opens the preview window"
        );
    }
    if std::io::stdin().is_terminal() {
        std::thread::park();
    } else {
        let _ = std::io::stdin().read_to_end(&mut Vec::new());
    }
    Ok(())
}
