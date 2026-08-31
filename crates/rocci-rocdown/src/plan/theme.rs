use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rocci_template::{
    LowerOptions, Segment, SourceFile, compile, format_diagnostic, wrap_type_module,
};

use crate::config::SiteConfig;
use crate::runtime;

#[derive(Debug, Clone)]
pub struct CompiledThemeModule {
    pub type_name: String,
    pub source_name: String,
    pub src: String,
    pub roc: String,
    pub segments: Vec<Segment>,
    pub styles: Vec<rocci_template::StyleArtifact>,
    pub components: Vec<String>,
    pub component_infos: Vec<rocci_template::ComponentInfo>,
}

pub(crate) fn validate_theme_painters(root: &Path, config: &SiteConfig) -> Result<()> {
    compile_theme_with_painters(root, config, false)?;
    Ok(())
}

pub(crate) fn compile_theme_with_painters(
    root: &Path,
    config: &SiteConfig,
    preview: bool,
) -> Result<(Vec<CompiledThemeModule>, Vec<crate::registry::InferredKind>)> {
    let (mut modules, pack_names) = compile_theme_modules(root, config)?;
    let inferred = inferred_kinds_from_pack(&modules, &pack_names)?;
    let allow_debug = preview || config.blocks.debug;
    let _guard = crate::registry::install_pack_kinds(&inferred);
    modules.push(block_painters_module(
        &modules,
        &pack_names,
        &config.blocks.override_map,
        allow_debug,
    )?);
    Ok((modules, inferred))
}

fn inferred_kinds_from_pack(
    modules: &[CompiledThemeModule],
    pack_names: &[String],
) -> Result<Vec<crate::registry::InferredKind>> {
    let infos: Vec<_> = modules
        .iter()
        .filter(|module| pack_names.contains(&module.type_name))
        .flat_map(|module| module.component_infos.iter().cloned())
        .collect();
    crate::registry::infer_pack_kinds(&infos)
}

pub(crate) fn site_has_block_pack(root: &Path, config: &SiteConfig) -> bool {
    if config.blocks.pack.is_some() {
        return true;
    }
    let Some(theme_dir) = theme_dir_for_pack(root, config) else {
        return false;
    };
    theme_dir.join("Blocks.rocci").is_file() || theme_dir.join("blocks").is_dir()
}

fn theme_dir_for_pack(root: &Path, config: &SiteConfig) -> Option<PathBuf> {
    if let Some(theme) = &config.build.theme {
        let path = root.join(theme);
        if path.is_dir() {
            return Some(path);
        }
        if path.is_file() {
            return path.parent().map(Path::to_path_buf);
        }
        return None;
    }
    let theme_dir = root.join("theme");
    theme_dir.is_dir().then_some(theme_dir)
}

pub(crate) fn infer_site_pack_kinds(
    root: &Path,
    config: &SiteConfig,
) -> Result<Vec<crate::registry::InferredKind>> {
    let mut modules = Vec::new();
    let theme_dir = theme_dir_for_pack(root, config);
    let pack_names = compile_block_pack(root, theme_dir.as_deref(), config, &mut modules)?;
    inferred_kinds_from_pack(&modules, &pack_names)
}

fn compile_theme_modules(
    root: &Path,
    config: &SiteConfig,
) -> Result<(Vec<CompiledThemeModule>, Vec<String>)> {
    let target = if let Some(theme) = &config.build.theme {
        let p = root.join(theme);
        if !p.exists() {
            bail!(
                "configured theme path `{theme}` does not exist in {}",
                root.display()
            );
        }
        Some(p)
    } else {
        let theme_dir = root.join("theme");
        let site_shell = root.join("theme/SiteShell.rocci");
        let rocdown_theme = root.join("theme/RocdownTheme.rocci");
        let root_site_shell = root.join("SiteShell.rocci");
        if site_shell.is_file()
            || rocdown_theme.is_file()
            || (theme_dir.is_dir() && has_rocci_files(&theme_dir))
        {
            Some(theme_dir)
        } else if root_site_shell.is_file() {
            Some(root_site_shell)
        } else {
            None
        }
    };

    if let Some(target) = target {
        let theme_dir = if target.is_dir() {
            target.clone()
        } else {
            target.parent().unwrap_or(root).to_path_buf()
        };
        let mut modules = compile_project_theme(root, &target)?;
        let pack_names = compile_block_pack(root, Some(&theme_dir), config, &mut modules)?;
        Ok((modules, pack_names))
    } else {
        let mut modules = compile_builtin_theme()?;
        let pack_names = compile_block_pack(root, None, config, &mut modules)?;
        Ok((modules, pack_names))
    }
}

