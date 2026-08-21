use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::catalog::{AppEntry, Catalog, DocsError, Hosting};
use crate::inventory::{PublishedFile, inventory_app};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageReport {
    pub apps: usize,
    pub files: usize,
}

pub fn live_demo_url(id: &str) -> String {
    format!("https://{id}.examples.rocci.dev")
}

pub fn stage(catalog: &Catalog, output: &Path) -> Result<StageReport, DocsError> {
    if output.exists() {
        fs::remove_dir_all(output).map_err(|source| DocsError::Io {
            path: output.to_path_buf(),
            source,
        })?;
    }
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
        write_file(page_path, &source_page(app, file))?;
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
            format!(" · [Open live demo]({})", live_demo_url(&app.id))
        } else {
            String::new()
        };
        rows.push_str(&format!(
            "| [{title}](/examples/{id}/) | {summary} | `{hosting}` · [source](/examples/{id}/source/){live} |\n",
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
and a complete highlighted source tree. Live demos use a separate hostname
when hosting is `live`; those origins are planned until a staging deploy has
served them.

Rocdown site examples are not generated here. Run them locally from the
repository:

| Example | Run |
| --- | --- |
| `rocdown/pages` | `rocdown run examples/rocdown/pages/Guide.rocdown` |
| `rocdown/pages` blocks | `rocdown run examples/rocdown/pages/Blocks.rocdown` |
| `rocdown/errors` | `rocdown run examples/rocdown/errors/ErrorDemo.rocdown` |
| `rocdown/site` | `rocdown build examples/rocdown/site --output dist` |
| `rocdown/counter` | `rocdown run examples/rocdown/counter` |
| `rocdown/hybrid` | `rocdown run examples/rocdown/hybrid` |

| App | Summary | Hosting |
| --- | --- | --- |
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

fn source_page(app: &AppEntry, file: &PublishedFile) -> String {
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

:include[path: "{include}"]
"#,
        name = escape_text(&file.relative),
        title = escape_text(&app.title),
        id = app.id,
        include = file.relative,
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
