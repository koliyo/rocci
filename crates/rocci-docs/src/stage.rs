use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::catalog::{AppEntry, Catalog, DocsError, Hosting};
use crate::extract::declarations_markdown;
use crate::inventory::{PublishedFile, inventory_app};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageReport {
    pub apps: usize,
    pub files: usize,
}

pub fn live_demo_url(id: &str) -> String {
    format!("https://{id}.examples.rocci.dev")
}

pub fn app_play_url(app: &AppEntry) -> String {
    match &app.live_url {
        Some(url) if !url.is_empty() => url.clone(),
        _ => live_demo_url(&app.id),
    }
}

pub fn stage(catalog: &Catalog, output: &Path) -> Result<StageReport, DocsError> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| DocsError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let staging = parent.join(format!(
        ".{}.staging-{}",
        output
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| "example-docs".into()),
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|source| DocsError::Io {
            path: staging.clone(),
            source,
        })?;
    }
    match stage_into(catalog, &staging) {
        Ok(report) => {
            replace_dir(&staging, output)?;
            Ok(report)
        }
        Err(err) => {
            let _ = fs::remove_dir_all(&staging);
            Err(err)
        }
    }
}

fn stage_into(catalog: &Catalog, output: &Path) -> Result<StageReport, DocsError> {
    fs::create_dir_all(output).map_err(|source| DocsError::Io {
        path: output.to_path_buf(),
        source,
    })?;

    let mut files = 0;
    write_file(output.join("index.rocdown"), &catalog_index(catalog))?;
    files += 1;

    for app in &catalog.apps {
        files += stage_app(catalog, app, output)?;
    }

    Ok(StageReport {
        apps: catalog.apps.len(),
        files,
    })
}

