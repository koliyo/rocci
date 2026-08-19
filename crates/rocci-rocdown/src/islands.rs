use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result, bail};
use rocci_roc_host::{InputFingerprint, NativeHost, compute_compile_hash, compute_gen_hash};
use rocci_template::{
    LowerOptions, MappedModule, SourceFile, StyleKind, format_diagnostic, remap_roc_output,
    type_name_from_path, wrap_type_module,
};

use crate::{BASIC_CLI_PLATFORM, CompileOptions, compile_islands};

pub const PLACEHOLDER: &str = "<!--rocci-island-->";
const BREAK: &str = "<!--rocci-island-break-->";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct EvaluatedIslands {
    pub html: Vec<String>,
    pub css: String,
}

pub fn fill_placeholders(article_html: &str, islands: &[String]) -> Result<String> {
    let expected = article_html.matches(PLACEHOLDER).count();
    if expected != islands.len() {
        bail!(
            "island count mismatch: article has {expected} slots, evaluation produced {}",
            islands.len()
        );
    }
    if expected == 0 {
        return Ok(article_html.to_string());
    }
    let mut parts = article_html.split(PLACEHOLDER);
    let mut out = String::new();
    if let Some(first) = parts.next() {
        out.push_str(first);
    }
    for (html, rest) in islands.iter().zip(parts) {
        out.push_str(html);
        out.push_str(rest);
    }
    Ok(out)
}

pub fn evaluate_page(source_path: &Path, source_name: &str, src: &str) -> Result<EvaluatedIslands> {
    let mut lower = LowerOptions::default();
    lower.embed_css = false;
    let compiled = compile_islands(
        SourceFile::new(source_name, src),
        &CompileOptions {
            lower,
            resolve_links: false,
            resolve_includes: false,
            check_assets: false,
            ..CompileOptions::default()
        },
    );
    for diagnostic in &compiled.diagnostics {
        if diagnostic.is_error() {
            eprintln!(
                "{}",
                format_diagnostic(SourceFile::new(source_name, src), diagnostic)
            );
        }
    }
    if compiled.has_errors() {
        let messages: Vec<_> = compiled
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        bail!(
            "failed to lower islands in {source_name}: {}",
            messages.join("; ")
        );
    }

    let css = compiled
        .styles
        .iter()
        .filter(|style| matches!(style.kind, StyleKind::File | StyleKind::Component))
        .map(|style| style.css.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if crate::lower::island_item_count(&compiled.document) == 0 {
        return Ok(EvaluatedIslands {
            html: Vec::new(),
            css,
        });
    }

    let html = render_islands(
        source_path,
        source_name,
        src,
        &compiled.roc,
        &compiled.segments,
    )?;
    Ok(EvaluatedIslands { html, css })
}

fn render_islands(
    source_path: &Path,
    source_name: &str,
    src: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
) -> Result<Vec<String>> {
    let type_name = type_name_from_path(source_path);
    let workspace = unique_temp("islands")?;
    fs::write(workspace.join("Html.roc"), crate::runtime::HTML)
        .with_context(|| format!("failed to write {}/Html.roc", workspace.display()))?;
    if let Some(src_dir) = source_path.parent() {
        rocci_cli::driver::copy_sibling_roc(src_dir, &workspace, &type_name)?;
    }
    let wrapped = wrap_type_module(roc, &type_name);
    fs::write(workspace.join(format!("{type_name}.roc")), &wrapped)
        .with_context(|| format!("failed to write {type_name}.roc"))?;
    let main = island_main(&type_name);
    fs::write(workspace.join("main.roc"), &main).context("failed to write island main.roc")?;

    let gen_hash = compute_gen_hash(
        env!("CARGO_PKG_VERSION"),
        "rocdown-islands",
        &[
            (&format!("{type_name}.roc"), wrapped.as_bytes()),
            ("main.roc", main.as_bytes()),
        ],
        &[("Html.roc", crate::runtime::HTML.as_bytes())],
    );
    let compile_hash = compute_compile_hash(
        &gen_hash,
        "roc",
        &format!("native:{}", env::consts::ARCH),
        "dev",
        BASIC_CLI_PLATFORM,
        env!("CARGO_PKG_VERSION"),
    );
    let fingerprints = [
        InputFingerprint::from_bytes(&format!("{type_name}.roc"), wrapped.as_bytes()),
        InputFingerprint::from_bytes("main.roc", main.as_bytes()),
        InputFingerprint::from_bytes("Html.roc", crate::runtime::HTML.as_bytes()),
    ];

    let host = NativeHost::default();
    let (apply_bin, _) = host
        .compile_or_cached(&workspace, &compile_hash, &fingerprints)
        .map_err(|err| {
            annotate_roc_error(
                source_name,
                src,
                roc,
                segments,
                &type_name,
                &err.to_string(),
            )
        })?;

    let output = Command::new(&apply_bin)
        .current_dir(&workspace)
        .output()
        .with_context(|| format!("failed to run {}", apply_bin.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let combined = format!("{stdout}{stderr}");
        bail!(annotate_roc_error(
            source_name,
            src,
            roc,
            segments,
            &type_name,
            &combined,
        ));
    }

    let _ = fs::remove_dir_all(&workspace);
    let body = stdout.trim_end_matches(['\r', '\n']);
    if body.is_empty() {
        bail!("island renderer produced no HTML for {source_name}");
    }
    Ok(body.split(BREAK).map(str::to_string).collect())
}

fn island_main(type_name: &str) -> String {
    format!(
        r#"app [main!] {{ pf: platform "{BASIC_CLI_PLATFORM}" }}

import pf.Stdout
import {type_name}
import Html

main! = |_args| {{
    parts = List.map({type_name}.rocci_islands({{}}), |html| Html.render(html))
    _ = Stdout.line!(Str.join_with(parts, "{BREAK}"))
    Ok({{}})
}}
"#
    )
}

fn annotate_roc_error(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    type_name: &str,
    output: &str,
) -> anyhow::Error {
    let mapped = remap_roc_output(
        output,
        &[MappedModule {
            type_name: type_name.to_string(),
            generated: roc.to_string(),
            source_name: filename.to_string(),
            source_src: source.to_string(),
            segments: segments.to_vec(),
        }],
    );
    if mapped.is_empty() {
        return anyhow::anyhow!("roc failed to render islands:\n{}", output.trim());
    }
    let frames: Vec<String> = mapped.iter().map(|frame| frame.message.clone()).collect();
    anyhow::anyhow!(
        "roc failed to render islands:\n{}\n{}",
        frames.join("\n"),
        output.trim()
    )
}

fn unique_temp(kind: &str) -> Result<PathBuf> {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("rocci-{kind}-{}-{n}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_placeholders_preserves_order() {
        let article = format!("<p>a</p>{PLACEHOLDER}<p>b</p>{PLACEHOLDER}");
        let filled = fill_placeholders(
            &article,
            &["<span>1</span>".into(), "<span>2</span>".into()],
        )
        .unwrap();
        assert_eq!(filled, "<p>a</p><span>1</span><p>b</p><span>2</span>");
    }

    #[test]
    fn fill_placeholders_rejects_count_mismatch() {
        let err = fill_placeholders(PLACEHOLDER, &[]).unwrap_err();
        assert!(err.to_string().contains("island count mismatch"), "{err}");
    }
}
