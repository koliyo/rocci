use std::path::Path;

use rocci_highlight::HighlightKind;
use rocci_rocdown::{
    CompileOptions, IncludeOptions, Item, MdNode, OriginKind, PageKind, SourceFile,
    classify_document, compile, format_ast, highlight_rocdown, is_static_document, load_page_docs,
    markdown_fragment_gated, render_article_gated, render_document, render_document_gated,
};

fn compile_with(src: &str) -> rocci_rocdown::CompileOutput {
    compile(
        SourceFile::new("test.rocdown", src),
        &CompileOptions::default(),
    )
}

fn ast(src: &str) -> String {
    let out = compile_with(src);
    format_ast(src, &out.document)
}

fn paragraph_children(out: &rocci_rocdown::CompileOutput) -> &[MdNode] {
    out.document
        .items
        .iter()
        .find_map(|item| match item {
            Item::Markdown(MdNode::Paragraph { children, .. }) => Some(children.as_slice()),
            _ => None,
        })
        .expect("expected a markdown paragraph")
}

fn interp_expr<'a>(src: &'a str, node: &MdNode) -> &'a str {
    match node {
        MdNode::Interpolation { expr, .. } => expr.of(src).trim(),
        other => panic!("expected interpolation, got {other:?}"),
    }
}

#[test]
fn published_date_splits_text_interp_text() {
    let src = "Published @{date}.\n";
    let out = compile_with(src);
    assert!(
        !out.diagnostics.iter().any(|d| d.is_error()),
        "{:?}",
        out.diagnostics
    );
    let children = paragraph_children(&out);
    assert_eq!(children.len(), 3, "{children:?}");
    match &children[0] {
        MdNode::Text { value, .. } => assert_eq!(value, "Published "),
        other => panic!("expected leading text, got {other:?}"),
    }
    assert_eq!(interp_expr(src, &children[1]), "date");
    match &children[2] {
        MdNode::Text { value, .. } => assert_eq!(value, "."),
        other => panic!("expected trailing text, got {other:?}"),
    }
    let dump = format_ast(src, &out.document);
    assert!(dump.contains("(interp date)"), "{dump}");
}

#[test]
fn method_call_and_if_expr_holes() {
    let src = "@{count.to_str()} then @{ if x { \"a\" } else { \"b\" } }\n";
    let dump = ast(src);
    assert!(dump.contains("(interp count.to_str())"), "{dump}");
    assert!(
        dump.contains("(interp if x { \"a\" } else { \"b\" })"),
        "{dump}"
    );
}

#[test]
fn nested_braces_and_strings_in_expr() {
    let src = "@{List.len(items)} and @{\"close } please\"}\n";
    let dump = ast(src);
    assert!(dump.contains("(interp List.len(items))"), "{dump}");
    assert!(dump.contains("(interp \"close } please\")"), "{dump}");
}

#[test]
fn escaped_at_brace_stays_text() {
    let src = "Use \\@{upstream} in prose.\n";
    let out = compile_with(src);
    assert!(
        !out.diagnostics.iter().any(|d| d.is_error()),
        "{:?}",
        out.diagnostics
    );
    let dump = format_ast(src, &out.document);
    assert!(!dump.contains("(interp"), "{dump}");
    assert!(dump.contains("@{upstream}"), "{dump}");
}

#[test]
fn even_backslashes_open_a_real_hole() {
    let src = "Use \\\\@{ident} in prose.\n";
    let out = compile_with(src);
    assert!(
        !out.diagnostics.iter().any(|d| d.is_error()),
        "{:?}",
        out.diagnostics
    );
    let dump = format_ast(src, &out.document);
    assert!(dump.contains("(interp ident)"), "{dump}");
    assert!(
        out.roc.contains("Html.text(ident)") || out.roc.contains(".text(ident)"),
        "{}",
        out.roc
    );
}

#[test]
fn table_cell_interpolates() {
    let src = "| col | val |\n| --- | --- |\n| a | @{x} |\n";
    let out = compile_with(src);
    let mut found = false;
    for item in &out.document.items {
        if let Item::Markdown(md) = item {
            md.walk(&mut |node| {
                if let MdNode::Interpolation { expr, .. } = node {
                    found |= expr.of(src).trim() == "x";
                }
            });
        }
    }
    assert!(
        found,
        "expected a table-cell interpolation, items={:?}",
        out.document.items
    );
    assert!(
        out.roc.contains("Html.text(x)") || out.roc.contains(".text(x)"),
        "{}",
        out.roc
    );
}

