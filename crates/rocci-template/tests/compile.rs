use rocci_template::{
    LowerOptions, OriginKind, SourceFile, Span, StyleKind, compile, format_ast,
    parse_component_params, strip_param_defaults,
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
    assert!(!out.roc.contains("@component"));
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
fn concatenates_sibling_nodes_and_for_loops_with_two_arg_concat() {
    let src = r#"
@component picker = |{ items }| {
    <div>
        <p>pick</p>
        @for item in items {
            <span>{item}</span>
        }
    </div>
}
"#;
    let out = compile_ok(src);
    assert!(out.roc.contains("List.concat("));
    assert!(!out.roc.contains("List.concat(["));
    assert!(out.roc.contains("List.map(items, |item|"));
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
@component hello = |{ name }| {
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
@component view = |{ state }| {
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
@component broken = |{}| {
    <Hello name={person.name}
}

@component ok = |{ name }| {
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
@component view = |{ ready }| {
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
@component page = |{}| {
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
    let typed = parse_component_params("|{ count: I64 }|", Span::new(0, "|{ count: I64 }|".len()));
    assert_eq!(typed.param_types, [("count".into(), "I64".into())]);
    let typed_default = parse_component_params(
        "|{ name: Str ?? \"World\" }|",
        Span::new(0, "|{ name: Str ?? \"World\" }|".len()),
    );
    assert_eq!(typed_default.param_types, [("name".into(), "Str".into())]);
    assert_eq!(
        typed_default.param_defaults,
        [("name".into(), "\"World\"".into())]
    );
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
@component hello = |{ name }| {
    <p>{name}</p>
}
@component badge = |{ tone }, content| {
    <span>{content}</span>
}
@component typed = |{ count: I64 }| {
    <p>{count.to_str()}</p>
}
@component modelView = |model| {
    <p>ok</p>
}
@component empty = |{}| {
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
    assert_eq!(typed.param_types, [("count".into(), "I64".into())]);
    assert!(typed.first_param_is_record);

    let model = find("modelView");
    assert_eq!(model.param_names, ["model"]);
    assert!(!model.first_param_is_record);

    let empty = find("empty");
    assert!(empty.param_names.is_empty());
    assert!(empty.first_param_is_record);
}

#[test]
fn rejects_component_keyword_after_name() {
    let src = r#"
hello = @component |{ name }| {
    <p>{name}</p>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("start of the declaration")
                && msg.contains("@component name = |params|")),
        "{errors:?}"
    );
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
@component hello = |{ name }| {
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

#[test]
fn lowers_fixtures_and_records_metadata() {
    let src = r#"
all_contacts = [{ first: "Carli", last: "Stoltenberg" }]

@fixture{target: todoItem}
todoItemTest = { item: { id: 123, text: "Buy milk" } }

@fixture {target: Search.results}
searchResultTest = { contacts: all_contacts, query: "Foo" }

@component todoItem = |{ item }| {
    <li>{item.text}</li>
}
"#;
    let out = compile_ok(src);
    assert!(!out.roc.contains("@fixture"));
    assert!(
        out.roc
            .contains("todoItemTest = { item: { id: 123, text: \"Buy milk\" } }")
    );
    assert!(
        out.roc
            .contains("searchResultTest = { contacts: all_contacts, query: \"Foo\" }")
    );
    assert!(
        out.roc
            .contains("all_contacts = [{ first: \"Carli\", last: \"Stoltenberg\" }]")
    );
    assert!(out.roc.contains("todoItem = |{ item }|"));

    assert_eq!(out.fixtures.len(), 2);
    let todo = out
        .fixtures
        .iter()
        .find(|fixture| fixture.name == "todoItemTest")
        .expect("todoItemTest");
    assert_eq!(todo.target, "todoItem");
    assert_eq!(todo.value, "{ item: { id: 123, text: \"Buy milk\" } }");

    let search = out
        .fixtures
        .iter()
        .find(|fixture| fixture.name == "searchResultTest")
        .expect("searchResultTest");
    assert_eq!(search.target, "Search.results");
    assert_eq!(search.value, "{ contacts: all_contacts, query: \"Foo\" }");
}

#[test]
fn formats_fixture_ast() {
    let src = r#"
@fixture{target: todoItem}
todoItemTest = { item: { id: 123, text: "Buy milk" } }

@component todoItem = |{ item }| {
    <li>{item.text}</li>
}
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert_eq!(
        ast,
        r#"(module
  (fixture todoItemTest target:todoItem
    (roc "{ item: { id: 123, text: \"Buy milk\" } }"))
  (component todoItem
    (params "|{ item }|")
    (element li
      (interp item.text))))
"#
    );
}

#[test]
fn rejects_unknown_local_fixture_target() {
    let src = r#"
@fixture{target: missing}
sample = { name: "Ada" }

@component hello = |{ name }| {
    <p>{name}</p>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("unknown fixture target `missing`")),
        "{errors:?}"
    );
}

#[test]
fn rejects_missing_fixture_target() {
    let src = r#"
@fixture
sample = { name: "Ada" }
"#;
    let errors = compile_err(src);
    assert!(
        errors.iter().any(|msg| msg.contains("{target: ...}")),
        "{errors:?}"
    );
}

#[test]
fn rejects_unknown_fixture_attribute() {
    let src = r#"
@fixture{name: hello}
sample = { name: "Ada" }

@component hello = |{ name }| {
    <p>{name}</p>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("unknown `@fixture` attribute `name`")),
        "{errors:?}"
    );
}

#[test]
fn rejects_fixture_inside_component_body() {
    let src = r#"
@component hello = |{ name }| {
    @fixture{target: hello}
    sample = { name: "Ada" }
    <p>{name}</p>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("`@fixture` is only valid at module level")),
        "{errors:?}"
    );
}

#[test]
fn lowers_datastar_action_to_helper_call() {
    let src = r#"
@component page = |{}| {
    <button data-on:click=@post("/x")>Go</button>
}
"#;
    let out = compile_ok(src);
    assert!(out.roc.contains("import Datastar"));
    assert!(out.roc.contains("Datastar.post(\"/x\")"));
}

#[test]
fn lowers_datastar_action_interpolated_uri() {
    let src = r#"
@component row = |{ item }| {
    <button data-on:click=@delete("/todos/${item.id}")>Delete</button>
}
"#;
    let out = compile_ok(src);
    assert!(out.roc.contains("Datastar.delete(\"/todos/${item.id}\")"));
}

#[test]
fn lowers_datastar_action_options_record() {
    let src = r#"
@component page = |{}| {
    <body data-init=@get("/sse", [OpenWhenHidden(Bool.true)])></body>
}
"#;
    let out = compile_ok(src);
    assert!(
        out.roc
            .contains("Datastar.get_with(\"/sse\", [OpenWhenHidden(Bool.true)])")
    );
}

#[test]
fn quoted_datastar_action_stays_static() {
    let src = r#"
@component page = |{}| {
    <button data-on:click="@post('/x')">Go</button>
}
"#;
    let out = compile_ok(src);
    assert!(!out.roc.contains("import Datastar"));
    assert!(out.roc.contains("@post('/x')"));
    assert!(!out.roc.contains("Datastar.post"));
}

#[test]
fn rejects_datastar_action_with_js_single_quotes() {
    let src = r#"
@component page = |{}| {
    <button data-on:click=@post('/x')>Go</button>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("Roc strings") && msg.contains("@post(\"/x\")")),
        "{errors:?}"
    );
}

#[test]
fn rejects_unknown_datastar_action() {
    let src = r#"
@component page = |{}| {
    <button data-on:click=@steer("/x")>Go</button>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("unknown Datastar action `@steer`")),
        "{errors:?}"
    );
}

#[test]
fn datastar_action_in_text_is_still_unknown_directive() {
    let src = r#"
@component page = |{}| {
    <p>@post("/x")</p>
}
"#;
    let errors = compile_err(src);
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("unknown directive `@post`")),
        "{errors:?}"
    );
}