fn has_rocci_files(dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "rocci") {
                return true;
            }
        }
    }
    false
}

fn compile_single_module(
    source_name: &str,
    type_name: &str,
    src: &str,
) -> Result<CompiledThemeModule> {
    let source_file = SourceFile::new(source_name, src);
    let compiled = compile(
        source_file,
        &LowerOptions {
            embed_css: false,
            html_type: "Str".to_string(),
            scope_file_css: type_name != "RocdownBase",
            ..LowerOptions::default()
        },
    );
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source_file, diagnostic));
    }
    if compiled.has_errors() {
        bail!("{source_name} compilation failed");
    }
    if compiled.roc.contains("import Datastar") {
        bail!("{source_name} uses Datastar, which the rocdown runtime does not stage");
    }
    Ok(CompiledThemeModule {
        type_name: type_name.to_string(),
        source_name: source_name.to_string(),
        src: src.to_string(),
        roc: wrap_type_module(&compiled.roc, type_name),
        segments: compiled.segments,
        styles: compiled.styles,
        components: compiled
            .components
            .iter()
            .map(|component| component.name.clone())
            .collect(),
        component_infos: compiled.components.clone(),
    })
}

fn compile_project_theme(root: &Path, target: &Path) -> Result<Vec<CompiledThemeModule>> {
    let mut modules = Vec::new();
    let theme_dir = if target.is_dir() {
        target.to_path_buf()
    } else {
        target.parent().unwrap_or(root).to_path_buf()
    };

    let mut rocci_files = Vec::new();
    if theme_dir.is_dir() {
        for entry in std::fs::read_dir(&theme_dir)
            .with_context(|| format!("failed to read {}", theme_dir.display()))?
        {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "rocci") {
                rocci_files.push(path);
            }
        }
    } else if target.is_file() {
        rocci_files.push(target.to_path_buf());
    }
    rocci_files.sort();

    for file in &rocci_files {
        let type_name = rocci_template::type_name_from_path(file);
        let src = std::fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        let rel_name = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");
        let module = compile_single_module(&rel_name, &type_name, &src)?;
        modules.push(module);
    }

    if !modules.iter().any(|m| m.type_name == "RocdownBase") {
        let base = compile_single_module("RocdownBase.rocci", "RocdownBase", runtime::BASE)?;
        modules.insert(0, base);
    }

    if !modules.iter().any(|m| m.type_name == "Breadcrumbs") {
        let breadcrumbs =
            compile_single_module("Breadcrumbs.rocci", "Breadcrumbs", runtime::BREADCRUMBS)?;
        modules.push(breadcrumbs);
    }

    if !modules.iter().any(|m| m.type_name == "NavList") {
        let nav_list = compile_single_module("NavList.rocci", "NavList", runtime::NAV_LIST)?;
        modules.push(nav_list);
    }

    if !modules.iter().any(|m| m.type_name == "PageOutline") {
        let page_outline =
            compile_single_module("PageOutline.rocci", "PageOutline", runtime::PAGE_OUTLINE)?;
        modules.push(page_outline);
    }

    if !modules.iter().any(|m| m.type_name == "DocsComponents") {
        let docs = compile_single_module("DocsComponents.rocci", "DocsComponents", runtime::DOCS)?;
        modules.push(docs);
    }

    if !modules.iter().any(|m| m.type_name == "BlockDebug") {
        let debug = compile_single_module("BlockDebug.rocci", "BlockDebug", runtime::BLOCK_DEBUG)?;
        modules.push(debug);
    }

    if !modules.iter().any(|m| m.type_name == "RocdownTheme") {
        if modules.iter().any(|m| m.type_name == "SiteShell") {
            let synth_roc = "import Html\nimport SiteShell\n\nRocdownTheme := [].{\n    siteShell = |view, content|\n        SiteShell.siteShell(view, content)\n}\n";
            modules.push(CompiledThemeModule {
                type_name: "RocdownTheme".to_string(),
                source_name: "RocdownTheme.roc".to_string(),
                src: synth_roc.to_string(),
                roc: synth_roc.to_string(),
                segments: Vec::new(),
                styles: Vec::new(),
                components: Vec::new(),
                component_infos: Vec::new(),
            });
        } else {
            bail!(
                "project theme in {} must define at least SiteShell.rocci or RocdownTheme.rocci",
                theme_dir.display()
            );
        }
    }

    Ok(modules)
}