#[test]
fn entity_adjacent_text_still_splits_the_hole() {
    let src = "A &amp; @{x} value.\n";
    let out = compile_with(src);
    let dump = format_ast(src, &out.document);
    assert!(dump.contains("(interp x)"), "{dump}");
    let children = paragraph_children(&out);
    assert!(
        children
            .iter()
            .any(|node| matches!(node, MdNode::Interpolation { .. })),
        "{children:?}"
    );
}

#[test]
fn inline_code_and_fences_stay_inert() {
    let src =
        "Use `@{upstream}` in a path.\n\n```\n@{date}\n```\n\n    indented @{date} stays code\n";
    let out = compile_with(src);
    assert!(
        !out.diagnostics.iter().any(|d| d.is_error()),
        "{:?}",
        out.diagnostics
    );
    let dump = format_ast(src, &out.document);
    assert!(!dump.contains("(interp"), "{dump}");
    assert!(dump.contains("(code @{upstream})"), "{dump}");
    assert!(dump.contains("(fence"), "{dump}");
}

#[test]
fn bare_brace_email_and_handle_stay_text() {
    let src = "See {date}. Email docs@example.com. Follow @roclang.\n";
    let dump = ast(src);
    assert!(!dump.contains("(interp"), "{dump}");
    assert!(dump.contains("{date}"), "{dump}");
    assert!(dump.contains("docs@example.com"), "{dump}");
    assert!(dump.contains("@roclang"), "{dump}");
}

#[test]
fn unterminated_hole_diagnoses_and_terminates() {
    let src = "Published @{date\n";
    let out = compile_with(src);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.contains("unterminated interpolation")),
        "{:?}",
        out.diagnostics
    );
    let dump = format_ast(src, &out.document);
    assert!(dump.contains("(interp date)"), "{dump}");
}

#[test]
fn lowers_hole_to_html_text_expression() {
    let src = "Published @{date}.\n";
    let out = compile_with(src);
    assert!(
        out.roc.contains("Html.text(date)") || out.roc.contains(".text(date)"),
        "{}",
        out.roc
    );
    assert!(
        !out.roc.contains("\"@{date}\""),
        "must not stringify the hole: {}",
        out.roc
    );
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::TextExpression),
        "{:?}",
        out.segments
            .iter()
            .map(|seg| seg.origin)
            .collect::<Vec<_>>()
    );
}

#[test]
fn hole_only_page_is_hydrate_without_roc() {
    let src = "Published @{x}.\n";
    let out = compile_with(src);
    let class = classify_document(src, &out.document, false);
    assert_eq!(class.kind, PageKind::Hydrate);
    assert_eq!(class.reason, "@{");
    assert_eq!(is_static_document(src, &out.document), Err("@{"));
}

#[test]
fn hole_with_roc_stays_hydrate_with_at_brace_reason() {
    let src = "@roc { x = \"hi\" }\n\nPublished @{x}.\n";
    let out = compile_with(src);
    let class = classify_document(src, &out.document, false);
    assert_eq!(class.kind, PageKind::Hydrate);
    assert_eq!(class.reason, "@{");
}

#[test]
fn note_body_hole_lowers_on_hydrate_page() {
    let src = "@roc { name = \"Ada\" }\n\n:note {{ Hello @{name}. }}\n";
    let out = compile_with(src);
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(
        out.roc.contains("Html.text(name)") || out.roc.contains(".text(name)"),
        "{}",
        out.roc
    );
    let class = classify_document(src, &out.document, false);
    assert_eq!(class.kind, PageKind::Hydrate);
}

#[test]
fn rust_article_html_does_not_emit_hole_glyphs() {
    let src = "Published @{date}.\n";
    let out = compile_with(src);
    let rendered = render_document_gated(&out.document);
    assert!(!rendered.html.contains("@{date}"), "{}", rendered.html);
    assert!(!rendered.html.contains("@{"), "{}", rendered.html);
    assert!(
        rendered.diagnostics.iter().any(|d| d.is_error()
            && d.message.contains("static Rust article path")
            && d.span.of(src).contains("@{")),
        "{:?}",
        rendered.diagnostics
    );
    let html = render_document(&out.document);
    assert!(!html.contains("@{date}"), "{html}");
}

