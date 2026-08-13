use rocci_template::{
    LowerOptions, OriginKind, SourceFile, Span, compile, format_ast, parse_component_params,
    strip_param_defaults,
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

fn compile_err(src: &str) -> Vec<String> {
    let out = compile(SourceFile::new("test.rocci", src), &LowerOptions::default());
    out.diagnostics
        .into_iter()
        .filter(|d| d.is_error())
        .map(|d| d.message)
        .collect()
}

#[test]
fn kitchen_sink_compiles_without_errors() {
    let src = include_str!("fixtures/kitchen_sink.rocci");
    let out = compile_ok(src);
    assert!(out.components.len() >= 5);
    assert!(out.components.iter().any(|c| c.name == "badge"
        && c.body_params == ["content"]
        && c.param_names == ["tone", "content"]
        && c.optional_params == ["tone"]
        && c.first_param_is_record));
    assert!(
        out.components.iter().any(|c| c.name == "hello"
            && c.param_names == ["name"]
            && c.optional_params == ["name"])
    );
    assert!(out.roc.contains("hello = |{ name }|"));
    assert!(!out.roc.contains("= component"));
    assert!(!out.roc.contains("name ??"));
    assert_eq!(out.roc, include_str!("fixtures/kitchen_sink.roc"));
}

#[test]
fn lowers_component_call_to_props_record() {
    let src = include_str!("fixtures/kitchen_sink.rocci");
    let out = compile_ok(src);
    assert!(out.roc.contains("hello(\n"));
    assert!(out.roc.contains("{ name: person.name }"));
    assert!(out.roc.contains("{ name: \"World\" }"));
}

#[test]
fn lowers_body_argument_and_html_child() {
    let src = include_str!("fixtures/kitchen_sink.rocci");
    let out = compile_ok(src);
    assert!(out.roc.contains("badge = |{ tone }, content|"));
    assert!(
        out.roc.contains("content,")
            || out.roc.contains("content\n")
            || out.roc.contains("[content]")
    );
    assert!(out.roc.contains("badge(\n"));
    assert!(out.roc.contains("{ tone: Positive }"));
    assert!(out.roc.contains("Html.text(\"Current count\")"));
}

#[test]
fn lowers_if_for_and_match() {
    let src = include_str!("fixtures/kitchen_sink.rocci");
    let out = compile_ok(src);
    assert!(out.roc.contains("match state {"));
    assert!(out.roc.contains("if List.isEmpty(items) {"));
    assert!(out.roc.contains("List.map(items, |item|"));
    assert!(out.roc.contains("} else if user.canRegister {"));
    assert!(out.roc.contains("Html.empty") || out.roc.contains("loginButton"));
}

#[test]
fn lowers_let_qualified_import_and_fragment() {
    let src = include_str!("fixtures/kitchen_sink.rocci");
    let out = compile_ok(src);
    assert!(
        out.roc
            .contains("visible = List.keepIf(items, |item| matches(item, query))")
    );
    assert!(out.roc.contains("Design.button("));
    assert!(out.roc.contains("Html.fragment("));
}

#[test]
fn preserves_roc_regions_and_parenthesized_header_records() {
    let src = include_str!("fixtures/kitchen_sink.rocci");
    let out = compile_ok(src);
    assert!(out.roc.contains("isVisible({ user, permissions })"));
    assert!(out.roc.contains("match ({ status, items }) {"));
    assert!(
        out.roc
            .contains("if selected { \"selected\" } else { \"\" }")
    );
    assert!(out.roc.contains("badgeClass = |tone| {"));
}

#[test]
fn maps_expressions_back_to_source() {
    let src = r#"
hello = component |{ name }| {
    <p>Hello, {name}</p>
}
"#;
    let out = compile_ok(src);
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::TextExpression
                && src[seg.source.as_range()].trim() == "name")
    );
    assert!(
        out.segments
            .iter()
            .any(|seg| seg.origin == OriginKind::OrdinaryRoc
                || seg.origin == OriginKind::ComponentSignature)
    );
}

#[test]
fn rejects_unparenthesized_header_record() {
    let src = r#"
view = component |{ state }| {
    @match { status, items } {
        _ => <Spinner />
    }
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("unparenthesized record")),
        "{errors:?}"
    );
}

#[test]
fn recovers_from_incomplete_tag_before_next_definition() {
    let src = r#"
broken = component |{}| {
    <Hello name={person.name}
}

ok = component |{ name }| {
    <p>{name}</p>
}
"#;
    let out = compile(SourceFile::new("test.rocci", src), &LowerOptions::default());
    assert!(out.has_errors());
    assert!(out.components.iter().any(|c| c.name == "ok"));
    assert!(out.roc.contains("ok = |{ name }|"));
}

