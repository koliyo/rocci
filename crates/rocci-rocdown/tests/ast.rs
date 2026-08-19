use rocci_rocdown::{
    CompileOptions, Diagnostic, OriginKind, SourceFile, collect_local_media, compile, format_ast,
    normalize_local_asset_url, parse,
};

#[test]
fn test_all_syntax_ast() {
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
fn colon_note_and_img_are_block_calls() {
    let src = "\
:note[title: \"Watch\"] {{
    Nested.

    :tip Inner.
}}

:img[src: \"./x.png\", alt: \"x\"]
";
    let parsed = parse(SourceFile::new("test.rocdown", src), false);
    assert!(
        !parsed.diagnostics.iter().any(Diagnostic::is_error),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(block note"), "{ast}");
    assert!(ast.contains("(block img"), "{ast}");
    assert!(
        !ast.contains("(docs "),
        "inspect should show block tags, not docs: {ast}"
    );
    let names: Vec<_> = parsed
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            rocci_rocdown::Item::Block(call) => Some(call.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["note", "img"]);
    let note = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "note" => Some(call),
            _ => None,
        })
        .unwrap();
    assert!(note.params.is_some());
    let nested = rocci_rocdown::parse_fragment(
        SourceFile::new("test.rocdown", src),
        note.content_span().expect("note content"),
        false,
    );
    assert!(nested.document.items.iter().any(|item| matches!(
        item,
        rocci_rocdown::Item::Block(call) if call.name == "tip"
    )));
}

fn ungram_productions(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let Some((name, _)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
            && name.chars().all(|ch| ch.is_ascii_alphanumeric())
        {
            names.push(name.to_string());
        }
    }
    names
}

fn toml_table_keys(src: &str, heading: &str) -> Vec<String> {
    let header = format!("[{heading}]");
    let mut keys = Vec::new();
    let mut in_section = false;
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == header;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, _)) = trimmed.split_once('=') else {
            continue;
        };
        keys.push(key.trim().trim_matches('"').to_string());
    }
    keys
}

fn classify_item(item: &rocci_rocdown::Item) -> &'static str {
    match item {
        rocci_rocdown::Item::Markdown(_) => "markdown",
        rocci_rocdown::Item::Page(_) => "page",
        rocci_rocdown::Item::Roc(_) => "roc",
        rocci_rocdown::Item::Render(_) => "render",
        rocci_rocdown::Item::Component(_) => "component",
        rocci_rocdown::Item::Fixture(_) => "fixture",
        rocci_rocdown::Item::Css(_) => "css",
        rocci_rocdown::Item::Context(_) => "context",
        rocci_rocdown::Item::Init(_) => "init",
        rocci_rocdown::Item::On(_) => "on",
        rocci_rocdown::Item::Use(_) => "use",
        rocci_rocdown::Item::Template(_) => "template",
        rocci_rocdown::Item::Block(_) => "block",
    }
}

#[test]
fn ungram_productions_are_classified_in_sidecar() {
    let ungram = include_str!("../Rocdown.AST.ungram");
    let sidecar = include_str!("../Rocdown.AST.toml");
    let productions = ungram_productions(ungram);
    let mut classified = Vec::new();
    for section in [
        "generated",
        "foreign",
        "opaque",
        "doc_only",
        "inline",
        "leaves",
    ] {
        classified.extend(toml_table_keys(sidecar, section));
    }
    for name in &productions {
        assert!(
            classified.iter().any(|key| key == name),
            "unclassified Rocdown production {name}"
        );
    }
    for key in &classified {
        assert!(
            productions.iter().any(|name| name == key),
            "sidecar key {key} is not a Rocdown ungram production"
        );
    }
}

#[test]
fn item_enum_has_no_paragraph_variant() {
    let _ = classify_item as fn(&rocci_rocdown::Item) -> &'static str;
}

#[test]
fn test_all_syntax_source_map_segments() {
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
fn test_all_syntax_routes_and_media() {
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
