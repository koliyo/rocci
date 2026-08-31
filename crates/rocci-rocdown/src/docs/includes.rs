use std::path::{Component, Path, PathBuf};

use rocci_template::{SourceFile, Span};

use crate::ast::MdNode;
use crate::catalog::CatalogDiagnostic;

use super::tree::nodes_from_items;
use super::{ArticleNode, BuildCtx, DocsAttrs, DocsNode, IncludeOrigin};

pub fn include_path_error(path: &str) -> Option<String> {
    if path.is_empty() {
        return Some("include path must not be empty".into());
    }
    if path.contains('\0') {
        return Some("include path must not contain NUL bytes".into());
    }
    if Path::new(path).is_absolute() {
        return Some("include path must be relative".into());
    }
    if path.contains("..") {
        return Some("include path must not contain `..`".into());
    }
    None
}

pub fn extract_region(text: &str, name: &str) -> Result<(String, usize, usize), String> {
    let start_marker = format!("docs-region: {name}");
    let end_marker = format!("docs-region-end: {name}");
    let mut start_line = None;
    let mut end_line = None;
    for (index, line) in text.lines().enumerate() {
        if line.contains(&start_marker) {
            if start_line.is_some() {
                return Err(format!("duplicate region `{name}`"));
            }
            start_line = Some(index);
        }
        if line.contains(&end_marker) {
            if end_line.is_some() {
                return Err(format!("duplicate region end `{name}`"));
            }
            end_line = Some(index);
        }
    }
    let Some(start) = start_line else {
        return Err(format!("missing region `{name}`"));
    };
    let Some(end) = end_line else {
        return Err(format!("unclosed region `{name}`"));
    };
    if end <= start {
        return Err(format!("region `{name}` ends before it starts"));
    }
    let excerpt = text
        .lines()
        .skip(start + 1)
        .take(end.saturating_sub(start + 1))
        .collect::<Vec<_>>()
        .join("\n");
    Ok((excerpt, start + 2, end))
}

pub fn extract_lines(text: &str, start: u32, end: u32) -> Result<(String, usize, usize), String> {
    if start == 0 || end == 0 || end < start {
        return Err("line range must be 1-based with end >= start".into());
    }
    let lines: Vec<&str> = text.lines().collect();
    let from = start as usize;
    let to = end as usize;
    if to > lines.len() {
        return Err(format!(
            "line range {start}-{end} is past the end of the file"
        ));
    }
    Ok((lines[from - 1..to].join("\n"), from, to))
}

