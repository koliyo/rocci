use std::{
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "rocs", about = "Static documentation generator built on Rocci")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile Rocdown pages into a static site.
    Build {
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Override `build.output` from rocs.toml.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Watch sources, rebuild, and serve with live reload.
    Run {
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Write preview output here instead of a temp directory.
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
    },
    /// Validate the documentation catalog without writing output.
    Check {
        #[arg(default_value = ".")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Print resolved catalog data.
    Inspect {
        #[command(subcommand)]
        target: InspectTarget,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PortArg {
    Auto,
    Exact(u16),
}

impl PortArg {
    fn resolve(self) -> Result<u16> {
        match self {
            Self::Auto => free_port(),
            Self::Exact(port) => {
                if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                    bail!("port {port} is already in use; pass --port auto or choose another port");
                }
                Ok(port)
            }
        }
    }
}

fn parse_port_arg(value: &str) -> Result<PortArg, String> {
    if value.eq_ignore_ascii_case("auto") {
        return Ok(PortArg::Auto);
    }
    match value.parse::<u16>() {
        Ok(0) => Err("port 0 is invalid; pass --port auto to pick a free port".into()),
        Ok(port) => Ok(PortArg::Exact(port)),
        Err(_) => Err(format!(
            "invalid port `{value}`; expected a number 1-65535 or `auto`"
        )),
    }
}

fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
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
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build { root, output } => {
            rocs::build_configured(&root, output.as_deref())?;
            Ok(())
        }
        Commands::Run {
            root,
            output,
            no_window,
            port,
        } => run(&root, output.as_deref(), no_window, port),
        Commands::Check { root, format } => {
            let report = rocs::check(&root)?;
            let rendered = report.render(match format {
                CheckFormatArg::Terminal => rocs::CheckFormat::Terminal,
                CheckFormatArg::Json => rocs::CheckFormat::Json,
            })?;
            if !rendered.is_empty() {
                println!("{rendered}");
            }
            if report.has_errors() {
                bail!("documentation catalog has errors");
            }
            Ok(())
        }
        Commands::Inspect { target } => {
            let (kind, root, page) = match &target {
                InspectTarget::Config { root } => (rocs::InspectKind::Config, root, None),
                InspectTarget::Catalog { root } => (rocs::InspectKind::Catalog, root, None),
                InspectTarget::Page { page, root } => {
                    (rocs::InspectKind::Page, root, Some(page.as_str()))
                }
                InspectTarget::Graph { root } => (rocs::InspectKind::Graph, root, None),
                InspectTarget::Nav { root } => (rocs::InspectKind::Nav, root, None),
                InspectTarget::Artifacts { root } => (rocs::InspectKind::Artifacts, root, None),
            };
            println!("{}", rocs::inspect(root, kind, page)?);
            Ok(())
        }
    }
}

fn run(
    root: &std::path::Path,
    output: Option<&std::path::Path>,
    no_window: bool,
    port: PortArg,
) -> Result<()> {
    let port = port.resolve()?;
    let server = rocs::run(root, output, port)?;
    eprintln!("rocs: serving {} at {}", server.title, server.url);
    if no_window {
        server.wait();
        return Ok(());
    }
    let result = rocci_wry::preview(rocci_wry::PreviewOptions {
        url: server.url.clone(),
        title: server.title.clone(),
        ..rocci_wry::PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn build_parses() {
        let cli = Cli::try_parse_from(["rocs", "build", "examples/rocs", "--output", "tmp-dist"])
            .unwrap();
        match cli.command {
            Commands::Build { root, output } => {
                assert_eq!(root, PathBuf::from("examples/rocs"));
                assert_eq!(output, Some(PathBuf::from("tmp-dist")));
            }
            _ => panic!("expected build"),
        }
    }

    #[test]
    fn run_parses_no_window_and_port() {
        let cli =
            Cli::try_parse_from(["rocs", "run", "docs", "--no-window", "--port", "8000"]).unwrap();
        match cli.command {
            Commands::Run {
                root,
                output,
                no_window,
                port,
            } => {
                assert_eq!(root, PathBuf::from("docs"));
                assert!(output.is_none());
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8000));
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn run_defaults_port_to_8000_without_window() {
        if std::env::var_os("ROC_BASIC_WEBSERVER_PORT").is_some() {
            return;
        }
        let cli = Cli::try_parse_from(["rocs", "run", "--no-window"]).unwrap();
        match cli.command {
            Commands::Run {
                no_window, port, ..
            } => {
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8000));
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn check_and_inspect_parse() {
        let cli = Cli::try_parse_from(["rocs", "check", "docs", "--format", "json"]).unwrap();
        match cli.command {
            Commands::Check { root, format } => {
                assert_eq!(root, PathBuf::from("docs"));
                assert!(matches!(format, CheckFormatArg::Json));
            }
            _ => panic!("expected check"),
        }
        let cli = Cli::try_parse_from(["rocs", "inspect", "page", "index", "docs"]).unwrap();
        match cli.command {
            Commands::Inspect { target } => match target {
                InspectTarget::Page { page, root } => {
                    assert_eq!(page, "index");
                    assert_eq!(root, PathBuf::from("docs"));
                }
                _ => panic!("expected page"),
            },
            _ => panic!("expected inspect"),
        }
    }
}
