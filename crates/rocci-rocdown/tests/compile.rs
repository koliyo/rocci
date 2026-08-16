use rocci_rocdown::{CompileOptions, OriginKind, SourceFile, compile, format_ast};
use rocci_template::LowerOptions;
use rocci_theme::ThemeOptions;

fn compile_ok(src: &str) -> rocci_rocdown::CompileOutput {
    let out = compile(
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

fn compile_err(src: &str) -> Vec<String> {
    let out = compile(
        SourceFile::new("test.rocdown", src),
        &CompileOptions::default(),
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
    this_is_displayed = Bool.true
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

Value: see below.

@render {
    Html.text(answer.to_str())
}
";
    let out = compile_ok(src);
    assert!(out.roc.contains("answer = 42"));
    assert!(out.roc.contains("Html.text(answer.to_str())"));
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

@render {
    Html.text(msg)
}
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
Before the card.

@render {
    Html.text(\"card\")
}

After the card.
";
    let out = compile_ok(src);
    assert!(out.roc.contains("Before the card."));
    assert!(out.roc.contains("Html.text(\"card\")"));
    assert!(out.roc.contains("After the card."));
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(p"));
    assert!(ast.contains("(render"));
}

#[test]
fn source_maps_cover_markdown_and_roc() {
    let src = "\
@roc {
answer = 1
}

# Hello

@render {
    Html.text(\"x\")
}
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
    let errs = compile_err("<div>nope</div>\n");
    assert!(
        errs.iter()
            .any(|msg| msg.contains("raw HTML is disabled in Rocdown"))
    );
}

#[test]
fn raw_html_can_be_enabled() {
    let src = "<div>trusted</div>\n";
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
            .any(|msg| msg.contains("text after a declaration's closing `}`"))
    );
}

#[test]
fn page_layout_and_route_are_emitted() {
    let src = r#"
@page {
    route: "/guides/rocdown/",
    layout: Docs.article,
    draft: Bool.false,
    meta: { title: "Rocdown" },
}

# Hello
"#;
    let out = compile_ok(src);
    assert_eq!(out.page_meta.route.as_deref(), Some("/guides/rocdown/"));
    assert!(!out.page_meta.draft);
    assert_eq!(out.page_meta.layout.as_deref(), Some("Docs.article"));
    assert!(
        out.roc
            .contains("Docs.article({ meta: rocci_meta, content: rocci_content({}) })")
    );
    assert!(out.roc.contains("rocci_meta = { title: \"Rocdown\" }"));
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
"#;
    let out = compile_ok(src);
    assert!(!out.roc.contains("rd-document"));
    assert!(!out.roc.contains("--rd-color-bg"));
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
    assert!(out.roc.contains("on_get_root!"));
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
    let src = include_str!("../../../examples/rocdown/Guide.rocdown");
    let out = compile(
        SourceFile::new("examples/rocdown/Guide.rocdown", src),
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
    assert!(out.roc.contains("feature_count = 3.I64"));
    assert!(out.roc.contains("charset"));
    assert!(out.roc.contains("\"main\""));
    assert!(out.roc.contains("featureCount = |{ count }|"));
    assert!(out.roc.contains("featureCount({ count: feature_count })"));
    assert!(out.roc.contains("language-roc"));
    assert!(out.roc.contains("docs@example.com"));
    assert!(out.roc.contains("@roclang"));
    assert!(!out.roc.contains("import Datastar"));
    assert_eq!(out.roc, include_str!("fixtures/guide.roc"));
}

#[test]
fn all_syntax_example_compiles() {
    let src = include_str!("../../../test/AllSyntax.rocdown");
    let out = compile(
        SourceFile::new("test/AllSyntax.rocdown", src),
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
    assert!(out.roc.contains("visible = List.keepIf"));
    assert!(out.roc.contains("if show_notice {"));
    assert!(out.roc.contains("List.map(visible, |item|"));
    assert!(out.roc.contains("List.concat("));
    assert!(out.roc.contains("match status {"));
    assert!(out.roc.contains("hello({ name: \"render\" })"));
    assert!(out.roc.contains("@if this is escaped"));
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
show = Bool.true
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

@if Bool.true {
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
    assert!(out.roc.contains("if Bool.true {"));
}

#[test]
fn else_attaches_across_blank_lines() {
    let src = r#"
@if Bool.true {
    <p>yes</p>
}

@else {
    <p>no</p>
}
"#;
    let out = compile_ok(src);
    assert!(out.roc.contains("if Bool.true {"));
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
