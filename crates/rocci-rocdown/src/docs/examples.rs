use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::catalog::CatalogDiagnostic;

use super::{BuildCtx, DocsNode, ExampleRecord, IncludeOrigin};

pub(crate) fn push_example(ctx: &mut BuildCtx<'_>, node: &DocsNode) {
    let origin = node.origin.clone().unwrap_or(IncludeOrigin {
        source_path: node
            .attrs
            .path
            .clone()
            .unwrap_or_else(|| ctx.source_path.to_string()),
        region: node.attrs.region.clone(),
        line_start: node.attrs.start,
        line_end: node.attrs.end,
    });
    if node.attrs.path.is_some() {
        ctx.snippet_paths.insert(origin.source_path.clone());
    }
    ctx.examples.push(ExampleRecord {
        id: node.attrs.id.clone().unwrap_or_default(),
        language: node.attrs.language.clone().unwrap_or_default(),
        path: node.attrs.path.clone(),
        region: node.attrs.region.clone(),
        test: node.attrs.test.clone(),
        expect: node.attrs.expect.clone(),
        allow_network: node.attrs.allow_network,
        origin,
        line: node.line,
    });
}

#[derive(Debug, Clone)]
pub struct ExampleTestOptions {
    pub root: PathBuf,
    pub timeout: Duration,
    pub allow_network: bool,
    pub update: bool,
}

pub fn run_examples(
    examples: &[ExampleRecord],
    options: &ExampleTestOptions,
) -> Vec<CatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    for example in examples {
        if example.test.is_empty() {
            continue;
        }
        if example.allow_network && !options.allow_network {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2603",
                &example.origin.source_path,
                format!(
                    "line {}: example requires network but examples.allow_network is false",
                    example.line
                ),
            ));
            continue;
        }
        let cwd = example
            .path
            .as_deref()
            .and_then(|path| options.root.join(path).parent().map(Path::to_path_buf))
            .unwrap_or_else(|| options.root.clone());
        let Some((program, args)) = example.test.split_first() else {
            continue;
        };
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(&cwd)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = match run_timed(&mut command, options.timeout) {
            Ok(output) => output,
            Err(CommandError::Timeout) => {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2603",
                    &example.origin.source_path,
                    format!("line {}: example command timed out", example.line),
                ));
                continue;
            }
            Err(CommandError::Io(err)) => {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2603",
                    &example.origin.source_path,
                    format!("line {}: failed to run example: {err}", example.line),
                ));
                continue;
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        if let Some(expect) = &example.expect {
            let expect_path = options.root.join(expect);
            if expect_path.is_file() {
                if options.update {
                    let _ = std::fs::write(&expect_path, stdout.as_bytes());
                } else {
                    let golden = std::fs::read_to_string(&expect_path).unwrap_or_default();
                    if stdout != golden {
                        diagnostics.push(CatalogDiagnostic::error(
                            "RD2603",
                            &example.origin.source_path,
                            format!(
                                "line {}: example output did not match golden file `{expect}`",
                                example.line
                            ),
                        ));
                    }
                }
            } else if !combined.contains(expect) {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2603",
                    &example.origin.source_path,
                    format!(
                        "line {}: example output did not contain `{expect}`",
                        example.line
                    ),
                ));
            }
        }
        if !output.status.success() {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2603",
                &example.origin.source_path,
                format!("line {}: example command failed", example.line),
            ));
        }
    }
    diagnostics
}

struct CommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

enum CommandError {
    Timeout,
    Io(std::io::Error),
}

fn run_timed(command: &mut Command, timeout: Duration) -> Result<CommandOutput, CommandError> {
    let mut child = command.spawn().map_err(CommandError::Io)?;
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let stdout_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stdout_pipe.take() {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stderr_pipe.take() {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CommandError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(err) => return Err(CommandError::Io(err)),
        }
    };
    Ok(CommandOutput {
        status,
        stdout: stdout_thread.join().unwrap_or_default(),
        stderr: stderr_thread.join().unwrap_or_default(),
    })
}
