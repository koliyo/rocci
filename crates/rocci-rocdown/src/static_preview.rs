use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rocci_template::{Diagnostic, SourceFile};

use crate::docs::{IncludeOptions, load_page_docs, render_article};
use crate::page::extract_page;
use crate::{CompileOptions, Document, Item, PageKind, classify_document, parse};

pub fn document_page_kind(document: &Document) -> PageKind {
    classify_document(document, false).kind
}

pub fn write_static_document_preview(
    input: &Path,
    out_dir: &Path,
    options: &CompileOptions,
) -> Result<String> {
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let parsed = parse(source, options.raw_html);
    if parsed.diagnostics.iter().any(Diagnostic::is_error) {
        bail!(
            "{}",
            parsed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.is_error())
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    if document_page_kind(&parsed.document) != PageKind::Static {
        bail!("{} is not a static document", input.display());
    }

    let html = render_static_preview_html(source, &parsed.document, options)?;
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    fs::write(out_dir.join("index.html"), &html)
        .with_context(|| format!("failed to write {}", out_dir.join("index.html").display()))?;

    let src_dir = input.parent().unwrap_or(Path::new("."));
    for (url, _) in crate::collect_local_media(source, &parsed.document) {
        let from = src_dir.join(&url);
        if from.is_file() {
            let to = out_dir.join(&url);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to).with_context(|| format!("failed to copy {}", from.display()))?;
        }
    }
    Ok(page_title(&parsed.document, source.src, input))
}

pub fn render_static_preview_html(
    source: SourceFile<'_>,
    document: &Document,
    options: &CompileOptions,
) -> Result<String> {
    let mut diagnostics = Vec::new();
    let mut page_meta = crate::PageMeta::default();
    for item in &document.items {
        if let Item::Page(page) = item {
            page_meta = extract_page(source.src, page.body, &mut diagnostics);
            break;
        }
    }
    if diagnostics.iter().any(Diagnostic::is_error) {
        bail!(
            "{}",
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.is_error())
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let theme = rocci_theme::resolve(
        page_meta.theme.as_deref(),
        page_meta.color_scheme.as_deref(),
        &options.theme,
    )
    .ok();

    let mut catalog_diags = Vec::new();
    let docs = load_page_docs(
        source,
        document,
        source.name,
        IncludeOptions {
            root: options
                .theme
                .source_dir
                .as_deref()
                .or_else(|| Path::new(source.name).parent())
                .unwrap_or(Path::new(".")),
            snippet_roots: &[],
        },
        &mut catalog_diags,
    );
    if catalog_diags.iter().any(|diagnostic| diagnostic.is_error()) {
        bail!(
            "{}",
            catalog_diags
                .iter()
                .filter(|diagnostic| diagnostic.is_error())
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let article = if docs.article.is_empty() {
        crate::render_document(document)
    } else {
        render_article(&docs.article)
    };

    let title = page_meta
        .title
        .clone()
        .unwrap_or_else(|| "Rocdown".to_string());
    Ok(wrap_document(&title, theme.as_ref(), &article))
}

fn page_title(document: &Document, src: &str, input: &Path) -> String {
    let mut diagnostics = Vec::new();
    for item in &document.items {
        if let Item::Page(page) = item {
            let meta = extract_page(src, page.body, &mut diagnostics);
            if let Some(title) = meta.title {
                return title;
            }
        }
    }
    input
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("rocdown")
        .to_string()
}

fn wrap_document(title: &str, theme: Option<&rocci_theme::ResolvedTheme>, article: &str) -> String {
    let mut theme_attr = String::new();
    let mut scheme_attr = String::new();
    let mut css = String::new();
    if let Some(theme) = theme.filter(|theme| !theme.is_none()) {
        theme_attr = format!(" data-rd-theme=\"{}\"", escape(theme.id.as_str()));
        if let Some(scheme) = theme.policy.html_attr() {
            scheme_attr = format!(" data-rd-color-scheme=\"{scheme}\"");
        }
        css = theme.css.clone();
    }
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\"{theme_attr}{scheme_attr}>\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>\n{css}\n</style>\n</head>\n<body>\n<main class=\"rd-article\">\n{article}\n</main>\n</body>\n</html>\n",
        escape(title)
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompileOptions;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rocci-static-preview-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn static_preview_renders_note_img_and_theme_without_roc() {
        let dir = temp_dir("note-img");
        let assets = dir.join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("mark.png"), b"png").unwrap();
        let input = dir.join("Report.rocdown");
        fs::write(
            &input,
            r#"@page {
    theme: "paper",
    meta: {
        title: "Report",
    },
}

# Hello

:note[title: "Watch"] {{
    Read this.
}}

:img[src: "assets/mark.png", alt: "Mark"]
"#,
        )
        .unwrap();

        let src = fs::read_to_string(&input).unwrap();
        let source = SourceFile::new("Report.rocdown", &src);
        let parsed = parse(source, false);
        assert_eq!(document_page_kind(&parsed.document), PageKind::Static);

        let out = dir.join("dist");
        let mut options = CompileOptions::default();
        options.theme.source_dir = Some(dir.clone());
        let title = write_static_document_preview(&input, &out, &options).unwrap();
        assert_eq!(title, "Report");
        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("data-rd-theme=\"paper\""));
        assert!(html.contains("--rd-color-bg"));
        assert!(html.contains("Hello"));
        assert!(html.contains("Read this"));
        assert!(html.contains("assets/mark.png"));
        assert!(out.join("assets/mark.png").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn interactive_document_is_not_static() {
        let src = "@on:get(\"/\") = |_, _request| {\n    Html.text(\"hi\")\n}\n";
        let source = SourceFile::new("Live.rocdown", src);
        let parsed = parse(source, false);
        assert_eq!(document_page_kind(&parsed.document), PageKind::Live);
    }
}
