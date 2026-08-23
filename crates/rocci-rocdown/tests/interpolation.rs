use rocci_rocdown::{CompileOptions, Item, MdNode, SourceFile, compile, format_ast};

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

fn paragraph_children<'a>(out: &'a rocci_rocdown::CompileOutput) -> &'a [MdNode] {
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
