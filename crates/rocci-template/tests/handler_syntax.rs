use rocci_template::{LowerOptions, SourceFile, compile, format_ast, inspect_handlers};

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

fn wrap_handler(header: &str) -> String {
    format!(
        r#"
{header} {{
    Html.text("ok")
}}

@component Unused = |{{}}| {{
    <p>x</p>
}}
"#
    )
}

#[test]
fn accepts_every_canonical_header() {
    let headers = [
        r#"@view("/")"#,
        r#"@patch("/p")"#,
        r#"@patch:put("/p")"#,
        r#"@patch:patch("/p")"#,
        r#"@patch:delete("/p")"#,
        r#"@command("/c")"#,
        r#"@command:put("/c")"#,
        r#"@command:patch("/c")"#,
        r#"@command:delete("/c")"#,
        r#"@live"#,
    ];
    for header in headers {
        let src = wrap_handler(header);
        let out = compile_ok(&src);
        let ast = format_ast(&src, &out.document);
        if header.starts_with("@view") {
            assert!(ast.contains("(view "), "{header}\n{ast}");
            assert_eq!(out.routes[0].method, "GET");
        } else if header.starts_with("@patch:put") {
            assert!(ast.contains("(patch PUT "), "{header}\n{ast}");
            assert_eq!(out.routes[0].method, "PUT");
        } else if header.starts_with("@patch:patch") {
            assert!(ast.contains("(patch PATCH "), "{header}\n{ast}");
            assert_eq!(out.routes[0].method, "PATCH");
        } else if header.starts_with("@patch:delete") {
            assert!(ast.contains("(patch DELETE "), "{header}\n{ast}");
            assert_eq!(out.routes[0].method, "DELETE");
        } else if header.starts_with("@patch") {
            assert!(ast.contains("(patch POST "), "{header}\n{ast}");
            assert_eq!(out.routes[0].method, "POST");
            assert_eq!(out.routes[0].respond, rocci_template::RespondKind::Patch);
        } else if header.starts_with("@command:put") {
            assert!(ast.contains("(command PUT "), "{header}\n{ast}");
        } else if header.starts_with("@command:patch") {
            assert!(ast.contains("(command PATCH "), "{header}\n{ast}");
        } else if header.starts_with("@command:delete") {
            assert!(ast.contains("(command DELETE "), "{header}\n{ast}");
        } else if header.starts_with("@command") {
            assert!(ast.contains("(command POST "), "{header}\n{ast}");
            assert_eq!(out.routes[0].respond, rocci_template::RespondKind::Command);
        } else {
            assert!(out.live.is_some(), "{header}");
        }
        let again = format_ast(&src, &out.document);
        assert_eq!(
            ast, again,
            "format_ast must be two-pass idempotent for {header}"
        );
    }
}

#[test]
fn rejects_post_and_get_suffixes() {
    let cases = [
        (
            r#"@patch:post("/x") { Html.text("x") }"#,
            "POST is the default",
        ),
        (
            r#"@command:post("/x") { Html.text("x") }"#,
            "POST is the default",
        ),
        (
            r#"@patch:get("/x") { Html.text("x") }"#,
            "`@patch` cannot use GET",
        ),
        (
            r#"@command:get("/x") { Html.text("x") }"#,
            "`@command` cannot use GET",
        ),
        (
            r#"@view:get("/") { Html.text("x") }"#,
            "`@view` has no method suffix",
        ),
        (
            r#"@patch:head("/x") { Html.text("x") }"#,
            "unknown HTTP method `head`",
        ),
    ];
    for (src, needle) in cases {
        let errors = compile_err(src);
        assert!(
            errors.iter().any(|msg| msg.contains(needle)),
            "expected `{needle}` in {errors:?} for {src}"
        );
    }
}

