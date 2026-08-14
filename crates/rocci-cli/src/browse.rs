use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    ComponentDecl, ComponentInfo, Document, LowerOptions, ModuleItem, SourceFile, TemplateBlock,
    TemplateItem, compile, format_diagnostic,
};

use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::serve;

const PLATFORM: &str = "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst";
const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";
const HTML_STUB: &str = include_str!("../../../examples/roc-counter/Html.roc");
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

pub fn browse(roots: &[PathBuf], no_window: bool) -> Result<()> {
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

    let groups = analyze_modules(&compiled_modules, &available);
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

    let port = serve::basic_webserver_port()?;
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
pub(crate) struct RecordField {
    pub name: String,
    pub kind: ParamKind,
    pub default_roc: Option<String>,
    pub default_display: String,
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
    Record(Vec<RecordField>),
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
            Self::Record(_) => "record",
        }
    }

    fn from_annotation(ty: &str) -> Option<Self> {
        let ty = ty.trim();
        match ty {
            "Str" => Some(Self::Str),
            "I64" => Some(Self::I64),
            "U64" => Some(Self::U64),
            "F64" => Some(Self::F64),
            "Dec" => Some(Self::Dec),
            "Bool" => Some(Self::Bool),
            _ => parse_record_annotation(ty),
        }
    }

    fn zero_roc(&self) -> String {
        match self {
            Self::Str | Self::BodyHtml => "\"\"".to_string(),
            Self::I64 => "0.I64".to_string(),
            Self::U64 => "0.U64".to_string(),
            Self::F64 | Self::Dec => "0.0".to_string(),
            Self::Bool => "False".to_string(),
            Self::Record(fields) => {
                if fields.is_empty() {
                    return "{}".to_string();
                }
                let inner = fields
                    .iter()
                    .map(|field| format!("{}: {}", field.name, field.kind.zero_roc()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {inner} }}")
            }
        }
    }

    fn zero_display(&self) -> String {
        match self {
            Self::Str | Self::BodyHtml | Self::Record(_) => String::new(),
            Self::I64 | Self::U64 | Self::F64 | Self::Dec => "0".to_string(),
            Self::Bool => "false".to_string(),
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
    }
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
                        param.default_display = kind.zero_display();
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
        TemplateItem::Interpolation(_) | TemplateItem::Text(_) | TemplateItem::Let(_) => {}
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
        return match parse_record_literal(trimmed) {
            Some(fields) => Inferred::Scalar(ParamKind::Record(fields)),
            None => Inferred::Unsupported("record".into()),
        };
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
        None => kind.zero_display(),
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
                decl.body.items.first(),
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
    Field { path: Vec<String>, leaf: FieldLeaf },
    List,
    Tag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldLeaf {
    Str,
    Bool,
    I64,
    Tag,
    List,
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
    let field_hints: Vec<(Vec<String>, FieldLeaf)> = hints
        .iter()
        .filter_map(|hint| match hint {
            UsageHint::Field { path, leaf } => Some((path.clone(), *leaf)),
            _ => None,
        })
        .collect();
    if !field_hints.is_empty() {
        if field_hints
            .iter()
            .any(|(_, leaf)| matches!(leaf, FieldLeaf::Tag | FieldLeaf::List))
        {
            return Some(Inferred::Unsupported("record".into()));
        }
        return Some(Inferred::Scalar(ParamKind::Record(
            record_fields_from_hints(&field_hints),
        )));
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

fn record_fields_from_hints(hints: &[(Vec<String>, FieldLeaf)]) -> Vec<RecordField> {
    let mut fields = Vec::new();
    for (path, leaf) in hints {
        insert_record_field(&mut fields, path, leaf_kind(*leaf), None);
    }
    fields
}

fn leaf_kind(leaf: FieldLeaf) -> ParamKind {
    match leaf {
        FieldLeaf::Str => ParamKind::Str,
        FieldLeaf::Bool => ParamKind::Bool,
        FieldLeaf::I64 => ParamKind::I64,
        FieldLeaf::Tag | FieldLeaf::List => ParamKind::Str,
    }
}

fn insert_record_field(
    fields: &mut Vec<RecordField>,
    path: &[String],
    kind: ParamKind,
    default_roc: Option<String>,
) {
    let Some((name, rest)) = path.split_first() else {
        return;
    };
    let default_display = default_roc
        .as_deref()
        .map(display_roc_literal)
        .unwrap_or_else(|| kind.zero_display());
    if rest.is_empty() {
        if let Some(existing) = fields.iter_mut().find(|field| field.name == *name) {
            existing.kind = merge_param_kind(&existing.kind, kind);
            if existing.default_roc.is_none() {
                existing.default_roc = default_roc;
                existing.default_display = default_display;
            }
        } else {
            fields.push(RecordField {
                name: name.clone(),
                kind,
                default_roc,
                default_display,
            });
        }
        return;
    }
    match fields.iter_mut().find(|field| field.name == *name) {
        Some(existing) => {
            if let ParamKind::Record(nested) = &mut existing.kind {
                insert_record_field(nested, rest, kind, default_roc);
            } else {
                let mut nested = Vec::new();
                insert_record_field(&mut nested, rest, kind, default_roc);
                existing.kind = ParamKind::Record(nested);
            }
        }
        None => {
            let mut nested = Vec::new();
            insert_record_field(&mut nested, rest, kind, default_roc);
            fields.push(RecordField {
                name: name.clone(),
                kind: ParamKind::Record(nested),
                default_roc: None,
                default_display: String::new(),
            });
        }
    }
}

fn merge_param_kind(left: &ParamKind, right: ParamKind) -> ParamKind {
    match (left, right) {
        (ParamKind::Record(existing), ParamKind::Record(incoming)) => {
            let mut fields = existing.clone();
            for field in incoming {
                insert_record_field(
                    &mut fields,
                    &[field.name.clone()],
                    field.kind,
                    field.default_roc,
                );
            }
            ParamKind::Record(fields)
        }
        (ParamKind::Record(existing), _) => ParamKind::Record(existing.clone()),
        (_, ParamKind::Record(incoming)) => ParamKind::Record(incoming),
        (ParamKind::I64, _) | (_, ParamKind::I64) => ParamKind::I64,
        (ParamKind::Bool, _) | (_, ParamKind::Bool) => ParamKind::Bool,
        _ => left.clone(),
    }
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
                if let rocci_template::AttrValue::Expr { expr } = attr.value {
                    classify_expr(param, expr.of(src), hints, ExprCtx::Value);
                }
            }
            for child in &el.children {
                walk_item(src, child, param, hints);
            }
        }
        TemplateItem::ComponentCall(call) => {
            for attr in &call.attrs {
                if let rocci_template::AttrValue::Expr { expr } = attr.value {
                    classify_expr(param, expr.of(src), hints, ExprCtx::Value);
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
                classify_expr(param, expr, hints, ExprCtx::Interpolate);
            }
        }
        TemplateItem::If(dir) => {
            let cond = dir.condition.of(src).trim();
            if cond == param || cond == format!("!{param}") {
                hints.push(UsageHint::Bool);
            } else {
                classify_expr(param, cond, hints, ExprCtx::If);
            }
            walk_block(src, &dir.then_body, param, hints);
            for (cond, body) in &dir.else_ifs {
                classify_expr(param, cond.of(src), hints, ExprCtx::If);
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
                classify_expr(param, collection, hints, ExprCtx::For);
            }
            walk_block(src, &dir.body, param, hints);
        }
        TemplateItem::Match(dir) => {
            let scrutinee = dir.scrutinee.of(src).trim();
            if scrutinee == param {
                hints.push(UsageHint::Tag);
            } else {
                classify_expr(param, scrutinee, hints, ExprCtx::Match);
            }
            for arm in &dir.arms {
                walk_item(src, &arm.value, param, hints);
            }
        }
        TemplateItem::Let(dir) => {
            classify_expr(param, dir.expr.of(src), hints, ExprCtx::Value);
        }
        TemplateItem::Text(_) => {}
    }
}

#[derive(Clone, Copy, Debug)]
enum ExprCtx {
    Value,
    Interpolate,
    If,
    Match,
    For,
}

fn classify_expr(param: &str, expr: &str, hints: &mut Vec<UsageHint>, ctx: ExprCtx) {
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
    if let Some((path, leaf)) = exact_field_access(param, expr, ctx) {
        hints.push(UsageHint::Field { path, leaf });
    }
}

fn exact_field_access(param: &str, expr: &str, ctx: ExprCtx) -> Option<(Vec<String>, FieldLeaf)> {
    let expr = expr.strip_prefix('!').unwrap_or(expr);
    let rest = expr.strip_prefix(param)?.strip_prefix('.')?;
    let (path, method) = parse_field_path(rest)?;
    if path.is_empty() {
        return None;
    }
    let leaf = match method {
        FieldMethod::ToStr => FieldLeaf::I64,
        FieldMethod::None => match ctx {
            ExprCtx::If => FieldLeaf::Bool,
            ExprCtx::Match => FieldLeaf::Tag,
            ExprCtx::For => FieldLeaf::List,
            ExprCtx::Interpolate | ExprCtx::Value => FieldLeaf::Str,
        },
    };
    Some((path, leaf))
}

#[derive(Clone, Copy, Debug)]
enum FieldMethod {
    None,
    ToStr,
}

fn parse_field_path(after: &str) -> Option<(Vec<String>, FieldMethod)> {
    let mut path = Vec::new();
    let mut rest = after;
    loop {
        let ident = take_ident(rest)?;
        rest = &rest[ident.len()..];
        if rest.starts_with('(') {
            let method = if ident == "to_str" || ident == "toStr" {
                FieldMethod::ToStr
            } else {
                return None;
            };
            return if path.is_empty() {
                None
            } else {
                Some((path, method))
            };
        }
        path.push(ident.to_string());
        if let Some(stripped) = rest.strip_prefix('.') {
            rest = stripped;
            continue;
        }
        break;
    }
    if rest.is_empty() {
        Some((path, FieldMethod::None))
    } else {
        None
    }
}

fn take_ident(input: &str) -> Option<&str> {
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    let end = chars
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
        .map(|(index, _)| index)
        .unwrap_or(input.len());
    Some(&input[..end])
}

fn parse_record_annotation(ty: &str) -> Option<ParamKind> {
    let inner = record_inner(ty)?;
    if inner.is_empty() {
        return Some(ParamKind::Record(Vec::new()));
    }
    let mut fields = Vec::new();
    for part in split_top_level(inner, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, rest) = split_once_top_level(part, ':')?;
        let name = take_ident(name.trim())?.to_string();
        let kind = ParamKind::from_annotation(rest.trim())?;
        fields.push(RecordField {
            default_display: kind.zero_display(),
            default_roc: None,
            name,
            kind,
        });
    }
    Some(ParamKind::Record(fields))
}

