use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    ComponentInfo, Document, FixtureInfo, LowerOptions, SourceFile, TemplateBlock, TemplateItem,
    compile, format_diagnostic,
};

use crate::datastar_asset;
use crate::error_page;
use crate::logs::Progress;
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::serve;
use crate::style;

pub(crate) const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";
const BROWSER_ROCCI: &str = include_str!("../../templates/browser/Browser.rocci");
const QUERY_ROC: &str = include_str!("../../templates/browser/Query.roc");
const BROWSER_CSS: &str = include_str!("../../templates/browser/assets/app.css");

const RESERVED_ROC: &[&str] = &[
    "main.roc",
    "Html.roc",
    "Datastar.roc",
    "Query.roc",
    "Catalog.roc",
    "Preview.roc",
    "Browser.roc",
];

pub fn browse(
    roots: &[PathBuf],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    verbose: bool,
    public: bool,
) -> Result<()> {
    let files = discover_rocci_files(roots)?;
    if files.is_empty() {
        bail!("no .rocci files found");
    }

    let compiled_modules = compile_modules(&files)?;
    let workspace = crate::driver::TempDir::create("browse")?;
    let mut copied = HashMap::new();
    let skip_names = reserved_and_generated(&compiled_modules);

    for module in &compiled_modules {
        let src_dir = module.path.parent().unwrap_or_else(|| Path::new("."));
        copy_sibling_roc(src_dir, &workspace.path, &skip_names, &mut copied)?;
    }
    crate::driver::rewrite_workspace_runtime_imports(&workspace.path)?;

    let mut available = copied_module_names(&copied);
    available.insert("Html".to_string());
    available.insert("Datastar".to_string());
    for module in &compiled_modules {
        available.insert(module.type_name.clone());
    }

    for module in &compiled_modules {
        fs::write(
            workspace.path.join(format!("{}.roc", module.type_name)),
            wrap_type_module(
                &crate::dispatch::rewrite_runtime_imports_for_pin(&module.roc, None),
                &module.type_name,
            ),
        )
        .with_context(|| format!("failed to write {}.roc", module.type_name))?;
    }

    let mut groups = analyze_modules(&compiled_modules, &available);
    attach_fixtures(&compiled_modules, &mut groups);
    fs::write(
        workspace.path.join("Catalog.roc"),
        generate_catalog_roc(&groups),
    )
    .context("failed to write Catalog.roc")?;
    fs::write(
        workspace.path.join("Preview.roc"),
        generate_preview_roc(&groups),
    )
    .context("failed to write Preview.roc")?;
    fs::write(workspace.path.join("Query.roc"), QUERY_ROC).context("failed to write Query.roc")?;
    fs::write(workspace.path.join("Browser.rocci"), BROWSER_ROCCI)
        .context("failed to write Browser.rocci")?;

    compile_browser(&workspace.path)?;

    let assets = workspace.path.join("assets");
    fs::create_dir_all(&assets)?;
    fs::write(assets.join("app.css"), BROWSER_CSS)?;
    datastar_asset::stage_into(&assets, datastar_asset::DEFAULT_VERSION)?;
    datastar_asset::print_hint(datastar_asset::DEFAULT_VERSION);

    fs::write(
        workspace.path.join("main.roc"),
        generate_main_roc_with_pin(&crate::dispatch::platform_pin_for_app_dir(&workspace.path)),
    )
    .context("failed to write main.roc")?;

    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}/");
    let invocation = crate::driver::RocInvocation {
        program: "roc",
        app_dir: workspace.path.clone(),
        roc_file: PathBuf::from("main.roc"),
        args: Vec::new(),
    };
    Progress::from_verbose(verbose).step(crate::logs::run_phase_start("roc", ""));
    let cmd = match crate::driver::prepare_roc_process(&invocation, port, public, verbose) {
        Ok(cmd) => cmd,
        Err(err) => {
            let html = error_page::render_roc_compile_error(&format!("{err:#}"), &[]);
            return serve::serve_html(
                port,
                500,
                &html,
                "rocci browse",
                no_window,
                live_reload,
                public,
            );
        }
    };
    let (mut child, mut tee) = serve::spawn_roc(cmd)?;

    match serve::wait_for_roc(
        &mut child,
        &mut tee,
        port,
        "/",
        Progress::from_verbose(verbose),
    )? {
        serve::RocStart::Ready => {}
        serve::RocStart::Failed(output) => {
            let html = error_page::render_roc_compile_error(&output, &[]);
            return serve::serve_html(
                port,
                500,
                &html,
                "rocci browse",
                no_window,
                live_reload,
                public,
            );
        }
    }

    let count: usize = groups.iter().map(|group| group.entries.len()).sum();
    println!(
        "{}",
        style::browsing(
            &format!("{count} components from {} file(s)", files.len()),
            &url
        )
    );
    serve::with_window(&mut child, &url, "rocci browse", no_window, live_reload)
}