#[test]
fn docs_forest_gate_does_not_emit_hole_glyphs() {
    let src = "Published @{date}.\n";
    let out = compile_with(src);
    let mut catalog = Vec::new();
    let docs = load_page_docs(
        SourceFile::new("test.rocdown", src),
        &out.document,
        "test.rocdown",
        IncludeOptions {
            root: Path::new("."),
            snippet_roots: &[],
        },
        &mut catalog,
    );
    let rendered = render_article_gated(&docs.article);
    assert!(!rendered.html.contains("@{date}"), "{}", rendered.html);
    assert!(
        rendered
            .diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.contains("static Rust article path")),
        "{:?}",
        rendered.diagnostics
    );
    let (markdown, md_diagnostics) = markdown_fragment_gated(&docs.article);
    assert!(!markdown.contains("@{date}"), "{markdown}");
    assert!(
        md_diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.contains("static Rust article path")),
        "{md_diagnostics:?}"
    );
}

#[test]
fn heading_hole_is_an_error_and_slug_stays_literal() {
    let src = "# Hello @{ver}\n";
    let out = compile_with(src);
    assert!(
        out.diagnostics.iter().any(|d| d.is_error()
            && d.message.contains("not allowed in headings")
            && d.span.of(src).contains("@{")),
        "{:?}",
        out.diagnostics
    );
    assert_eq!(out.headings.len(), 1);
    assert_eq!(out.headings[0].id, "hello-ver");
    assert!(
        out.headings[0].text.contains("@{ver}"),
        "{:?}",
        out.headings[0].text
    );
    assert!(
        !out.roc.contains("Html.text(ver)") && !out.roc.contains(".text(ver)"),
        "{}",
        out.roc
    );
}

#[test]
fn colon_heading_hole_is_an_error() {
    let src = ":h2 Hello @{ver}\n";
    let out = compile_with(src);
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.is_error() && d.message.contains("not allowed in headings")),
        "{:?}",
        out.diagnostics
    );
}

#[test]
fn link_destination_at_brace_stays_literal_href() {
    let src = "[t](@{url})\n";
    let out = compile_with(src);
    let dump = format_ast(src, &out.document);
    assert!(
        dump.contains("(a @{url})") || dump.contains("@{url}"),
        "{dump}"
    );
    assert!(
        !out.roc.contains("Html.text(url)") && !out.roc.contains(".text(url)"),
        "destination must not interpolate: {}",
        out.roc
    );
}

#[test]
fn link_text_interpolates_destination_does_not() {
    let src = "[see @{title}](/x/)\n";
    let out = compile_with(src);
    let dump = format_ast(src, &out.document);
    assert!(dump.contains("(interp title)"), "{dump}");
    assert!(
        out.roc.contains("Html.text(title)") || out.roc.contains(".text(title)"),
        "{}",
        out.roc
    );
}

#[test]
fn image_alt_does_not_interpolate_url() {
    let src = "![alt @{x}](./a.png)\n";
    let out = compile_with(src);
    let dump = format_ast(src, &out.document);
    assert!(!dump.contains("(interp x)"), "{dump}");
    assert!(
        !out.roc.contains("Html.text(x)") && !out.roc.contains(".text(x)"),
        "image url/alt must not interpolate in v1: {}",
        out.roc
    );
}

#[test]
fn footnote_body_may_interpolate_label_does_not() {
    let src = "Claim.[^lab]\n\n[^lab]: Hello @{name}.\n";
    let out = compile_with(src);
    let mut found = false;
    for item in &out.document.items {
        if let Item::Markdown(MdNode::FootnoteDefinition { .. }) = item {
            item_walk_interp(item, &mut found);
        }
    }
    assert!(found, "footnote body should contain an interpolation node");
    assert!(
        out.roc.contains("Html.text(name)") || out.roc.contains(".text(name)"),
        "{}",
        out.roc
    );
}

fn item_walk_interp(item: &Item, found: &mut bool) {
    if let Item::Markdown(md) = item {
        md.walk(&mut |node| {
            if matches!(node, MdNode::Interpolation { .. }) {
                *found = true;
            }
        });
    }
}

#[test]
fn interpolation_highlights_delimiters_and_expr() {
    let src = "Hello @{name}.\n";
    let spans = highlight_rocdown(src);
    let painted: Vec<_> = spans
        .iter()
        .map(|s| (src.get(s.start()..s.end()).unwrap_or(""), s.kind))
        .collect();
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Punctuation
                && src.get(s.start()..s.end()) == Some("@{")),
        "{painted:?}"
    );
    assert!(
        spans
            .iter()
            .any(|s| s.kind == HighlightKind::Punctuation
                && src.get(s.start()..s.end()) == Some("}")),
        "{painted:?}"
    );
    let expr = src.find("name").expect("name");
    assert!(
        spans.iter().any(|s| {
            s.kind != HighlightKind::Punctuation && s.start() <= expr && s.end() >= expr + 4
        }),
        "expr should be painted as Roc: {painted:?}"
    );
}
