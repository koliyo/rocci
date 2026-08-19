use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    Diagnostic, InitInfo, MappedModule, RouteInfo, SourceFile, type_name_from_path,
};
use rocci_theme::ThemeOptions;

use crate::links::{
    PageRef, has_scheme, is_document_href, normalize_components, normalize_join, percent_decode,
    split_fragment,
};
use crate::{CompileOptions, compile};

#[derive(Debug, Clone)]
pub struct StandaloneModule {
    pub type_name: String,
    pub roc: String,
    pub state_type: Option<String>,
    pub init: Option<InitInfo>,
    pub routes: Vec<RouteInfo>,
    pub mapped: MappedModule,
    pub local_assets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StandaloneFailedFile {
    pub name: String,
    pub src: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct StandalonePlan {
    pub primary_name: String,
    pub modules: Vec<StandaloneModule>,
    pub redirect_trailing_slash: bool,
}

pub enum StandaloneReady {
    Ready(StandalonePlan),
    Failed(Vec<StandaloneFailedFile>),
}

pub fn plan_standalone(primary: &Path, theme: &ThemeOptions) -> Result<StandaloneReady> {
    let primary = if primary.is_absolute() {
        primary.to_path_buf()
    } else {
        std::env::current_dir()?.join(primary)
    };
    if !primary.is_file() {
        bail!("no such Rocdown file: {}", primary.display());
    }
    let primary = primary.canonicalize().unwrap_or(primary);

    let mut modules = Vec::new();
    let mut failures = Vec::new();
    let inputs = linked_standalone_inputs(&primary)?;
    let root = primary
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut sources = Vec::with_capacity(inputs.len());
    for input in &inputs {
        let src = fs::read_to_string(input)
            .with_context(|| format!("failed to read {}", input.display()))?;
        sources.push(src);
    }

    let pages: Vec<PageRef> = inputs
        .iter()
        .zip(sources.iter())
        .map(|(path, src)| preview_page_ref(path, src, &primary, &root))
        .collect();
    let type_names = unique_type_names(&root, &inputs);

    for (((input, src), page), type_name) in inputs
        .iter()
        .zip(sources.iter())
        .zip(pages.iter())
        .zip(type_names.iter())
    {
        let name = input.display().to_string();
        let source = SourceFile::new(&name, src);
        let default_route = if input == &primary || page.explicit_route {
            None
        } else {
            Some(page.route.clone())
        };
        let options = CompileOptions {
            theme: theme.clone(),
            check_assets: true,
            pages: pages.clone(),
            default_route,
            ..CompileOptions::default()
        };
        let compiled = compile(source, &options);

        if compiled.has_errors() {
            failures.push(StandaloneFailedFile {
                name,
                src: src.clone(),
                diagnostics: compiled.diagnostics,
            });
            continue;
        }

        let local_assets = crate::collect_local_media(source, &compiled.document)
            .into_iter()
            .map(|(url, _)| url)
            .collect();

        modules.push(StandaloneModule {
            type_name: type_name.clone(),
            roc: compiled.roc.clone(),
            state_type: compiled.state_type,
            init: compiled.init,
            routes: compiled.routes,
            mapped: MappedModule {
                type_name: type_name.clone(),
                generated: compiled.roc,
                source_name: name,
                source_src: src.clone(),
                segments: compiled.segments,
            },
            local_assets,
        });
    }

    if !failures.is_empty() {
        return Ok(StandaloneReady::Failed(failures));
    }

    let redirect_trailing_slash = redirect_trailing_slash_for(&root);

    Ok(StandaloneReady::Ready(StandalonePlan {
        primary_name: type_names[0].clone(),
        modules,
        redirect_trailing_slash,
    }))
}

pub fn linked_standalone_inputs(primary: &Path) -> Result<Vec<PathBuf>> {
    let primary = primary
        .canonicalize()
        .unwrap_or_else(|_| primary.to_path_buf());
    let mut files = Vec::new();
    let mut seen = HashSet::new();
    enqueue(&mut files, &mut seen, primary.clone());

    if primary.extension().is_some_and(|ext| ext == "rocdown")
        && let Some(dir) = primary.parent()
    {
        for path in discover_rocdown_files(dir)? {
            if path.extension().is_some_and(|ext| ext == "rocdown") {
                enqueue(&mut files, &mut seen, path.canonicalize().unwrap_or(path));
            }
        }
    }

    let mut index = 0;
    while index < files.len() {
        let path = files[index].clone();
        index += 1;
        let Ok(src) = fs::read_to_string(&path) else {
            continue;
        };
        for target in linked_document_targets(&path, &src) {
            enqueue(&mut files, &mut seen, target);
        }
    }

    if let Some(position) = files.iter().position(|path| path == &primary) {
        files.swap(0, position);
    } else {
        files.insert(0, primary);
    }
    Ok(files)
}

pub fn discover_rocdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "rocdown" || ext == "md" || ext == "markdown")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn enqueue(files: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>, path: PathBuf) {
    if seen.insert(path.clone()) {
        files.push(path);
    }
}

fn preview_page_ref(path: &Path, src: &str, primary: &Path, root: &Path) -> PageRef {
    let mut page = crate::page_ref_from_source(path, src);
    if !page.explicit_route {
        page.route = if path == primary {
            "/".to_string()
        } else {
            derived_preview_route(root, path)
        };
    }
    page
}

fn derived_preview_route(root: &Path, file: &Path) -> String {
    let rel = file
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| {
            file.file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| file.to_path_buf())
        });
    let mut route = String::from("/");
    let rel = rel.to_string_lossy().replace('\\', "/");
    route.push_str(rel.trim_start_matches('/'));
    route
}

