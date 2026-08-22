use std::fs;
use std::path::{Path, PathBuf};

use rocci_rocdown::{
    CompileOptions, MarkdownBodyOptions, MdNode, OriginKind, PageRef, SourceFile, Span, compile,
    compile_islands, format_ast, index_pages_in_dir, parse_markdown_body,
};
use rocci_template::LowerOptions;
use rocci_theme::ThemeOptions;

fn compile_with(src: &str, options: CompileOptions) -> rocci_rocdown::CompileOutput {
    compile(SourceFile::new("test.rocdown", src), &options)
}

#[test]
fn body_only_markdown_keeps_original_offsets_and_parses_footnotes() {
    let src = "---\ntype: Test\n---\n\n# Body\n\nClaim.[^source]\n\n[^source]: Evidence.\n";
    let body_start = src.find("# Body").unwrap();
    let parsed = parse_markdown_body(
        SourceFile::new("test.md", src),
        Span::new(body_start, src.len()),
        MarkdownBodyOptions {
            raw_html: false,
            footnotes: true,
        },
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.headings[0].span.of(src), "# Body");
    assert!(parsed.document.items.iter().any(|item| matches!(
        item,
        rocci_rocdown::Item::Markdown(MdNode::FootnoteDefinition { name, .. }) if name == "source"
    )));
}

fn page(stem: &str, route: &str, headings: &[&str]) -> PageRef {
    PageRef {
        stem: stem.to_string(),
        file_name: format!("{stem}.rocdown"),
        path: PathBuf::new(),
        route: route.to_string(),
        explicit_route: true,
        heading_ids: headings.iter().map(|id| id.to_string()).collect(),
    }
}

fn compile_ok_pages(src: &str, pages: Vec<PageRef>) -> rocci_rocdown::CompileOutput {
    let out = compile_with(
        src,
        CompileOptions {
            pages,
            ..CompileOptions::default()
        },
    );
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    out
}

fn compile_ok(src: &str) -> rocci_rocdown::CompileOutput {
    compile_ok_pages(src, Vec::new())
}

#[test]
fn compile_output_includes_parse_and_lower_timings() {
    let out = compile_ok("Hello from Rocdown.\n");
    let _ = out.timings.parse_ms;
    let _ = out.timings.lower_ms;
    assert!(out.timings.parse_ms < 10_000);
    assert!(out.timings.lower_ms < 10_000);
}

fn compile_err(src: &str) -> Vec<String> {
    compile_err_pages(src, Vec::new())
}

fn compile_err_pages(src: &str, pages: Vec<PageRef>) -> Vec<String> {
    let out = compile_with(
        src,
        CompileOptions {
            pages,
            ..CompileOptions::default()
        },
    );
    out.diagnostics
        .into_iter()
        .filter(|d| d.is_error())
        .map(|d| d.message)
        .collect()
}

#[test]
fn at_signs_in_prose_are_literal() {
    let src = "\
Email help@example.com.
Follow @roclang.
Use `@component` in a Rocci file.
[mail](mailto:docs@example.com)

    indented @roc is just code

\\@roc { is prose here }
";
    let out = compile_ok(src);
    assert!(out.roc.contains("help@example.com"));
    assert!(out.roc.contains("@roclang"));
    assert!(out.roc.contains("@component"));
    assert!(out.roc.contains("@roc { is prose here }"));
    assert!(!out.roc.contains("feature_count"));
    assert!(out.items_have_no_roc_decl());
}

trait NoRoc {
    fn items_have_no_roc_decl(&self) -> bool;
}

impl NoRoc for rocci_rocdown::CompileOutput {
    fn items_have_no_roc_decl(&self) -> bool {
        !self
            .document
            .items
            .iter()
            .any(|item| matches!(item, rocci_rocdown::Item::Roc(_)))
    }
}

