use rocci_template::{
    LowerOptions, OriginKind, SourceFile, compile, generated_to_source, map_source_span,
    project_type_module, source_to_generated, wrap_type_module,
};

fn compile_ok(src: &str) -> rocci_template::CompileOutput {
    let out = compile(SourceFile::new("test.rocci", src), &LowerOptions::default());
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

fn ident_offset(src: &str, ident: &str, after: &str) -> u32 {
    let from = src.find(after).unwrap_or_else(|| panic!("missing {after}"));
    let rel = src[from..]
        .find(ident)
        .unwrap_or_else(|| panic!("missing {ident} after {after}"));
    (from + rel) as u32
}

#[test]
fn interpolation_maps_to_generated_ident_not_html_text() {
    let src = r#"
@component Hello = |{ title }| {
    <p>{title}</p>
}
"#;
    let out = compile_ok(src);
    let title = ident_offset(src, "title", "{title}");
    let mapped = source_to_generated(src, &out.roc, &out.segments, title).expect("title map");
    assert_eq!(mapped.origin, OriginKind::TextExpression);
    let generated = &out.roc[mapped.offset as usize..];
    assert!(generated.starts_with("title"), "{generated}");
    assert!(
        !generated.starts_with("Html"),
        "mapped onto Html.text scaffolding: {generated}"
    );

    let back = generated_to_source(src, &out.roc, &out.segments, mapped.offset).expect("roundtrip");
    assert_eq!(back.offset, title);
    assert_eq!(back.origin, OriginKind::TextExpression);

    let span = map_source_span(
        src,
        &out.roc,
        &out.segments,
        rocci_template::Span::new(title as usize, title as usize + 5),
    )
    .expect("title span");
    assert_eq!(span.of(&out.roc), "title");
}

#[test]
fn trimmed_interpolation_maps_interior_only() {
    let src = r#"
@component Hello = |{ title }| {
    <p>{  title  }</p>
}
"#;
    let out = compile_ok(src);
    let open = src.find("{  title  }").expect("interp");
    let leading = (open + 1) as u32;
    assert!(source_to_generated(src, &out.roc, &out.segments, leading).is_none());
    let title = ident_offset(src, "title", "{  title  }");
    let mapped = source_to_generated(src, &out.roc, &out.segments, title).expect("title");
    assert_eq!(
        &out.roc[mapped.offset as usize..mapped.offset as usize + 5],
        "title"
    );
}

#[test]
fn roc_block_maps_verbatim() {
    let src = r#"
import Html

hello = |name| Html.text(name)

@component Card = |{ title }| {
    <div>{hello(title)}</div>
}
"#;
    let out = compile_ok(src);
    let hello = ident_offset(src, "hello", "hello = |name|");
    let mapped = source_to_generated(src, &out.roc, &out.segments, hello).expect("hello");
    assert_eq!(mapped.origin, OriginKind::OrdinaryRoc);
    assert!(out.roc[mapped.offset as usize..].starts_with("hello"));
}

#[test]
fn handler_body_maps() {
    let src = r#"
@get:view("/") {
    Html.text("ok")
}

@component Page = |{}| {
    <html><body></body></html>
}
"#;
    let out = compile_ok(src);
    let html = ident_offset(src, "Html", "Html.text");
    let mapped = source_to_generated(src, &out.roc, &out.segments, html).expect("Html");
    assert!(out.roc[mapped.offset as usize..].starts_with("Html"));
}

#[test]
fn scaffolding_and_markup_do_not_map() {
    let src = r#"
@component Hello = |{ title }| {
    <p class="card">{title}</p>
}
"#;
    let out = compile_ok(src);
    let tag = ident_offset(src, "p", "<p class");
    assert!(
        source_to_generated(src, &out.roc, &out.segments, tag).is_none(),
        "markup should not map as Roc"
    );
    let class = ident_offset(src, "card", "class=\"card\"");
    assert!(source_to_generated(src, &out.roc, &out.segments, class).is_none());
}

#[test]
fn wrap_indent_preserves_interpolation_map() {
    let src = r#"
@component Hello = |{ title }| {
    <p>{title}</p>
}
"#;
    let out = compile_ok(src);
    let title = ident_offset(src, "title", "{title}");
    let projection = project_type_module(&out.roc, &out.segments, "Hello");
    assert_eq!(projection.roc, wrap_type_module(&out.roc, "Hello"));
    let mapped =
        source_to_generated(src, &projection.roc, &projection.segments, title).expect("projected");
    assert_eq!(
        &projection.roc[mapped.offset as usize..mapped.offset as usize + 5],
        "title"
    );
    assert!(!projection.roc[mapped.offset as usize..].starts_with("Html"));
}
