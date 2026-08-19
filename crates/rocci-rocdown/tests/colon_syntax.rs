use rocci_rocdown::{
    BlockContent, CompileOptions, Diagnostic, SourceFile, compile, format_ast, highlight_rocdown,
    parse,
};

fn parse_src(src: &str) -> rocci_rocdown::ParseOutput {
    parse(SourceFile::new("test.rocdown", src), false)
}

fn error_messages(parsed: &rocci_rocdown::ParseOutput) -> Vec<String> {
    parsed
        .diagnostics
        .iter()
        .filter(|d| Diagnostic::is_error(d))
        .map(|d| d.message.clone())
        .collect()
}

fn block_names(parsed: &rocci_rocdown::ParseOutput) -> Vec<&str> {
    parsed
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            rocci_rocdown::Item::Block(call) => Some(call.name.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn syntax_v2_fixture_parses_colon_blocks() {
    let src = include_str!("../../../test/syntax-v2.rocdown");
    let parsed = parse(SourceFile::new("test/syntax-v2.rocdown", src), false);
    let errs = error_messages(&parsed);
    assert!(errs.is_empty(), "{errs:?}");
    let ast = format_ast(src, &parsed.document);
    for needle in [
        "(block note)",
        "(block h2)",
        "(block caution)",
        "(block steps)",
        "(block tabs)",
        "(block figure)",
        "(block badge)",
    ] {
        assert!(ast.contains(needle), "missing {needle} in {ast}");
    }
    assert!(
        !ast.contains("(docs "),
        "colon syntax should inspect as block, not docs: {ast}"
    );
}

#[test]
fn docs_note_still_parses_beside_colon_note() {
    let src = "\
@docs note {
    title: \"Legacy\"

    Still works.
}

:note Also a note.
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    assert_eq!(block_names(&parsed), ["note", "note"]);
}

#[test]
fn line_scope_does_not_eat_the_following_paragraph() {
    let src = "\
:note This is the note.

This is a paragraph.
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(block note)"), "{ast}");
    assert!(ast.contains("(p"), "{ast}");
    assert!(ast.contains("This is a paragraph"), "{ast}");
    let note = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "note" => Some(call),
            _ => None,
        })
        .unwrap();
    assert!(matches!(note.content, Some(BlockContent::Line(_))));
    assert_eq!(note.content_span().unwrap().of(src), "This is the note.");
}

#[test]
fn space_after_colon_and_mid_paragraph_stay_markdown() {
    let src = "\
: definition stays Markdown.

A sentence with :note in the middle.
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    assert!(block_names(&parsed).is_empty());
    let ast = format_ast(src, &parsed.document);
    assert!(!ast.contains("(block "), "{ast}");
}

#[test]
fn fenced_braces_inside_section_are_opaque() {
    let src = "\
:note {{
    A stray brace { and closer } in prose.

    ```roc
    pair = { a: 1, b: 2 }
    ```
}}
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    assert_eq!(block_names(&parsed), ["note"]);
    let note = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "note" => Some(call),
            _ => None,
        })
        .unwrap();
    let body = note.content_span().unwrap().of(src);
    assert!(body.contains("pair = { a: 1, b: 2 }"), "{body}");
}

#[test]
fn nested_tabs_with_named_closer() {
    let src = "\
:tabs[group: \"os\", kind: \"tool\"]
    :tab[id: \"mac\", label: \"macOS\"] Hello mac.
    :tab[id: \"win\", label: \"Windows\"] Hello win.
:end.tabs
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    assert_eq!(block_names(&parsed), ["tabs"]);
    let tabs = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "tabs" => Some(call),
            _ => None,
        })
        .unwrap();
    assert!(matches!(tabs.content, Some(BlockContent::End(_))));
    let nested = rocci_rocdown::parse_fragment(
        SourceFile::new("test.rocdown", src),
        tabs.content_span().unwrap(),
        false,
    );
    let names: Vec<_> = nested
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            rocci_rocdown::Item::Block(call) => Some(call.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["tab", "tab"]);
}

#[test]
fn unclosed_section_is_an_error() {
    let src = ":note {{\n    still open\n";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter().any(|msg| msg.contains("unterminated `{{`")),
        "{errs:?}"
    );
}

#[test]
fn mismatched_end_kind_is_an_error() {
    let src = "\
:tabs[group: \"os\", kind: \"tool\"]
    :tab[id: \"mac\", label: \"macOS\"] Hello.
:end.foo
";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains(":end.foo") || msg.contains("unmatched")),
        "{errs:?}"
    );
}

#[test]
fn end_inside_fence_is_not_a_closer() {
    let src = "\
:note {{
    ```text
    :end.note
    ```
}}
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    assert_eq!(block_names(&parsed), ["note"]);
    let note = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "note" => Some(call),
            _ => None,
        })
        .unwrap();
    assert!(note.content_span().unwrap().of(src).contains(":end.note"));
}

#[test]
fn bare_end_is_an_error_and_not_a_block() {
    let src = ":end\n";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:end` requires a kind")),
        "{errs:?}"
    );
    assert!(block_names(&parsed).is_empty());
}

#[test]
fn unknown_kind_and_module_collision_are_diagnostics() {
    let src = "\
:widget Hello.

:page Not a page declaration.
";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("unknown article kind `:widget`")),
        "{errs:?}"
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:page` collides with a reserved module name")),
        "{errs:?}"
    );
}

#[test]
fn parent_child_and_required_fields_use_the_registry() {
    let src = "\
:tab[id: \"a\", label: \"A\"] Orphan tab.

:tabs Hello without params.
";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:tab` is only valid inside `:tabs`")),
        "{errs:?}"
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:tabs` requires `group`")),
        "{errs:?}"
    );
}

#[test]
fn colon_note_does_not_crash_highlight_or_compile() {
    let src = ":note Hello from colon syntax.\n";
    let _ = highlight_rocdown(src);
    let out = compile(
        SourceFile::new("test.rocdown", src),
        &CompileOptions::default(),
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
}

#[test]
fn img_bracket_params_parse() {
    let src = ":img[src: \"./x.png\", alt: \"x\"]\n";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let img = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "img" => Some(call),
            _ => None,
        })
        .unwrap();
    let params = img.params.as_ref().expect("img params");
    let names: Vec<_> = params
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    assert_eq!(names, ["src", "alt"]);
}
