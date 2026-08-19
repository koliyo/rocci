use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_desktop::PreviewOptions;
use rocci_template::{Diagnostic, LowerOptions, Segment, SourceFile, compile, format_diagnostic};

use crate::datastar_asset;
use crate::driver::{self, GenericAppPlan, GenericModule, ResolvedEntry};
use crate::error_page::{FailedFile, MappedModule};
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::runtime_assets;
use crate::serve;
use crate::style;

pub fn run(file: &Path, args: &[String], no_window: bool, port: serve::PortArg) -> Result<()> {
    if is_standalone_file(file) {
        return run_standalone(file, args, no_window, port);
    }
    let resolved = resolve_entry(file)?;
    datastar_asset::ensure_app(&resolved.app_dir, datastar_asset::HintMode::Print)?;
    runtime_assets::stage_into(&resolved.app_dir)?;
    let compiled = compile_rocci_app(&resolved.app_dir)?;
    if !compiled.failures.is_empty() {
        return driver::serve_template_errors(
            &compiled.failures,
            port,
            no_window,
            &driver::window_title(&resolved),
        );
    }
    driver::execute_resolved_entry(
        &resolved,
        args,
        no_window,
        port,
        &compiled.maps,
        None,
        compiled.profile,
        compiled.inspect_pages,
    )
}

fn resolve_entry(file: &Path) -> Result<ResolvedEntry> {
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };

    if path.is_dir() {
        let roc_file = path.join("main.roc");
        if !roc_file.is_file() {
            if crate::path_hint::looks_like_okf_bundle(&path) {
                bail!(
                    "no main.roc in {}; preview OKF knowledge bundles with `rocci-okf run {}`",
                    path.display(),
                    file.display()
                );
            }
            let standalone = suggest_standalone(&path)?;
            if standalone.is_empty() {
                bail!("no main.roc in {}", path.display());
            }
            bail!(
                "no main.roc in {}; run a standalone file with `rocci run {}`",
                path.display(),
                standalone[0].display()
            );
        }
        return Ok(ResolvedEntry {
            app_dir: path,
            roc_file: PathBuf::from("main.roc"),
        });
    }

    if !path.is_file() {
        bail!("no such Roc app: {}", path.display());
    }

    let ext = path.extension().and_then(|e| e.to_str());
    if ext != Some("roc") {
        if ext == Some("rocdown") || ext == Some("md") || ext == Some("markdown") {
            if ext != Some("rocdown") && crate::path_hint::looks_like_okf_file(&path) {
                bail!(
                    "unsupported file extension for `rocci run`: {}; preview OKF knowledge records with `rocci-okf run {}`",
                    path.display(),
                    file.display()
                );
            }
            bail!(
                "unsupported file extension for `rocci run`: {}; run Markdown and Rocdown documents with `rocdown run {}`",
                path.display(),
                file.display()
            );
        }
        bail!(
            "unsupported file extension for `rocci run`: {}; expected a .roc or .rocci file",
            path.display()
        );
    }

    let roc_file = path
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("main.roc"));
    let app_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));

    if !app_dir.join(&roc_file).is_file() {
        bail!("no such Roc app: {}", path.display());
    }

    Ok(ResolvedEntry { app_dir, roc_file })
}

fn suggest_standalone(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(discover_standalone(dir)?
        .into_iter()
        .map(|path| {
            path.strip_prefix(env::current_dir().unwrap_or_else(|_| dir.to_path_buf()))
                .map(Path::to_path_buf)
                .unwrap_or(path)
        })
        .collect())
}

fn is_standalone_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "rocci")
}

fn discover_standalone(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.is_file() && is_standalone_file(&path) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn run_standalone(
    file: &Path,
    args: &[String],
    no_window: bool,
    port: serve::PortArg,
) -> Result<()> {
    let path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir()?.join(file)
    };
    if !path.is_file() {
        bail!("no such Rocci file: {}", path.display());
    }
    let src_dir = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current directory"));

    let title = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("rocci")
        .to_string();
    let plan = match plan_standalone(&path)? {
        (StandaloneReady::Failed(files), _, _) => {
            return driver::serve_template_errors(&files, port, no_window, &title);
        }
        (StandaloneReady::Ready(plan), profile, inspect_pages) => (plan, profile, inspect_pages),
    };
    let (plan, profile, inspect_pages) = plan;
    let options = driver::DriverOptions {
        args: args.to_vec(),
        no_window,
        port,
        db_path: None,
        title,
        preview_path: None,
        profile,
        inspect_pages,
        state_key: None,
    };
    driver::execute_app_plan(&plan, &src_dir, &options)
}