#[test]
fn fenced_code_is_never_executed() {
    let src = "\
```roc
@roc {
    this_is_displayed = True
}
```
";
    let out = compile_ok(src);
    assert!(out.roc.contains("language-roc"));
    assert!(out.items_have_no_roc_decl());
}

#[test]
fn declarations_work_with_leading_indent() {
    let src = "\
    @roc {
    answer = 42
    }

@component
Show = |{ text }| {
    <span>{text}</span>
}

Value: see below.

@render Show({ text: answer.to_str() })
";
    let out = compile_ok(src);
    assert!(out.roc.contains("answer = 42"));
    assert!(out.roc.contains("show("));
    assert!(out.roc.contains("answer.to_str()"));
}

#[test]
fn declarations_inside_lists_and_quotes_stay_markdown() {
    let src = "\
- item
  @roc {
  captured = 1
  }

> @render {
> Html.text(\"nope\")
> }

@roc {
real = 2
}
";
    let out = compile_ok(src);
    let roc_items = out
        .document
        .items
        .iter()
        .filter(|item| matches!(item, rocci_rocdown::Item::Roc(_)))
        .count();
    assert_eq!(roc_items, 1);
    assert!(out.roc.contains("real = 2"));
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(roc real = 2)"));
}

#[test]
fn braces_in_strings_do_not_end_roc_blocks() {
    let src = r#"
@roc {
msg = "close } please"
nested = { a: 1, b: { c: 2 } }
# } comment
}

@component
Show = |{ text }| {
    <span>{text}</span>
}

@render Show({ text: msg })
"#;
    let out = compile_ok(src);
    assert!(out.roc.contains(r#"msg = "close } please""#));
    assert!(out.roc.contains("nested = { a: 1, b: { c: 2 } }"));
}

#[test]
fn css_nested_rules_and_media() {
    let src = r#"
@css {
    .card { color: red; }
    @media (min-width: 40rem) {
        .card { color: blue; }
    }
}

# Title
"#;
    let out = compile_ok(src);
    let file = out
        .styles
        .iter()
        .find(|style| style.kind == rocci_rocdown::StyleKind::File)
        .expect("file css");
    assert!(file.css.contains("@media"));
    assert!(file.css.contains(".card"));
}

#[test]
fn link_references_resolve_across_declarations() {
    let src = "\
See [docs].

@roc {
x = 1
}

[docs]: https://roc-lang.org
";
    let out = compile_ok(src);
    assert!(out.roc.contains("https://roc-lang.org"));
    assert!(
        out.links
            .iter()
            .any(|link| link.url == "https://roc-lang.org")
    );
}

#[test]
fn render_keeps_paragraph_boundaries() {
    let src = "\
@component
Card = |_| {
    <span>card</span>
}

Before the card.

@render Card({})

After the card.
";
    let out = compile_ok(src);
    assert!(out.roc.contains("Before the card."));
    assert!(out.roc.contains("card("));
    assert!(out.roc.contains("After the card."));
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(p"));
    assert!(ast.contains("(render Card)"));
}

#[test]
fn source_maps_cover_markdown_and_roc() {
    let src = "\
@roc {
answer = 1
}

@component
Mark = |_| {
    <span>x</span>
}

# Hello

@render Mark({})
";
    let out = compile_ok(src);
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::MarkdownText
                || seg.origin == OriginKind::MarkdownStructure)
    );
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::RocBlock)
    );
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::RenderRoc)
    );
}

#[test]
fn raw_html_is_rejected_by_default() {
    let errs = compile_err("hello <em>nope</em> there\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("raw HTML is disabled in Rocdown"))
    );
}

#[test]
fn raw_html_can_be_enabled() {
    let src = "hello <em>trusted</em> there\n";
    let out = compile(
        SourceFile::new("test.rocdown", src),
        &CompileOptions {
            lower: LowerOptions::default(),
            raw_html: true,
            ..CompileOptions::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.roc.contains("dangerously_include_unescaped_html"));
}

#[test]
fn duplicate_page_and_unknown_fields_are_errors() {
    let errs = compile_err(
        "\
@page { extra: 1 }
@page { route: \"/x\" }
",
    );
    assert!(errs.iter().any(|msg| msg.contains("unknown `@page` field")));
    assert!(errs.iter().any(|msg| msg.contains("duplicate `@page`")));
}

#[test]
fn text_after_closing_brace_is_an_error() {
    let errs = compile_err(
        "\
@roc { x = 1 } trailing

# Hi
",
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("text after a declaration must be on the next line"))
    );
}

#[test]
fn page_layout_and_route_are_emitted() {
    let src = r#"
@page {
    route: "/guides/rocdown/",
    layout: Docs.article,
    draft: False,
    meta: { title: "Rocdown", description: "Markdown-first pages" },
}

# Hello
"#;
    let out = compile_ok(src);
    assert_eq!(out.page_meta.route.as_deref(), Some("/guides/rocdown/"));
    assert!(out.page_meta.id.is_none());
    assert!(out.page_meta.aliases.is_empty());
    assert!(!out.page_meta.draft);
    assert_eq!(out.page_meta.layout.as_deref(), Some("Docs.article"));
    assert_eq!(
        out.page_meta.description.as_deref(),
        Some("Markdown-first pages")
    );
    assert!(
        out.roc
            .contains("Docs.article({ meta: rocci_meta, content: rocci_content({}) })")
    );
    assert!(
        out.roc
            .contains("rocci_meta = { title: \"Rocdown\", description: \"Markdown-first pages\" }")
    );
    assert!(
        out.routes
            .iter()
            .any(|route| route.method == "GET" && route.path == "/guides/rocdown/")
    );
    assert!(
        out.routes
            .iter()
            .any(|route| route.method == "GET" && route.path == "/")
    );
}

#[test]
fn page_draft_rejects_bool_dot_true() {
    let errs = compile_err("@page {\n    draft: Bool.true,\n}\n\n# Hi\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`draft` must be `True` or `False`")),
        "{errs:?}"
    );
}

#[test]
fn article_params_reject_bool_dot_true() {
    let errs = compile_err(":details[summary: \"More\", open: Bool.true] Nested.\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("Roc booleans are `True` and `False`, not `Bool.true`")),
        "{errs:?}"
    );
}

#[test]
fn site_chrome_layout_is_not_emitted_as_a_roc_call() {
    let src = r#"
@page {
    route: "/",
    layout: "docs",
    meta: { title: "Counter" },
}

@patch("/actions/x") = |_, _request| {
    <p>ok</p>
}

# Hello
"#;
    let out = compile_ok(src);
    assert_eq!(out.page_meta.layout.as_deref(), Some("docs"));
    assert!(!out.roc.contains("docs({ meta:"), "{}", out.roc);
    assert!(out.roc.contains("rocci_content"), "{}", out.roc);
}

#[test]
fn page_theme_and_color_scheme_are_extracted() {
    let src = r#"
@page {
    route: "/x/",
    theme: "rocdown:rocci",
    color_scheme: "dark",
    meta: { title: "Hi" },
}

# Hello
"#;
    let out = compile_ok(src);
    assert_eq!(out.page_meta.theme.as_deref(), Some("rocdown:rocci"));
    assert_eq!(out.page_meta.color_scheme.as_deref(), Some("dark"));
    assert_eq!(
        out.theme.as_ref().map(|theme| theme.id.as_str()),
        Some("rocci")
    );
    assert!(out.roc.contains("data-rd-theme"));
    assert!(out.roc.contains("\"rocci\""));
    assert!(out.roc.contains("data-rd-color-scheme"));
    assert!(out.roc.contains("color-scheme"));
    assert!(out.roc.contains("--rd-color-accent"));
    assert!(out.roc.contains("#48eda4"));
    assert!(out.roc.contains("light-dark("));
    assert!(out.roc.contains("rd-header-1"));
    assert!(
        out.styles
            .iter()
            .any(|style| style.kind == rocci_rocdown::StyleKind::Theme)
    );
}

#[test]
fn page_id_and_aliases_are_extracted() {
    let src = r#"
@page {
    id: "guides.install",
    route: "/install/",
    aliases: ["/getting-started/install/", "/gs/install/"],
    meta: { title: "Install" },
}

# Install
"#;
    let out = compile_ok(src);
    assert_eq!(out.page_meta.id.as_deref(), Some("guides.install"));
    assert_eq!(
        out.page_meta.aliases,
        vec!["/getting-started/install/", "/gs/install/"]
    );
}

#[test]
fn page_id_and_aliases_reject_invalid_values() {
    let errs = compile_err(
        r#"
@page {
    id: "/not-an-id",
    aliases: ["relative", "/ok/../secret/"],
}

# X
"#,
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`id` must not start with `/`")),
        "{errs:?}"
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("must be an absolute URL path")),
        "{errs:?}"
    );
    assert!(errs.iter().any(|msg| msg.contains("`..`")), "{errs:?}");
}

#[test]
fn default_theme_is_paper() {
    let out = compile_ok("# Hello\n");
    assert!(out.roc.contains("rd-document"));
    assert!(out.roc.contains("\"paper\""));
    assert!(out.roc.contains("--rd-font-body"));
    assert!(out.roc.contains("rd-header-1"));
}

#[test]
fn none_theme_skips_injection() {
    let src = r#"
@page { theme: "none" }

# Hello

## Section
"#;
    let out = compile_ok(src);
    assert!(!out.roc.contains("rd-document"));
    assert!(!out.roc.contains("--rd-color-bg"));
    assert!(!out.roc.contains("\"rd-toc\""));
    assert!(!out.roc.contains("\"rd-toc-menu\""));
    assert!(!out.roc.contains("On this page"));
    assert!(!out.roc.contains("requestAnimationFrame"));
}

#[test]
fn default_shell_emits_toc_for_h2_and_h3() {
    let src = "# Title\n\n## Alpha\n\n### Beta\n\n#### Gamma\n";
    let out = compile_ok(src);
    assert!(out.roc.contains("\"rd-toc\""));
    assert!(out.roc.contains("\"rd-toc-menu\""));
    assert!(out.roc.contains("\"rd-shell\""));
    assert!(out.roc.contains("On this page"));
    assert!(out.roc.contains("\"#alpha\""));
    assert!(out.roc.contains("\"#beta\""));
    assert!(out.roc.contains("rd-toc-level-3"));
    assert!(out.roc.contains("requestAnimationFrame"));
    assert!(!out.roc.contains("\"#title\""));
    assert!(!out.roc.contains("\"#gamma\""));
}

#[test]
fn default_shell_omits_toc_without_outline_headings() {
    let out = compile_ok("# Hello\n\nA paragraph.\n");
    assert!(!out.roc.contains("\"rd-toc\""));
    assert!(!out.roc.contains("\"rd-toc-menu\""));
    assert!(!out.roc.contains("\"rd-shell\""));
    assert!(!out.roc.contains("On this page"));
    assert!(!out.roc.contains("requestAnimationFrame"));
}

#[test]
fn custom_layout_omits_toc() {
    let src = r#"
@page { layout: Docs.article }

# Title

## Section
"#;
    let out = compile_ok(src);
    assert!(
        out.roc
            .contains("Docs.article({ meta: rocci_meta, content: rocci_content({}) })")
    );
    assert!(!out.roc.contains("\"rd-toc\""));
    assert!(!out.roc.contains("\"rd-toc-menu\""));
    assert!(!out.roc.contains("\"rd-shell\""));
    assert!(!out.roc.contains("On this page"));
    assert!(!out.roc.contains("requestAnimationFrame"));
}

#[test]
fn unknown_theme_is_an_error() {
    let errs = compile_err("@page { theme: \"nope\" }\n");
    assert!(errs.iter().any(|msg| msg.contains("unknown theme `nope`")));
}

#[test]
fn invalid_color_scheme_is_an_error() {
    let errs = compile_err("@page { color_scheme: \"sepia\" }\n");
    assert!(errs.iter().any(|msg| msg.contains("color scheme")));
}

#[test]
fn cli_default_theme_applies_without_page_theme() {
    let out = compile(
        SourceFile::new("test.rocdown", "# Hello\n"),
        &CompileOptions {
            theme: ThemeOptions {
                default_id: Some("rocci".into()),
                ..ThemeOptions::default()
            },
            ..CompileOptions::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.roc.contains("\"rocci\""));
    assert!(out.roc.contains("#48eda4"));
}

#[test]
fn theme_css_is_emitted_before_file_css() {
    let src = r#"
@page { theme: "paper" }

@css {
    body { margin: 1rem; }
}

# Hello
"#;
    let out = compile_ok(src);
    let theme_at = out.roc.find("--rd-color-bg").expect("theme css");
    let file_at = out.roc.find("margin: 1rem").expect("file css");
    assert!(theme_at < file_at);
}

#[test]
fn static_page_emits_no_datastar() {
    let src = "# Hello\n\nA paragraph.\n";
    let out = compile_ok(src);
    assert!(!out.roc.contains("import Datastar"));
    assert!(out.roc.contains("rocci_page"));
    assert!(out.roc.contains("on_get_root! = |_state, _request|"));
    assert!(out.roc.contains("charset"));
    assert!(out.roc.contains("\"main\""));
}

#[test]
fn heading_ids_disambiguate_duplicates() {
    let src = "# Hello\n\n# Hello\n";
    let out = compile_ok(src);
    assert_eq!(out.headings.len(), 2);
    assert_eq!(out.headings[0].id, "hello");
    assert_eq!(out.headings[1].id, "hello-1");
    assert!(out.roc.contains("\"hello\""));
    assert!(out.roc.contains("\"hello-1\""));
}

#[test]
fn guide_example_compiles() {
    let src = include_str!("../../../examples/rocdown/pages/Guide.rocdown");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/pages");
    let out = compile(
        SourceFile::new("examples/rocdown/pages/Guide.rocdown", src),
        &CompileOptions {
            pages: index_pages_in_dir(&dir),
            ..CompileOptions::default()
        },
    );
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(out.roc.contains("feature_count = 3.I64"));
    assert!(out.roc.contains("charset"));
    assert!(out.roc.contains("\"main\""));
    assert!(out.roc.contains("featureCount = |{ count }|"));
    assert!(out.roc.contains("featureCount("));
    assert!(out.roc.contains("{ count: feature_count }"));
    assert!(out.roc.contains("language-roc"));
    assert!(out.roc.contains("docs@example.com"));
    assert!(out.roc.contains("@roclang"));
    assert!(!out.roc.contains("import Datastar"));
    assert!(out.roc.contains("rocdown-blocks"));
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/guide.roc");
    if std::env::var("UPDATE_FIXTURES").ok().as_deref() == Some("1") {
        fs::write(&fixture, &out.roc).unwrap();
    }
    assert_eq!(out.roc, include_str!("fixtures/guide.roc"));
}

#[test]
fn errors_demo_example_compiles() {
    let src = include_str!("../../../examples/rocdown/errors/ErrorDemo.rocdown");
    let out = compile(
        SourceFile::new("examples/rocdown/errors/ErrorDemo.rocdown", src),
        &CompileOptions::default(),
    );
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(out.roc.contains("demoLinks = |{}|"));
    assert!(out.roc.contains("href"));
    assert!(out.roc.contains("/missing"));
}

#[test]
fn blocks_example_contains_narrow_viewport_fixture() {
    let src = include_str!("../../../examples/rocdown/pages/Blocks.rocdown");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown/pages");
    let out = compile(
        SourceFile::new("examples/rocdown/pages/Blocks.rocdown", src),
        &CompileOptions {
            pages: index_pages_in_dir(&dir),
            ..CompileOptions::default()
        },
    );
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(out.roc.contains("rd-table-wrap"));
    assert!(out.roc.contains("golf-overflow-cell"));
    assert!(out.roc.contains("rd-docs-tab"));
    assert!(out.roc.contains("\"rd-toc-menu\""));
    assert!(out.roc.contains("nested-outline-heading"));
}

fn compile_all_syntax() -> rocci_rocdown::CompileOutput {
    let src = include_str!("../../../test/AllSyntax.rocdown");
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test");
    compile(
        SourceFile::new("test/AllSyntax.rocdown", src),
        &CompileOptions {
            pages: index_pages_in_dir(&test_dir),
            resolve_includes: false,
            ..CompileOptions::default()
        },
    )
}

#[test]
fn all_syntax_example_compiles() {
    let out = compile_all_syntax();
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(out.roc.contains("visible = List.keep_if"));
    assert!(out.roc.contains("if show_notice {"));
    assert!(out.roc.contains("List.map(visible, |item|"));
    assert!(out.roc.contains("List.concat("));
    assert!(out.roc.contains("match status {"));
    assert!(out.roc.contains("hello({ name: \"render\" })"));
    assert!(out.roc.contains("@if this is escaped"));
    assert!(out.roc.contains("rd-docs-aside rd-docs-block rd-docs-note"));
    assert!(out.roc.contains("on_get_all_syntax! = |_state, _request|"));
    assert!(
        out.roc
            .contains("on_post_actions_all_syntax_ping! = |_, _request|")
    );
    assert!(
        !out.roc.contains("Bool.true"),
        "Roc expressions must use True/False, not Bool.true:\n{}",
        out.roc
    );
    assert!(
        out.routes
            .iter()
            .any(|route| route.method == "POST" && route.path == "/actions/all-syntax/ping")
    );
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/all_syntax.roc");
    if std::env::var("UPDATE_FIXTURES").ok().as_deref() == Some("1") {
        fs::write(&fixture, &out.roc).unwrap();
    }
    assert_eq!(out.roc, include_str!("fixtures/all_syntax.roc"));
}

#[test]
fn unknown_at_name_is_markdown() {
    let src = "@roclang is a handle\n";
    let out = compile_ok(src);
    assert!(out.roc.contains("@roclang is a handle"));
}

#[test]
fn top_level_if_for_match_and_let() {
    let src = r#"
@roc {
items = [{ name: "a" }]
show = True
status = Ready
}

# Title

@let visible = items

@if show {
    <p>Shown</p>
} @else {
    <p>Hidden</p>
}

@for item in visible {
    <li>{item.name}</li>
}

@match status {
    Ready => <p>Ready</p>
    Loading => <p>Wait</p>
}
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(if)"));
    assert!(ast.contains("(for item)"));
    assert!(ast.contains("(match)"));
    assert!(ast.contains("(let visible)"));
    assert!(out.roc.contains("visible = items"));
    assert!(out.roc.contains("if show {"));
    assert!(out.roc.contains("} else {"));
    assert!(out.roc.contains("List.map(visible, |item|"));
    assert!(out.roc.contains("List.concat("));
    assert!(out.roc.contains("match status {"));
    assert!(out.roc.contains("\"Title\""));
}

#[test]
fn escaped_and_fenced_if_stay_markdown() {
    let src = "\
\\@if show { not a directive }

```roc
@if show {
    <p>fenced</p>
}
```
";
    let out = compile_ok(src);
    assert!(out.roc.contains("@if show { not a directive }"));
    assert!(out.roc.contains("language-roc"));
    assert!(
        !out.document
            .items
            .iter()
            .any(|item| { matches!(item, rocci_rocdown::Item::Template(_)) })
    );
}

#[test]
fn if_inside_lists_stays_markdown() {
    let src = "\
- item
  @if show { captured }

@if True {
    <p>yes</p>
}
";
    let out = compile_ok(src);
    let templates = out
        .document
        .items
        .iter()
        .filter(|item| matches!(item, rocci_rocdown::Item::Template(_)))
        .count();
    assert_eq!(templates, 1);
    assert!(out.roc.contains("if True {"));
}

#[test]
fn else_attaches_across_blank_lines() {
    let src = r#"
@if True {
    <p>yes</p>
}

@else {
    <p>no</p>
}
"#;
    let out = compile_ok(src);
    assert!(out.roc.contains("if True {"));
    assert!(out.roc.contains("} else {"));
    assert!(out.roc.contains("\"yes\""));
    assert!(out.roc.contains("\"no\""));
    let templates = out
        .document
        .items
        .iter()
        .filter(|item| matches!(item, rocci_rocdown::Item::Template(_)))
        .count();
    assert_eq!(templates, 1);
}

#[test]
fn wiki_and_markdown_page_links_resolve() {
    let pages = vec![page("Foo", "/guides/foo/", &["hello"])];
    let src = "\
See [[Foo]], [[Foo|label]], and [[Foo#hello]].

Also [md](Foo.rocdown) and [rel](./Foo.rocdown).

Also [plain md](Foo.md) and [rel md](./Foo.md).

[ref]: Foo.rocdown
And [ref text][ref].
";
    let out = compile_ok_pages(src, pages);
    assert!(out.roc.contains("\"/guides/foo/\""));
    assert!(out.roc.contains("\"/guides/foo/#hello\""));
    assert!(out.roc.contains("\"label\""));
    assert!(
        out.links
            .iter()
            .filter(|link| link.url.starts_with("/guides/foo/"))
            .count()
            >= 7
    );
}

#[test]
fn nested_markdown_page_links_resolve_to_preview_routes() {
    let pages = vec![
        PageRef {
            stem: "Plan".to_string(),
            file_name: "Plan.md".to_string(),
            path: PathBuf::from("Plan.md"),
            route: "/".to_string(),
            explicit_route: false,
            heading_ids: vec!["plan".to_string()],
        },
        PageRef {
            stem: "About".to_string(),
            file_name: "About.md".to_string(),
            path: PathBuf::from("docs/About.md"),
            route: "/docs/About.md".to_string(),
            explicit_route: false,
            heading_ids: vec!["about".to_string()],
        },
    ];
    let out = compile(
        SourceFile::new(
            "Plan.md",
            "See [about](docs/About.md) and [heading](docs/About.md#about).\n",
        ),
        &CompileOptions {
            pages: pages.clone(),
            ..CompileOptions::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.roc.contains("\"/docs/About.md\""));
    assert!(out.roc.contains("\"/docs/About.md#about\""));

    let back = compile(
        SourceFile::new("docs/About.md", "Back to [plan](../Plan.md).\n"),
        &CompileOptions {
            pages,
            default_route: Some("/docs/About.md".to_string()),
            ..CompileOptions::default()
        },
    );
    assert!(!back.has_errors(), "{:?}", back.diagnostics);
    assert!(back.roc.contains("\"/\""));
    assert!(
        back.routes
            .iter()
            .any(|route| route.method == "GET" && route.path == "/docs/About.md")
    );
}

#[test]
fn absolute_document_path_suffix_matches_page() {
    let pages = vec![PageRef {
        stem: "boundary".to_string(),
        file_name: "static-okf-boundary.md".to_string(),
        path: PathBuf::from("knowledge/decisions/static-okf-boundary.md"),
        route: "/knowledge/decisions/static-okf-boundary.md".to_string(),
        explicit_route: false,
        heading_ids: vec![],
    }];
    let out = compile(
        SourceFile::new(
            "knowledge/decisions/foo.md",
            "See [okf](/decisions/static-okf-boundary.md).\n",
        ),
        &CompileOptions {
            pages,
            ..CompileOptions::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(
        out.roc
            .contains("\"/knowledge/decisions/static-okf-boundary.md\"")
    );
}

#[test]
fn unmatched_absolute_markdown_path_is_not_a_route_error() {
    let pages = vec![page("Foo", "/guides/foo/", &[])];
    let out = compile(
        SourceFile::new("test.rocdown", "[go](/missing.md)\n"),
        &CompileOptions {
            pages,
            ..CompileOptions::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.roc.contains("\"/missing.md\""));
}

#[test]
fn unknown_page_and_heading_are_errors() {
    let pages = vec![page("Foo", "/guides/foo/", &["hello"])];
    let unknown = compile_err_pages("[[Missing]]\n", pages.clone());
    assert!(
        unknown
            .iter()
            .any(|msg| msg.contains("unknown Rocdown page `Missing`")),
        "{unknown:?}"
    );
    let heading = compile_err_pages("[[Foo#nope]]\n", pages);
    assert!(
        heading
            .iter()
            .any(|msg| msg.contains("unknown heading `nope` on page `Foo`")),
        "{heading:?}"
    );
    let same_page = compile_err("# Hello\n\n[x](#missing)\n");
    assert!(
        same_page
            .iter()
            .any(|msg| msg.contains("unknown heading `missing`")),
        "{same_page:?}"
    );
}

#[test]
fn unknown_route_and_collision_are_errors() {
    let pages = vec![page("Foo", "/guides/foo/", &[]), page("Bar", "/dup/", &[])];
    let route = compile_err_pages("[go](/nope/)\n", pages.clone());
    assert!(
        route
            .iter()
            .any(|msg| msg.contains("unknown Rocdown route `/nope/`")),
        "{route:?}"
    );
    let collision = compile(
        SourceFile::new("test.rocdown", "@page { route: \"/dup/\" }\n"),
        &CompileOptions {
            pages,
            ..CompileOptions::default()
        },
    );
    assert!(
        collision.diagnostics.iter().any(|d| d
            .message
            .contains("`@page.route` `/dup/` is also used by Bar.rocdown")),
        "{:?}",
        collision.diagnostics
    );
}

#[test]
fn http_and_autolink_are_unchanged() {
    let out = compile_ok("See [site](https://roc-lang.org) and docs@example.com.\n");
    assert!(out.roc.contains("https://roc-lang.org"));
    assert!(out.roc.contains("mailto:docs@example.com"));
}

#[test]
fn autolink_angle_brackets_are_not_html_islands() {
    let out = compile_ok("<https://roc-lang.org>\n");
    assert!(out.roc.contains("https://roc-lang.org"));
    assert!(
        !out.document
            .items
            .iter()
            .any(|item| matches!(item, rocci_rocdown::Item::Template(_)))
    );
}

#[test]
fn top_level_html_islands_instantiate_components() {
    let src = r#"
@component
Hello = |{ name }| {
    <p>{name}</p>
}

<Hello name="Ada" />

<div class="callout">
    <p>note</p>
</div>
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(call hello)"));
    assert!(ast.contains("(element div)"));
    assert!(out.roc.contains("hello("));
    assert!(out.roc.contains("{ name: \"Ada\" }"));
    assert!(out.roc.contains("\"div\""));
    assert!(out.roc.contains("\"callout\""));
}

#[test]
fn top_level_html_component_tags_pass_children() {
    let src = r#"
@component
Badge = |{ tone }, content| {
    <span class={tone}>{content}</span>
}

<Badge tone="ok">
    <p>child</p>
</Badge>
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(call badge)"));
    assert!(out.roc.contains("badge("));
    assert!(out.roc.contains("{ tone: \"ok\" }"));
    assert!(out.roc.contains("\"p\""));
    assert!(out.roc.contains("\"child\""));
}

#[test]
fn render_call_uses_pascal_case_and_lowers_to_camel_case() {
    let src = r#"
@component
MyComponent = |{ num }| {
    <p>{num.to_str()}</p>
}

@render MyComponent({ num: 1 })
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(render MyComponent)"), "{ast}");
    assert!(out.roc.contains("myComponent("));
    assert!(out.roc.contains("{ num: 1 }"));
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::RenderRoc)
    );
}

#[test]
fn render_brace_body_is_a_removal_error() {
    let errs = compile_err("@render {\n    Html.text(\"x\")\n}\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("PascalCase call") && msg.contains("`{ }` body")),
        "{errs:?}"
    );
}

#[test]
fn render_html_tag_payload_is_an_error() {
    let errs = compile_err("@render <MyComponent num=\"1\" />\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("not an HTML tag") && msg.contains("standalone")),
        "{errs:?}"
    );
}

#[test]
fn render_camel_case_target_is_an_error() {
    let src = r#"
@component
Hello = |{ name }| {
    <p>{name}</p>
}

@render hello({ name: "x" })
"#;
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("PascalCase") && msg.contains("Hello")),
        "{errs:?}"
    );
}

#[test]
fn island_lowering_keeps_markdown_off_the_roc_path() {
    let src = r#"
@roc {
feature_count = 3.I64
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Rocdown

<FeatureCount count={feature_count} />
"#;
    let out = compile_islands(
        SourceFile::new("guide.rocdown", src),
        &CompileOptions::default(),
    );
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(out.roc.contains("rocci_islands"));
    assert!(out.roc.contains("featureCount("));
    assert!(out.roc.contains("feature_count"));
    assert!(!out.roc.contains("rd-header-1"), "{}", out.roc);
    assert!(!out.roc.contains("rocci_content"), "{}", out.roc);
}

fn live_sqlite_helper_page() -> &'static str {
    r#"
@page {
    route: "/",
    layout: "docs",
    meta: { title: "Counter" },
}

@roc {
import pf.Sqlite

read_count! = |db|
    Sqlite.query!(
        {
            db,
            query: "SELECT value FROM counter WHERE id = 1",
            params: {},
            limits: Sqlite.default_query_limits,
        },
    )
}

@patch("/actions/counter/sync") = |_, _request| {
    count = read_count!(db)?
    counterCard({ count: count.value })
}

@component
CounterCard = |{ count }| {
    <output>{count.to_str()}</output>
}

@render CounterCard({ count: 0.I64 })
"#
}

fn compile_islands_ok(src: &str) -> rocci_rocdown::CompileOutput {
    let out = compile_islands(
        SourceFile::new("test.rocdown", src),
        &CompileOptions::default(),
    );
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    out
}

#[test]
fn compile_islands_omits_service_only_roc_helpers() {
    let out = compile_islands_ok(live_sqlite_helper_page());
    assert!(out.roc.contains("rocci_islands"), "{}", out.roc);
    assert!(out.roc.contains("counterCard"), "{}", out.roc);
    assert!(!out.roc.contains("read_count!"), "{}", out.roc);
    assert!(!out.roc.contains("import pf.Sqlite"), "{}", out.roc);
}

#[test]
fn compile_keeps_service_roc_helpers() {
    let out = compile_ok(live_sqlite_helper_page());
    assert!(out.roc.contains("read_count!"), "{}", out.roc);
    assert!(out.roc.contains("import pf.Sqlite"), "{}", out.roc);
}

#[test]
fn compile_islands_keeps_roc_used_from_render() {
    let src = r#"
@roc {
feature_count = 3.I64
format_count = |n| n.to_str()
unused_helper = |_| "no"
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

@render FeatureCount({ count: feature_count, caption: format_count(feature_count) })
"#;
    let out = compile_islands_ok(src);
    assert!(out.roc.contains("rocci_islands"), "{}", out.roc);
    assert!(out.roc.contains("feature_count"), "{}", out.roc);
    assert!(out.roc.contains("format_count"), "{}", out.roc);
    assert!(!out.roc.contains("unused_helper"), "{}", out.roc);
}

#[test]
fn compile_islands_keeps_datastar_for_root_action_elements() {
    let src = r#"
@roc {
import Datastar
}

<div data-init=@get("/sse", [OpenWhenHidden(True)])></div>
"#;
    let out = compile_islands_ok(src);
    assert!(out.roc.contains("import Datastar"), "{}", out.roc);
    assert!(
        out.roc
            .contains("Datastar.get_with(\"/sse\", [OpenWhenHidden(True)])"),
        "{}",
        out.roc
    );
}

#[test]
fn html_inside_lists_and_fences_is_not_an_island() {
    let src = "\
- item
  <Hello />

```html
<div>fenced</div>
```

<Hello />
";
    let out = compile(
        SourceFile::new("test.rocdown", src),
        &CompileOptions::default(),
    );
    let templates = out
        .document
        .items
        .iter()
        .filter(|item| matches!(item, rocci_rocdown::Item::Template(_)))
        .count();
    assert_eq!(templates, 1);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.message.contains("raw HTML is disabled in Rocdown")),
        "{:?}",
        out.diagnostics
    );
}

#[test]
fn inline_html_in_a_paragraph_is_still_rejected() {
    let errs = compile_err("a <em>x</em> b\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("raw HTML is disabled in Rocdown"))
    );
}

#[test]
fn top_level_if_cannot_declare_component() {
    let errs = compile_err(
        r#"
@if True {
    @component Foo = |{}| {
        <p>no</p>
    }
}
"#,
    );
    assert!(
        errs.iter().any(|msg| msg
            .contains("`@component` is only valid at document root, not inside a template body")),
        "{errs:?}"
    );
}

#[test]
fn default_compile_omits_framework_source() {
    let out = compile_ok("# Hello\n\nA paragraph.\n");
    assert!(!out.roc.contains("rocci_docs_source"));
    assert!(!out.roc.contains("rocs_source"));
    assert!(!out.roc.contains("import DocsModel"));
    assert!(!out.roc.contains("import RocsModel"));
}

#[test]
fn resolve_links_false_leaves_relative_hrefs() {
    let src = "[x](guide.rocdown)\n";
    let pages = vec![page("guide", "/guide/", &[])];
    let resolved = compile_ok_pages(src, pages.clone());
    assert!(resolved.roc.contains("\"/guide/\""));
    assert!(!resolved.roc.contains("\"guide.rocdown\""));

    let unresolved = compile(
        SourceFile::new("test.rocdown", src),
        &CompileOptions {
            pages,
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(!unresolved.has_errors(), "{:?}", unresolved.diagnostics);
    assert!(unresolved.roc.contains("\"guide.rocdown\""));
    assert!(!unresolved.roc.contains("\"/guide/\""));
}

#[test]
fn resolve_links_false_skips_route_collisions() {
    let pages = vec![page("Foo", "/dup/", &[]), page("Bar", "/dup/", &[])];
    let out = compile(
        SourceFile::new("test.rocdown", "@page { route: \"/dup/\" }\n"),
        &CompileOptions {
            pages,
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    assert!(
        !out.diagnostics
            .iter()
            .any(|d| d.message.contains("also used by")),
        "{:?}",
        out.diagnostics
    );
}

#[test]
fn docs_note_is_parsed_and_lowered() {
    let src = "\
# Guide

:note[title: \"Deprecation\"] {{
    Do not use `foo` in production.
}}
";
    let out = compile_ok(src);
    assert!(
        out.document.items.iter().any(|item| matches!(
            item,
            rocci_rocdown::Item::Block(call) if call.name == "note"
        )),
        "{:?}",
        out.document.items
    );
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(block note"), "{ast}");
    assert!(out.roc.contains("data-rocci-docs"));
    assert!(out.roc.contains("rd-docs-note"));
    assert!(out.roc.contains("Deprecation"));
}

#[test]
fn kebab_docs_kinds_are_parsed() {
    let src = "\
:link-card[href: \"/guide/\", title: \"Guide\"]
";
    let out = compile_ok(src);
    let kinds: Vec<_> = out
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            rocci_rocdown::Item::Block(call) => Some(call.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, ["link-card"]);

    let errs = compile_err(":api-operation[id: \"get\"]\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:api-operation` is not an authorable article kind")),
        "{errs:?}"
    );
}

