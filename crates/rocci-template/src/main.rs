use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use rocci_template::{CompileOutput, LowerOptions, SourceFile, compile, format_diagnostic};

#[derive(Parser)]
#[command(
    name = "rocci-template",
    about = "Parse and lower .rocci templates to ordinary Roc"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lower a .rocci module to Roc and print or write the result.
    Compile {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show generated Roc, components, and source-map segments.
    Inspect { input: PathBuf },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    match Cli::parse().command {
        Commands::Compile { input, output } => {
            let (name, src) = read_input(&input)?;
            let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
            print_diagnostics(&compiled, &name, &src);
            if compiled.has_errors() {
                return Ok(ExitCode::from(1));
            }
            match output {
                Some(path) => {
                    fs::write(&path, &compiled.roc)
                        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
                }
                None => {
                    io::stdout()
                        .write_all(compiled.roc.as_bytes())
                        .map_err(|err| format!("failed to write stdout: {err}"))?;
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Inspect { input } => {
            let (name, src) = read_input(&input)?;
            let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
            print_diagnostics(&compiled, &name, &src);
            println!("# components ({})", compiled.components.len());
            for component in &compiled.components {
                let body = if component.body_params.is_empty() {
                    "props".to_string()
                } else {
                    format!("props, {}", component.body_params.join(", "))
                };
                println!("- {} ({body})", component.name);
            }
            println!("\n# generated roc\n{}", compiled.roc);
            println!("# segments ({})", compiled.segments.len());
            for segment in &compiled.segments {
                let (sline, scol) = SourceFile::new(&name, &src).line_col(segment.source.start);
                println!(
                    "- generated {}..{} <- {}:{}:{} {} ({})",
                    segment.generated.start,
                    segment.generated.end,
                    name,
                    sline,
                    scol,
                    segment.origin,
                    snippet(&src, segment.source.start, segment.source.end),
                );
            }
            Ok(if compiled.has_errors() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

fn read_input(path: &Path) -> Result<(String, String), String> {
    if path.as_os_str() == "-" {
        let mut src = String::new();
        io::stdin()
            .read_to_string(&mut src)
            .map_err(|err| format!("failed to read stdin: {err}"))?;
        return Ok(("<stdin>".to_string(), src));
    }
    let src = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok((path.display().to_string(), src))
}

fn print_diagnostics(compiled: &CompileOutput, name: &str, src: &str) {
    let source = SourceFile::new(name, src);
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
}

fn snippet(src: &str, start: u32, end: u32) -> String {
    let start = start as usize;
    let end = (end as usize).min(src.len());
    let start = start.min(end);
    let text = src[start..end].replace('\n', "\\n");
    if text.len() > 48 {
        format!("{}…", &text[..48])
    } else {
        text
    }
}