enum StandaloneReady {
    Ready(GenericAppPlan),
    Failed(Vec<FailedFile>),
}

fn plan_standalone(
    primary: &Path,
) -> Result<(
    StandaloneReady,
    crate::profile::ProfileSnapshot,
    Vec<crate::inspect::InspectPage>,
)> {
    let mut rec = crate::profile::SpanRecorder::new();
    let mut modules = Vec::new();
    let mut failures = Vec::new();
    let mut inspect_pages = Vec::new();
    let primary = primary
        .canonicalize()
        .unwrap_or_else(|_| primary.to_path_buf());
    let inputs = vec![primary.clone()];
    for input in inputs {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src)?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        if compiled.failed {
            failures.push(FailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }
        inspect_pages.push(compiled.inspect);
        let type_name = type_name_from_path(&input);
        modules.push(GenericModule {
            type_name: type_name.clone(),
            roc: compiled.roc.clone(),
            state_type: compiled.state_type,
            init: compiled.init,
            routes: compiled.routes,
            mapped: MappedModule {
                type_name,
                generated: compiled.roc,
                source_name: name,
                source_src: src,
                segments: compiled.segments,
            },
            local_assets: compiled.local_assets,
        });
    }
    let profile = rec.finish();
    if !failures.is_empty() {
        return Ok((StandaloneReady::Failed(failures), profile, inspect_pages));
    }
    Ok((
        StandaloneReady::Ready(GenericAppPlan {
            primary_name: type_name_from_path(&primary),
            modules,
            redirect_trailing_slash: redirect_trailing_slash_for(
                primary.parent().unwrap_or_else(|| Path::new(".")),
            ),
        }),
        profile,
        inspect_pages,
    ))
}

fn redirect_trailing_slash_for(dir: &Path) -> bool {
    let path = dir.join("rocci.toml");
    if !path.is_file() {
        return true;
    }
    Config::from_file(path)
        .map(|config| config.http.redirect_trailing_slash)
        .unwrap_or(true)
}

