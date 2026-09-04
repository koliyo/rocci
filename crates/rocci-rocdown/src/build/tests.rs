use super::*;
use std::process::Command;
use std::sync::Mutex;

pub(crate) static ROC_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn lock_roc() -> std::sync::MutexGuard<'static, ()> {
    ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner())
}

pub(crate) fn skip_without_roc() -> bool {
    if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() != Some("1") {
        eprintln!("skipping: ROCCI_REQUIRE_ROC is not 1");
        return true;
    }
    let help_ok = Command::new("roc")
        .arg("help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !help_ok {
        panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
    }
    false
}

fn temp_dir(name: &str) -> PathBuf {
    unique_temp(name).unwrap()
}

fn assert_goto_chrome(html: &str) {
    let lower = html.to_ascii_lowercase();
    assert!(
        lower.contains("<script") && html.contains("goto."),
        "expected hashed goto chrome script\n{html}"
    );
    assert!(
        html.contains("script-src 'self'") || html.contains("script-src &#39;self&#39;"),
        "{html}"
    );
    assert!(
        html.contains("connect-src 'self'") || html.contains("connect-src &#39;self&#39;"),
        "{html}"
    );
    assert!(
        !html.contains("script-src 'none'") && !html.contains("script-src &#39;none&#39;"),
        "{html}"
    );
    assert!(
        !html.contains("/assets/datastar") && !html.contains("datastar.js"),
        "{html}"
    );
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

#[test]
fn ensure_page_files_errors_when_html_is_missing() {
    let staging = temp_dir("missing-html");
    fs::write(staging.join("index.html"), "<!DOCTYPE html>").unwrap();
    let err = ensure_page_files(&staging, ["index.html", "404.html", "guide/index.html"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("apply did not write page HTML"), "{err}");
    assert!(err.contains("404.html"), "{err}");
    assert!(err.contains("guide/index.html"), "{err}");
    assert!(!err.contains("index.html,"), "{err}");
    let _ = fs::remove_dir_all(&staging);
}

#[test]
fn two_page_build_writes_shell_and_escapes() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/site");
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
    assert!(index.contains("href=\"/guide\""));
    assert!(index.contains("class=\"rd-paragraph\""));
    for html in [&index, &guide] {
        assert!(html.contains("rel=\"stylesheet\""));
        assert!(html.contains("Content-Security-Policy"));
        assert_goto_chrome(html);
        let style_idx = html.find("<style");
        if let Some(idx) = style_idx {
            let window = &html[idx..idx.saturating_add(80).min(html.len())];
            assert!(
                !window.contains(":scope") && !window.contains("--canvas"),
                "theme CSS should not be inlined: {window}"
            );
        }
    }
    let not_found = fs::read_to_string(output.join("404.html")).unwrap();
    assert!(not_found.contains("skip-link"));
    assert!(not_found.contains("Page not found"));
    assert!(not_found.contains("id=\"main-content\"") || not_found.contains("id='main-content'"));
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn docs_components_render_asides_tabs_and_includes() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("docs-src");
    fs::write(
        root.join("snippet.rs"),
        "// docs-region: hello\nfn hello() {}\n// docs-region-end: hello\n",
    )
    .unwrap();
    write_page(
        &root,
        "index.rocdown",
        "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n\n:include[path: \"snippet.rs\", region: \"hello\"]\n",
    );
    let output = temp_dir("docs-out");
    build(&root, &output).unwrap();
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("data-rocci-docs=\"note\""), "{html}");
    assert!(html.contains("rd-docs-label"), "{html}");
    assert!(
        html.contains("<p class=\"rd-paragraph\">Read this.</p>"),
        "{html}"
    );
    assert!(!html.contains("&lt;p"), "{html}");
    assert!(html.contains("data-rocci-docs=\"tabs\""), "{html}");
    assert!(html.contains("aria-label=\"macOS\""), "{html}");
    assert!(html.contains("Linux panel"), "{html}");
    assert!(html.contains("fn hello()"), "{html}");
    assert!(!html.contains("role=\"tablist\""), "{html}");
    assert_goto_chrome(&html);
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn block_pack_overrides_note_html() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("block-pack-src");
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
        </head>
        <body>{content}</body>
    </html>
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Note = |{ title }, content|
    <section data-test-note data-title={title}>{content}</section>
"#,
    )
    .unwrap();
    write_page(
        &root,
        "index.rocdown",
        "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n",
    );
    let output = temp_dir("block-pack-out");
    build(&root, &output).unwrap();
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("data-test-note"), "{html}");
    assert!(html.contains("data-title=\"Watch\""), "{html}");
    assert!(!html.contains("data-rocci-docs=\"note\""), "{html}");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn pack_custom_kind_paints_callout_html() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("block-pack-callout-src");
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
        </head>
        <body>{content}</body>
    </html>
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("theme/Blocks.rocci"),
        r#"