fn scope_id(css: &str) -> &str {
    let marker = r#"data-rocci-css~=""#;
    let start = css.find(marker).expect("scope attr") + marker.len();
    let end = css[start..].find('"').expect("scope end") + start;
    &css[start..end]
}

#[test]
fn lowers_file_and_component_css() {
    let src = r#"
@css {
    .card { padding: 1rem; }
}

@component hello = |{ name }| {
    @css {
        .greeting { color: navy; }
        p { margin: 0; }
    }
    <p class="greeting">Hello, {name}</p>
}

@component other = |{}| {
    <div class="card"></div>
}
"#;
    let out = compile_ok(src);
    assert_eq!(out.styles.len(), 2);
    let file = out
        .styles
        .iter()
        .find(|style| style.kind == StyleKind::File)
        .expect("file style");
    let hello = out
        .styles
        .iter()
        .find(|style| style.kind == StyleKind::Component && style.name == "hello")
        .expect("hello style");
    let file_id = scope_id(&file.css);
    let hello_id = scope_id(&hello.css);
    assert_ne!(file_id, hello_id);
    assert!(file.css.contains(".card { padding: 1rem; }"));
    assert!(hello.css.contains(".greeting { color: navy; }"));
    assert!(out.roc.contains("\"style\""));
    assert!(
        out.roc
            .contains(&format!(r#"data-rocci-css~=\"{hello_id}\""#))
    );
    assert!(out.roc.contains(&format!("\"{file_id} {hello_id}\"")));
    assert!(out.roc.contains(&format!("\"{file_id}\"")));
    assert!(!out.roc.contains("@css"));
}

#[test]
fn isolates_component_css_and_does_not_stamp_child_calls() {
    let src = r#"
@component parent = |{}| {
    @css {
        .x { color: red; }
    }
    <div class="x">
        <Child />
    </div>
}

@component child = |{}| {
    @css {
        .x { color: blue; }
    }
    <span class="x">ok</span>
}
"#;
    let out = compile_ok(src);
    let parent = out
        .styles
        .iter()
        .find(|style| style.name == "parent")
        .expect("parent");
    let child = out
        .styles
        .iter()
        .find(|style| style.name == "child")
        .expect("child");
    let parent_id = scope_id(&parent.css);
    let child_id = scope_id(&child.css);
    assert_ne!(parent_id, child_id);
    assert!(out.roc.contains(&format!("\"{parent_id}\"")));
    assert!(out.roc.contains(&format!("\"{child_id}\"")));
    let child_call = out.roc.find("child(\n").expect("child call");
    let after = &out.roc[child_call..child_call + 40];
    assert!(
        !after.contains("data-rocci-css"),
        "component calls must not receive a scope attribute: {after}"
    );
}

#[test]
fn scans_nested_css_strings_and_comments() {
    let src = r#"
@component box = |{}| {
    @css {
        .card { content: "{"; /* } */ }
        .quote { content: '}'; }
    }
    <section class="card"></section>
}
"#;
    let out = compile_ok(src);
    let css = &out.styles[0].css;
    assert!(css.contains(r#".card { content: "{"; /* } */ }"#));
    assert!(css.contains(".quote { content: '}'; }"));
}

#[test]
fn formats_css_in_lisp_ast() {
    let src = r#"
@css {
    .card { padding: 1rem; }
}

@component hello = |{ name }| {
    @css {
        .greeting { color: navy; }
    }
    <p class="greeting">{name}</p>
}
"#;
    let out = compile_ok(src);
    let ast = format_ast(src, &out.document);
    assert!(ast.contains("(css \".card { padding: 1rem; }\")"));
    assert!(ast.contains("(css \".greeting { color: navy; }\")"));
}

#[test]
fn rejects_css_after_markup_and_inside_if() {
    let after_markup = compile_err(
        r#"
@component hello = |{}| {
    <p>Hi</p>
    @css { .x { color: red; } }
}
"#,
    );
    assert!(
        after_markup
            .iter()
            .any(|msg| msg.contains("`@css` must appear before render-producing items")),
        "{after_markup:?}"
    );

    let nested = compile_err(
        r#"
@component hello = |{}| {
    @if True {
        @css { .x { color: red; } }
        <p>Hi</p>
    }
}
"#,
    );
    assert!(
        nested
            .iter()
            .any(|msg| msg.contains("`@css` is only valid at the start of a component body")),
        "{nested:?}"
    );
}
