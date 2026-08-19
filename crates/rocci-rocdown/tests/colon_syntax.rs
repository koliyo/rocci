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
        "(block note",
        "(block h2",
        "(block caution",
        "(block steps",
        "(block tabs",
        "(block figure",
        "(block badge",
    ] {
        assert!(ast.contains(needle), "missing {needle} in {ast}");
    }
    assert!(
        !ast.contains("(docs "),
        "colon syntax should inspect as block, not docs: {ast}"
    );
}

#[test]
fn leftover_docs_and_img_are_removal_errors() {
    let src = "\
@docs note {
    title: \"Legacy\"

    Still works.
}

@img {
    src: \"x.png\"
    alt: \"x\"
}

:note Also a note.
";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`@docs` was removed") && msg.contains(":note")),
        "{errs:?}"
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`@img` was removed") && msg.contains(":img[")),
        "{errs:?}"
    );
    assert_eq!(block_names(&parsed), ["note"]);
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
    assert!(ast.contains("(block note line"), "{ast}");
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
:tabs.begin[group: \"os\", kind: \"tool\"]
    :tab[id: \"mac\", label: \"macOS\"] Hello mac.
    :tab[id: \"win\", label: \"Windows\"] Hello win.
:tabs.end
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
:tabs.begin[group: \"os\", kind: \"tool\"]
    :tab[id: \"mac\", label: \"macOS\"] Hello.
:foo.end
";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("unclosed `:tabs.begin`")
                || msg.contains("unmatched `:foo.end`")),
        "{errs:?}"
    );
}

#[test]
fn end_inside_fence_is_not_a_closer() {
    let src = "\
:note {{
    ```text
    :note.end
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
    assert!(note.content_span().unwrap().of(src).contains(":note.end"));
}

#[test]
fn leftover_end_dot_kind_is_a_removal_error() {
    let src = ":end.tabs\n";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:end.tabs` was removed") && msg.contains("`:tabs.end`")),
        "{errs:?}"
    );
    assert!(block_names(&parsed).is_empty());
}

#[test]
fn leftover_bare_end_is_a_removal_error() {
    let src = ":end\n";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:end` was removed") && msg.contains("`:kind.end`")),
        "{errs:?}"
    );
    assert!(block_names(&parsed).is_empty());
}

#[test]
fn begin_cannot_mix_with_double_braces() {
    let src = "\
:tabs.begin[group: \"os\", kind: \"tool\"] {{
    :tab[id: \"mac\", label: \"macOS\"] Hello.
}}
";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("cannot mix with a double-brace body")),
        "{errs:?}"
    );
}

#[test]
fn brace_form_does_not_use_named_closer() {
    let src = "\
:tabs[group: \"os\", kind: \"tool\"] {{
    :tab[id: \"mac\", label: \"macOS\"] Hello.
}}
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let tabs = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "tabs" => Some(call),
            _ => None,
        })
        .unwrap();
    assert!(matches!(tabs.content, Some(BlockContent::Brace(_))));
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
fn tabs_reject_markdown_and_non_tab_children() {
    let stray = "\
:tabs.begin[group: \"os\", kind: \"platform\"]
    A stray paragraph.

    :tab[id: \"mac\", label: \"macOS\"] Hello.
:tabs.end
";
    let parsed = parse_src(stray);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:tabs` cannot contain Markdown")),
        "{errs:?}"
    );

    let nested = "\
:tabs.begin[group: \"os\", kind: \"platform\"]
    :note Nested aside.
    :tab[id: \"mac\", label: \"macOS\"] Hello.
:tabs.end
";
    let parsed = parse_src(nested);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:tabs` cannot contain `:note`")),
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

#[test]
fn atx_heading_inspects_as_block_h2() {
    let src = "# Title\n\n## Install\n";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(block h1 line"), "{ast}");
    assert!(ast.contains("(block h2 line"), "{ast}");
    assert!(!ast.contains("(h 1"), "{ast}");
    assert!(!ast.contains("(h 2"), "{ast}");
    let ids: Vec<_> = parsed.headings.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(ids, ["title", "install"]);
}

#[test]
fn explicit_heading_id_wins_over_slug() {
    let src = "\
## Install

:h2[id: \"from-source\"] Building from source
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(block h2"), "{ast}");
    let ids: Vec<_> = parsed.headings.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(ids, ["install", "from-source"]);
    let texts: Vec<_> = parsed.headings.iter().map(|h| h.text.as_str()).collect();
    assert_eq!(texts, ["Install", "Building from source"]);
}

#[test]
fn block_level_markdown_image_becomes_img_block() {
    let src = "See ![inline](a.png) here.\n\n![Hero](hero.png)\n";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(block img"), "{ast}");
    assert!(ast.contains("(p"), "{ast}");
    let imgs: Vec<_> = parsed
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            rocci_rocdown::Item::Block(call) if call.name == "img" => Some(call),
            _ => None,
        })
        .collect();
    assert_eq!(imgs.len(), 1);
    let src_field = imgs[0]
        .params
        .as_ref()
        .unwrap()
        .fields
        .iter()
        .find(|field| field.name == "src")
        .unwrap();
    match &src_field.value {
        rocci_rocdown::ParamValue::StringLit { value, .. } => assert_eq!(value, "hero.png"),
        other => panic!("expected string src, got {other:?}"),
    }
}

#[test]
fn format_ast_shows_params_and_content_scope() {
    let src = "\
:note Don't do this.

:note[title: \"Watch\"] {{
Nested section body.
}}

:tabs.begin[group: \"os\", kind: \"platform\"]
    :tab[id: \"mac\", label: \"macOS\"] Mac panel.
    :tab[id: \"linux\", label: \"Linux\"] Linux panel.
:tabs.end

:img[src: \"./x.png\", alt: \"x\"]
";
    let parsed = parse_src(src);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(block note line"), "{ast}");
    assert!(ast.contains("(block note section title Watch"), "{ast}");
    assert!(
        ast.contains("(block tabs end group os kind platform"),
        "{ast}"
    );
    assert!(ast.contains("(block tab line id mac label macOS"), "{ast}");
    assert!(ast.contains("(block img src ./x.png alt x)"), "{ast}");
}

#[test]
fn end_marker_is_highlighted() {
    let src = "\
:tabs.begin[group: \"os\", kind: \"platform\"]
    :tab[id: \"a\", label: \"A\"] A.
:tabs.end
";
    let spans = highlight_rocdown(src);
    let closer: Vec<_> = spans
        .iter()
        .map(|span| (span.span.of(src), span.kind))
        .collect();
    assert!(
        closer.iter().any(|(text, kind)| {
            *text == ":tabs.end" && *kind == rocci_highlight::HighlightKind::Keyword
        }),
        "{closer:?}"
    );
}

fn temp_use_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rocdown-use-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_callout(dir: &std::path::Path) {
    std::fs::write(
        dir.join("Callout.rocci"),
        include_str!("../../../test/Callout.rocci"),
    )
    .unwrap();
}

#[test]
fn use_maps_exported_component_to_article_kind() {
    let dir = temp_use_dir("ok");
    write_callout(&dir);
    let path = dir.join("Page.rocdown");
    let src = "@use \"./Callout.rocci\"\n\n:callout[tone: \"warn\"] Be careful.\n";
    std::fs::write(&path, src).unwrap();
    let parsed = parse(SourceFile::new(&path.to_string_lossy(), src), false);
    assert!(
        error_messages(&parsed).is_empty(),
        "{:?}",
        parsed.diagnostics
    );
    let ast = format_ast(src, &parsed.document);
    assert!(ast.contains("(use ./Callout.rocci)"), "{ast}");
    assert!(ast.contains("(block callout"), "{ast}");
    let out = compile(
        SourceFile::new(&path.to_string_lossy(), src),
        &CompileOptions::default(),
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(
        out.roc.contains("callout("),
        "imported kind should call the component:\n{}",
        out.roc
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unknown_callout_without_use_remains_an_error() {
    let src = ":callout[tone: \"warn\"] Be careful.\n";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("unknown article kind `:callout`")),
        "{errs:?}"
    );
}

#[test]
fn use_missing_file_is_a_diagnostic() {
    let dir = temp_use_dir("missing");
    let path = dir.join("Page.rocdown");
    let src = "@use \"./Missing.rocci\"\n\n:callout Hi.\n";
    std::fs::write(&path, src).unwrap();
    let parsed = parse(SourceFile::new(&path.to_string_lossy(), src), false);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter().any(|msg| msg.contains("does not exist")),
        "{errs:?}"
    );
    assert!(
        errs.iter()
            .any(|msg| msg.contains("unknown article kind `:callout`")),
        "{errs:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn use_without_component_export_is_a_diagnostic() {
    let dir = temp_use_dir("css-only");
    std::fs::write(
        dir.join("Empty.rocci"),
        "import Html\n\n@css { .x { color: red; } }\n",
    )
    .unwrap();
    let path = dir.join("Page.rocdown");
    let src = "@use \"./Empty.rocci\"\n\n:callout Hi.\n";
    std::fs::write(&path, src).unwrap();
    let parsed = parse(SourceFile::new(&path.to_string_lossy(), src), false);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("does not export an `@component`")),
        "{errs:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn use_is_a_reserved_module_name() {
    let src = ":use Hello.\n";
    let parsed = parse_src(src);
    let errs = error_messages(&parsed);
    assert!(
        errs.iter()
            .any(|msg| msg.contains("`:use` collides with a reserved module name")),
        "{errs:?}"
    );
}
