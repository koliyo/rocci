use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use rocci_template::{
    CompileOutput, LowerOptions, SourceFile, compile, format_ast, format_diagnostic,
};

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
    /// Build a .rocci module to Roc and print or write the result.
    Build {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show generated Roc, components, source-map segments, and optional AST.
    Inspect {
        input: PathBuf,
        /// Also print the parse tree as an S-expression.
        #[arg(long)]
        ast: bool,
    },
    /// Print the parse tree as a LISPy S-expression.
    Ast { input: PathBuf },
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
        Commands::Build { input, output } => {
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
        Commands::Inspect { input, ast } => {
            let (name, src) = read_input(&input)?;
            let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
            print_diagnostics(&compiled, &name, &src);
            print_inspect(&compiled, &name, &src, ast);
            Ok(if compiled.has_errors() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        Commands::Ast { input } => {
            let (name, src) = read_input(&input)?;
            let compiled = compile(SourceFile::new(&name, &src), &LowerOptions::default());
            print_diagnostics(&compiled, &name, &src);
            print!("{}", format_ast(&src, &compiled.document));
            Ok(if compiled.has_errors() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

fn print_inspect(compiled: &CompileOutput, name: &str, src: &str, ast: bool) {
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
    println!("# styles ({})", compiled.styles.len());
    for style in &compiled.styles {
        let kind = match style.kind {
            rocci_template::StyleKind::File => "file",
            rocci_template::StyleKind::Component => "component",
            rocci_template::StyleKind::Theme => "theme",
        };
        println!("- {} {} ({} bytes)", kind, style.name, style.css.len());
    }
    if ast {
        println!("\n# ast\n{}", format_ast(src, &compiled.document));
    }
    println!("\n# generated roc\n{}", compiled.roc);
    println!("# segments ({})", compiled.segments.len());
    for segment in &compiled.segments {
        let (sline, scol) = SourceFile::new(name, src).line_col(segment.source.start);
        println!(
            "- generated {}..{} <- {}:{}:{} {} ({})",
            segment.generated.start,
            segment.generated.end,
            name,
            sline,
            scol,
            segment.origin,
            snippet(src, segment.source.start, segment.source.end),
        );
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