#[test]
fn nested_docs_and_escaped_docs_are_distinct() {
    let src = "\
:steps.begin
    :step[title: \"Install\"] Run the installer.
:steps.end

\\@docs note { not a directive }
";
    let out = compile_ok(src);
    let docs: Vec<_> = out
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            rocci_rocdown::Item::Block(call) => Some(call.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(docs, ["steps"]);
    let nested = rocci_rocdown::parse_fragment(
        SourceFile::new("test.rocdown", src),
        out.document
            .items
            .iter()
            .find_map(|item| match item {
                rocci_rocdown::Item::Block(call) => call.content_span(),
                _ => None,
            })
            .unwrap(),
        false,
    );
    assert!(nested.document.items.iter().any(|item| matches!(
        item,
        rocci_rocdown::Item::Block(call) if call.name == "step"
    )));
    assert!(out.roc.contains("@docs note { not a directive }"));
}

#[test]
fn render_inside_docs_is_an_error() {
    let src = "\
:note {{
    @render {
        Html.text(\"nope\")
    }
}}
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`@render` is not allowed inside an article block")),
        "{errs:?}"
    );
}

#[test]
fn embedded_languages_rocdown_fixture_compiles() {
    let src = include_str!("../../../test/EmbeddedLanguages.rocdown");
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test/EmbeddedLanguages.rocdown")
        .canonicalize()
        .expect("canonicalize path");
    let out = compile(
        SourceFile::new(&path.display().to_string(), src),
        &CompileOptions::default(),
    );
    assert!(
        out.diagnostics
            .iter()
            .all(|d| d.severity != rocci_template::Severity::Error),
        "{:?}",
        out.diagnostics
    );
}

