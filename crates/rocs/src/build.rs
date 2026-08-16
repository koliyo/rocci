use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    LowerOptions, MappedModule, SourceFile, compile, format_diagnostic, remap_roc_output,
    type_name_from_path, wrap_type_module,
};

use crate::BASIC_CLI_PLATFORM;
use crate::article::{is_static_document, render_document};
use crate::catalog::{self, RouteHint, SourcePage};
use crate::runtime;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn build(root: &Path, output: &Path) -> Result<BuildReport> {
    let root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        env::current_dir()?.join(root)
    };
    let output = if output.is_absolute() {
        output.to_path_buf()
    } else {
        env::current_dir()?.join(output)
    };

    let files = discover_rocdown(&root)?;
    let mut sources = Vec::new();

    for path in &files {
        let src = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let name = path.display().to_string();
        let relative_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| name.clone());
        let page_id = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty())
            .unwrap_or_else(|| "page".to_string());
        let compiled = rocci_rocdown::compile(
            SourceFile::new(&name, &src),
            &rocci_rocdown::CompileOptions {
                resolve_links: false,
                ..rocci_rocdown::CompileOptions::default()
            },
        );
        for diagnostic in &compiled.diagnostics {
            eprintln!(
                "{}",
                format_diagnostic(SourceFile::new(&name, &src), diagnostic)
            );
        }
        if compiled.has_errors() {
            bail!("template compilation failed for {}", path.display());
        }
        if compiled.roc.contains("import Datastar") {
            bail!(
                "{} uses Datastar, which the rocs runtime does not stage",
                path.display()
            );
        }
        if let Some(layout) = compiled.page_meta.layout.as_deref() {
            bail!(
                "{} uses layout `{layout}`, which static rocs pages do not support yet",
                path.display()
            );
        }
        if let Err(kind) = is_static_document(&compiled.document) {
            bail!(
                "{} contains {kind}; static rocs pages cannot include Roc/Rocci islands yet",
                path.display()
            );
        }
        let title = compiled
            .page_meta
            .title
            .clone()
            .or_else(|| {
                compiled
                    .headings
                    .first()
                    .map(|heading| heading.text.clone())
            })
            .unwrap_or_else(|| page_id.clone());
        let route_hint = match compiled.page_meta.route {
            Some(route) => RouteHint::Explicit(route),
            None => RouteHint::Derived,
        };
        sources.push(SourcePage {
            id: page_id,
            source_path: relative_name,
            route_hint,
            title,
            article_html: render_document(&compiled.document),
        });
    }

    let pages = catalog::validate(&sources).map_err(anyhow::Error::msg)?;

    let theme_src = runtime::THEME;
    let theme_compiled = compile(
        SourceFile::new("RocsTheme.rocci", theme_src),
        &LowerOptions::default(),
    );
    for diagnostic in &theme_compiled.diagnostics {
        eprintln!(
            "{}",
            format_diagnostic(SourceFile::new("RocsTheme.rocci", theme_src), diagnostic)
        );
    }
    if theme_compiled.has_errors() {
        bail!("RocsTheme.rocci compilation failed");
    }
    if theme_compiled.roc.contains("import Datastar") {
        bail!("RocsTheme.rocci uses Datastar, which the rocs runtime does not stage");
    }
    let theme_wrapped = wrap_type_module(&theme_compiled.roc, "RocsTheme");
    let mut generated_roc_bytes = theme_wrapped.len();
    let maps = vec![MappedModule {
        type_name: "RocsTheme".into(),
        generated: theme_wrapped.clone(),
        source_name: "RocsTheme.rocci".into(),
        source_src: theme_src.to_string(),
        segments: theme_compiled.segments,
    }];

    let workspace = unique_temp("ws")?;
    let staging = unique_temp("stage")?;
    runtime::stage_into(&workspace)?;
    generated_roc_bytes += runtime::runtime_bytes();

    let articles = workspace.join("articles");
    fs::create_dir_all(&articles).context("failed to create articles directory")?;
    let mut index_pages = Vec::new();
    for page in &pages {
        let type_name = type_name_from_path(Path::new(&page.source_path));
        let article_rel = format!("articles/{type_name}.html");
        let title_rel = format!("articles/{type_name}.title");
        fs::write(workspace.join(&article_rel), &page.article_html)
            .with_context(|| format!("failed to write {article_rel}"))?;
        fs::write(workspace.join(&title_rel), &page.title)
            .with_context(|| format!("failed to write {title_rel}"))?;
        if let Some(parent) = Path::new(&page.output_path).parent() {
            if parent != Path::new("") {
                fs::create_dir_all(staging.join(parent)).with_context(|| {
                    format!("failed to create {}", staging.join(parent).display())
                })?;
            }
        }
        index_pages.push(IndexedPage {
            title_path: title_rel,
            article_path: article_rel,
            output_path: page.output_path.clone(),
        });
    }

    let pages_roc = pages_source(&index_pages);
    generated_roc_bytes += pages_roc.len();
    fs::write(workspace.join("RocsPages.roc"), &pages_roc)
        .context("failed to write RocsPages.roc")?;
    fs::write(workspace.join("RocsTheme.roc"), &theme_wrapped)
        .context("failed to write RocsTheme.roc")?;
    let main_roc = main_roc();
    generated_roc_bytes += main_roc.len();
    fs::write(workspace.join("main.roc"), main_roc).context("failed to write main.roc")?;

    eprintln!("rocs: generated {generated_roc_bytes} bytes of Roc, compiling with roc");
    let roc_started = Instant::now();
    let roc_output = invoke_roc(&workspace, &staging, &maps)
        .with_context(|| format!("workspace {}", workspace.display()))?;
    let roc_ms = roc_started.elapsed().as_millis();
    eprintln!("rocs: roc finished in {roc_ms}ms");
    if !roc_output.is_empty() {
        eprint!("{roc_output}");
    }

    commit_output(&staging, &output)?;
    let _ = fs::remove_dir_all(&workspace);

    Ok(BuildReport {
        generated_roc_bytes,
        roc_ms,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub generated_roc_bytes: usize,
    pub roc_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedPage {
    pub title_path: String,
    pub article_path: String,
    pub output_path: String,
}

pub fn discover_rocdown(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(root)
        .with_context(|| format!("failed to read {}", root.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "rocdown"))
        .collect();
    files.sort();
    if files.is_empty() {
        bail!("no .rocdown files in {}", root.display());
    }
    Ok(files)
}

pub fn pages_source(pages: &[IndexedPage]) -> String {
    let mut pages = pages.to_vec();
    pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    let mut out = String::from("RocsPages := [].{\n    pages = [\n");
    for page in &pages {
        out.push_str("        {\n            title_path: ");
        push_roc_string(&mut out, &page.title_path);
        out.push_str(",\n            article_path: ");
        push_roc_string(&mut out, &page.article_path);
        out.push_str(",\n            output_path: ");
        push_roc_string(&mut out, &page.output_path);
        out.push_str(",\n        },\n");
    }
    out.push_str("    ]\n}\n");
    out
}

fn push_roc_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn main_roc() -> String {
    format!(
        "\
app [main!] {{ pf: platform \"{BASIC_CLI_PLATFORM}\" }}

import RocsBuild

main! = |_args| {{
    RocsBuild.run!({{}})?
    Ok({{}})
}}
"
    )
}

fn invoke_roc(workspace: &Path, staging: &Path, maps: &[MappedModule]) -> Result<String> {
    let output = Command::new("roc")
        .arg("main.roc")
        .current_dir(workspace)
        .env("ROCS_STAGING", staging)
        .output()
        .context("failed to invoke roc")?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = if stdout.is_empty() {
        stderr.clone()
    } else if stderr.is_empty() {
        stdout.clone()
    } else {
        format!("{stdout}{stderr}")
    };
    if output.status.success() {
        return Ok(combined);
    }
    let mapped = remap_roc_output(&combined, maps);
    for frame in mapped {
        eprintln!("{}", frame.render_for_stderr());
    }
    if !combined.trim().is_empty() {
        eprintln!("{}", combined.trim_end());
    }
    bail!(
        "roc rocs build failed{}",
        if combined.trim().is_empty() {
            String::new()
        } else {
            format!("\n{}", combined.trim_end())
        }
    );
}

fn commit_output(staging: &Path, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let prev_name = match output.file_name() {
        Some(name) => {
            let mut prev = name.to_os_string();
            prev.push(".prev");
            prev
        }
        None => "output.prev".into(),
    };
    let prev = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(prev_name);
    if prev.exists() {
        fs::remove_dir_all(&prev).with_context(|| format!("failed to clear {}", prev.display()))?;
    }
    if output.exists() {
        fs::rename(output, &prev)
            .with_context(|| format!("failed to move {} aside", output.display()))?;
        if let Err(err) = fs::rename(staging, output) {
            let _ = fs::rename(&prev, output);
            return Err(err).with_context(|| {
                format!(
                    "failed to replace {} with staged rocs output",
                    output.display()
                )
            });
        }
        let _ = fs::remove_dir_all(&prev);
    } else if let Err(err) = fs::rename(staging, output) {
        return Err(err)
            .with_context(|| format!("failed to move staged rocs output to {}", output.display()));
    }
    Ok(())
}

fn unique_temp(kind: &str) -> Result<PathBuf> {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("rocs-{kind}-{}-{n}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ROC_LOCK: Mutex<()> = Mutex::new(());

    fn skip_without_roc() -> bool {
        let available = Command::new("roc")
            .arg("help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if available {
            return false;
        }
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
            panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
        }
        eprintln!("skipping: roc not on PATH");
        true
    }

    fn temp_dir(name: &str) -> PathBuf {
        unique_temp(name).unwrap()
    }

    fn write_page(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join(name), body).unwrap();
    }

    fn collect_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut files = Vec::new();
        fn walk(dir: &Path, prefix: &Path, files: &mut Vec<(String, Vec<u8>)>) {
            let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let path = entry.path();
                let rel = prefix.join(entry.file_name());
                if path.is_dir() {
                    walk(&path, &rel, files);
                } else {
                    files.push((rel.to_string_lossy().into_owned(), fs::read(path).unwrap()));
                }
            }
        }
        walk(dir, Path::new(""), &mut files);
        files
    }

    fn sample_pages() -> [IndexedPage; 2] {
        [
            IndexedPage {
                title_path: "articles/Index.title".into(),
                article_path: "articles/Index.html".into(),
                output_path: "index.html".into(),
            },
            IndexedPage {
                title_path: "articles/Guide.title".into(),
                article_path: "articles/Guide.html".into(),
                output_path: "guide/index.html".into(),
            },
        ]
    }

    #[test]
    fn pages_source_is_invariant_under_shuffle() {
        let src = pages_source(&sample_pages());
        let mut shuffled = sample_pages();
        shuffled.reverse();
        assert_eq!(src, pages_source(&shuffled));
        let guide = src.find("output_path: \"guide/index.html\"").unwrap();
        let index = src.find("output_path: \"index.html\"").unwrap();
        assert!(guide < index);
        assert!(src.contains("articles/Guide.html"));
        assert!(src.contains("articles/Index.html"));
    }

    #[test]
    fn two_page_build_writes_shell_and_escapes() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocs");
        let output = temp_dir("out");
        let report = build(&root, &output).unwrap();
        assert!(report.generated_roc_bytes > 0);
        let index = fs::read_to_string(output.join("index.html")).unwrap();
        let guide = fs::read_to_string(output.join("guide/index.html")).unwrap();
        assert!(index.contains("Ampersand &amp; Company"));
        assert!(!index.contains("Ampersand & Company"));
        assert!(guide.contains("Guide"));
        for html in [&index, &guide] {
            assert!(html.contains("skip-link"));
            assert!(html.contains("<main"));
            assert!(html.contains("id=\"main-content\"") || html.contains("id='main-content'"));
            assert!(html.contains("<!DOCTYPE html>"));
        }
        assert!(index.contains("href=\"/guide/\""));
        assert!(index.contains("class=\"rd-paragraph\""));
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn duplicate_routes_fail_in_catalog_and_preserve_output() {
        let root = temp_dir("dup-src");
        write_page(
            &root,
            "alpha.rocdown",
            "@page { route: \"/same/\", meta: { title: \"Alpha\" } }\n\n# Alpha\n",
        );
        write_page(
            &root,
            "beta.rocdown",
            "@page { route: \"/same/\", meta: { title: \"Beta\" } }\n\n# Beta\n",
        );
        let output = temp_dir("dup-out");
        fs::write(output.join("keep.txt"), "preserve me").unwrap();
        let err = build(&root, &output).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("alpha.rocdown"), "{message}");
        assert!(message.contains("beta.rocdown"), "{message}");
        assert!(message.contains("duplicate route"), "{message}");
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).unwrap(),
            "preserve me"
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn island_pages_are_rejected_without_roc() {
        let root = temp_dir("island-src");
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n@render {\n    Html.text(\"nope\")\n}\n",
        );
        let output = temp_dir("island-out");
        let err = build(&root, &output).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("@render"), "{message}");
        assert!(message.contains("islands"), "{message}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn repeat_build_is_byte_identical() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocs");
        let first = temp_dir("det-a");
        let second = temp_dir("det-b");
        build(&root, &first).unwrap();
        build(&root, &second).unwrap();
        assert_eq!(collect_files(&first), collect_files(&second));
        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }
}