fn discover_rocci(app_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(app_dir).with_context(|| format!("failed to read {}", app_dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rocci") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn generated_module_path(rocci: &Path) -> PathBuf {
    rocci.with_extension("roc")
}

pub fn compile_rocci_modules(app_dir: &Path) -> Result<()> {
    let compiled = compile_rocci_app(app_dir)?;
    if !compiled.failures.is_empty() {
        bail!("template compilation failed");
    }
    Ok(())
}

struct CompiledApp {
    failures: Vec<FailedFile>,
    maps: Vec<MappedModule>,
    profile: crate::profile::ProfileSnapshot,
    inspect_pages: Vec<crate::inspect::InspectPage>,
}

fn compile_rocci_app(app_dir: &Path) -> Result<CompiledApp> {
    let mut rec = crate::profile::SpanRecorder::new();
    let mut failures = Vec::new();
    let mut maps = Vec::new();
    let mut inspect_pages = Vec::new();
    for input in discover_rocci(app_dir)? {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src)?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        if compiled.failed {
            failures.push(FailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }
        if inspect_pages.is_empty() {
            inspect_pages.push(compiled.inspect);
        }
        let type_name = type_name_from_path(&input);
        let output = generated_module_path(&input);
        rec.span("write", || {
            fs::write(&output, wrap_type_module(&compiled.roc, &type_name))
                .with_context(|| format!("failed to write {}", output.display()))
        })?;
        maps.push(MappedModule {
            type_name,
            generated: compiled.roc,
            source_name: name,
            source_src: src,
            segments: compiled.segments,
        });
    }
    Ok(CompiledApp {
        failures,
        maps,
        profile: rec.finish(),
        inspect_pages,
    })
}

struct CompiledSource {
    roc: String,
    state_type: Option<String>,
    init: Option<rocci_template::InitInfo>,
    routes: Vec<rocci_template::RouteInfo>,
    failed: bool,
    diagnostics: Vec<Diagnostic>,
    segments: Vec<Segment>,
    local_assets: Vec<String>,
    timings: rocci_template::CompileTimings,
    inspect: crate::inspect::InspectPage,
}

fn compile_source(name: &str, src: &str) -> Result<CompiledSource> {
    let source = SourceFile::new(name, src);
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    let failed = compiled.has_errors();
    let route = crate::driver::preview_path(&compiled.routes);
    let inspect = crate::inspect::InspectPage::from_rocci_compile(&route, name, src, &compiled);
    Ok(CompiledSource {
        roc: compiled.roc,
        state_type: compiled.state_type,
        init: compiled.init,
        routes: compiled.routes,
        failed,
        diagnostics: compiled.diagnostics,
        segments: compiled.segments,
        local_assets: Vec::new(),
        timings: compiled.timings,
        inspect,
    })
}

pub fn run_bundled(resources: &Path) -> Result<()> {
    let config = Config::from_file(resources.join("rocci.toml"))?;
    let app_dir = resources.join("app");
    let server = app_dir.join("server");
    if !server.is_file() {
        bail!("bundled app is missing {}", server.display());
    }

    let port = if config.http.port == 0 {
        serve::PortArg::Auto.resolve()?
    } else {
        serve::PortArg::Exact(config.http.port).resolve()?
    };
    let url = format!("http://127.0.0.1:{port}/");
    let window = config.windows.first();
    let title = window
        .map(|window| window.title.as_str())
        .unwrap_or(&config.app.name)
        .to_string();
    let width = window.map(|window| window.width).unwrap_or(1040.0);
    let height = window.map(|window| window.height).unwrap_or(760.0);

    let mut cmd = Command::new(&server);
    cmd.current_dir(&app_dir)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to start {}", server.display()))?;
    if let Err(err) = serve::wait_for_server(&mut child, port) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }

    println!("{}", style::serving(&app_dir.display().to_string(), &url));
    let state_key = format!("rocci:{}", config.app.identifier);
    let preview_result = rocci_desktop::preview(PreviewOptions {
        url,
        title,
        width,
        height,
        devtools: config.development.devtools,
        state_key: Some(state_key),
        inspector_url: None,
        source_root: None,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    let _ = child.kill();
    let _ = child.wait();
    preview_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{roc_command, roc_invocation, window_title};

    fn temp_app(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("rocci-run-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn resolve_entry_uses_file_name_and_parent_dir() {
        let dir = temp_app("file");
        let main = dir.join("main.roc");
        fs::write(&main, "app").unwrap();
        let resolved = resolve_entry(&main).unwrap();
        assert_eq!(resolved.app_dir, dir);
        assert_eq!(resolved.roc_file, PathBuf::from("main.roc"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_entry_directory_uses_main_roc() {
        let dir = temp_app("dir");
        fs::write(dir.join("main.roc"), "app").unwrap();
        let resolved = resolve_entry(&dir).unwrap();
        assert_eq!(resolved.app_dir, dir);
        assert_eq!(resolved.roc_file, PathBuf::from("main.roc"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_entry_rejects_missing_app() {
        let dir = temp_app("missing");
        let err = resolve_entry(&dir.join("main.roc"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("no such Roc app"));
        let err = resolve_entry(&dir).unwrap_err().to_string();
        assert!(err.contains("no main.roc"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_entry_directory_suggests_standalone_rocci() {
        let dir = temp_app("standalone-hint");
        fs::write(dir.join("Counter.rocci"), "").unwrap();
        let err = resolve_entry(&dir).unwrap_err().to_string();
        assert!(err.contains("no main.roc"));
        assert!(err.contains("rocci run"));
        assert!(err.contains("Counter.rocci"));
        cleanup(&dir);
    }

    #[test]
    fn discover_rocci_is_non_recursive_and_ignores_other_extensions() {
        let dir = temp_app("discover");
        fs::write(dir.join("Snake.rocci"), "").unwrap();
        fs::write(dir.join("Game.roc"), "").unwrap();
        fs::write(dir.join("notes.txt"), "").unwrap();
        let nested = dir.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("Other.rocci"), "").unwrap();

        let found = discover_rocci(&dir).unwrap();
        assert_eq!(found, vec![dir.join("Snake.rocci")]);
        cleanup(&dir);
    }

    #[test]
    fn generated_module_uses_stem() {
        let input = Path::new("examples/snake/Snake.rocci");
        assert_eq!(
            generated_module_path(input),
            PathBuf::from("examples/snake/Snake.roc")
        );
        assert_eq!(type_name_from_path(input), "Snake");
    }

    #[test]
    fn compile_writes_wrapped_type_module() {
        let dir = temp_app("compile");
        fs::write(
            dir.join("Hello.rocci"),
            "import Html\n\n@component Hello = |{ name }| {\n    <p>{name}</p>\n}\n",
        )
        .unwrap();
        compile_rocci_modules(&dir).unwrap();
        let generated = fs::read_to_string(dir.join("Hello.roc")).unwrap();
        assert!(generated.starts_with("import Html\n\nHello := [].{\n"));
        assert!(generated.contains("    hello = |{ name }| {"));
        cleanup(&dir);
    }

    #[test]
    fn roc_invocation_forwards_args_and_runs_from_app_dir() {
        let resolved = ResolvedEntry {
            app_dir: PathBuf::from("/tmp/app"),
            roc_file: PathBuf::from("main.roc"),
        };
        let invocation = roc_invocation(&resolved, &["--".into(), "arg1".into()]);
        assert_eq!(invocation.program, "roc");
        assert_eq!(invocation.app_dir, PathBuf::from("/tmp/app"));
        assert_eq!(invocation.roc_file, PathBuf::from("main.roc"));
        assert_eq!(invocation.args, vec!["--".to_string(), "arg1".to_string()]);

        let cmd = roc_command(&invocation, 9001);
        assert_eq!(cmd.get_program(), "roc");
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["main.roc", "--", "arg1"]);
        assert_eq!(cmd.get_current_dir(), Some(Path::new("/tmp/app")));
        let port = cmd
            .get_envs()
            .find(|(key, _)| *key == "ROC_BASIC_WEBSERVER_PORT")
            .and_then(|(_, value)| value)
            .unwrap();
        assert_eq!(port, "9001");
    }

    #[test]
    fn window_title_uses_app_directory_name() {
        let resolved = ResolvedEntry {
            app_dir: PathBuf::from("/tmp/snake"),
            roc_file: PathBuf::from("main.roc"),
        };
        assert_eq!(window_title(&resolved), "snake");
    }

    #[test]
    fn resolve_entry_rejects_unsupported_file_extensions() {
        let dir = temp_app("unsupported-ext");
        let txt_file = dir.join("notes.txt");
        fs::write(&txt_file, "hello").unwrap();
        let err = resolve_entry(&txt_file).unwrap_err().to_string();
        assert!(err.contains("unsupported file extension"));
        assert!(err.contains("expected a .roc or .rocci file"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_entry_suggests_rocdown_for_markdown_documents() {
        let dir = temp_app("markdown-hint");
        let md_file = dir.join("PLAN.md");
        fs::write(&md_file, "# Plan").unwrap();
        let err = resolve_entry(&md_file).unwrap_err().to_string();
        assert!(err.contains("unsupported file extension"));
        assert!(err.contains("rocdown run"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_entry_suggests_okf_for_knowledge_records() {
        let dir = temp_app("okf-hint");
        let md_file = dir.join("plan.md");
        fs::write(
            &md_file,
            "---\ntype: Implementation Plan\ntitle: Plan\nauthority: exploratory\n---\n\n# Plan\n",
        )
        .unwrap();
        let err = resolve_entry(&md_file).unwrap_err().to_string();
        assert!(err.contains("unsupported file extension"));
        assert!(err.contains("rocci-okf run"));
        cleanup(&dir);
    }

    #[test]
    fn resolve_entry_suggests_okf_for_knowledge_bundle() {
        let dir = temp_app("okf-bundle-hint");
        fs::write(
            dir.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        let err = resolve_entry(&dir).unwrap_err().to_string();
        assert!(err.contains("rocci-okf run"));
        cleanup(&dir);
    }
}