fn compile_builtin_theme() -> Result<Vec<CompiledThemeModule>> {
    let base = compile_single_module("RocdownBase.rocci", "RocdownBase", runtime::BASE)?;
    let breadcrumbs =
        compile_single_module("Breadcrumbs.rocci", "Breadcrumbs", runtime::BREADCRUMBS)?;
    let nav_list = compile_single_module("NavList.rocci", "NavList", runtime::NAV_LIST)?;
    let page_outline =
        compile_single_module("PageOutline.rocci", "PageOutline", runtime::PAGE_OUTLINE)?;
    let theme = compile_single_module("RocdownTheme.rocci", "RocdownTheme", runtime::THEME)?;
    let docs = compile_single_module("DocsComponents.rocci", "DocsComponents", runtime::DOCS)?;
    let debug = compile_single_module("BlockDebug.rocci", "BlockDebug", runtime::BLOCK_DEBUG)?;
    Ok(vec![
        base,
        breadcrumbs,
        nav_list,
        page_outline,
        theme,
        docs,
        debug,
    ])
}

fn compile_block_pack(
    root: &Path,
    theme_dir: Option<&Path>,
    config: &SiteConfig,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<Vec<String>> {
    if let Some(pack) = &config.blocks.pack {
        return compile_pack_path(root, pack, modules);
    }
    if modules.iter().any(|module| module.type_name == "Blocks") {
        return Ok(vec!["Blocks".to_string()]);
    }
    let Some(theme_dir) = theme_dir else {
        return Ok(Vec::new());
    };
    let blocks_file = theme_dir.join("Blocks.rocci");
    if blocks_file.is_file() {
        return Ok(vec![ensure_theme_module(root, &blocks_file, modules)?]);
    }
    let pack_dir = theme_dir.join("blocks");
    if !pack_dir.is_dir() {
        return Ok(Vec::new());
    }
    compile_pack_dir(root, &pack_dir, modules)
}

fn compile_pack_path(
    root: &Path,
    pack: &str,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<Vec<String>> {
    let path = root.join(pack);
    if !path.exists() {
        bail!("blocks.pack `{pack}` does not exist in {}", root.display());
    }
    if path.is_dir() {
        return compile_pack_dir(root, &path, modules);
    }
    if path.extension().is_some_and(|ext| ext == "rocci") {
        Ok(vec![ensure_theme_module(root, &path, modules)?])
    } else {
        bail!("blocks.pack `{pack}` must be a .rocci file or directory of .rocci files");
    }
}

fn compile_pack_dir(
    root: &Path,
    dir: &Path,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rocci") {
            files.push(path);
        }
    }
    files.sort();
    let mut names = Vec::new();
    for file in files {
        names.push(ensure_theme_module(root, &file, modules)?);
    }
    Ok(names)
}