pub fn resolve_include_path(from_file: &str, path: &str) -> Result<PathBuf, String> {
    if let Some(err) = include_path_error(path) {
        return Err(err);
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
                return Err("include path must not contain `..`".into());
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Ok(out)
}

pub(crate) fn include_node(
    ctx: &mut BuildCtx<'_>,
    span: Span,
    attrs: DocsAttrs,
    line: u32,
) -> Option<DocsNode> {
    let Some(path) = attrs.path.clone() else {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2501",
            ctx.source_path,
            format!("line {line}: `:include` requires `path`"),
        ));
        return None;
    };
    if attrs.start.is_some() || attrs.end.is_some() {
        ctx.diagnostics.push(CatalogDiagnostic::warning(
            "RD2504",
            ctx.source_path,
            format!("line {line}: include line ranges are fragile; prefer a named region"),
        ));
    }
    let resolved = match resolve_allowed_path(ctx, &path) {
        Ok(path) => path,
        Err(err) => {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2501",
                ctx.source_path,
                format!("line {line}: {err}"),
            ));
            return None;
        }
    };
    if ctx.stack.iter().any(|seen| seen == &resolved) {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2505",
            ctx.source_path,
            format!(
                "line {line}: cyclic include `{}`",
                display_rel(ctx, &resolved)
            ),
        ));
        return None;
    }
    let contents = match std::fs::read_to_string(&resolved) {
        Ok(contents) => contents,
        Err(_) => {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2501",
                ctx.source_path,
                format!(
                    "line {line}: missing include `{}`",
                    display_rel(ctx, &resolved)
                ),
            ));
            return None;
        }
    };
    let (excerpt, line_start, line_end) = if let Some(region) = attrs.region.as_deref() {
        match extract_region(&contents, region) {
            Ok((excerpt, start, end)) => (excerpt, Some(start as u32), Some(end as u32)),
            Err(err) => {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2502",
                    ctx.source_path,
                    format!("line {line}: {err}"),
                ));
                return None;
            }
        }
    } else if let (Some(start), Some(end)) = (attrs.start, attrs.end) {
        match extract_lines(&contents, start, end) {
            Ok((excerpt, from, to)) => (excerpt, Some(from as u32), Some(to as u32)),
            Err(err) => {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2501",
                    ctx.source_path,
                    format!("line {line}: {err}"),
                ));
                return None;
            }
        }
    } else {
        (contents, None, None)
    };
    let origin = IncludeOrigin {
        source_path: authored_origin_path(ctx, &path, &resolved),
        region: attrs.region.clone(),
        line_start,
        line_end,
    };
    ctx.origins.push(origin.clone());
    ctx.snippet_paths.insert(origin.source_path.clone());
    if resolved.extension().and_then(|ext| ext.to_str()) == Some("rocdown") {
        ctx.stack.push(resolved.clone());
        let included = crate::parse(SourceFile::new(&origin.source_path, &excerpt), false);
        if included
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
        {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2503",
                ctx.source_path,
                format!(
                    "line {line}: included Rocdown `{}` has parse errors",
                    origin.source_path
                ),
            ));
        }
        let children = nodes_from_items(ctx, &included.document.items, Some("include"));
        ctx.stack.pop();
        return Some(DocsNode {
            kind: "include".into(),
            attrs,
            children,
            origin: Some(origin),
            line,
        });
    }
    let language = attrs
        .language
        .clone()
        .or_else(|| {
            resolved
                .extension()
                .and_then(|ext| ext.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let code = ArticleNode::Markdown(MdNode::CodeBlock {
        info: language,
        literal: excerpt,
        span,
    });
    Some(DocsNode {
        kind: "include".into(),
        attrs,
        children: vec![code],
        origin: Some(origin),
        line,
    })
}

fn resolve_allowed_path(ctx: &BuildCtx<'_>, path: &str) -> Result<PathBuf, String> {
    let from = ctx.includes.root.join(ctx.source_path);
    let relative = resolve_include_path(&from.to_string_lossy(), path)?;
    let mut candidates = vec![
        ctx.includes.root.join(path),
        ctx.includes.root.join(&relative),
    ];
    if let Some(parent) = Path::new(ctx.source_path).parent() {
        candidates.push(ctx.includes.root.join(parent).join(path));
    }
    for root in ctx.includes.snippet_roots {
        candidates.push(root.join(path));
    }
    let mut allowed = vec![ctx.includes.root.to_path_buf()];
    allowed.extend(ctx.includes.snippet_roots.iter().cloned());
    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let canonical = std::fs::canonicalize(&candidate)
            .map_err(|_| format!("include path `{}` is not readable", path))?;
        if !allowed.iter().any(|root| {
            std::fs::canonicalize(root)
                .ok()
                .is_some_and(|root| canonical.starts_with(root))
        }) {
            return Err(format!(
                "include path `{path}` escapes allowed snippet roots"
            ));
        }
        return Ok(canonical);
    }
    Err(format!("missing include `{path}`"))
}

fn authored_origin_path(ctx: &BuildCtx<'_>, authored: &str, resolved: &Path) -> String {
    if include_path_error(authored).is_none() && !authored.is_empty() {
        authored.replace('\\', "/")
    } else {
        display_rel(ctx, resolved)
    }
}

fn display_rel(ctx: &BuildCtx<'_>, path: &Path) -> String {
    let root = std::fs::canonicalize(ctx.includes.root)
        .unwrap_or_else(|_| ctx.includes.root.to_path_buf());
    if let Ok(rel) = path.strip_prefix(&root) {
        return rel.to_string_lossy().replace('\\', "/");
    }
    if let Ok(canonical) = std::fs::canonicalize(path)
        && let Ok(rel) = canonical.strip_prefix(&root)
    {
        return rel.to_string_lossy().replace('\\', "/");
    }
    for snippet_root in ctx.includes.snippet_roots {
        let snippet_root =
            std::fs::canonicalize(snippet_root).unwrap_or_else(|_| snippet_root.clone());
        if let Ok(rel) = path.strip_prefix(&snippet_root) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("include")
        .to_string()
}