fn replace_dir(staging: &Path, output: &Path) -> Result<(), DocsError> {
    let backup = output.with_file_name(format!(
        ".{}.old-{}",
        output
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| "example-docs".into()),
        std::process::id()
    ));
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|source| DocsError::Io {
            path: backup.clone(),
            source,
        })?;
    }
    if output.exists() {
        fs::rename(output, &backup).map_err(|source| DocsError::Io {
            path: output.to_path_buf(),
            source,
        })?;
        if let Err(source) = fs::rename(staging, output) {
            let _ = fs::rename(&backup, output);
            return Err(DocsError::Io {
                path: output.to_path_buf(),
                source,
            });
        }
        let _ = fs::remove_dir_all(&backup);
    } else {
        fs::rename(staging, output).map_err(|source| DocsError::Io {
            path: output.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn stage_app(catalog: &Catalog, app: &AppEntry, output: &Path) -> Result<usize, DocsError> {
    let app_src = catalog.root.join(&app.path);
    let app_out = output.join(&app.id);
    fs::create_dir_all(app_out.join("source")).map_err(|source| DocsError::Io {
        path: app_out.join("source"),
        source,
    })?;
    fs::create_dir_all(app_out.join("snippets")).map_err(|source| DocsError::Io {
        path: app_out.join("snippets"),
        source,
    })?;

    let mut written = 0;
    written += copy_authored_pages(&app_src, &app_out)?;

    let published = inventory_app(&catalog.root, app)?;
    write_file(
        app_out.join("source/index.rocdown"),
        &source_index(app, &published),
    )?;
    written += 1;

    for file in &published {
        let snippet = app_out.join("snippets").join(&file.relative);
        if let Some(parent) = snippet.parent() {
            fs::create_dir_all(parent).map_err(|source| DocsError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::copy(&file.absolute, &snippet).map_err(|source| DocsError::Io {
            path: snippet.clone(),
            source,
        })?;
        written += 1;

        let page_rel = source_page_rel(&file.relative);
        let page_path = app_out.join("source").join(page_rel);
        if let Some(parent) = page_path.parent() {
            fs::create_dir_all(parent).map_err(|source| DocsError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let rocci_src = if file.relative.ends_with(".rocci") {
            Some(
                fs::read_to_string(&file.absolute).map_err(|source| DocsError::Io {
                    path: file.absolute.clone(),
                    source,
                })?,
            )
        } else {
            None
        };
        write_file(page_path, &source_page(app, file, rocci_src.as_deref()))?;
        written += 1;
    }
    Ok(written)
}

fn copy_authored_pages(app_src: &Path, app_out: &Path) -> Result<usize, DocsError> {
    let mut written = 0;
    copy_rocdown(app_src, app_src, app_out, &mut written)?;
    Ok(written)
}

fn copy_rocdown(
    app_src: &Path,
    dir: &Path,
    app_out: &Path,
    written: &mut usize,
) -> Result<(), DocsError> {
    let entries = fs::read_dir(dir).map_err(|source| DocsError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DocsError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|source| DocsError::Io {
            path: path.clone(),
            source,
        })?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            let name = entry.file_name();
            if matches!(
                name.to_string_lossy().as_ref(),
                "generated" | "target" | "dist" | ".git"
            ) {
                continue;
            }
            copy_rocdown(app_src, &path, app_out, written)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rocdown") {
            continue;
        }
        let relative = path
            .strip_prefix(app_src)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let dest = app_out.join(&relative);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|source| DocsError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::copy(&path, &dest).map_err(|source| DocsError::Io { path: dest, source })?;
        *written += 1;
    }
    Ok(())
}

fn catalog_index(catalog: &Catalog) -> String {
    let mut rows = String::new();
    for app in &catalog.apps {
        let live = if app.hosting == Hosting::Live {
            let planned = if app.live_url.as_deref().unwrap_or("").is_empty() {
                " (planned)"
            } else {
                ""
            };
            format!(" · [Open live demo]({}){planned}", app_play_url(app))
        } else {
            String::new()
        };
        let role = if app.complexity.is_empty() {
            "—"
        } else {
            app.complexity.as_str()
        };
        let persistence = if app.persistence.is_empty() {
            "—"
        } else {
            app.persistence.as_str()
        };
        rows.push_str(&format!(
            "| [{title}](/examples/{id}/) | {role} | {persistence} | {summary} | `{hosting}` · [source](/examples/{id}/source/){live} |\n",
            title = app.title,
            id = app.id,
            summary = app.summary,
            hosting = app.hosting.as_str(),
        ));
    }
    format!(
        r#"@page {{
    layout: "docs",
    aliases: ["/docs/examples/"],
    meta: {{
        title: "Examples",
        description: "Cataloged Rocci applications with authored docs and a full source tree.",
    }},
}}

# Examples

These pages come from `examples/rocci/apps.toml`. Each app has authored docs
and a complete highlighted source tree. Source and local run paths stay useful
without a public demo.

Rocdown is an optional experimental document layer. See [Rocdown](/docs/rocdown/).

Live demo hostnames (`<id>.examples.rocci.dev`) are **planned** until a staging
deploy has served them. Only catalog `live` apps advertise those URLs.

Roles: **learning** (first component and first app), **reference** (handler
matrix), **pattern** (Datastar gallery), **advanced** (Snake stress demo).

| App | Role | Persistence | Summary | Hosting |
| --- | --- | --- | --- | --- |
{rows}"#
    )
}

fn source_index(app: &AppEntry, files: &[PublishedFile]) -> String {
    let mut items = String::new();
    for file in files {
        let href = format!(
            "/examples/{id}/source/{path}/",
            id = app.id,
            path = source_page_id(&file.relative)
        );
        items.push_str(&format!("- [{name}]({href})\n", name = file.relative));
    }
    format!(
        r#"@page {{
    layout: "docs",
    meta: {{
        title: "{title} source",
        description: "Published source files for {title}.",
    }},
}}

# {title} source

{items}"#,
        title = escape_text(&app.title),
    )
}

fn source_page(app: &AppEntry, file: &PublishedFile, rocci_src: Option<&str>) -> String {
    let docs = rocci_src
        .map(declarations_markdown)
        .filter(|md| !md.is_empty())
        .unwrap_or_default();
    format!(
        r#"@page {{
    layout: "docs",
    meta: {{
        title: "{name}",
        description: "Published source for {title}: {name}.",
    }},
}}

# {name}

See the [{title} tutorial](/examples/{id}/) for context.

{docs}:include[path: "{include}"]
"#,
        name = escape_text(&file.relative),
        title = escape_text(&app.title),
        id = app.id,
        include = file.relative,
        docs = docs,
    )
}

fn source_page_rel(relative: &str) -> String {
    format!("{}.rocdown", source_page_id(relative))
}

fn source_page_id(relative: &str) -> String {
    relative.replace('/', "--").replace('.', "-")
}

fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn write_file(path: PathBuf, contents: &str) -> Result<(), DocsError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| DocsError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&path, contents).map_err(|source| DocsError::Io { path, source })
}

#[cfg(test)]
mod tests {
    #[test]
    fn explicit_live_url_overrides_example_hostname() {
        let app = crate::catalog::AppEntry {
            id: "blocks".into(),
            path: "custom/blocks".into(),
            title: "Rocci Blocks".into(),
            summary: "demo".into(),
            entry: ".".into(),
            hosting: crate::catalog::Hosting::Live,
            files: Vec::new(),
            audience: String::new(),
            purpose: String::new(),
            complexity: String::new(),
            persistence: String::new(),
            support: String::new(),
            live_url: Some("https://rocci.dev/play/blocks/".into()),
        };
        assert_eq!(super::app_play_url(&app), "https://rocci.dev/play/blocks/");
        assert_eq!(
            super::live_demo_url("blocks"),
            "https://blocks.examples.rocci.dev"
        );
    }
}