fn ensure_theme_module(
    root: &Path,
    file: &Path,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<String> {
    let type_name = rocci_template::type_name_from_path(file);
    let rel_name = file
        .strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    if let Some(existing) = modules.iter().find(|module| module.source_name == rel_name) {
        return Ok(existing.type_name.clone());
    }
    let src = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read {}", file.display()))?;
    modules.push(compile_single_module(&rel_name, &type_name, &src)?);
    Ok(type_name)
}

pub(crate) fn roc_fn_name(component: &str) -> String {
    let mut chars = component.chars();
    let first = chars.next().unwrap();
    first.to_lowercase().chain(chars).collect()
}

pub(crate) fn widget_kind_render_arms() -> String {
    let mut out = String::new();
    for spec in crate::registry::widget_specs() {
        if !spec.paints_as_widget() {
            continue;
        }
        out.push_str(&widget_kind_render_arm(*spec));
    }
    out
}

fn widget_kind_render_arm(spec: crate::registry::KindSpec) -> String {
    let painter = roc_fn_name(spec.component);
    let record = paint_record(spec);
    match spec.name {
        "tabs" => format!(
            "        Tabs(seg) => {{\n            (items, after) = render_tab_items!(segments, index + 1, seg.child_count)?\n            body = html_from_records(items)\n            Ok((BlockPainters.tabs({record}, body), after))\n        }}\n"
        ),
        "steps" => format!(
            "        Steps(seg) => {{\n            (items, after) = render_step_items!(segments, index + 1, seg.child_count)?\n            body = html_from_records(items)\n            Ok((BlockPainters.steps({record}, body), after))\n        }}\n"
        ),
        "card-grid" => format!(
            "        CardGrid(seg) => {{\n            (items, after) = render_card_items!(segments, index + 1, seg.child_count)?\n            body = html_from_records(items)\n            Ok((BlockPainters.cardGrid({record}, body), after))\n        }}\n"
        ),
        _ if spec.paint_content() => format!(
            "        {}(seg) => {{\n            (body, after) = render_children!(segments, index + 1, seg.child_count)?\n            Ok((BlockPainters.{painter}({record}, body), after))\n        }}\n",
            spec.component
        ),
        _ => format!(
            "        {}(seg) =>\n            Ok((BlockPainters.{painter}({record}), index + 1))\n",
            spec.component
        ),
    }
}

fn paint_record(spec: crate::registry::KindSpec) -> String {
    let props: Vec<String> = spec
        .paint_fields()
        .iter()
        .map(|field| format!("{}: seg.{}", field.prop, field.prop))
        .collect();
    if props.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", props.join(", "))
    }
}

fn module_exports_component(module: &CompiledThemeModule, component: &str) -> bool {
    let key = roc_fn_name(component);
    module
        .components
        .iter()
        .any(|name| name == component || roc_fn_name(name) == key)
}

fn lookup_painter_source<'a>(
    component: &str,
    pack_modules: &[&'a CompiledThemeModule],
    docs: Option<&'a CompiledThemeModule>,
) -> Option<&'a str> {
    for module in pack_modules {
        if module_exports_component(module, component) {
            return Some(module.type_name.as_str());
        }
    }
    if docs.is_some_and(|module| module_exports_component(module, component)) {
        return Some("DocsComponents");
    }
    None
}

fn debug_params_roc(spec: crate::registry::KindSpec) -> String {
    let fields = spec.paint_fields();
    if fields.is_empty() {
        return "\"\"".to_string();
    }
    let parts: Vec<String> = fields
        .iter()
        .map(|field| match field.ty {
            crate::registry::PaintType::Str => field.prop.to_string(),
            crate::registry::PaintType::Bool => {
                format!("(if {} {{ \"true\" }} else {{ \"false\" }})", field.prop)
            }
        })
        .collect();
    if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else {
        format!("Str.join_with([{}], \"\\n\")", parts.join(", "))
    }
}

