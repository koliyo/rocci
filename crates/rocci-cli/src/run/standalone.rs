use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;
use rocci_desktop::PreviewOptions;
use rocci_template::{Diagnostic, LowerOptions, Segment, SourceFile, compile, format_diagnostic};

use crate::driver::{EXTRACTED_STYLESHEET_HREF, GenericAppPlan, GenericModule};
use crate::error_page::{FailedFile, MappedModule};
use crate::logs::{self, Progress};
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::serve;
use crate::style;

use super::{StandaloneReady, discover_standalone_tree, standalone_app_root};

pub fn standalone_island_lower_options() -> LowerOptions {
    LowerOptions {
        embed_css: false,
        html_type: "Html.Node".to_string(),
        ..LowerOptions::default()
    }
}

pub fn standalone_http_module_lower_options() -> LowerOptions {
    LowerOptions {
        embed_css: false,
        html_type: "Html.Node".to_string(),
        stylesheet_href: Some(EXTRACTED_STYLESHEET_HREF.to_string()),
        ..LowerOptions::default()
    }
}

pub fn standalone_http_module_app_plan(primary: &Path) -> Result<GenericAppPlan> {
    match plan_standalone(
        primary,
        &standalone_http_module_lower_options(),
        Progress::default(),
    )? {
        (StandaloneReady::Ready(plan), _, _) => Ok(plan),
        (StandaloneReady::Failed(files), _, _) => {
            let name = files
                .first()
                .map(|file| file.name.as_str())
                .unwrap_or("template");
            bail!("template compilation failed for {name}")
        }
    }
}

pub fn standalone_island_app_plan(primary: &Path) -> Result<GenericAppPlan> {
    match plan_standalone(
        primary,
        &standalone_island_lower_options(),
        Progress::default(),
    )? {
        (StandaloneReady::Ready(plan), _, _) => Ok(plan),
        (StandaloneReady::Failed(files), _, _) => {
            let name = files
                .first()
                .map(|file| file.name.as_str())
                .unwrap_or("template");
            bail!("template compilation failed for {name}")
        }
    }
}

pub(crate) fn plan_standalone(
    primary: &Path,
    lower: &LowerOptions,
    progress: Progress,
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
    let mut inputs = vec![primary.clone()];
    let app_root = standalone_app_root(&primary);
    for path in discover_standalone_tree(&app_root)? {
        let path = path.canonicalize().unwrap_or(path);
        if path != primary {
            inputs.push(path);
        }
    }
    let templates_started = Instant::now();
    progress.step(logs::run_phase_start(
        "templates",
        &format!("modules={}", inputs.len()),
    ));
    for input in inputs {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src, lower)?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        progress.detail(logs::run_module_detail(
            &name,
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            compiled.timings.lower_ms,
        ));
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
            lives: compiled.lives,
            routes: compiled.routes,
            mapped: MappedModule {
                type_name,
                generated: compiled.roc,
                source_name: name,
                source_src: src,
                segments: compiled.segments,
            },
            local_assets: compiled.local_assets,
            styles: compiled.styles,
        });
    }
    let profile = rec.finish();
    progress.step(logs::run_phase_done(
        "templates",
        templates_started.elapsed().as_millis(),
        "",
    ));
    if !failures.is_empty() {
        return Ok((StandaloneReady::Failed(failures), profile, inspect_pages));
    }
    // Primary first so dispatch / @init / @get:live come from the entry file.
    if let Some(index) = modules
        .iter()
        .position(|module| module.type_name == type_name_from_path(&primary))
    {
        modules.swap(0, index);
    }
    Ok((
        StandaloneReady::Ready(GenericAppPlan {
            primary_name: type_name_from_path(&primary),
            modules,
            redirect_trailing_slash: redirect_trailing_slash_for(&app_root),
            log_handlers: false,
            platform: None,
        }),
        profile,
        inspect_pages,
    ))
}

pub fn standalone_app_plan(primary: &Path) -> Result<GenericAppPlan> {
    match plan_standalone(primary, &LowerOptions::default(), Progress::default())? {
        (StandaloneReady::Ready(plan), _, _) => {
            ensure_unique_process_init(&plan)?;
            Ok(plan)
        }
        (StandaloneReady::Failed(files), _, _) => {
            let name = files
                .first()
                .map(|file| file.name.as_str())
                .unwrap_or("template");
            bail!("template compilation failed for {name}")
        }
    }
}

fn module_has_process_init(module: &GenericModule) -> bool {
    module.init.is_some() || module.state_type.is_some()
}

