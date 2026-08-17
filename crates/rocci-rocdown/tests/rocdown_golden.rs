use rocci_rocdown::{
    CompileOptions, Diagnostic, OriginKind, SourceFile, collect_local_media, compile, format_ast,
    normalize_local_asset_url, parse,
};

#[test]
fn golden_all_syntax_ast() {
    let src = include_str!("../../../test/AllSyntax.rocdown");
    let source = SourceFile::new("test/AllSyntax.rocdown", src);
    let parsed = parse(source, false);
    assert!(
        !parsed.diagnostics.iter().any(Diagnostic::is_error),
        "{:?}",
        parsed.diagnostics
    );

    let ast = format_ast(src, &parsed.document);
    let expected = include_str!("fixtures/all_syntax.ast");
    assert_eq!(ast.trim(), expected.trim(), "AST S-expression mismatch");
}

#[test]
fn golden_all_syntax_source_map_segments() {
    let src = include_str!("../../../test/AllSyntax.rocdown");
    let source = SourceFile::new("test/AllSyntax.rocdown", src);
    let out = compile(source, &CompileOptions::default());
    assert!(!out.has_errors(), "{:?}", out.diagnostics);

    // Verify origin kinds are present
    assert!(
        out.segments
            .iter()
            .any(|s| s.origin == OriginKind::MarkdownText),
        "missing MarkdownText segment"
    );
    assert!(
        out.segments
            .iter()
            .any(|s| s.origin == OriginKind::MarkdownStructure),
        "missing MarkdownStructure segment"
    );
    assert!(
        out.segments
            .iter()
            .any(|s| s.origin == OriginKind::RocBlock),
        "missing RocBlock segment"
    );
    assert!(
        out.segments
            .iter()
            .any(|s| s.origin == OriginKind::RenderRoc),
        "missing RenderRoc segment"
    );

    // Verify all segments have valid generated and source spans
    for (i, seg) in out.segments.iter().enumerate() {
        assert!(
            seg.generated.end as usize <= out.roc.len(),
            "segment {i} generated span {:?} exceeds generated length {}",
            seg.generated,
            out.roc.len()
        );
        assert!(
            seg.source.end as usize <= src.len(),
            "segment {i} source span {:?} exceeds source length {}",
            seg.source,
            src.len()
        );
    }
}

#[test]
fn golden_all_syntax_routes_and_media() {
    let src = include_str!("../../../test/AllSyntax.rocdown");
    let source = SourceFile::new("test/AllSyntax.rocdown", src);
    let out = compile(source, &CompileOptions::default());
    assert!(!out.has_errors(), "{:?}", out.diagnostics);

    // Verify page route
    assert_eq!(out.page_meta.route.as_deref(), Some("/all-syntax/"));
    assert_eq!(out.page_meta.title.as_deref(), Some("All syntax"));
    assert!(!out.page_meta.draft);

    // Verify routes
    assert!(
        out.routes
            .iter()
            .any(|r| r.method == "GET" && r.path == "/all-syntax/"),
        "missing GET /all-syntax/ route"
    );

    // Verify local media discovery
    let media = collect_local_media(source, &out.document);
    assert_eq!(media.len(), 1);
    assert_eq!(media[0].0, "./img/yammi_banana.png");
    let normalized = normalize_local_asset_url(&media[0].0);
    assert_eq!(normalized.as_deref(), Some("img/yammi_banana.png"));
}