#[test]
fn img_with_src_and_width_is_parsed_and_lowered() {
    let src = "\
# Image Demo

:img[src: \"img/yammi_banana.png\", alt: \"A banana\", width: \"50px\"]
";
    let out = compile_ok(src);
    assert!(out.document.items.iter().any(|item| matches!(
        item,
        rocci_rocdown::Item::Block(call) if call.name == "img"
    )));
    assert!(out.roc.contains("Html.void_element("));
    assert!(out.roc.contains("\"img\""));
    assert!(out.roc.contains("\"rd-image\""));
    assert!(out.roc.contains("\"img/yammi_banana.png\""));
    assert!(out.roc.contains("\"50px\""));
}

#[test]
fn img_with_all_optional_fields() {
    let src = "\
:img[src: \"img/banana.png\", alt: \"A tasty banana\", title: \"Yummy banana\", width: \"100px\", height: \"80px\", class: \"hero-img\", loading: \"lazy\", decoding: \"async\"]
";
    let out = compile_ok(src);
    assert!(out.roc.contains("rd-image hero-img"));
    assert!(out.roc.contains("\"img/banana.png\""));
    assert!(out.roc.contains("\"A tasty banana\""));
    assert!(out.roc.contains("\"Yummy banana\""));
    assert!(out.roc.contains("\"100px\""));
    assert!(out.roc.contains("\"80px\""));
    assert!(out.roc.contains("\"lazy\""));
    assert!(out.roc.contains("\"async\""));
}

#[test]
fn img_without_explicit_sizing_is_valid() {
    let src = "\
:img[src: \"img/simple.png\", alt: \"Simple image\"]
";
    let out = compile_ok(src);
    assert!(out.roc.contains("\"rd-image\""));
    assert!(out.roc.contains("\"img/simple.png\""));
    assert!(!out.roc.contains("\"width\""));
    assert!(!out.roc.contains("\"height\""));
}

#[test]
fn img_missing_src_is_an_error() {
    let src = "\
:img[width: \"50px\"]
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("missing required field `src` in `:img`")),
        "{errs:?}"
    );
}

