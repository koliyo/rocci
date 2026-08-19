//! Resolve `@use "./Module.rocci"` into imported article kinds.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use rocci_template::{
    Diagnostic, LowerOptions, ModuleItem, SourceFile, Span, StyleArtifact, pascal_to_camel,
};

use crate::ast::{Document, Item, UseDecl};
use crate::docs::include_path_error;
use crate::page::string_literal;
use crate::registry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedKind {
    pub kind: String,
    pub component: String,
    pub roc_name: String,
}

#[derive(Clone, Debug)]
pub struct CompiledUse {
    pub roc: String,
    pub styles: Vec<StyleArtifact>,
    pub kinds: Vec<ImportedKind>,
    pub defaults: Vec<(String, Vec<(String, String)>)>,
    pub has_datastar: bool,
}

pub fn imported_kind_names(
    source: SourceFile<'_>,
    document: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut seen = HashMap::new();
    for decl in use_decls(document) {
        let kinds = load_kind_exports(source, decl, diagnostics);
        for kind in kinds {
            if let Some(previous) = seen.insert(kind.kind.clone(), decl.path.clone()) {
                diagnostics.push(Diagnostic::error(
                    decl.path_span,
                    format!(
                        "duplicate imported kind `:{}` from `{previous}` and `{}`",
                        kind.kind, decl.path
                    ),
                ));
                continue;
            }
            names.insert(kind.kind);
        }
    }
    names
}

pub fn compile_modules(
    source: SourceFile<'_>,
    document: &Document,
    options: &LowerOptions,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<CompiledUse> {
    let mut compiled = Vec::new();
    for decl in use_decls(document) {
        let Some(path) = resolved_use_file(source, decl) else {
            continue;
        };
        let Ok(src) = fs::read_to_string(&path) else {
            continue;
        };
        let name = path.display().to_string();
        let file = SourceFile::new(&name, &src);
        let output = rocci_template::compile(file, options);
        if output.has_errors() {
            let first = output
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.is_error())
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("module has errors");
            diagnostics.push(Diagnostic::error(
                decl.path_span,
                format!("failed to compile `{}`: {first}", decl.path),
            ));
            continue;
        }
        let kinds = kinds_from_document(&output.document, decl, diagnostics);
        if kinds.is_empty() {
            continue;
        }
        let defaults = output
            .components
            .iter()
            .map(|component| (component.name.clone(), component.param_defaults.clone()))
            .collect();
        compiled.push(CompiledUse {
            roc: strip_runtime_imports(&output.roc),
            styles: output.styles,
            kinds,
            defaults,
            has_datastar: output.roc.contains("import Datastar"),
        });
    }
    compiled
}

fn use_decls(document: &Document) -> impl Iterator<Item = &UseDecl> {
    document.items.iter().filter_map(|item| match item {
        Item::Use(decl) => Some(decl),
        _ => None,
    })
}

fn load_kind_exports(
    source: SourceFile<'_>,
    decl: &UseDecl,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ImportedKind> {
    let Some(path) = resolve_use_path(source.name, &decl.path, decl.path_span, diagnostics) else {
        return Vec::new();
    };
    let src = match fs::read_to_string(&path) {
        Ok(src) => src,
        Err(_) => {
            diagnostics.push(Diagnostic::error(
                decl.path_span,
                format!("`{}` does not exist", decl.path),
            ));
            return Vec::new();
        }
    };
    let name = path.display().to_string();
    let file = SourceFile::new(&name, &src);
    let parsed = rocci_template::parse(file);
    if parsed.diagnostics.iter().any(Diagnostic::is_error) {
        let first = parsed
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.is_error())
            .map(|diagnostic| diagnostic.message.as_str())
            .unwrap_or("module has errors");
        diagnostics.push(Diagnostic::error(
            decl.path_span,
            format!("failed to parse `{}`: {first}", decl.path),
        ));
        return Vec::new();
    }
    kinds_from_document(&parsed.document, decl, diagnostics)
}

fn kinds_from_document(
    document: &rocci_template::Document,
    decl: &UseDecl,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ImportedKind> {
    let mut kinds = Vec::new();
    for item in &document.items {
        let ModuleItem::Component(component) = item else {
            continue;
        };
        let kind = pascal_to_kebab(&component.name.name);
        if registry::lookup(&kind).is_some() || registry::module_collision(&kind) {
            diagnostics.push(Diagnostic::error(
                decl.path_span,
                format!(
                    "`{}` collides with builtin kind `:{kind}`",
                    component.name.name
                ),
            ));
            continue;
        }
        kinds.push(ImportedKind {
            kind,
            component: component.name.name.clone(),
            roc_name: pascal_to_camel(&component.name.name),
        });
    }
    if kinds.is_empty() {
        diagnostics.push(Diagnostic::error(
            decl.path_span,
            format!("`{}` does not export an `@component`", decl.path),
        ));
    }
    kinds
}

fn resolved_use_file(source: SourceFile<'_>, decl: &UseDecl) -> Option<PathBuf> {
    let mut ignored = Vec::new();
    resolve_use_path(source.name, &decl.path, decl.path_span, &mut ignored)
        .filter(|path| path.is_file())
}

fn resolve_use_path(
    from_file: &str,
    path: &str,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<PathBuf> {
    if path.is_empty() {
        diagnostics.push(Diagnostic::error(
            span,
            "expected a `.rocci` module path after `@use`".to_string(),
        ));
        return None;
    }
    if let Some(err) = include_path_error(path) {
        diagnostics.push(Diagnostic::error(
            span,
            err.replace("include path", "`@use` path"),
        ));
        return None;
    }
    if !path.ends_with(".rocci") {
        diagnostics.push(Diagnostic::error(
            span,
            format!("`@use` requires a `.rocci` module, not `{path}`"),
        ));
        return None;
    }
    let from_file = from_file.strip_prefix("file://").unwrap_or(from_file);
    let base = Path::new(from_file)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let joined = base.join(path);
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                diagnostics.push(Diagnostic::error(
                    span,
                    "`@use` path must not contain `..`".to_string(),
                ));
                return None;
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Some(out)
}

pub fn pascal_to_kebab(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('-');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn strip_runtime_imports(roc: &str) -> String {
    let mut out = String::new();
    for line in roc.lines() {
        let trimmed = line.trim();
        if trimmed == "import Html" || trimmed == "import Datastar" {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn parse_use_path(src: &str, at: usize, end: usize) -> (String, Span) {
    let mut cur = rocci_template::Cursor::at(src, at);
    cur.eat('@');
    cur.scan_ident();
    cur.skip_trivia();
    let start = cur.pos;
    cur.skip_string();
    let path_end = cur.pos.min(end);
    let span = Span::new(start, path_end);
    let path = string_literal(src, span).unwrap_or_default();
    (path, span)
}

#[cfg(test)]
mod tests {
    use super::pascal_to_kebab;

    #[test]
    fn pascal_component_names_become_kebab_kinds() {
        assert_eq!(pascal_to_kebab("Callout"), "callout");
        assert_eq!(pascal_to_kebab("LinkCard"), "link-card");
        assert_eq!(pascal_to_kebab("FileTree"), "file-tree");
    }
}