fn unique_type_names(root: &Path, inputs: &[PathBuf]) -> Vec<String> {
    let preferred: Vec<String> = inputs
        .iter()
        .map(|path| type_name_from_path(path))
        .collect();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for name in &preferred {
        *counts.entry(name.clone()).or_insert(0) += 1;
    }
    let mut used = HashSet::new();
    inputs
        .iter()
        .zip(preferred)
        .map(|(path, stem)| {
            let mut name = if counts.get(&stem) == Some(&1) {
                stem
            } else {
                pascal_from_rel(path.strip_prefix(root).unwrap_or(path))
            };
            if name.is_empty() {
                name = "View".to_string();
            }
            if !used.insert(name.clone()) {
                let mut index = 2;
                while !used.insert(format!("{name}{index}")) {
                    index += 1;
                }
                name = format!("{name}{index}");
            }
            name
        })
        .collect()
}

fn pascal_from_rel(rel: &Path) -> String {
    let mut out = String::new();
    let file_name = rel.file_name();
    for component in rel.components() {
        let raw = component.as_os_str();
        let text = if file_name == Some(raw) {
            Path::new(raw)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| raw.to_string_lossy().into_owned())
        } else {
            raw.to_string_lossy().into_owned()
        };
        out.push_str(&pascal_segment(&text));
    }
    if out.is_empty() {
        "View".to_string()
    } else {
        out
    }
}

fn pascal_segment(text: &str) -> String {
    let mut out = String::new();
    let mut cap_next = true;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            if cap_next {
                out.extend(ch.to_uppercase());
                cap_next = false;
            } else {
                out.push(ch);
            }
        } else {
            cap_next = true;
        }
    }
    out
}

fn linked_document_targets(from: &Path, src: &str) -> Vec<PathBuf> {
    let name = from.display().to_string();
    let parsed = crate::parse(SourceFile::new(&name, src), false);
    let mut targets = Vec::new();
    let mut seen = HashSet::new();
    for link in parsed.links {
        let decoded = percent_decode(&link.url);
        let (path, _) = split_fragment(&decoded);
        if let Some(target) = resolve_document_target(from, path)
            && seen.insert(target.clone())
        {
            targets.push(target);
        }
    }
    targets
}

fn resolve_document_target(from: &Path, href: &str) -> Option<PathBuf> {
    if href.is_empty() || has_scheme(href) {
        return None;
    }
    if href.starts_with('/') {
        return resolve_absolute_document(from, href);
    }
    let parent = from.parent()?;
    if is_document_href(href) {
        let candidate = normalize_join(parent, href);
        return existing_document(&candidate);
    }
    if href.contains('/') || href.contains('\\') {
        return None;
    }
    let stem = href
        .strip_suffix(".rocdown")
        .or_else(|| href.strip_suffix(".markdown"))
        .or_else(|| href.strip_suffix(".md"))
        .unwrap_or(href);
    if stem.is_empty() {
        return None;
    }
    for ext in ["rocdown", "md", "markdown"] {
        let candidate = parent.join(format!("{stem}.{ext}"));
        if let Some(path) = existing_document(&candidate) {
            return Some(path);
        }
    }
    None
}

fn resolve_absolute_document(from: &Path, href: &str) -> Option<PathBuf> {
    if !is_document_href(href) {
        return None;
    }
    let rel = href.trim_start_matches('/');
    let mut dir = from.parent()?;
    for _ in 0..16 {
        let candidate = normalize_components(dir.join(rel));
        if let Some(path) = existing_document(&candidate) {
            return Some(path);
        }
        dir = dir.parent()?;
    }
    None
}

fn existing_document(path: &Path) -> Option<PathBuf> {
    if path.is_file()
        && path
            .extension()
            .is_some_and(|ext| ext == "rocdown" || ext == "md" || ext == "markdown")
    {
        Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
    } else {
        None
    }
}

fn redirect_trailing_slash_for(dir: &Path) -> bool {
    let path = dir.join("rocci.toml");
    if !path.is_file() {
        return true;
    }
    rocci_core::Config::from_file(path)
        .map(|config| config.http.redirect_trailing_slash)
        .unwrap_or(true)
}