import Html

@component Callout = |{ tone ?? "note" }, content|
    <aside data-test-callout data-tone={tone}>{content}</aside>
"#,
    )
    .unwrap();
    write_page(
        &root,
        "index.rocdown",
        "# Home\n\n:callout[tone: \"warn\"] {{\n    Watch this.\n}}\n",
    );
    let output = temp_dir("block-pack-callout-out");
    build(&root, &output).unwrap();
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("data-test-callout"), "{html}");
    assert!(html.contains("data-tone=\"warn\""), "{html}");
    assert!(html.contains("Watch this."), "{html}");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn debug_painter_emits_unfinished_markup() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("block-debug-src");
    fs::create_dir_all(root.join("theme")).unwrap();
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
        </head>
        <body>{content}</body>
    </html>
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("theme/DocsComponents.rocci"),
        r#"
import Html

@component Tip = |{ title }, content|
    <p data-stub-tip data-title={title}>{content}</p>
"#,
    )
    .unwrap();
    fs::write(
        root.join("rocdown.toml"),
        r#"
[site]
title = "Rocci"

[blocks]
debug = true
"#,
    )
    .unwrap();
    write_page(
        &root,
        "index.rocdown",
        "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n",
    );
    let output = temp_dir("block-debug-out");
    build(&root, &output).unwrap();
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("data-rocci-block-debug"), "{html}");
    assert!(html.contains("data-kind=\"note\""), "{html}");
    assert!(html.contains("Watch"), "{html}");
    assert!(!html.contains("rd-docs-note"), "{html}");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn dual_apply_paints_widgets_and_splices_islands() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("dual-apply-src");
    write_page(
        &root,
        "index.rocdown",
        "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n",
    );
    write_page(
        &root,
        "widgets.rocdown",
        r#"
@page {
    route: "/widgets/",
    meta: { title: "Widgets" },
}

@roc {
feature_count = 3.I64
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Widgets

<FeatureCount count={feature_count} />
"#,
    );
    write_page(
        &root,
        "live.rocdown",
        r#"
@page {
    route: "/live/",
    meta: { title: "Live" },
}

@component
RevealTip = |{ open }| {
    <div id="reveal-tip">
        @if open {
            <p>Hide tip</p>
        } @else {
            <p>This block is closed until the server sends the open markup.</p>
        }
    </div>
}

@post:fragment("/actions/reveal/show") = |_, _request| {
    revealTip({ open: True })
}

# Live

@render RevealTip({ open: False })
"#,
    );
    write_page(
        &root,
        "about.rocdown",
        r#"
@page {
    route: "/about/",
    meta: { title: "About" },
}

# About

Static neighbor.
"#,
    );
    let output = temp_dir("dual-apply-out");
    let report = build(&root, &output).unwrap();
    assert!(report.datastar);
    let home = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(home.contains("data-rocci-docs=\"note\""), "{home}");
    assert!(home.contains("data-rocci-docs=\"tabs\""), "{home}");
    assert!(home.contains("Mac panel"), "{home}");
    assert_goto_chrome(&home);

    let widgets = fs::read_to_string(output.join("widgets/index.html")).unwrap();
    assert!(widgets.contains("3 core ideas"), "{widgets}");
    assert_goto_chrome(&widgets);

    let live = fs::read_to_string(output.join("live/index.html")).unwrap();
    assert!(live.contains("reveal-tip"), "{live}");
    assert!(
        live.contains("/assets/datastar.") || live.contains("datastar."),
        "{live}"
    );

    let about = fs::read_to_string(output.join("about/index.html")).unwrap();
    assert!(about.contains("Static neighbor."), "{about}");
    assert_goto_chrome(&about);
    let pages: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
    let kind = |route: &str| {
        pages
            .as_array()
            .unwrap()
            .iter()
            .find(|page| page["route"] == route)
            .unwrap()
            .clone()
    };
    assert_eq!(kind("/")["kind"], "static");
    assert_eq!(kind("/widgets")["kind"], "hydrate");
    assert_eq!(kind("/live")["kind"], "live");
    assert_eq!(kind("/about")["kind"], "static");
    assert!(kind("/about").get("datastar").is_none());
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn project_theme_renders_article_html_unescaped() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("theme-html-src");
    fs::create_dir_all(root.join("theme")).unwrap();
    write_page(
        &root,
        "index.rocdown",
        "# Welcome\n\nHello from Markdown.\n",
    );
    fs::write(
        root.join("theme/SiteShell.rocci"),
        r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
            <link rel="stylesheet" href={view.resources.stylesheet} />
        </head>
        <body>
            <main id="main-content">{content}</main>
        </body>
    </html>
}
"#,
    )
    .unwrap();
    let output = temp_dir("theme-html-out");
    build(&root, &output).unwrap();
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("<h1 class=\"rd-header-1\""), "{html}");
    assert!(!html.contains("&lt;h1"), "{html}");
    assert!(
        html.contains("<p class=\"rd-paragraph\">Hello from Markdown.</p>"),
        "{html}"
    );
    let stylesheet = html
        .split("href=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("stylesheet href");
    let css_path = output.join(stylesheet.trim_start_matches('/'));
    let css = fs::read_to_string(&css_path)
        .unwrap_or_else(|_| panic!("missing stylesheet {}", css_path.display()));
    assert!(css.contains("--canvas"), "{css}");
    assert!(css.contains(".rd-header-1"), "{css}");
    let _ = fs::remove_dir_all(&root);
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
fn cdn_only_rejects_live_pages_and_preserves_output() {
    let root = temp_dir("cdn-only-src");
    write_page(
        &root,
        "index.rocdown",
        "@page { route: \"/\", meta: { title: \"Live\" } }\n\n@post:fragment(\"/actions/x\") = |_, _request| {\n    Html.text(\"x\")\n}\n\n# Live\n",
    );
    let output = temp_dir("cdn-only-out");
    fs::write(output.join("keep.txt"), "preserve me").unwrap();
    let err = build_configured_with_options(
        &root,
        Some(&output),
        BuildOptions {
            host: None,
            cdn_only: true,
        },
    )
    .unwrap_err();
    let message = format!("{err:#}");
    assert!(message.contains("RD2302"), "{message}");
    assert!(message.contains("CDN-only"), "{message}");
    assert_eq!(
        fs::read_to_string(output.join("keep.txt")).unwrap(),
        "preserve me"
    );
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn cdn_only_allows_static_pages() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("cdn-only-static-src");
    write_page(
        &root,
        "index.rocdown",
        "@page { route: \"/\", meta: { title: \"Home\" } }\n\n# Home\n",
    );
    let output = temp_dir("cdn-only-static-out");
    let report = build_configured_with_options(
        &root,
        Some(&output),
        BuildOptions {
            host: None,
            cdn_only: true,
        },
    )
    .unwrap();
    assert!(!report.datastar);
    assert!(report.service_routes.is_empty());
    assert!(output.join("index.html").is_file());
    assert!(!output.join("islands.json").is_file());
    let pages: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
    assert_eq!(pages[0]["kind"], "static");
    assert!(pages[0].get("datastar").is_none());
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn hydrate_pages_splice_component_html_without_scripts() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("hydrate-src");
    write_page(
        &root,
        "index.rocdown",
        r#"
@page {
    route: "/",
    meta: { title: "Hydrate" },
}

@roc {
feature_count = 3.I64
}

@css {
    .feature-count { color: teal; }
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Rocdown

Email docs@example.com.

<FeatureCount count={feature_count} />

After the island.
"#,
    );
    let output = temp_dir("hydrate-out");
    build(&root, &output).unwrap();
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("<h1 class=\"rd-header-1\""), "{html}");
    assert!(html.contains("Rocdown"), "{html}");
    assert!(html.contains("docs@example.com"), "{html}");
    assert!(html.contains("3 core ideas"), "{html}");
    assert!(html.contains("After the island."), "{html}");
    assert_goto_chrome(&html);
    let stylesheet = html
        .split("href=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("stylesheet href");
    let css_path = output.join(stylesheet.trim_start_matches('/'));
    let css = fs::read_to_string(&css_path)
        .unwrap_or_else(|_| panic!("missing stylesheet {}", css_path.display()));
    assert!(css.contains(".feature-count"), "{css}");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn live_pages_splice_component_html_and_stage_datastar() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = temp_dir("live-src");
    write_page(
        &root,
        "index.rocdown",
        r#"
@page {
    route: "/",
    meta: { title: "Live" },
}

@component
RevealTip = |{ open }| {
    <div id="reveal-tip">
        @if open {
            <p>Hide tip</p>
        } @else {
            <>
                <p>This block is closed until the server sends the open markup.</p>
                <button type="button" data-on:click=@post("/actions/reveal/show")>
                    Show tip
                </button>
            </>
        }
    </div>
}

@post:fragment("/actions/reveal/show") = |_, _request| {
    revealTip({ open: True })
}

# Live

Prose stays Markdown.

@render RevealTip({ open: False })
"#,
    );
    write_page(
        &root,
        "about.rocdown",
        r#"
@page {
    route: "/about/",
    meta: { title: "About" },
}

# About

Static neighbor.
"#,
    );
    let output = temp_dir("live-out");
    let report = build(&root, &output).unwrap();
    assert!(report.datastar);
    assert!(
        report
            .pages
            .iter()
            .any(|page| page.kind == crate::article::PageKind::Live && page.datastar),
        "{:?}",
        report.pages
    );
    assert!(
        report
            .service_routes
            .iter()
            .any(|route| route.method == "POST" && route.path == "/actions/reveal/show"),
        "{:?}",
        report.service_routes
    );
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("<h1 class=\"rd-header-1\""), "{html}");
    assert!(html.contains("Live"), "{html}");
    assert!(html.contains("Prose stays Markdown."), "{html}");
    assert!(
        html.contains("id=\"reveal-tip\"") || html.contains("id=&#34;reveal-tip&#34;"),
        "{html}"
    );
    assert!(html.contains("Show tip"), "{html}");
    assert!(html.contains("<script"), "{html}");
    assert!(
        html.contains("/assets/datastar.") || html.contains("datastar."),
        "{html}"
    );
    assert!(
        html.contains("script-src 'self'") || html.contains("script-src &#39;self&#39;"),
        "{html}"
    );
    assert!(
        html.contains("unsafe-eval") || html.contains("unsafe-eval"),
        "{html}"
    );
    assert!(
        html.contains("connect-src 'self'") || html.contains("connect-src &#39;self&#39;"),
        "{html}"
    );
    assert!(
        !html.contains("script-src 'none'") && !html.contains("script-src &#39;none&#39;"),
        "{html}"
    );

    let about = fs::read_to_string(output.join("about/index.html")).unwrap();
    assert!(about.contains("Static neighbor."), "{about}");
    assert_goto_chrome(&about);
    let pages: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
    let live = pages
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["route"] == "/")
        .unwrap();
    assert_eq!(live["kind"], "live");
    assert_eq!(live["datastar"], true);
    let about_entry = pages
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["route"] == "/about")
        .unwrap();
    assert_eq!(about_entry["kind"], "static");
    assert!(about_entry.get("datastar").is_none());
    let islands: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("islands.json")).unwrap()).unwrap();
    assert_eq!(islands["service_origin"], "");
    assert!(
        islands["routes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["method"] == "POST" && route["path"] == "/actions/reveal/show"),
        "{islands}"
    );
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn repeat_hybrid_build_is_byte_identical() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/hybrid");
    let first = temp_dir("hybrid-det-a");
    let second = temp_dir("hybrid-det-b");
    build(&root, &first).unwrap();
    build(&root, &second).unwrap();
    assert_eq!(collect_files(&first), collect_files(&second));
    let _ = fs::remove_dir_all(&first);
    let _ = fs::remove_dir_all(&second);
}

