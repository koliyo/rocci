use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    ComponentDecl, ComponentInfo, Document, FixtureInfo, LowerOptions, ModuleItem, SourceFile,
    TemplateBlock, TemplateItem, compile, format_diagnostic,
};

use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::serve;

const PLATFORM: &str = "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst";
const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";
const HTML_STUB: &str = include_str!("../../../examples/counter/Html.roc");
const BROWSER_ROCCI: &str = include_str!("../templates/browser/Browser.rocci");
const QUERY_ROC: &str = include_str!("../templates/browser/Query.roc");
const BROWSER_CSS: &str = include_str!("../templates/browser/assets/app.css");
const DATASTAR_JS: &[u8] = include_bytes!("../../../assets/datastar.js");

const RESERVED_ROC: &[&str] = &[
    "main.roc",
    "Html.roc",
    "Query.roc",
    "Catalog.roc",
    "Preview.roc",
    "Browser.roc",
];

pub fn browse(roots: &[PathBuf], no_window: bool, port: serve::PortArg) -> Result<()> {
    let files = discover_rocci_files(roots)?;
    if files.is_empty() {
        bail!("no .rocci files found");
    }

    let compiled_modules = compile_modules(&files)?;
    let workspace = TempDir::create()?;
    let mut copied = HashMap::new();
    let skip_names = reserved_and_generated(&compiled_modules);

    for module in &compiled_modules {
        let src_dir = module.path.parent().unwrap_or_else(|| Path::new("."));
        copy_sibling_roc(src_dir, &workspace.path, &skip_names, &mut copied)?;
    }

    if !workspace.path.join("Html.roc").is_file() {
        fs::write(workspace.path.join("Html.roc"), HTML_STUB)
            .context("failed to write Html.roc stub")?;
    }

    let mut available = copied_module_names(&copied);
    available.insert("Html".to_string());
    for module in &compiled_modules {
        available.insert(module.type_name.clone());
    }

    for module in &compiled_modules {
        fs::write(
            workspace.path.join(format!("{}.roc", module.type_name)),
            wrap_type_module(&module.roc, &module.type_name),
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
    fs::write(assets.join("datastar.js"), DATASTAR_JS)?;

    fs::write(workspace.path.join("main.roc"), generate_main_roc())
        .context("failed to write main.roc")?;

    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}/");
    let mut child = Command::new("roc")
        .arg("main.roc")
        .current_dir(&workspace.path)
        .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start `roc`; is it on PATH?")?;

    if let Err(err) = serve::wait_for_server(&mut child, port) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }

    let count: usize = groups.iter().map(|group| group.entries.len()).sum();
    println!(
        "Browsing {count} components from {} file(s) at {url}",
        files.len()
    );
    serve::with_window(&mut child, &url, "rocci browse", no_window)
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
                "warning: skipping {} (template compilation failed)",
                path.display()
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
                    "warning: skipping {} from {} (already copied from {})",
                    name,
                    path.display(),
                    prev.display()
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
        wrap_type_module(&compiled.roc, "Browser"),
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
    fn as_str(&self) -> &'static str {
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

    fn from_annotation(ty: &str) -> Option<Self> {
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

    fn zero_roc(&self) -> &'static str {
        match self {
            Self::Str | Self::BodyHtml => "\"\"",
            Self::I64 => "0.I64",
            Self::U64 => "0.U64",
            Self::F64 | Self::Dec => "0.0",
            Self::Bool => "False",
        }
    }

    fn zero_display(&self) -> &'static str {
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
                {
                    if let Some(kind) =
                        known.get(&(call.path.roc_name.clone(), attr.name.name.clone()))
                    {
                        *found = Some(kind.clone());
                    }
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

pub(crate) fn infer_params(
    src: &str,
    info: &ComponentInfo,
    document: &Document,
) -> Vec<BrowseParam> {
    let decl = find_decl(document, &info.name);
    info.param_names
        .iter()
        .map(|name| {
            let required = !info.optional_params.iter().any(|param| param == name);
            let is_body = info.body_params.iter().any(|param| param == name);
            let default_roc = info
                .param_defaults
                .iter()
                .find(|(param, _)| param == name)
                .map(|(_, value)| value.clone());
            let annotation = info
                .param_types
                .iter()
                .find(|(param, _)| param == name)
                .map(|(_, ty)| ty.as_str());
            let inferred = infer_one(
                name,
                is_body,
                annotation,
                default_roc.as_deref(),
                src,
                decl.map(|decl| &decl.body),
            );
            let (kind, reason) = match inferred {
                Inferred::Scalar(kind) => (Some(kind), String::new()),
                Inferred::Unsupported(reason) => (None, reason),
            };
            let default_display = match &kind {
                Some(kind) => display_default(kind, default_roc.as_deref()),
                None => String::new(),
            };
            BrowseParam {
                name: name.clone(),
                required,
                kind,
                reason,
                default_roc,
                default_display,
                is_body,
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Inferred {
    Scalar(ParamKind),
    Unsupported(String),
}

fn infer_one(
    name: &str,
    is_body: bool,
    annotation: Option<&str>,
    default_roc: Option<&str>,
    src: &str,
    body: Option<&TemplateBlock>,
) -> Inferred {
    if is_body {
        return Inferred::Scalar(ParamKind::BodyHtml);
    }
    if let Some(ty) = annotation {
        return match ParamKind::from_annotation(ty) {
            Some(kind) => Inferred::Scalar(kind),
            None => Inferred::Unsupported(format!("type `{ty}`")),
        };
    }
    if let Some(default) = default_roc {
        match infer_from_default(default) {
            Inferred::Scalar(kind) => return Inferred::Scalar(kind),
            Inferred::Unsupported(_) => {}
        }
    }
    if let Some(body) = body
        && let Some(inferred) = infer_from_usage(src, body, name)
    {
        return inferred;
    }
    Inferred::Unsupported("no scalar type".into())
}

fn infer_from_default(expr: &str) -> Inferred {
    let trimmed = expr.trim();
    if trimmed == "Bool.true" || trimmed == "Bool.false" || trimmed == "True" || trimmed == "False"
    {
        return Inferred::Scalar(ParamKind::Bool);
    }
    if is_i64(trimmed) {
        return Inferred::Scalar(ParamKind::I64);
    }
    if is_float(trimmed) {
        return Inferred::Scalar(ParamKind::F64);
    }
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return Inferred::Scalar(ParamKind::Str);
    }
    if trimmed.starts_with('[') {
        return Inferred::Unsupported("list".into());
    }
    if trimmed.starts_with('{') {
        return Inferred::Unsupported("record".into());
    }
    if trimmed
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        return Inferred::Unsupported(format!("tag `{trimmed}`"));
    }
    Inferred::Unsupported(format!("default `{trimmed}`"))
}

fn is_i64(value: &str) -> bool {
    let rest = value.strip_prefix('-').unwrap_or(value);
    !rest.is_empty() && rest.bytes().all(|ch| ch.is_ascii_digit())
}

fn is_float(value: &str) -> bool {
    let rest = value.strip_prefix('-').unwrap_or(value);
    if rest.is_empty() || rest == "." || !rest.contains('.') {
        return false;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for ch in rest.chars() {
        if ch == '.' {
            if seen_dot {
                return false;
            }
            seen_dot = true;
        } else if ch.is_ascii_digit() {
            seen_digit = true;
        } else {
            return false;
        }
    }
    seen_digit
}

fn display_default(kind: &ParamKind, default_roc: Option<&str>) -> String {
    match default_roc {
        Some(value) => display_roc_literal(value),
        None => kind.zero_display().to_string(),
    }
}

fn display_roc_literal(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "Bool.true" || trimmed == "True" {
        return "true".to_string();
    }
    if trimmed == "Bool.false" || trimmed == "False" {
        return "false".to_string();
    }
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return unescape_roc_string(&trimmed[1..trimmed.len() - 1]);
    }
    trimmed.to_string()
}

fn unescape_roc_string(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn find_decl<'a>(document: &'a Document, name: &str) -> Option<&'a ComponentDecl> {
    document.items.iter().find_map(|item| match item {
        ModuleItem::Component(decl) if decl.name.name == name => Some(decl),
        _ => None,
    })
}

fn component_is_html_document(document: &Document, roc_name: &str) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Component(decl) if decl.name.name == roc_name => {
            matches!(
                decl.body.items.iter().find(|item| !item.is_preamble()),
                Some(TemplateItem::Element(el)) if el.name.name == "html"
            )
        }
        _ => false,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UsageHint {
    Str,
    Bool,
    I64,
    Record,
    List,
    Tag,
}

fn infer_from_usage(src: &str, body: &TemplateBlock, param: &str) -> Option<Inferred> {
    let mut hints = Vec::new();
    walk_block(src, body, param, &mut hints);
    if hints.is_empty() {
        return None;
    }
    if hints.contains(&UsageHint::List) {
        return Some(Inferred::Unsupported("list".into()));
    }
    if hints.contains(&UsageHint::Tag) {
        return Some(Inferred::Unsupported("tag".into()));
    }
    if hints.contains(&UsageHint::Record) {
        return Some(Inferred::Unsupported("record".into()));
    }
    if hints.contains(&UsageHint::I64) {
        return Some(Inferred::Scalar(ParamKind::I64));
    }
    if hints.contains(&UsageHint::Bool) && !hints.contains(&UsageHint::Str) {
        return Some(Inferred::Scalar(ParamKind::Bool));
    }
    if hints.contains(&UsageHint::Str) {
        return Some(Inferred::Scalar(ParamKind::Str));
    }
    if hints.contains(&UsageHint::Bool) {
        return Some(Inferred::Scalar(ParamKind::Bool));
    }
    None
}

fn walk_block(src: &str, body: &TemplateBlock, param: &str, hints: &mut Vec<UsageHint>) {
    for item in &body.items {
        walk_item(src, item, param, hints);
    }
}

fn walk_item(src: &str, item: &TemplateItem, param: &str, hints: &mut Vec<UsageHint>) {
    match item {
        TemplateItem::Element(el) => {
            for attr in &el.attrs {
                match attr.value {
                    rocci_template::AttrValue::Expr { expr } => {
                        classify_expr(param, expr.of(src), hints);
                    }
                    rocci_template::AttrValue::Action { args, .. } => {
                        classify_expr(param, args.of(src), hints);
                    }
                    _ => {}
                }
            }
            for child in &el.children {
                walk_item(src, child, param, hints);
            }
        }
        TemplateItem::ComponentCall(call) => {
            for attr in &call.attrs {
                match attr.value {
                    rocci_template::AttrValue::Expr { expr } => {
                        classify_expr(param, expr.of(src), hints);
                    }
                    rocci_template::AttrValue::Action { args, .. } => {
                        classify_expr(param, args.of(src), hints);
                    }
                    _ => {}
                }
            }
            if let Some(children) = &call.children {
                for child in children {
                    walk_item(src, child, param, hints);
                }
            }
        }
        TemplateItem::Fragment(frag) => {
            for child in &frag.children {
                walk_item(src, child, param, hints);
            }
        }
        TemplateItem::Interpolation(interp) => {
            let expr = interp.expr.of(src).trim();
            if expr == param {
                hints.push(UsageHint::Str);
            } else {
                classify_expr(param, expr, hints);
            }
        }
        TemplateItem::If(dir) => {
            let cond = dir.condition.of(src).trim();
            if cond == param || cond == format!("!{param}") {
                hints.push(UsageHint::Bool);
            } else {
                classify_expr(param, cond, hints);
            }
            walk_block(src, &dir.then_body, param, hints);
            for (cond, body) in &dir.else_ifs {
                classify_expr(param, cond.of(src), hints);
                walk_block(src, body, param, hints);
            }
            if let Some(body) = &dir.else_body {
                walk_block(src, body, param, hints);
            }
        }
        TemplateItem::For(dir) => {
            let collection = dir.collection.of(src).trim();
            if collection == param {
                hints.push(UsageHint::List);
            } else {
                classify_expr(param, collection, hints);
            }
            walk_block(src, &dir.body, param, hints);
        }
        TemplateItem::Match(dir) => {
            let scrutinee = dir.scrutinee.of(src).trim();
            if scrutinee == param {
                hints.push(UsageHint::Tag);
            } else {
                classify_expr(param, scrutinee, hints);
            }
            for arm in &dir.arms {
                walk_item(src, &arm.value, param, hints);
            }
        }
        TemplateItem::Let(dir) => {
            classify_expr(param, dir.expr.of(src), hints);
        }
        TemplateItem::Text(_) | TemplateItem::Css(_) => {}
    }
}

fn classify_expr(param: &str, expr: &str, hints: &mut Vec<UsageHint>) {
    let expr = expr.trim();
    if expr.is_empty() || expr == param {
        return;
    }
    if expr == format!("{param}.to_str()")
        || expr == format!("Num.toStr({param})")
        || expr == format!("Num.to_str({param})")
    {
        hints.push(UsageHint::I64);
        return;
    }
    if is_list_expr(param, expr) {
        hints.push(UsageHint::List);
        return;
    }
    if is_record_expr(param, expr) {
        hints.push(UsageHint::Record);
    }
}

fn is_record_expr(param: &str, expr: &str) -> bool {
    let expr = expr.strip_prefix('!').unwrap_or(expr);
    expr.starts_with(param)
        && expr[param.len()..].starts_with('.')
        && expr != format!("{param}.to_str()")
        && expr != format!("{param}.toStr()")
}

fn is_list_expr(param: &str, expr: &str) -> bool {
    [
        format!("List.is_empty({param})"),
        format!("List.isEmpty({param})"),
        format!("List.len({param})"),
        format!("List.map({param}"),
        format!("List.keep_if({param}"),
        format!("List.fold({param}"),
        format!("List.concat({param}"),
        format!("List.get({param}"),
        format!("List.append({param}"),
    ]
    .iter()
    .any(|needle| expr.contains(needle.as_str()))
}

fn roc_imports(src: &str) -> Vec<String> {
    src.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("import ")
                .map(|rest| rest.split_whitespace().next().unwrap_or(rest).to_string())
        })
        .collect()
}

fn missing_imports(src: &str, available: &HashSet<String>) -> Vec<String> {
    roc_imports(src)
        .into_iter()
        .filter(|name| {
            !name.starts_with("pf.") && !name.starts_with("http.") && !available.contains(name)
        })
        .collect()
}

fn form_params(entry: &CatalogEntry) -> Vec<&BrowseParam> {
    entry
        .params
        .iter()
        .filter(|param| param.kind.is_some())
        .collect()
}

fn fixture_scalars(value: &str) -> Vec<(String, String)> {
    top_level_fields(value)
        .into_iter()
        .filter_map(|(name, expr)| fixture_scalar_display(&expr).map(|display| (name, display)))
        .collect()
}

fn fixture_scalar_display(expr: &str) -> Option<String> {
    let trimmed = strip_num_suffix(expr.trim());
    if trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed.starts_with('(')
        || trimmed.contains('(')
    {
        return None;
    }
    match infer_from_default(trimmed) {
        Inferred::Scalar(
            ParamKind::Str
            | ParamKind::I64
            | ParamKind::U64
            | ParamKind::F64
            | ParamKind::Dec
            | ParamKind::Bool,
        ) => Some(display_roc_literal(trimmed)),
        _ => None,
    }
}

fn strip_num_suffix(value: &str) -> &str {
    for suffix in [".I64", ".U64", ".F64", ".Dec"] {
        if let Some(rest) = value.strip_suffix(suffix) {
            return rest;
        }
    }
    value
}

fn record_has_field(record: &str, name: &str) -> bool {
    top_level_fields(record)
        .iter()
        .any(|(field, _)| field == name)
}

fn top_level_fields(record: &str) -> Vec<(String, String)> {
    let trimmed = record.trim();
    let Some(inner) = trimmed.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return Vec::new();
    };
    split_top_level(inner, ',')
        .into_iter()
        .filter_map(|part| {
            let (name, value) = split_top_level_once(part, ':')?;
            let name = name.trim();
            if name.is_empty() {
                None
            } else {
                Some((name.to_string(), value.trim().to_string()))
            }
        })
        .collect()
}

