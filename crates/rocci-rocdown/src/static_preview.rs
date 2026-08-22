use std::fs;
use std::path::Path;
use std::time::Instant;

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

    let started = Instant::now();
    let theme = rocci_theme::resolve(
        page_meta.theme.as_deref(),
        page_meta.color_scheme.as_deref(),
        &options.theme,
    )?;
    rocci_cli::logs::emit(format!(
        "rocdown: resolved theme in {}ms",
        started.elapsed().as_millis()
    ));

    let mut asset_diags = Vec::new();
    crate::img::check_document_assets(source, document, options, &mut asset_diags);
    if asset_diags.iter().any(Diagnostic::is_error) {
        bail!(
            "{}",
            asset_diags
                .iter()
                .filter(|diagnostic| diagnostic.is_error())
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let started = Instant::now();
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
    rocci_cli::logs::emit(format!(
        "rocdown: article tree in {}ms ({} nodes)",
        started.elapsed().as_millis(),
        docs.article.len()
    ));

    let started = Instant::now();
    let article = if docs.article.is_empty() {
        crate::render_document(document)
    } else {
        render_article(&docs.article)
    };
    rocci_cli::logs::emit(format!(
        "rocdown: article HTML in {}ms ({} bytes)",
        started.elapsed().as_millis(),
        article.len()
    ));

    let title = page_meta
        .title
        .clone()
        .unwrap_or_else(|| "Rocdown".to_string());
    Ok(wrap_document(&title, &theme, &article))
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

fn wrap_document(title: &str, theme: &rocci_theme::ResolvedTheme, article: &str) -> String {
    let mut html_attrs = String::from(" lang=\"en\"");
    let mut scheme_meta = String::new();
    let mut css = String::new();
    if !theme.is_none() {
        html_attrs.push_str(" class=\"rd-document\"");
        html_attrs.push_str(&format!(" data-rd-theme=\"{}\"", escape(theme.id.as_str())));
        if let Some(scheme) = theme.policy.html_attr() {
            html_attrs.push_str(&format!(" data-rd-color-scheme=\"{scheme}\""));
        }
        scheme_meta = format!(
            "<meta name=\"color-scheme\" content=\"{}\">\n",
            escape(theme.policy.meta_content())
        );
        css = theme.css.clone();
    }
    format!(
        "<!DOCTYPE html>\n<html{html_attrs}>\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n{scheme_meta}<title>{}</title>\n<style>\n{css}\n</style>\n</head>\n<body>\n<main class=\"rd-article\">\n{article}\n</main>\n</body>\n</html>\n",
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
        assert!(
            html.contains("class=\"rd-document\""),
            "theme CSS requires html.rd-document\n{html}"
        );
        assert!(html.contains("data-rd-theme=\"paper\""));
        assert!(html.contains("name=\"color-scheme\""));
        assert!(html.contains("--rd-color-bg"));
        assert!(html.contains("class=\"rd-header-1\"") || html.contains("<h1"));
        assert!(html.contains("<img"));
        assert!(html.contains("Hello"));
        assert!(html.contains("Read this"));
        assert!(html.contains("assets/mark.png"));
        assert!(out.join("assets/mark.png").is_file());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn branding_report_static_preview_completes_quickly() {
        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../archive/reports/branding/BRANDING_AND_COMMUNITY_REPORT.rocdown");
        if !input.is_file() {
            return;
        }
        let dir = temp_dir("branding-report");
        let started = Instant::now();
        let mut options = CompileOptions::default();
        options.theme.source_dir = input.parent().map(|parent| parent.to_path_buf());
        write_static_document_preview(&input, &dir, &options).unwrap();
        let ms = started.elapsed().as_millis();
        eprintln!("branding static preview {ms}ms");
        assert!(ms < 5_000, "static preview of branding report took {ms}ms");
        let html = fs::read_to_string(dir.join("index.html")).unwrap();
        assert!(html.contains("Rocci branding"));
        assert!(
            html.contains("class=\"rd-document\""),
            "branding preview must stamp html.rd-document"
        );
        assert!(html.contains("assets/rocci-logo-folded-r.png"));
        assert!(
            html.contains("<h2 class=\"rd-header-2\" id=\"searchability-and-seo\">"),
            "headings must render as elements, not raw markdown"
        );
        assert!(
            !html.contains("\n## Searchability"),
            "ATX heading source must not leak into the article HTML\n"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn interactive_document_is_not_static() {
        let src = "@get:view(\"/\") = |_, _request| {\n    Html.text(\"hi\")\n}\n";
        let source = SourceFile::new("Live.rocdown", src);
        let parsed = parse(source, false);
        assert_eq!(document_page_kind(&parsed.document), PageKind::Live);
    }
}