#[test]
fn counter_example_builds_live_with_static_neighbor() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/counter");
    let output = temp_dir("counter-out");
    let report = build(&root, &output).unwrap();
    assert!(report.datastar);
    assert!(
        report
            .pages
            .iter()
            .any(|page| page.kind == crate::article::PageKind::Live && page.datastar),
        "{:?}",
        report.pages
    );
    assert!(
        report
            .pages
            .iter()
            .any(|page| page.kind == crate::article::PageKind::Static && !page.datastar),
        "{:?}",
        report.pages
    );
    let index = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(
        index.contains("id=\"counter\"") || index.contains("id='counter'"),
        "{index}"
    );
    assert!(
        index.contains("rd-document"),
        "site shell should stamp html.rd-document\n{index}"
    );
    assert!(
        index.contains("rel=\"stylesheet\"") && index.contains("/assets/"),
        "{index}"
    );
    assert!(
        !index.contains("site-grid is-plain"),
        "index should use the docs layout\n{index}"
    );
    assert!(
        index.contains("/assets/datastar.") || index.contains("datastar."),
        "{index}"
    );
    let stamp = index
        .split("id=\"counter\"")
        .nth(1)
        .and_then(|rest| rest.split("data-rocci-css=\"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .unwrap_or("");
    assert!(!stamp.is_empty(), "missing counter css stamp\n{index}");
    let stylesheet = index
        .split("href=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("stylesheet href");
    let css = fs::read_to_string(output.join(stylesheet.trim_start_matches('/')))
        .unwrap_or_else(|_| panic!("missing stylesheet {stylesheet}"));
    assert!(
        css.contains(stamp),
        "theme.css should include island stamp {stamp}"
    );
    assert!(
        css.contains("border-radius: 16px"),
        "island card CSS should be in the hashed stylesheet"
    );
    let about = fs::read_to_string(output.join("about/index.html")).unwrap();
    assert!(about.contains("static CDN HTML"), "{about}");
    assert_goto_chrome(&about);
    assert!(
        !about.contains("/assets/datastar") && !about.contains("datastar.js"),
        "{about}"
    );
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn repeat_build_is_byte_identical() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/site");
    let first = temp_dir("det-a");
    let second = temp_dir("det-b");
    build(&root, &first).unwrap();
    build(&root, &second).unwrap();
    assert_eq!(collect_files(&first), collect_files(&second));
    let _ = fs::remove_dir_all(&first);
    let _ = fs::remove_dir_all(&second);
}

#[test]
fn session_reuses_apply_binary_when_roc_sources_are_unchanged() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/site");
    let output = temp_dir("session-out");
    let mut session = BuildSession::create().unwrap();
    let first = session.rebuild(&root, &output).unwrap();
    assert!(output.join("index.html").is_file());
    let second = session.rebuild(&root, &output).unwrap();
    assert!(
        !second.recompiled,
        "unchanged Roc sources should reuse the apply binary (first recompiled={})",
        first.recompiled
    );
    assert!(output.join("index.html").is_file());
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn session_rebuild_failure_preserves_output() {
    let root = temp_dir("session-fail-src");
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
    let output = temp_dir("session-fail-out");
    fs::write(output.join("keep.txt"), "preserve me").unwrap();
    let mut session = BuildSession::create().unwrap();
    let err = session.rebuild(&root, &output).unwrap_err();
    let message = format!("{err:#}");
    assert!(message.contains("duplicate route"), "{message}");
    assert_eq!(
        fs::read_to_string(output.join("keep.txt")).unwrap(),
        "preserve me"
    );
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn wasm_host_build() {
    if skip_without_roc() {
        return;
    }
    let _lock = lock_roc();
    let root = temp_dir("wasm-host-src");
    write_page(
        &root,
        "index.rocdown",
        "@page { route: \"/\", meta: { title: \"Wasm Test\" } }\n\n# Wasm Documentation\nThis was rendered via Wasm host.\n",
    );
    let output = temp_dir("wasm-host-out");
    let _report = match build_with_host(&root, &output, rocci_roc_host::HostChoice::Wasm) {
        Ok(report) => report,
        Err(err) => {
            let message = format!("{err:#}");
            if message.contains("MissingRelocCode") {
                eprintln!("skipping: roc build --target=wasm32 hit compiler MissingRelocCode");
                return;
            }
            panic!("{message}");
        }
    };
    assert!(output.join("index.html").is_file());
    let html = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(html.contains("Wasm Documentation"), "{html}");
    assert!(
        html.contains("<h1 class=\"rd-header-1\""),
        "wasm apply must splice the Markdown blob into the theme article slot\n{html}"
    );
    assert!(!html.contains("&lt;h1"), "{html}");
    let native_out = temp_dir("wasm-host-native-out");
    build_with_host(&root, &native_out, rocci_roc_host::HostChoice::Native).unwrap();
    let native = fs::read_to_string(native_out.join("index.html")).unwrap();
    let article = |page: &str| {
        let start = page
            .find("<article")
            .and_then(|idx| page[idx..].find('>').map(|rel| idx + rel + 1))
            .expect("article open");
        let end = page.find("</article>").expect("article close");
        page[start..end].to_string()
    };
    assert_eq!(article(&html), article(&native));
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
    let _ = fs::remove_dir_all(&native_out);
}
