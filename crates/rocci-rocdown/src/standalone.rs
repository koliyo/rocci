use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    Diagnostic, InitInfo, MappedModule, RouteInfo, SourceFile, type_name_from_path,
};
use rocci_theme::ThemeOptions;

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

    let mut modules = Vec::new();
    let mut failures = Vec::new();
    let inputs = linked_standalone_inputs(&primary)?;

    let pages = primary
        .parent()
        .map(crate::index_pages_in_dir)
        .unwrap_or_default();

    for input in inputs {
        let src = fs::read_to_string(&input)
            .with_context(|| format!("failed to read {}", input.display()))?;
        let name = input.display().to_string();
        let source = SourceFile::new(&name, &src);

        let options = CompileOptions {
            theme: theme.clone(),
            check_assets: true,
            pages: pages.clone(),
            ..CompileOptions::default()
        };
        let compiled = compile(source, &options);

        if compiled.has_errors() {
            failures.push(StandaloneFailedFile {
                name,
                src,
                diagnostics: compiled.diagnostics,
            });
            continue;
        }

        let type_name = type_name_from_path(&input);
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
                type_name,
                generated: compiled.roc,
                source_name: name,
                source_src: src,
                segments: compiled.segments,
            },
            local_assets,
        });
    }

    if !failures.is_empty() {
        return Ok(StandaloneReady::Failed(failures));
    }

    let redirect_trailing_slash =
        redirect_trailing_slash_for(primary.parent().unwrap_or_else(|| Path::new(".")));

    Ok(StandaloneReady::Ready(StandalonePlan {
        primary_name: type_name_from_path(&primary),
        modules,
        redirect_trailing_slash,
    }))
}

pub fn linked_standalone_inputs(primary: &Path) -> Result<Vec<PathBuf>> {
    let primary = primary
        .canonicalize()
        .unwrap_or_else(|_| primary.to_path_buf());
    if !primary.extension().is_some_and(|ext| ext == "rocdown") {
        return Ok(vec![primary]);
    }
    let Some(dir) = primary.parent() else {
        return Ok(vec![primary]);
    };
    let mut files: Vec<PathBuf> = discover_rocdown_files(dir)?
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "rocdown"))
        .map(|path| path.canonicalize().unwrap_or(path))
        .collect();
    files.sort();
    files.dedup();
    if let Some(index) = files.iter().position(|path| path == &primary) {
        files.swap(0, index);
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

fn redirect_trailing_slash_for(dir: &Path) -> bool {
    let path = dir.join("rocci.toml");
    if !path.is_file() {
        return true;
    }
    rocci_core::Config::from_file(path)
        .map(|config| config.http.redirect_trailing_slash)
        .unwrap_or(true)
}