#[test]
fn unknown_directive_suggests_if() {
    let src = r#"
view = component |{ ready }| {
    @fi ready {
        <Ready />
    }
}
"#;
    let errors = compile_err(src);
    assert!(
        errors.iter().any(|msg| msg.contains("did you mean `@if`")),
        "{errors:?}"
    );
}

#[test]
fn discards_indentation_between_tags() {
    let src = r#"
page = component |{}| {
    <div>
        <span>a</span>
        <span>b</span>
    </div>
}
"#;
    let out = compile_ok(src);
    assert!(!out.roc.contains("Html.text(\"\\n"));
    assert!(out.roc.contains("Html.text(\"a\")"));
    assert!(out.roc.contains("Html.text(\"b\")"));
}

#[test]
fn extracts_component_param_names() {
    let cases: &[(&str, bool, &[&str], &[&str], &[&str])] = &[
        ("|{ name }|", true, &["name"], &[], &[]),
        ("|{ person, count }|", true, &["person", "count"], &[], &[]),
        (
            "|{ tone }, content|",
            true,
            &["tone", "content"],
            &["content"],
            &[],
        ),
        ("|{ count: I64 }|", true, &["count"], &[], &[]),
        ("|{ name ?? \"World\" }|", true, &["name"], &[], &["name"]),
        (
            "|{ tone ?? Neutral }, content|",
            true,
            &["tone", "content"],
            &["content"],
            &["tone"],
        ),
        ("|model|", false, &["model"], &[], &[]),
        ("|{}|", true, &[], &[], &[]),
        ("|{ }|", true, &[], &[], &[]),
    ];
    for (src, is_record, names, body, optional) in cases {
        let parsed = parse_component_params(src, Span::new(0, src.len()));
        assert_eq!(parsed.first_param_is_record, *is_record, "{src}");
        assert_eq!(parsed.param_names, *names, "{src}");
        assert_eq!(parsed.body_params, *body, "{src}");
        assert_eq!(parsed.optional_params, *optional, "{src}");
    }
    let parsed = parse_component_params(
        "|{ name ?? \"World\" }|",
        Span::new(0, "|{ name ?? \"World\" }|".len()),
    );
    assert_eq!(parsed.param_defaults, [("name".into(), "\"World\"".into())]);
}

#[test]
fn strips_param_defaults_for_generated_roc() {
    assert_eq!(
        strip_param_defaults("|{ name ?? \"World\" }|"),
        "|{ name }|"
    );
    assert_eq!(
        strip_param_defaults("|{ tone ?? Neutral }, content|"),
        "|{ tone }, content|"
    );
    assert_eq!(
        strip_param_defaults("|{ person, count }|"),
        "|{ person, count }|"
    );
}

#[test]
fn compile_records_param_names_on_components() {
    let src = r#"
hello = component |{ name }| {
    <p>{name}</p>
}
badge = component |{ tone }, content| {
    <span>{content}</span>
}
typed = component |{ count: I64 }| {
    <p>{count.to_str()}</p>
}
modelView = component |model| {
    <p>ok</p>
}
empty = component |{}| {
    <p>empty</p>
}
"#;
    let out = compile_ok(src);
    let find = |name: &str| {
        out.components
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing {name}"))
    };

    let hello = find("hello");
    assert_eq!(hello.param_names, ["name"]);
    assert!(hello.optional_params.is_empty());
    assert!(hello.first_param_is_record);

    let badge = find("badge");
    assert_eq!(badge.param_names, ["tone", "content"]);
    assert_eq!(badge.body_params, ["content"]);
    assert!(badge.first_param_is_record);

    let typed = find("typed");
    assert_eq!(typed.param_names, ["count"]);
    assert!(typed.first_param_is_record);

    let model = find("modelView");
    assert_eq!(model.param_names, ["model"]);
    assert!(!model.first_param_is_record);

    let empty = find("empty");
    assert!(empty.param_names.is_empty());
    assert!(empty.first_param_is_record);
}

#[test]
fn package_has_no_runtime_dependencies() {
    let manifest = include_str!("../Cargo.toml");
    for forbidden in ["tokio", "axum", "wry", "tao", "hyper", "reqwest"] {
        assert!(
            !manifest.contains(forbidden),
            "rocci-template must not depend on {forbidden}"
        );
    }
}

#[test]
fn formats_lisp_ast() {
    let src = r#"
hello = component |{ name }| {
    <p class="greeting">Hello, {name}</p>
}
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert_eq!(
        ast,
        r#"(module
  (component hello
    (params "|{ name }|")
    (element p
      (attr class "greeting")
      (text "Hello, ")
      (interp name))))
"#
    );
}