fn process_init_file_name(module: &GenericModule) -> &str {
    Path::new(&module.mapped.source_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(module.type_name.as_str())
}

pub(crate) fn ensure_unique_process_init(plan: &GenericAppPlan) -> Result<()> {
    let inits: Vec<_> = plan
        .modules
        .iter()
        .filter(|module| module_has_process_init(module))
        .collect();
    match inits.as_slice() {
        [] => Ok(()),
        [only] if only.type_name == plan.primary_name => Ok(()),
        [only] => bail!(
            "process `@init` is in `{}`; run that file or the app directory",
            process_init_file_name(only)
        ),
        many => {
            let names = many
                .iter()
                .map(|module| format!("`{}`", process_init_file_name(module)))
                .collect::<Vec<_>>()
                .join(", ");
            bail!("multiple process `@init` / `@context` modules in one app: {names}")
        }
    }
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

pub(crate) fn discover_rocci(app_dir: &Path) -> Result<Vec<PathBuf>> {
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

pub(crate) fn generated_module_path(rocci: &Path) -> PathBuf {
    rocci.with_extension("roc")
}

pub fn compile_rocci_modules(app_dir: &Path) -> Result<()> {
    let compiled = compile_rocci_app(app_dir, Progress::default())?;
    if !compiled.failures.is_empty() {
        bail!("template compilation failed");
    }
    Ok(())
}

pub(crate) struct CompiledApp {
    pub(crate) failures: Vec<FailedFile>,
    pub(crate) maps: Vec<MappedModule>,
    pub(crate) profile: crate::profile::ProfileSnapshot,
    pub(crate) inspect_pages: Vec<crate::inspect::InspectPage>,
}

pub(crate) fn compile_rocci_app(app_dir: &Path, progress: Progress) -> Result<CompiledApp> {
    let mut rec = crate::profile::SpanRecorder::new();
    let mut failures = Vec::new();
    let mut maps = Vec::new();
    let mut inspect_pages = Vec::new();
    let inputs = discover_rocci(app_dir)?;
    let templates_started = Instant::now();
    progress.step(logs::run_phase_start(
        "templates",
        &format!("modules={}", inputs.len()),
    ));
    for input in inputs {
        let src = rec.span("read", || {
            fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))
        })?;
        let name = input.display().to_string();
        let compiled = compile_source(&name, &src, &LowerOptions::default())?;
        rec.push(
            "parse",
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            None,
        );
        rec.push("generate", compiled.timings.lower_ms, None);
        progress.detail(logs::run_module_detail(
            &name,
            compiled.timings.parse_ms + compiled.timings.validate_ms,
            compiled.timings.lower_ms,
        ));
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
    progress.step(logs::run_phase_done(
        "templates",
        templates_started.elapsed().as_millis(),
        "",
    ));
    Ok(CompiledApp {
        failures,
        maps,
        profile: rec.finish(),
        inspect_pages,
    })
}

pub(crate) struct CompiledSource {
    pub(crate) roc: String,
    pub(crate) state_type: Option<String>,
    pub(crate) init: Option<rocci_template::InitInfo>,
    pub(crate) lives: Vec<rocci_template::LiveInfo>,
    pub(crate) routes: Vec<rocci_template::RouteInfo>,
    pub(crate) failed: bool,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) segments: Vec<Segment>,
    pub(crate) local_assets: Vec<String>,
    pub(crate) styles: Vec<String>,
    pub(crate) timings: rocci_template::CompileTimings,
    pub(crate) inspect: crate::inspect::InspectPage,
}

pub(crate) fn compile_source(
    name: &str,
    src: &str,
    lower: &LowerOptions,
) -> Result<CompiledSource> {
    let source = SourceFile::new(name, src);
    let compiled = compile(source, lower);
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
        lives: compiled.lives,
        routes: compiled.routes,
        failed,
        diagnostics: compiled.diagnostics,
        segments: compiled.segments,
        local_assets: Vec::new(),
        styles: compiled
            .styles
            .iter()
            .map(|style| style.css.clone())
            .collect(),
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
    let window = config.windows.first();
    let title = window
        .map(|window| window.title.as_str())
        .unwrap_or(&config.app.name)
        .to_string();
    let width = window.map(|window| window.width).unwrap_or(1040.0);
    let height = window.map(|window| window.height).unwrap_or(760.0);
    let path = window.map(|window| window.url.as_str()).unwrap_or("/");
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let url = format!("http://127.0.0.1:{port}{path}");

    let mut cmd = Command::new(&server);
    cmd.current_dir(&app_dir)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to start {}", server.display()))?;
    if let Err(err) = serve::wait_for_server(&mut child, port, crate::logs::Progress::default()) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }

    println!("{}", style::serving(&app_dir.display().to_string(), &url));
    crate::serve::emit_preview_ready(&url);
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
        live_reload: true,
        ..PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    let _ = child.kill();
    let _ = child.wait();
    preview_result
}