fn props_pattern(spec: crate::registry::KindSpec) -> String {
    let fields = spec.paint_fields();
    if fields.is_empty() {
        "{}".to_string()
    } else {
        format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|field| field.prop)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn emit_debug_painter(roc: &mut String, spec: crate::registry::KindSpec) {
    let painter = roc_fn_name(spec.component);
    let props = props_pattern(spec);
    let params = debug_params_roc(spec);
    roc.push_str("    ");
    roc.push_str(&painter);
    if spec.paint_content() {
        roc.push_str(" = |");
        roc.push_str(&props);
        roc.push_str(", content|\n        BlockDebug.debug({ kind: \"");
        roc.push_str(spec.name);
        roc.push_str("\", params: ");
        roc.push_str(&params);
        roc.push_str(" }, content)\n");
    } else {
        roc.push_str(" = |");
        roc.push_str(&props);
        roc.push_str("|\n        BlockDebug.debug({ kind: \"");
        roc.push_str(spec.name);
        roc.push_str("\", params: ");
        roc.push_str(&params);
        roc.push_str(" }, Html.empty)\n");
    }
}

fn block_painters_module(
    modules: &[CompiledThemeModule],
    pack_names: &[String],
    overrides: &BTreeMap<String, String>,
    allow_debug: bool,
) -> Result<CompiledThemeModule> {
    let pack_modules: Vec<&CompiledThemeModule> = modules
        .iter()
        .filter(|module| pack_names.contains(&module.type_name))
        .collect();
    let docs = modules
        .iter()
        .find(|module| module.type_name == "DocsComponents");
    enum Binding {
        Component {
            painter: String,
            source: String,
            callee: String,
            paint_content: bool,
        },
        Debug(crate::registry::KindSpec),
    }
    let mut bindings = Vec::new();
    let mut uses_debug = false;
    let mut uses_html = false;
    for spec in crate::registry::widget_specs()
        .into_iter()
        .copied()
        .filter(|spec| spec.paints_as_widget())
    {
        let painter = roc_fn_name(spec.component);
        let (paint_as, required) = match overrides.get(spec.name) {
            Some(component) => (component.as_str(), true),
            None => (spec.component, false),
        };
        let callee = roc_fn_name(paint_as);
        match lookup_painter_source(paint_as, &pack_modules, docs) {
            Some(source) => bindings.push(Binding::Component {
                painter,
                source: source.to_string(),
                callee,
                paint_content: spec.paint_content(),
            }),
            None if required => {
                bail!(
                    "blocks.override `{}` names unknown component `{paint_as}`",
                    spec.name
                );
            }
            None if allow_debug => {
                uses_debug = true;
                if !spec.paint_content() {
                    uses_html = true;
                }
                bindings.push(Binding::Debug(spec));
            }
            None => {
                bail!(
                    "no renderer bound for kind `{}`; set [blocks] debug = true to paint a debug placeholder",
                    spec.name
                );
            }
        }
    }
    let mut imports = vec!["DocsComponents".to_string()];
    if uses_debug {
        imports.push("BlockDebug".to_string());
    }
    if uses_html {
        imports.push("Html".to_string());
    }
    for binding in &bindings {
        if let Binding::Component { source, .. } = binding
            && !imports.contains(source)
        {
            imports.push(source.clone());
        }
    }
    let mut roc = String::new();
    for import in &imports {
        roc.push_str("import ");
        roc.push_str(import);
        roc.push('\n');
    }
    roc.push_str("\nBlockPainters := [].{\n");
    for binding in &bindings {
        match binding {
            Binding::Component {
                painter,
                source,
                callee,
                paint_content,
            } => {
                roc.push_str("    ");
                roc.push_str(painter);
                if *paint_content {
                    roc.push_str(" = |props, content|\n        ");
                    roc.push_str(source);
                    roc.push('.');
                    roc.push_str(callee);
                    roc.push_str("(props, content)\n");
                } else {
                    roc.push_str(" = |props|\n        ");
                    roc.push_str(source);
                    roc.push('.');
                    roc.push_str(callee);
                    roc.push_str("(props)\n");
                }
            }
            Binding::Debug(spec) => emit_debug_painter(&mut roc, *spec),
        }
    }
    roc.push_str("}\n");
    Ok(CompiledThemeModule {
        type_name: "BlockPainters".to_string(),
        source_name: "BlockPainters.roc".to_string(),
        src: roc.clone(),
        roc,
        segments: Vec::new(),
        styles: Vec::new(),
        components: Vec::new(),
        component_infos: Vec::new(),
    })
}