#[test]
fn img_unknown_field_is_an_error() {
    let src = "\
:img[src: \"img/banana.png\", alt: \"A tasty banana\", bad_field: \"value\"]
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("unknown field `bad_field` in `:img`")),
        "{errs:?}"
    );
}

#[test]
fn escaped_img_and_img_in_fences_are_inert() {
    let src = "\
\\@img {
    src: \"not-an-img\"
}

```rocdown
@img {
    src: \"in-fence\"
}
```

- list
  @img {
      src: \"in-list\"
  }
";
    let out = compile_ok(src);
    assert!(!out.document.items.iter().any(|item| matches!(
        item,
        rocci_rocdown::Item::Block(call) if call.name == "img"
    )));
    assert!(out.roc.contains("@img {"));
    assert!(out.roc.contains("not-an-img"));
}

#[test]
fn img_nested_inside_docs_component() {
    let src = "\
:figure[caption: \"Architecture\", credit: \"Rocci docs\"] {{
    :img[src: \"diagram.png\", alt: \"Diagram\", width: \"400px\"]
}}
";
    let out = compile_ok(src);
    assert!(out.roc.contains("rd-docs-figure"));
    assert!(out.roc.contains("\"diagram.png\""));
    assert!(out.roc.contains("\"400px\""));
    assert!(out.roc.contains("\"rd-docs-caption\""));
    assert!(out.roc.contains("\"Architecture\""));
    assert!(out.roc.contains("\"rd-docs-credit\""));
    assert!(out.roc.contains("\"Rocci docs\""));
}