fn parse_record_literal(expr: &str) -> Option<Vec<RecordField>> {
    let inner = record_inner(expr)?;
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut fields = Vec::new();
    for part in split_top_level(inner, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, rest) = split_once_top_level(part, ':')?;
        let name = take_ident(name.trim())?.to_string();
        match infer_from_default(rest.trim()) {
            Inferred::Scalar(kind) => {
                let default_display = display_default(&kind, Some(rest.trim()));
                fields.push(RecordField {
                    name,
                    default_roc: Some(rest.trim().to_string()),
                    default_display,
                    kind,
                });
            }
            Inferred::Unsupported(_) => return None,
        }
    }
    Some(fields)
}

fn record_inner(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    trimmed.strip_prefix('{')?.strip_suffix('}').map(str::trim)
}

fn split_once_top_level(input: &str, sep: char) -> Option<(&str, &str)> {
    let mut depth: usize = 0;
    let mut chars = input.char_indices().peekable();
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
                return Some((&input[..i], &input[i + c.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
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

struct FlatField {
    name: String,
    required: bool,
    kind: &'static str,
    value: String,
}

fn form_params(entry: &CatalogEntry) -> Vec<FlatField> {
    entry.params.iter().flat_map(flatten_param).collect()
}

fn flatten_param(param: &BrowseParam) -> Vec<FlatField> {
    match &param.kind {
        Some(ParamKind::Record(fields)) => flatten_record(&param.name, param.required, fields),
        Some(kind) => vec![FlatField {
            name: param.name.clone(),
            required: param.required,
            kind: kind.as_str(),
            value: param.default_display.clone(),
        }],
        None => Vec::new(),
    }
}

fn flatten_record(prefix: &str, required: bool, fields: &[RecordField]) -> Vec<FlatField> {
    fields
        .iter()
        .flat_map(|field| {
            let name = format!("{prefix}.{}", field.name);
            match &field.kind {
                ParamKind::Record(nested) => flatten_record(&name, required, nested),
                kind => vec![FlatField {
                    name,
                    required,
                    kind: kind.as_str(),
                    value: field.default_display.clone(),
                }],
            }
        })
        .collect()
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
                roc_string(param.kind),
                roc_string(&param.value),
            ));
            if index + 1 != params.len() {
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
    let mut imports: Vec<String> = groups
        .iter()
        .filter(|group| group.import_ok && group.entries.iter().any(|entry| entry.previewable))
        .map(|group| group.module.clone())
        .collect();
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
            let call = generate_runtime_call(&group.module, entry);
            let render = if entry.full_document {
                call
            } else {
                format!("shell({call})")
            };
            out.push_str(&format!(
                "            {} => {render}\n",
                roc_string(&entry.id)
            ));
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
    value_expr_kind(kind, &param.name, param.default_roc.as_deref())
}

fn value_expr_kind(kind: &ParamKind, key: &str, default_roc: Option<&str>) -> String {
    if let ParamKind::Record(fields) = kind {
        if fields.is_empty() {
            return "{}".to_string();
        }
        let inner = fields
            .iter()
            .map(|field| {
                format!(
                    "{}: {}",
                    field.name,
                    value_expr_kind(
                        &field.kind,
                        &format!("{key}.{}", field.name),
                        field.default_roc.as_deref()
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return format!("{{ {inner} }}");
    }
    let fallback = default_roc
        .map(str::to_string)
        .unwrap_or_else(|| kind.zero_roc());
    let quoted = roc_string(key);
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
        ParamKind::Record(_) => unreachable!(),
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
    match Catalog.find(id) {{
        Ok(selected) =>
            html_ok(
                Html.render(
                    Browser.inspectorPage({{
                        groups: Catalog.groups,
                        selected: selected,
                        fields: fields(selected, args),
                        preview_url: preview_url(selected, args),
                    }}),
                ),
            )
        Err(_) => html_ok(Html.render(Browser.homePage({{ groups: Catalog.groups }})))
    }}
}}

preview = |args| {{
    id = Query.arg_str(args, "id") ?? ""
    html_ok(Html.render(Preview.render(id, args)))
}}

fields = |selected, args|
    List.map(
        selected.params,
        |param| {{
            {{
                name: param.name,
                required: param.required,
                kind: param.kind,
                value: Query.arg_str(args, param.name) ?? param.value,
            }}
        }},
    )

preview_url = |selected, args| {{
    suffix =
        List.fold(
            selected.params,
            "",
            |acc, param| {{
                value = Query.arg_str(args, param.name) ?? param.value
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
hello = @component |{ name ?? "Roc" }| {
    <p>{name}</p>
}
typed = @component |{ count: I64 }| {
    <p>{count.to_str()}</p>
}
badge = @component |{ tone ?? Neutral }, content| {
    <span>{content}</span>
}
card = @component |{ count }| {
    <output>{count.to_str()}</output>
}
flag = @component |{ full }| {
    @if full {
        <p>full</p>
    }
}
items = @component |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
contact = @component |{ contact }| {
    <p>{contact.first}</p>
}
title = @component |{ title }| {
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
        assert!(contact.previewable, "{}", contact.reason);
        match &contact.params[0].kind {
            Some(ParamKind::Record(fields)) => {
                let names: Vec<_> = fields.iter().map(|field| field.name.as_str()).collect();
                assert_eq!(names, ["first"]);
                assert_eq!(fields[0].kind, ParamKind::Str);
            }
            other => panic!("expected record, got {other:?}"),
        }

        let title = entry_for(src, "title");
        assert!(title.previewable);
        assert_eq!(title.params[0].kind, Some(ParamKind::Str));
        assert!(title.params[0].required);
    }

    #[test]
    fn passthrough_inherits_sibling_param_kind() {
        let src = r#"
card = @component |{ count }| {
    <output>{count.to_str()}</output>
}
page = @component |{ count }| {
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
    fn record_fields_flatten_into_forms_and_preview_calls() {
        let src = r#"
card = @component |{ contact }| {
    <p>{contact.first} {contact.last}</p>
}
nested = @component |{ person ?? { name: "Ada", age: 1 } }| {
    <p>{person.name}</p>
}
"#;
        let card = entry_for(src, "card");
        assert!(card.previewable, "{}", card.reason);
        let groups = vec![ModuleGroup {
            module: "Demo".into(),
            file: "Demo.rocci".into(),
            import_ok: true,
            entries: vec![card, entry_for(src, "nested")],
        }];
        let catalog = generate_catalog_roc(&groups);
        assert!(catalog.contains("name: \"contact.first\""));
        assert!(catalog.contains("name: \"contact.last\""));
        assert!(catalog.contains("name: \"person.name\""));
        assert!(catalog.contains("name: \"person.age\""));

        let preview = generate_preview_roc(&groups);
        assert!(preview.contains(
            "Demo.card({ contact: { first: Query.arg_str(args, \"contact.first\") ?? \"\", last: Query.arg_str(args, \"contact.last\") ?? \"\" } })"
        ));
        assert!(preview.contains("person.name"));
        assert!(preview.contains("?? \"Ada\""));
        assert!(preview.contains("?? 1"));
    }

    #[test]
    fn catalog_and_preview_generation() {
        let src = r#"
hello = @component |{ name ?? "Roc" }| {
    <p>{name}</p>
}
items = @component |{ items }| {
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
    fn preview_omits_modules_with_missing_imports() {
        let src = r#"
hello = @component |{}| {
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
    }
}
