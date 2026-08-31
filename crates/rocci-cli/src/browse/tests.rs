use super::*;

fn temp_root(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("rocci-browse-test-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn compile_src(src: &str) -> rocci_template::CompileOutput {
    compile(SourceFile::new("test.rocci", src), &LowerOptions::default())
}

fn entry_for(src: &str, name: &str) -> CatalogEntry {
    let out = compile_src(src);
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let info = out
        .components
        .iter()
        .find(|component| component.name == name)
        .unwrap();
    catalog_entry(src, "Demo", "Demo.rocci", info, &out.document)
}

#[test]
fn discover_is_recursive_and_skips_target() {
    let dir = temp_root("discover");
    fs::write(dir.join("Top.rocci"), "").unwrap();
    let nested = dir.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("Other.rocci"), "").unwrap();
    let target = dir.join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("Skip.rocci"), "").unwrap();
    let git = dir.join(".git");
    fs::create_dir(&git).unwrap();
    fs::write(git.join("Hidden.rocci"), "").unwrap();
    fs::write(dir.join("notes.txt"), "").unwrap();

    let found = discover_rocci_files(std::slice::from_ref(&dir)).unwrap();
    assert_eq!(
        found,
        vec![dir.join("Top.rocci"), nested.join("Other.rocci")]
    );
    cleanup(&dir);
}

#[test]
fn discover_file_root_and_rejects_duplicates() {
    let dir = temp_root("dup");
    let a = dir.join("a");
    let b = dir.join("b");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    fs::write(a.join("Foo.rocci"), "").unwrap();
    fs::write(b.join("Foo.rocci"), "").unwrap();
    let err = discover_rocci_files(&[a.clone(), b.clone()])
        .unwrap_err()
        .to_string();
    assert!(err.contains("duplicate module name `Foo`"), "{err}");

    let only = discover_rocci_files(&[a.join("Foo.rocci")]).unwrap();
    assert_eq!(only, vec![a.join("Foo.rocci")]);
    cleanup(&dir);
}

#[test]
fn infers_annotation_default_body_and_usage() {
    let src = r#"
@component Hello = |{ name ?? "Roc" }| {
    <p>{name}</p>
}
@component Typed = |{ count: I64 }| {
    <p>{count.to_str()}</p>
}
@component Badge = |{ tone : [Neutral] ?? Neutral }, content| {
    <span>{content}</span>
}
@component Card = |{ count }| {
    <output>{count.to_str()}</output>
}
@component Flag = |{ full }| {
    @if full {
        <p>full</p>
    }
}
@component Items = |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
@component Contact = |{ contact }| {
    <p>{contact.first}</p>
}
@component Title = |{ title }| {
    <h1>{title}</h1>
}
"#;
    let hello = entry_for(src, "hello");
    assert!(hello.previewable);
    assert_eq!(hello.params[0].kind, Some(ParamKind::Str));
    assert_eq!(hello.params[0].default_display, "Roc");
    assert!(!hello.params[0].required);

    let typed = entry_for(src, "typed");
    assert!(typed.previewable);
    assert_eq!(typed.params[0].kind, Some(ParamKind::I64));

    let badge = entry_for(src, "badge");
    assert!(badge.previewable);
    assert!(badge.params[0].kind.is_none());
    assert_eq!(badge.params[1].kind, Some(ParamKind::BodyHtml));
    assert!(badge.params[1].is_body);

    let card = entry_for(src, "card");
    assert!(card.previewable);
    assert_eq!(card.params[0].kind, Some(ParamKind::I64));

    let flag = entry_for(src, "flag");
    assert!(flag.previewable);
    assert_eq!(flag.params[0].kind, Some(ParamKind::Bool));

    let items = entry_for(src, "items");
    assert!(!items.previewable);
    assert!(items.reason.contains("list"));

    let contact = entry_for(src, "contact");
    assert!(!contact.previewable);
    assert!(contact.reason.contains("record"));

    let title = entry_for(src, "title");
    assert!(title.previewable);
    assert_eq!(title.params[0].kind, Some(ParamKind::Str));
    assert!(title.params[0].required);
}

#[test]
fn passthrough_inherits_sibling_param_kind() {
    let src = r#"
@component Card = |{ count }| {
    <output>{count.to_str()}</output>
}
@component Page = |{ count }| {
    <html><body><Card count={count} /></body></html>
}
"#;
    let out = compile_src(src);
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let mut entries: Vec<_> = out
        .components
        .iter()
        .map(|info| catalog_entry(src, "Demo", "Demo.rocci", info, &out.document))
        .collect();
    propagate_passthrough(src, &out.document, &mut entries);
    let page = entries.iter().find(|entry| entry.name == "page").unwrap();
    assert!(page.previewable, "{}", page.reason);
    assert_eq!(page.params[0].kind, Some(ParamKind::I64));
    assert!(page.full_document);
}