#[test]
fn rejects_removed_on_and_action_experiments() {
    let cases = [
        (
            r#"@on:get("/") { Html.text("x") }"#,
            "`@on` was removed",
            "@view",
        ),
        (
            r#"@on:post("/x") { Html.text("x") }"#,
            "`@on` was removed",
            "@patch",
        ),
        (
            r#"@on:post("/x") json { "{\"a\":1}" }"#,
            "`@on` was removed",
            "@command",
        ),
        (
            r#"@on:delete("/x") json { "{\"a\":1}" }"#,
            "`@on` was removed",
            "@command:delete",
        ),
        (
            r#"@action[patch]:delete("/x") { Html.text("x") }"#,
            "`@action` was not adopted",
            "@patch",
        ),
        (
            r#"@action:delete[patch]("/x") { Html.text("x") }"#,
            "`@action` was not adopted",
            "@command",
        ),
        (
            r#"@patch("/x") -> html { Html.text("x") }"#,
            "`->` does not select a response",
            "@patch",
        ),
        (
            r#"@patch[html]("/x") { Html.text("x") }"#,
            "selector brackets are not part of `@patch`",
            "@patch",
        ),
        (
            r#"@command("/x") json { { count: 1 } }"#,
            "`json` was removed",
            "@command",
        ),
    ];
    for (src, needle, rewrite) in cases {
        let errors = compile_err(src);
        assert!(
            errors.iter().any(|msg| msg.contains(needle)),
            "expected `{needle}` in {errors:?} for {src}"
        );
        assert!(
            errors.iter().any(|msg| msg.contains(rewrite)),
            "expected rewrite `{rewrite}` in {errors:?} for {src}"
        );
    }
}

#[test]
fn recovers_malformed_handler_and_keeps_later_valid_decl() {
    let src = r#"
@component First = |{}| {
    <p>one</p>
}

@patch("/broken"
@css { .x { color: red; } }

@view("/") {
    Html.text("ok")
}

@command("/c") {
    { count: 1 }
}

@component Second = |{}| {
    <p>two</p>
}
"#;
    let out = compile(SourceFile::new("test.rocci", src), &LowerOptions::default());
    assert!(out.has_errors(), "malformed header must diagnose");
    assert!(
        out.components
            .iter()
            .any(|component| component.name == "first")
    );
    assert!(
        out.components
            .iter()
            .any(|component| component.name == "second"),
        "later component must survive recovery: {:?}",
        out.components
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        out.routes
            .iter()
            .any(|route| route.path == "/" && route.method == "GET"),
        "{:?}",
        out.routes
    );
    assert!(
        out.routes.iter().any(
            |route| route.path == "/c" && route.respond == rocci_template::RespondKind::Command
        ),
        "{:?}",
        out.routes
    );
}

#[test]
fn recovers_unclosed_handler_among_roc_and_style() {
    let src = r#"
helper = || 1

@patch("/x") {
    Html.text("unclosed")

@css { .ok { color: blue; } }

@view("/") {
    Html.text("ok")
}
"#;
    let out = compile(SourceFile::new("test.rocci", src), &LowerOptions::default());
    assert!(
        out.routes.iter().any(|route| route.path == "/"),
        "{:?}",
        out.routes
    );
}

#[test]
fn missing_equals_and_path_are_diagnosed() {
    let errors = compile_err(
        r#"
@patch = |state| {
    Html.text("x")
}
"#,
    );
    assert!(
        errors
            .iter()
            .any(|msg| msg.contains("expected `(\"path\")`")),
        "{errors:?}"
    );
    let errors = compile_err(
        r#"
@view("/") = {
    Html.text("x")
}
"#,
    );
    assert!(
        errors.iter().any(|msg| msg.contains("expected `|params|`")),
        "{errors:?}"
    );
}

#[test]
fn inspects_kind_method_path_and_role() {
    let src = r#"
@view("/") { Html.text("v") }
@patch("/p") { Html.text("p") }
@patch:patch("/pp") { Html.text("pp") }
@command:delete("/c") { { n: 1 } }
@live { Html.text("l") }

@component Unused = |{}| {
    <p>x</p>
}
"#;
    let out = compile_ok(src);
    let lines: Vec<_> = inspect_handlers(&out.document)
        .into_iter()
        .map(|handler| handler.line())
        .collect();
    assert_eq!(
        lines,
        vec![
            "view GET \"/\" document",
            "patch POST \"/p\" patch",
            "patch PATCH \"/pp\" patch",
            "command DELETE \"/c\" command",
            "live GET \"/sse\" live",
        ]
    );
}