#[test]
fn img_missing_alt_is_an_error() {
    let src = "\
:img[src: \"img/simple.png\"]
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:img` requires `alt` for meaningful images")),
        "{errs:?}"
    );
}

#[test]
fn img_decorative_emits_empty_alt() {
    let src = "\
:img[src: \"img/divider.png\", decorative: True]
";
    let out = compile_ok(src);
    assert!(out.roc.contains("\"img/divider.png\""));
    assert!(out.roc.contains(".attribute(\"alt\", \"\")"));
}

#[test]
fn img_decorative_rejects_nonempty_alt() {
    let src = "\
:img[src: \"img/divider.png\", alt: \"Divider\", decorative: True]
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("decorative `:img` must not set a non-empty `alt`")),
        "{errs:?}"
    );
}

#[test]
fn img_invalid_loading_and_decoding_are_errors() {
    let src = "\
:img[src: \"img/simple.png\", alt: \"Simple\", loading: \"whenever\", decoding: \"fast\"]
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`loading` must be one of")),
        "{errs:?}"
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`decoding` must be one of")),
        "{errs:?}"
    );
}

#[test]
fn ordinary_footnotes_render_with_backlinks() {
    let src = "\
Claim.[^source]

[^source]: Evidence.
";
    let out = compile_ok(src);
    assert!(out.roc.contains("rd-footnote-ref"));
    assert!(out.roc.contains("data-footnote-ref"));
    assert!(out.roc.contains("rd-footnotes"));
    assert!(out.roc.contains("Footnotes"));
    assert!(out.roc.contains("data-footnote-backref"));
    assert!(out.roc.contains("fn-source"));
    assert!(out.roc.contains("fnref-source"));
    assert!(out.roc.contains("Evidence."));
}

#[test]
fn missing_footnote_definition_is_an_error() {
    let src = "Claim.[^missing]\n";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("footnote `[^missing]` has no definition")),
        "{errs:?}"
    );
}

#[test]
fn duplicate_footnote_definition_is_an_error() {
    let src = "\
Claim.[^source]

[^source]: One.
[^source]: Two.
";
    let errs = compile_err(src);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("duplicate footnote definition `[^source]`")),
        "{errs:?}"
    );
}

#[test]
fn inline_code_and_fenced_footnote_patterns_do_not_require_definitions() {
    let src = "\
`[^label]` in inline code is not a reference.

````rocdown
Claim.[^note]

[^note]: In fence.
````
";
    let out = compile_ok(src);
    assert!(!out.roc.contains("rd-footnotes"));
}

#[test]
fn missing_local_asset_is_diagnosed_when_enabled() {
    let src = "\
:img[src: \"missing-file.png\", alt: \"Missing\"]
";
    let out = compile_with(
        src,
        CompileOptions {
            check_assets: true,
            theme: ThemeOptions {
                source_dir: Some(std::env::temp_dir()),
                ..ThemeOptions::default()
            },
            ..CompileOptions::default()
        },
    );
    assert!(
        out.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error() && diagnostic.message.contains("missing local asset")
        }),
        "{:?}",
        out.diagnostics
    );
}

#[test]
fn dotted_relative_img_resolves_against_source_dir() {
    let dir = std::env::temp_dir().join(format!("rocci-rocdown-rel-img-{}", std::process::id()));
    fs::create_dir_all(dir.join("img")).unwrap();
    fs::write(dir.join("img/dot.png"), b"png").unwrap();
    let src = "\
:img[src: \"./img/dot.png\", alt: \"Dot\"]
";
    let out = compile_with(
        src,
        CompileOptions {
            check_assets: true,
            theme: ThemeOptions {
                source_dir: Some(dir.clone()),
                ..ThemeOptions::default()
            },
            ..CompileOptions::default()
        },
    );
    let _ = fs::remove_dir_all(&dir);
    assert!(
        !out.has_errors(),
        "{}",
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(out.roc.contains("./img/dot.png"));
}

#[test]
fn parent_relative_img_is_rejected_when_checking_assets() {
    let src = "\
:img[src: \"../secret.png\", alt: \"Secret\"]
";
    let out = compile_with(
        src,
        CompileOptions {
            check_assets: true,
            theme: ThemeOptions {
                source_dir: Some(std::env::temp_dir()),
                ..ThemeOptions::default()
            },
            ..CompileOptions::default()
        },
    );
    assert!(
        out.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("is not a path under the source file")
        }),
        "{:?}",
        out.diagnostics
    );
}