#[test]
fn catalog_and_preview_generation() {
    let src = r#"
@component Hello = |{ name ?? "Roc" }| {
    <p>{name}</p>
}
@component Items = |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
"#;
    let hello = entry_for(src, "hello");
    let items = entry_for(src, "items");
    let groups = vec![ModuleGroup {
        module: "Demo".into(),
        file: "Demo.rocci".into(),
        import_ok: true,
        entries: vec![hello, items],
    }];
    let catalog = generate_catalog_roc(&groups);
    assert!(catalog.contains("Demo.hello"));
    assert!(catalog.contains("Demo.items"));
    assert!(catalog.contains("previewable: True"));
    assert!(catalog.contains("previewable: False"));
    assert!(catalog.contains("kind: \"str\""));

    let preview = generate_preview_roc(&groups);
    assert!(preview.contains("import Demo"));
    assert!(preview.contains("Demo.hello({ name: Query.arg_str(args, \"name\") ?? \"Roc\" })"));
    assert!(!preview.contains("Demo.items("));
    assert!(preview.contains("shell("));
}

#[test]
fn fixtures_make_list_components_previewable_and_fill_scalars() {
    let src = r#"
@component Hello = |{ name }| {
    <p>{name}</p>
}
@fixture{target: Hello}
helloTest = { name: "Ada" }

@component Items = |{ items }| {
    @for item in items {
        <li>{item}</li>
    }
}
@fixture{target: Items}
itemsTest = { items: ["milk", "eggs"] }
"#;
    let groups = groups_with_fixtures(src);
    let hello = groups[0]
        .entries
        .iter()
        .find(|entry| entry.name == "hello")
        .unwrap();
    assert!(hello.previewable);
    assert_eq!(hello.fixtures.len(), 1);
    assert_eq!(hello.fixtures[0].name, "helloTest");
    assert_eq!(
        hello.fixtures[0].scalars,
        vec![("name".into(), "Ada".into())]
    );

    let items = groups[0]
        .entries
        .iter()
        .find(|entry| entry.name == "items")
        .unwrap();
    assert!(items.previewable, "{}", items.reason);
    assert_eq!(items.fixtures[0].name, "itemsTest");
    assert!(items.fixtures[0].scalars.is_empty());

    let catalog = generate_catalog_roc(&groups);
    assert!(catalog.contains("helloTest"));
    assert!(catalog.contains("value: \"Ada\""));
    assert!(catalog.contains("itemsTest"));

    let preview = generate_preview_roc(&groups);
    assert!(
        preview
            .contains("Demo.hello({ name: Query.arg_str(args, \"name\") ?? Demo.helloTest.name })")
    );
    assert!(preview.contains("Demo.items(Demo.itemsTest)"));
    assert!(preview.contains("\"helloTest\" =>"));
}

#[test]
fn fixture_numeric_overlays_use_typed_literals() {
    let src = r#"
@component Card = |{ count }| {
    <output>{count.to_str()}</output>
}
@fixture{target: Card}
cardTest = { count: 3 }
"#;
    let groups = groups_with_fixtures(src);
    let preview = generate_preview_roc(&groups);
    assert!(preview.contains("Demo.card({ count: Query.arg_i64(args, \"count\") ?? 3.I64 })"));
}

fn groups_with_fixtures(src: &str) -> Vec<ModuleGroup> {
    let out = compile_src(src);
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let module = CompiledModule {
        path: PathBuf::from("Demo.rocci"),
        type_name: "Demo".into(),
        roc: out.roc,
        document: out.document,
        components: out.components,
        fixtures: out.fixtures,
        src: src.to_string(),
    };
    let available = HashSet::from(["Html".into(), "Demo".into()]);
    let mut groups = analyze_modules(std::slice::from_ref(&module), &available);
    attach_fixtures(std::slice::from_ref(&module), &mut groups);
    groups
}

#[test]
fn preview_omits_modules_with_missing_imports() {
    let src = r#"
@component Hello = |{}| {
    <p>ok</p>
}
"#;
    let hello = entry_for(src, "hello");
    let groups = vec![ModuleGroup {
        module: "Demo".into(),
        file: "Demo.rocci".into(),
        import_ok: false,
        entries: vec![hello],
    }];
    let preview = generate_preview_roc(&groups);
    assert!(!preview.contains("import Demo"));
    assert!(!preview.contains("Demo.hello"));
}

#[test]
fn query_decode_helpers_match_form_values() {
    assert_eq!(display_roc_literal("\"Roc\""), "Roc");
    assert_eq!(display_roc_literal("Bool.true"), "true");
    assert_eq!(display_roc_literal("0"), "0");
    assert_eq!(ParamKind::from_annotation("I64"), Some(ParamKind::I64));
    assert_eq!(ParamKind::from_annotation("List(Item)"), None);
    assert_eq!(ParamKind::from_annotation("{ first: Str }"), None);
    assert_eq!(
        infer_from_default("\"Ada\""),
        Inferred::Scalar(ParamKind::Str)
    );
    assert_eq!(infer_from_default("12"), Inferred::Scalar(ParamKind::I64));
    assert_eq!(
        infer_from_default("Bool.false"),
        Inferred::Scalar(ParamKind::Bool)
    );
    assert!(matches!(
        infer_from_default("Neutral"),
        Inferred::Unsupported(_)
    ));
    assert_eq!(
        fixture_scalars("{ name: \"Ada\", contacts: all_contacts, count: 3, full: True }"),
        vec![
            ("name".into(), "Ada".into()),
            ("count".into(), "3".into()),
            ("full".into(), "true".into()),
        ]
    );
}