pub(crate) fn discover_rocci_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in roots {
        let path = abs(root)?;
        if path.is_file() {
            if path.extension().and_then(|ext| ext.to_str()) != Some("rocci") {
                bail!("{} is not a .rocci file", path.display());
            }
            files.push(path);
        } else if path.is_dir() {
            walk_dir(&path, &mut files)?;
        } else {
            bail!("no such path: {}", path.display());
        }
    }
    files.sort();
    files.dedup();

    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    for file in &files {
        let name = type_name_from_path(file);
        if let Some(prev) = seen.get(&name)
            && prev != file
        {
            bail!(
                "duplicate module name `{name}` from {} and {}",
                prev.display(),
                file.display()
            );
        }
        seen.insert(name, file.clone());
    }
    Ok(files)
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if skip_dir_name(&name) {
                continue;
            }
            walk_dir(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rocci") {
            files.push(path);
        }
    }
    Ok(())
}

fn skip_dir_name(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

fn abs(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

struct CompiledModule {
    path: PathBuf,
    type_name: String,
    roc: String,
    document: Document,
    components: Vec<ComponentInfo>,
    fixtures: Vec<FixtureInfo>,
    src: String,
}

fn compile_modules(files: &[PathBuf]) -> Result<Vec<CompiledModule>> {
    let mut modules = Vec::new();
    for path in files {
        let src = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let name = path.display().to_string();
        let source = SourceFile::new(&name, &src);
        let compiled = compile(source, &LowerOptions::default());
        for diagnostic in &compiled.diagnostics {
            eprintln!("{}", format_diagnostic(source, diagnostic));
        }
        if compiled.has_errors() {
            eprintln!(
                "{}",
                style::warning(&format!(
                    "skipping {} (template compilation failed)",
                    path.display()
                ))
            );
            continue;
        }
        if compiled.components.is_empty() {
            continue;
        }
        modules.push(CompiledModule {
            path: path.clone(),
            type_name: type_name_from_path(path),
            roc: compiled.roc,
            document: compiled.document,
            components: compiled.components,
            fixtures: compiled.fixtures,
            src,
        });
    }
    if modules.is_empty() {
        bail!("no components found in the given roots");
    }
    Ok(modules)
}

fn reserved_and_generated(modules: &[CompiledModule]) -> HashSet<String> {
    let mut skip: HashSet<String> = RESERVED_ROC.iter().map(|name| name.to_string()).collect();
    for module in modules {
        skip.insert(format!("{}.roc", module.type_name));
    }
    skip
}

fn copy_sibling_roc(
    src_dir: &Path,
    dest: &Path,
    skip: &HashSet<String>,
    copied: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    if !src_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("roc") {
            continue;
        }
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().into_owned();
        if skip.contains(&name) {
            continue;
        }
        if let Some(prev) = copied.get(&name) {
            let prev_bytes = fs::read(prev).unwrap_or_default();
            let next_bytes = fs::read(&path).unwrap_or_default();
            if prev_bytes != next_bytes {
                eprintln!(
                    "{}",
                    style::warning(&format!(
                        "skipping {} from {} (already copied from {})",
                        name,
                        path.display(),
                        prev.display()
                    ))
                );
            }
            continue;
        }
        fs::copy(&path, dest.join(&file_name))
            .with_context(|| format!("failed to copy {}", path.display()))?;
        copied.insert(name, path);
    }
    Ok(())
}

fn copied_module_names(copied: &HashMap<String, PathBuf>) -> HashSet<String> {
    copied
        .keys()
        .filter_map(|name| Path::new(name).file_stem()?.to_str().map(str::to_string))
        .collect()
}

fn compile_browser(workspace: &Path) -> Result<()> {
    let input = workspace.join("Browser.rocci");
    let src = fs::read_to_string(&input).context("failed to read Browser.rocci")?;
    let source = SourceFile::new("Browser.rocci", &src);
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    if compiled.has_errors() {
        bail!("browser template compilation failed");
    }
    fs::write(
        workspace.join("Browser.roc"),
        wrap_type_module(
            &crate::dispatch::rewrite_runtime_imports_for_pin(&compiled.roc, None),
            "Browser",
        ),
    )
    .context("failed to write Browser.roc")?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParamKind {
    Str,
    I64,
    U64,
    F64,
    Dec,
    Bool,
    BodyHtml,
}

impl ParamKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Str => "str",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::F64 => "f64",
            Self::Dec => "dec",
            Self::Bool => "bool",
            Self::BodyHtml => "body",
        }
    }

    pub(crate) fn from_annotation(ty: &str) -> Option<Self> {
        match ty.trim() {
            "Str" => Some(Self::Str),
            "I64" => Some(Self::I64),
            "U64" => Some(Self::U64),
            "F64" => Some(Self::F64),
            "Dec" => Some(Self::Dec),
            "Bool" => Some(Self::Bool),
            _ => None,
        }
    }

    pub(crate) fn zero_roc(&self) -> &'static str {
        match self {
            Self::Str | Self::BodyHtml => "\"\"",
            Self::I64 => "0.I64",
            Self::U64 => "0.U64",
            Self::F64 | Self::Dec => "0.0",
            Self::Bool => "False",
        }
    }

    pub(crate) fn zero_display(&self) -> &'static str {
        match self {
            Self::Str | Self::BodyHtml => "",
            Self::I64 | Self::U64 | Self::F64 | Self::Dec => "0",
            Self::Bool => "false",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BrowseParam {
    pub name: String,
    pub required: bool,
    pub kind: Option<ParamKind>,
    pub reason: String,
    pub default_roc: Option<String>,
    pub default_display: String,
    pub is_body: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BrowseFixture {
    pub name: String,
    pub module: String,
    pub value: String,
    pub scalars: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogEntry {
    pub id: String,
    pub module: String,
    pub name: String,
    pub file: String,
    pub previewable: bool,
    pub reason: String,
    pub full_document: bool,
    pub first_param_is_record: bool,
    pub params: Vec<BrowseParam>,
    pub fixtures: Vec<BrowseFixture>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModuleGroup {
    pub module: String,
    pub file: String,
    pub import_ok: bool,
    pub entries: Vec<CatalogEntry>,
}

fn analyze_modules(modules: &[CompiledModule], available: &HashSet<String>) -> Vec<ModuleGroup> {
    modules
        .iter()
        .map(|module| analyze_module(module, available))
        .collect()
}

fn analyze_module(module: &CompiledModule, available: &HashSet<String>) -> ModuleGroup {
    let missing = missing_imports(&module.roc, available);
    let import_ok = missing.is_empty();
    let import_reason = if import_ok {
        String::new()
    } else {
        format!("missing import {}", missing.join(", "))
    };
    let file = module.path.display().to_string();
    let mut entries: Vec<CatalogEntry> = module
        .components
        .iter()
        .map(|info| {
            catalog_entry(
                &module.src,
                &module.type_name,
                &file,
                info,
                &module.document,
            )
        })
        .collect();
    propagate_passthrough(&module.src, &module.document, &mut entries);
    if !import_ok {
        for entry in &mut entries {
            entry.previewable = false;
            entry.reason = import_reason.clone();
        }
    }
    ModuleGroup {
        module: module.type_name.clone(),
        file,
        import_ok,
        entries,
    }
}

fn catalog_entry(
    src: &str,
    type_name: &str,
    file: &str,
    info: &ComponentInfo,
    document: &Document,
) -> CatalogEntry {
    let params = infer_params(src, info, document);
    let missing: Vec<&str> = params
        .iter()
        .filter(|param| param.required && param.kind.is_none())
        .map(|param| param.name.as_str())
        .collect();
    let previewable = missing.is_empty();
    let reason = if previewable {
        String::new()
    } else {
        format!(
            "cannot preview: {} {}",
            if missing.len() == 1 {
                "param"
            } else {
                "params"
            },
            missing
                .iter()
                .map(|name| {
                    let why = params
                        .iter()
                        .find(|param| param.name == *name)
                        .map(|param| param.reason.as_str())
                        .unwrap_or("unsupported");
                    format!("`{name}` ({why})")
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    CatalogEntry {
        id: format!("{type_name}.{}", info.name),
        module: type_name.to_string(),
        name: info.name.clone(),
        file: file.to_string(),
        previewable,
        reason,
        full_document: component_is_html_document(document, &info.name),
        first_param_is_record: info.first_param_is_record,
        params,
        fixtures: Vec::new(),
    }
}

fn attach_fixtures(modules: &[CompiledModule], groups: &mut [ModuleGroup]) {
    let fixtures: Vec<(&str, &FixtureInfo)> = modules
        .iter()
        .flat_map(|module| {
            module
                .fixtures
                .iter()
                .map(|fixture| (module.type_name.as_str(), fixture))
        })
        .collect();
    for (module_name, fixture) in fixtures {
        for group in groups.iter_mut() {
            for entry in &mut group.entries {
                if !fixture_targets(entry, module_name, fixture) {
                    continue;
                }
                entry.fixtures.push(BrowseFixture {
                    name: fixture.name.clone(),
                    module: module_name.to_string(),
                    value: fixture.value.clone(),
                    scalars: fixture_scalars(&fixture.value),
                });
            }
        }
    }
    for group in groups {
        if !group.import_ok {
            continue;
        }
        for entry in &mut group.entries {
            if entry.fixtures.is_empty() {
                continue;
            }
            entry.previewable = true;
            entry.reason.clear();
        }
    }
}

fn fixture_targets(entry: &CatalogEntry, module_name: &str, fixture: &FixtureInfo) -> bool {
    if fixture.target.contains('.') {
        fixture.target == entry.id
    } else {
        module_name == entry.module && fixture.target == entry.name
    }
}

fn can_preview_from_form(entry: &CatalogEntry) -> bool {
    entry
        .params
        .iter()
        .all(|param| !param.required || param.kind.is_some())
}

fn refresh_previewable(entry: &mut CatalogEntry) {
    let missing: Vec<String> = entry
        .params
        .iter()
        .filter(|param| param.required && param.kind.is_none())
        .map(|param| format!("`{}` ({})", param.name, param.reason))
        .collect();
    entry.previewable = missing.is_empty();
    entry.reason = if entry.previewable {
        String::new()
    } else {
        format!(
            "cannot preview: {} {}",
            if missing.len() == 1 {
                "param"
            } else {
                "params"
            },
            missing.join(", ")
        )
    };
}

fn propagate_passthrough(src: &str, document: &Document, entries: &mut [CatalogEntry]) {
    for _ in 0..entries.len() {
        let known: HashMap<(String, String), ParamKind> = entries
            .iter()
            .flat_map(|entry| {
                entry.params.iter().filter_map(|param| {
                    param
                        .kind
                        .clone()
                        .map(|kind| ((entry.name.clone(), param.name.clone()), kind))
                })
            })
            .collect();
        let mut changed = false;
        for entry in entries.iter_mut() {
            let Some(decl) = find_decl(document, &entry.name) else {
                continue;
            };
            for param in &mut entry.params {
                if param.kind.is_some() {
                    continue;
                }
                if let Some(kind) = passthrough_kind(src, &decl.body, &param.name, &known) {
                    param.kind = Some(kind.clone());
                    param.reason.clear();
                    if param.default_display.is_empty() {
                        param.default_display = kind.zero_display().to_string();
                    }
                    changed = true;
                }
            }
            refresh_previewable(entry);
        }
        if !changed {
            break;
        }
    }
}

fn passthrough_kind(
    src: &str,
    body: &TemplateBlock,
    param: &str,
    known: &HashMap<(String, String), ParamKind>,
) -> Option<ParamKind> {
    let mut found = None;
    find_passthrough(src, body, param, known, &mut found);
    found
}

fn find_passthrough(
    src: &str,
    body: &TemplateBlock,
    param: &str,
    known: &HashMap<(String, String), ParamKind>,
    found: &mut Option<ParamKind>,
) {
    for item in &body.items {
        find_passthrough_item(src, item, param, known, found);
    }
}

fn find_passthrough_item(
    src: &str,
    item: &TemplateItem,
    param: &str,
    known: &HashMap<(String, String), ParamKind>,
    found: &mut Option<ParamKind>,
) {
    match item {
        TemplateItem::Element(el) => {
            for child in &el.children {
                find_passthrough_item(src, child, param, known, found);
            }
        }
        TemplateItem::ComponentCall(call) => {
            for attr in &call.attrs {
                if let rocci_template::AttrValue::Expr { expr } = attr.value
                    && expr.of(src).trim() == param
                    && let Some(kind) =
                        known.get(&(call.path.roc_name.clone(), attr.name.name.clone()))
                {
                    *found = Some(kind.clone());
                }
            }
            if let Some(children) = &call.children {
                for child in children {
                    find_passthrough_item(src, child, param, known, found);
                }
            }
        }
        TemplateItem::Fragment(frag) => {
            for child in &frag.children {
                find_passthrough_item(src, child, param, known, found);
            }
        }
        TemplateItem::If(dir) => {
            find_passthrough(src, &dir.then_body, param, known, found);
            for (_, body) in &dir.else_ifs {
                find_passthrough(src, body, param, known, found);
            }
            if let Some(body) = &dir.else_body {
                find_passthrough(src, body, param, known, found);
            }
        }
        TemplateItem::For(dir) => find_passthrough(src, &dir.body, param, known, found),
        TemplateItem::Match(dir) => {
            for arm in &dir.arms {
                find_passthrough_item(src, &arm.value, param, known, found);
            }
        }
        TemplateItem::Interpolation(_)
        | TemplateItem::Text(_)
        | TemplateItem::Let(_)
        | TemplateItem::Css(_) => {}
    }
}

mod codegen;
mod infer;
use codegen::*;
use infer::*;

#[cfg(test)]
mod tests;