fn split_top_level(inner: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut part_start = 0;
    let mut chars = inner.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '"' => {
                while let Some((_, next)) = chars.next() {
                    if next == '\\' {
                        chars.next();
                    } else if next == '"' {
                        break;
                    }
                }
            }
            c if c == sep && depth == 0 => {
                parts.push(&inner[part_start..i]);
                part_start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&inner[part_start..]);
    parts
}

fn split_top_level_once(part: &str, sep: char) -> Option<(&str, &str)> {
    let mut depth: usize = 0;
    let mut chars = part.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '"' => {
                while let Some((_, next)) = chars.next() {
                    if next == '\\' {
                        chars.next();
                    } else if next == '"' {
                        break;
                    }
                }
            }
            c if c == sep && depth == 0 => {
                return Some((&part[..i], &part[i + c.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn generate_catalog_roc(groups: &[ModuleGroup]) -> String {
    let mut out = String::from("Catalog := [].{\n    groups = [\n");
    for (index, group) in groups.iter().enumerate() {
        out.push_str("        {\n");
        out.push_str(&format!(
            "            mod_name: {},\n",
            roc_string(&group.module)
        ));
        out.push_str("            entries: [\n");
        for (entry_index, entry) in group.entries.iter().enumerate() {
            out.push_str(&catalog_entry_roc(entry, "                "));
            if entry_index + 1 != group.entries.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("            ],\n        }");
        if index + 1 != groups.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(
        "    ]\n\n    find = |id|\n        List.fold(\n            groups,\n            Err(NotFound),\n            |acc, group|\n                match acc {\n                    Ok(found) => Ok(found)\n                    Err(err) =>\n                        match List.get(List.keep_if(group.entries, |entry| entry.id == id), 0) {\n                            Ok(entry) => Ok(entry)\n                            Err(_) => Err(err)\n                        }\n                },\n        )\n}\n",
    );
    out
}

fn catalog_entry_roc(entry: &CatalogEntry, indent: &str) -> String {
    let params = form_params(entry);
    let mut out = format!("{indent}{{\n");
    out.push_str(&format!("{indent}    id: {},\n", roc_string(&entry.id)));
    out.push_str(&format!(
        "{indent}    mod_name: {},\n",
        roc_string(&entry.module)
    ));
    out.push_str(&format!("{indent}    name: {},\n", roc_string(&entry.name)));
    out.push_str(&format!("{indent}    file: {},\n", roc_string(&entry.file)));
    out.push_str(&format!(
        "{indent}    previewable: {},\n",
        roc_bool(entry.previewable)
    ));
    out.push_str(&format!(
        "{indent}    reason: {},\n",
        roc_string(&entry.reason)
    ));
    if params.is_empty() {
        out.push_str(&format!("{indent}    params: [],\n"));
    } else {
        out.push_str(&format!("{indent}    params: [\n"));
        for (index, param) in params.iter().enumerate() {
            out.push_str(&format!(
                "{indent}        {{ name: {}, required: {}, kind: {}, value: {} }}",
                roc_string(&param.name),
                roc_bool(param.required),
                roc_string(param.kind.as_ref().map(ParamKind::as_str).unwrap_or("")),
                roc_string(&param.default_display),
            ));
            if index + 1 != params.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(&format!("{indent}    ],\n"));
    }
    if entry.fixtures.is_empty() {
        out.push_str(&format!("{indent}    fixtures: [],\n"));
    } else {
        out.push_str(&format!("{indent}    fixtures: [\n"));
        for (index, fixture) in entry.fixtures.iter().enumerate() {
            out.push_str(&fixture_roc(fixture, &format!("{indent}        ")));
            if index + 1 != entry.fixtures.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(&format!("{indent}    ],\n"));
    }
    out.push_str(&format!("{indent}}}"));
    out
}

fn fixture_roc(fixture: &BrowseFixture, indent: &str) -> String {
    let mut out = format!("{indent}{{\n");
    out.push_str(&format!(
        "{indent}    name: {},\n",
        roc_string(&fixture.name)
    ));
    out.push_str(&format!(
        "{indent}    source: {},\n",
        roc_string(&fixture.value)
    ));
    if fixture.scalars.is_empty() {
        out.push_str(&format!("{indent}    scalars: [],\n"));
    } else {
        out.push_str(&format!("{indent}    scalars: [\n"));
        for (index, (name, value)) in fixture.scalars.iter().enumerate() {
            out.push_str(&format!(
                "{indent}        {{ name: {}, value: {} }}",
                roc_string(name),
                roc_string(value)
            ));
            if index + 1 != fixture.scalars.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(&format!("{indent}    ],\n"));
    }
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn generate_preview_roc(groups: &[ModuleGroup]) -> String {
    let mut imports: Vec<String> = Vec::new();
    for group in groups {
        if !group.import_ok {
            continue;
        }
        for entry in &group.entries {
            if !entry.previewable {
                continue;
            }
            imports.push(group.module.clone());
            for fixture in &entry.fixtures {
                imports.push(fixture.module.clone());
            }
        }
    }
    imports.sort();
    imports.dedup();

    let mut out = String::from("import Html\nimport Query\n");
    for name in &imports {
        out.push_str("import ");
        out.push_str(name);
        out.push('\n');
    }
    out.push_str("\nPreview := [].{\n    render = |id, args|\n        match id {\n");
    for group in groups {
        if !group.import_ok {
            continue;
        }
        for entry in &group.entries {
            if !entry.previewable {
                continue;
            }
            out.push_str(&preview_id_arm(&group.module, entry));
        }
    }
    out.push_str("            _ => shell(Html.text(\"Unknown component\"))\n        }\n}\n\n");
    out.push_str(
        r#"shell = |node|
    Html.element(
        "html",
        [Html.attribute("lang", "en")],
        [
            Html.element(
                "head",
                [],
                [
                    Html.void_element("meta", [Html.attribute("charset", "utf-8")]),
                    Html.element("title", [], [Html.text("rocci browse")]),
                ],
            ),
            Html.element("body", [], [node]),
        ],
    )
"#,
    );
    out
}

fn preview_id_arm(module: &str, entry: &CatalogEntry) -> String {
    let wrap = |call: String| {
        if entry.full_document {
            call
        } else {
            format!("shell({call})")
        }
    };
    if entry.fixtures.is_empty() {
        return format!(
            "            {} => {}\n",
            roc_string(&entry.id),
            wrap(generate_runtime_call(module, entry))
        );
    }
    let mut out = format!(
        "            {} =>\n                match Query.arg_str(args, \"fixture\") ?? \"\" {{\n",
        roc_string(&entry.id)
    );
    for fixture in &entry.fixtures {
        out.push_str(&format!(
            "                    {} => {}\n",
            roc_string(&fixture.name),
            wrap(generate_fixture_call(module, entry, fixture))
        ));
    }
    let fallback = if can_preview_from_form(entry) {
        generate_runtime_call(module, entry)
    } else {
        generate_fixture_call(module, entry, &entry.fixtures[0])
    };
    out.push_str(&format!(
        "                    _ => {}\n                }}\n",
        wrap(fallback)
    ));
    out
}

fn generate_fixture_call(module: &str, entry: &CatalogEntry, fixture: &BrowseFixture) -> String {
    let fixture_ref = format!("{}.{}", fixture.module, fixture.name);
    let fields: Vec<String> = entry
        .params
        .iter()
        .filter(|param| !param.is_body && record_has_field(&fixture.value, &param.name))
        .map(|param| {
            let value = match &param.kind {
                Some(_) => overlay_expr(param, &fixture_ref, fixture),
                None => format!("{fixture_ref}.{}", param.name),
            };
            format!("{}: {}", param.name, value)
        })
        .collect();
    let has_scalar_overlay = entry.params.iter().any(|param| {
        !param.is_body && param.kind.is_some() && record_has_field(&fixture.value, &param.name)
    });
    let bodies: Vec<&BrowseParam> = entry.params.iter().filter(|param| param.is_body).collect();
    let mut call_args = Vec::new();
    if entry.first_param_is_record {
        if !has_scalar_overlay {
            call_args.push(fixture_ref);
        } else {
            call_args.push(format!("{{ {} }}", fields.join(", ")));
        }
    } else {
        call_args.push(fixture_ref);
    }
    for param in bodies {
        call_args.push(value_expr(param));
    }
    format!("{module}.{}({})", entry.name, call_args.join(", "))
}

fn overlay_expr(param: &BrowseParam, fixture_ref: &str, fixture: &BrowseFixture) -> String {
    let quoted = roc_string(&param.name);
    let fallback = overlay_fallback(param, fixture_ref, fixture);
    match param.kind.as_ref().unwrap() {
        ParamKind::Str => format!("Query.arg_str(args, {quoted}) ?? {fallback}"),
        ParamKind::I64 => format!("Query.arg_i64(args, {quoted}) ?? {fallback}"),
        ParamKind::U64 => format!("Query.arg_u64(args, {quoted}) ?? {fallback}"),
        ParamKind::F64 => format!("Query.arg_f64(args, {quoted}) ?? {fallback}"),
        ParamKind::Dec => format!("Query.arg_dec(args, {quoted}) ?? {fallback}"),
        ParamKind::Bool => format!("Query.arg_bool(args, {quoted}) ?? {fallback}"),
        ParamKind::BodyHtml => {
            format!("Html.text(Query.arg_str(args, {quoted}) ?? {fallback})")
        }
    }
}

fn overlay_fallback(param: &BrowseParam, fixture_ref: &str, fixture: &BrowseFixture) -> String {
    let field = top_level_fields(&fixture.value)
        .into_iter()
        .find(|(name, _)| name == &param.name)
        .map(|(_, expr)| expr);
    match param.kind.as_ref().unwrap() {
        ParamKind::I64 => numeric_literal_fallback(field.as_deref(), "I64")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        ParamKind::U64 => numeric_literal_fallback(field.as_deref(), "U64")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        ParamKind::F64 => numeric_literal_fallback(field.as_deref(), "F64")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        ParamKind::Dec => numeric_literal_fallback(field.as_deref(), "Dec")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        _ => format!("{fixture_ref}.{}", param.name),
    }
}

fn numeric_literal_fallback(field: Option<&str>, suffix: &str) -> Option<String> {
    let bare = strip_num_suffix(field?.trim());
    let ok = match suffix {
        "I64" | "U64" => is_i64(bare),
        "F64" | "Dec" => is_i64(bare) || is_float(bare),
        _ => false,
    };
    ok.then(|| format!("{bare}.{suffix}"))
}

fn generate_runtime_call(module: &str, entry: &CatalogEntry) -> String {
    let props: Vec<&BrowseParam> = entry.params.iter().filter(|param| !param.is_body).collect();
    let bodies: Vec<&BrowseParam> = entry.params.iter().filter(|param| param.is_body).collect();
    let mut call_args = Vec::new();
    if entry.first_param_is_record {
        let fields: Vec<String> = props.iter().filter_map(|param| field_expr(param)).collect();
        if fields.is_empty() {
            call_args.push("{}".to_string());
        } else {
            call_args.push(format!("{{ {} }}", fields.join(", ")));
        }
    } else if let Some(param) = props.first() {
        call_args.push(value_expr(param));
    }
    for param in bodies {
        call_args.push(value_expr(param));
    }
    format!("{module}.{}({})", entry.name, call_args.join(", "))
}

fn field_expr(param: &BrowseParam) -> Option<String> {
    match &param.kind {
        Some(_) => Some(format!("{}: {}", param.name, value_expr(param))),
        None => param
            .default_roc
            .as_ref()
            .map(|default| format!("{}: {}", param.name, default)),
    }
}

fn value_expr(param: &BrowseParam) -> String {
    let Some(kind) = &param.kind else {
        return param
            .default_roc
            .clone()
            .unwrap_or_else(|| "Html.empty".to_string());
    };
    let fallback = param
        .default_roc
        .clone()
        .unwrap_or_else(|| kind.zero_roc().to_string());
    let quoted = roc_string(&param.name);
    match kind {
        ParamKind::Str => format!("Query.arg_str(args, {quoted}) ?? {fallback}"),
        ParamKind::I64 => format!("Query.arg_i64(args, {quoted}) ?? {fallback}"),
        ParamKind::U64 => format!("Query.arg_u64(args, {quoted}) ?? {fallback}"),
        ParamKind::F64 => format!("Query.arg_f64(args, {quoted}) ?? {fallback}"),
        ParamKind::Dec => format!("Query.arg_dec(args, {quoted}) ?? {fallback}"),
        ParamKind::Bool => format!("Query.arg_bool(args, {quoted}) ?? {fallback}"),
        ParamKind::BodyHtml => {
            format!("Html.text(Query.arg_str(args, {quoted}) ?? {fallback})")
        }
    }
}

fn generate_main_roc() -> String {
    format!(
        r#"app [Context, program] {{
    pf: platform "{PLATFORM}",
    http: "{HTTP_PKG}",
}}

import pf.Path
import pf.Server
import http.Method
import http.Response
import Browser
import Catalog
import Html
import Preview
import Query

Context : {{}}

program = {{ init!, respond!, shutdown! }}

init! : () => Try({{ config : Server.Config, context : Context }}, [Exit(I64), ..])
init! = || {{
    assets = Server.file_root({{
        id: "assets",
        path: Path.utf8("assets"),
    }})
    config =
        Server.default_config
        .with_file_roots([assets])
        .with_native_routes({{
            files: [
                Server.static_mount({{ at: "/assets", files: assets }}),
            ],
            liveness: [],
            readiness: [],
        }})
    Ok({{ config, context: {{}} }})
}}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, _context| {{
    path =
        match request.target() {{
            Resource({{ raw_path, .. }}) => raw_path
            _ => ""
        }}
    query =
        match request.target() {{
            Resource({{ raw_query: Present(q), .. }}) => q
            _ => ""
        }}
    args = Query.parse(query)

    match (Method.to_str(request.method()), path) {{
        ("GET", "/") => html_ok(Html.render(Browser.homePage({{ groups: Catalog.groups }})))
        ("GET", "/c") => inspector(args)
        ("GET", "/preview") => preview(args)
        _ =>
            Ok(
                Server.respond(
                    Response.from_status(404)
                    .with_body(Str.to_utf8("Not found")),
                ),
            )
    }}
}}

shutdown! : Server.ShutdownReason, Context => Try({{}}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({{}})

inspector = |args| {{
    id = Query.arg_str(args, "id") ?? ""
    requested = Query.arg_str(args, "fixture") ?? ""
    match Catalog.find(id) {{
        Ok(selected) => {{
            chosen = chosen_fixture(selected, requested)
            html_ok(
                Html.render(
                    Browser.inspectorPage({{
                        groups: Catalog.groups,
                        selected: selected,
                        fields: fields(selected, args, chosen),
                        preview_url: preview_url(selected, args, chosen),
                        selected_fixture: chosen.name,
                    }}),
                ),
            )
        }}
        Err(_) => html_ok(Html.render(Browser.homePage({{ groups: Catalog.groups }})))
    }}
}}

preview = |args| {{
    id = Query.arg_str(args, "id") ?? ""
    html_ok(Html.render(Preview.render(id, args)))
}}

empty_fixture = {{ name: "", source: "", scalars: [] }}

chosen_fixture = |selected, requested|
    match List.get(List.keep_if(selected.fixtures, |item| item.name == requested), 0) {{
        Ok(found) => found
        Err(_) =>
            match List.get(selected.fixtures, 0) {{
                Ok(first) => first
                Err(_) => empty_fixture
            }}
    }}

fields = |selected, args, chosen|
    List.map(
        selected.params,
        |param| {{
            from_fixture =
                match List.get(List.keep_if(chosen.scalars, |item| item.name == param.name), 0) {{
                    Ok(item) => item.value
                    Err(_) => param.value
                }}
            {{
                name: param.name,
                required: param.required,
                kind: param.kind,
                value: Query.arg_str(args, param.name) ?? from_fixture,
            }}
        }},
    )

preview_url = |selected, args, chosen| {{
    fixture_q =
        if chosen.name == "" {{
            ""
        }} else {{
            "&fixture=${{Query.encode(chosen.name)}}"
        }}
    suffix =
        List.fold(
            selected.params,
            fixture_q,
            |acc, param| {{
                from_fixture =
                    match List.get(List.keep_if(chosen.scalars, |item| item.name == param.name), 0) {{
                        Ok(item) => item.value
                        Err(_) => param.value
                    }}
                value = Query.arg_str(args, param.name) ?? from_fixture
                "${{acc}}&${{Query.encode(param.name)}}=${{Query.encode(value)}}"
            }},
        )
    "/preview?id=${{Query.encode(selected.id)}}${{suffix}}"
}}

html_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{{ name: "Content-Type", value: "text/html; charset=utf-8" }}])
            .with_body(Str.to_utf8(body)),
        ),
    )
"#
    )
}

fn roc_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn roc_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn create() -> Result<Self> {
        let path = std::env::temp_dir().join(format!("rocci-browse-{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("failed to clear {}", path.display()))?;
        }
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create {}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let dir =
            env::temp_dir().join(format!("rocci-browse-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn compile_src(src: &str) -> rocci_template::CompileOutput {
        compile(SourceFile::new("test.rocci", src), &LowerOptions::default())
    }

    fn entry_for(src: &str, name: &str) -> CatalogEntry {
        let out = compile_src(src);
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let info = out
            .components
            .iter()
            .find(|component| component.name == name)
            .unwrap();
        catalog_entry(src, "Demo", "Demo.rocci", info, &out.document)
    }

    #[test]
    fn discover_is_recursive_and_skips_target() {
        let dir = temp_root("discover");
        fs::write(dir.join("Top.rocci"), "").unwrap();
        let nested = dir.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("Other.rocci"), "").unwrap();
        let target = dir.join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("Skip.rocci"), "").unwrap();
        let git = dir.join(".git");
        fs::create_dir(&git).unwrap();
        fs::write(git.join("Hidden.rocci"), "").unwrap();
        fs::write(dir.join("notes.txt"), "").unwrap();

        let found = discover_rocci_files(&[dir.clone()]).unwrap();
        assert_eq!(
            found,
            vec![dir.join("Top.rocci"), nested.join("Other.rocci")]
        );
        cleanup(&dir);
    }

    #[test]
    fn discover_file_root_and_rejects_duplicates() {
        let dir = temp_root("dup");
        let a = dir.join("a");
        let b = dir.join("b");
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        fs::write(a.join("Foo.rocci"), "").unwrap();
        fs::write(b.join("Foo.rocci"), "").unwrap();
        let err = discover_rocci_files(&[a.clone(), b.clone()])
            .unwrap_err()
            .to_string();
        assert!(err.contains("duplicate module name `Foo`"), "{err}");

        let only = discover_rocci_files(&[a.join("Foo.rocci")]).unwrap();
        assert_eq!(only, vec![a.join("Foo.rocci")]);
        cleanup(&dir);
    }

    #[test]
    fn infers_annotation_default_body_and_usage() {
        let src = r#"
@component hello = |{ name ?? "Roc" }| {
    <p>{name}</p>
}
@component typed = |{ count: I64 }| {
    <p>{count.to_str()}</p>
}
@component badge = |{ tone ?? Neutral }, content| {
    <span>{content}</span>
}
@component card = |{ count }| {
    <output>{count.to_str()}</output>
}
@component flag = |{ full }| {
    @if full {
        <p>full</p>
    }
}
@component items = |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
@component contact = |{ contact }| {
    <p>{contact.first}</p>
}
@component title = |{ title }| {
    <h1>{title}</h1>
}
"#;
        let hello = entry_for(src, "hello");
        assert!(hello.previewable);
        assert_eq!(hello.params[0].kind, Some(ParamKind::Str));
        assert_eq!(hello.params[0].default_display, "Roc");
        assert!(!hello.params[0].required);

        let typed = entry_for(src, "typed");
        assert!(typed.previewable);
        assert_eq!(typed.params[0].kind, Some(ParamKind::I64));

        let badge = entry_for(src, "badge");
        assert!(badge.previewable);
        assert!(badge.params[0].kind.is_none());
        assert_eq!(badge.params[1].kind, Some(ParamKind::BodyHtml));
        assert!(badge.params[1].is_body);

        let card = entry_for(src, "card");
        assert!(card.previewable);
        assert_eq!(card.params[0].kind, Some(ParamKind::I64));

        let flag = entry_for(src, "flag");
        assert!(flag.previewable);
        assert_eq!(flag.params[0].kind, Some(ParamKind::Bool));

        let items = entry_for(src, "items");
        assert!(!items.previewable);
        assert!(items.reason.contains("list"));

        let contact = entry_for(src, "contact");
        assert!(!contact.previewable);
        assert!(contact.reason.contains("record"));

        let title = entry_for(src, "title");
        assert!(title.previewable);
        assert_eq!(title.params[0].kind, Some(ParamKind::Str));
        assert!(title.params[0].required);
    }

    #[test]
    fn passthrough_inherits_sibling_param_kind() {
        let src = r#"
@component card = |{ count }| {
    <output>{count.to_str()}</output>
}
@component page = |{ count }| {
    <html><body><Card count={count} /></body></html>
}
"#;
        let out = compile_src(src);
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let mut entries: Vec<_> = out
            .components
            .iter()
            .map(|info| catalog_entry(src, "Demo", "Demo.rocci", info, &out.document))
            .collect();
        propagate_passthrough(src, &out.document, &mut entries);
        let page = entries.iter().find(|entry| entry.name == "page").unwrap();
        assert!(page.previewable, "{}", page.reason);
        assert_eq!(page.params[0].kind, Some(ParamKind::I64));
        assert!(page.full_document);
    }

    #[test]
    fn catalog_and_preview_generation() {
        let src = r#"
@component hello = |{ name ?? "Roc" }| {
    <p>{name}</p>
}
@component items = |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
"#;
        let hello = entry_for(src, "hello");
        let items = entry_for(src, "items");
        let groups = vec![ModuleGroup {
            module: "Demo".into(),
            file: "Demo.rocci".into(),
            import_ok: true,
            entries: vec![hello, items],
        }];
        let catalog = generate_catalog_roc(&groups);
        assert!(catalog.contains("Demo.hello"));
        assert!(catalog.contains("Demo.items"));
        assert!(catalog.contains("previewable: True"));
        assert!(catalog.contains("previewable: False"));
        assert!(catalog.contains("kind: \"str\""));

        let preview = generate_preview_roc(&groups);
        assert!(preview.contains("import Demo"));
        assert!(preview.contains("Demo.hello({ name: Query.arg_str(args, \"name\") ?? \"Roc\" })"));
        assert!(!preview.contains("Demo.items("));
        assert!(preview.contains("shell("));
    }

    #[test]
    fn fixtures_make_list_components_previewable_and_fill_scalars() {
        let src = r#"
@component hello = |{ name }| {
    <p>{name}</p>
}
@fixture{target: hello}
helloTest = { name: "Ada" }

@component items = |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
@fixture{target: items}
itemsTest = { items: ["milk", "eggs"] }
"#;
        let groups = groups_with_fixtures(src);
        let hello = groups[0]
            .entries
            .iter()
            .find(|entry| entry.name == "hello")
            .unwrap();
        assert!(hello.previewable);
        assert_eq!(hello.fixtures.len(), 1);
        assert_eq!(hello.fixtures[0].name, "helloTest");
        assert_eq!(
            hello.fixtures[0].scalars,
            vec![("name".into(), "Ada".into())]
        );

        let items = groups[0]
            .entries
            .iter()
            .find(|entry| entry.name == "items")
            .unwrap();
        assert!(items.previewable, "{}", items.reason);
        assert_eq!(items.fixtures[0].name, "itemsTest");
        assert!(items.fixtures[0].scalars.is_empty());

        let catalog = generate_catalog_roc(&groups);
        assert!(catalog.contains("helloTest"));
        assert!(catalog.contains("value: \"Ada\""));
        assert!(catalog.contains("itemsTest"));

        let preview = generate_preview_roc(&groups);
        assert!(preview.contains(
            "Demo.hello({ name: Query.arg_str(args, \"name\") ?? Demo.helloTest.name })"
        ));
        assert!(preview.contains("Demo.items(Demo.itemsTest)"));
        assert!(preview.contains("\"helloTest\" =>"));
    }

    #[test]
    fn fixture_numeric_overlays_use_typed_literals() {
        let src = r#"
@component card = |{ count }| {
    <output>{count.to_str()}</output>
}
@fixture{target: card}
cardTest = { count: 3 }
"#;
        let groups = groups_with_fixtures(src);
        let preview = generate_preview_roc(&groups);
        assert!(preview.contains("Demo.card({ count: Query.arg_i64(args, \"count\") ?? 3.I64 })"));
    }

    fn groups_with_fixtures(src: &str) -> Vec<ModuleGroup> {
        let out = compile_src(src);
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let module = CompiledModule {
            path: PathBuf::from("Demo.rocci"),
            type_name: "Demo".into(),
            roc: out.roc,
            document: out.document,
            components: out.components,
            fixtures: out.fixtures,
            src: src.to_string(),
        };
        let available = HashSet::from(["Html".into(), "Demo".into()]);
        let mut groups = analyze_modules(std::slice::from_ref(&module), &available);
        attach_fixtures(std::slice::from_ref(&module), &mut groups);
        groups
    }

    #[test]
    fn preview_omits_modules_with_missing_imports() {
        let src = r#"
@component hello = |{}| {
    <p>ok</p>
}
"#;
        let hello = entry_for(src, "hello");
        let groups = vec![ModuleGroup {
            module: "Demo".into(),
            file: "Demo.rocci".into(),
            import_ok: false,
            entries: vec![hello],
        }];
        let preview = generate_preview_roc(&groups);
        assert!(!preview.contains("import Demo"));
        assert!(!preview.contains("Demo.hello"));
    }

    #[test]
    fn query_decode_helpers_match_form_values() {
        assert_eq!(display_roc_literal("\"Roc\""), "Roc");
        assert_eq!(display_roc_literal("Bool.true"), "true");
        assert_eq!(display_roc_literal("0"), "0");
        assert_eq!(ParamKind::from_annotation("I64"), Some(ParamKind::I64));
        assert_eq!(ParamKind::from_annotation("List(Item)"), None);
        assert_eq!(ParamKind::from_annotation("{ first: Str }"), None);
        assert_eq!(
            infer_from_default("\"Ada\""),
            Inferred::Scalar(ParamKind::Str)
        );
        assert_eq!(infer_from_default("12"), Inferred::Scalar(ParamKind::I64));
        assert_eq!(
            infer_from_default("Bool.false"),
            Inferred::Scalar(ParamKind::Bool)
        );
        assert!(matches!(
            infer_from_default("Neutral"),
            Inferred::Unsupported(_)
        ));
        assert_eq!(
            fixture_scalars("{ name: \"Ada\", contacts: all_contacts, count: 3, full: True }"),
            vec![
                ("name".into(), "Ada".into()),
                ("count".into(), "3".into()),
                ("full".into(), "true".into()),
            ]
        );
    }
}
