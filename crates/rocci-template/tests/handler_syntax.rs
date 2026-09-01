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
    compile(SourceFile::new("test.rocci", src), &LowerOptions::default())
        .diagnostics
        .into_iter()
        .filter(|d| d.is_error())
        .map(|d| match d.code {
            Some(code) => format!("{code}: {}", d.message),
            None => d.message,
        })
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
fn accepts_every_canonical_header_as_a_typed_route() {
    let cases = [
        (r#"@get:view("/")"#, "view", "GET", "document"),
        (r#"@get:fragment("/f")"#, "fragment", "GET", "fragment"),
        (r#"@get:live("/sse")"#, "live", "GET", "live"),
        (r#"@post:fragment("/f")"#, "fragment", "POST", "fragment"),
        (r#"@put:fragment("/f")"#, "fragment", "PUT", "fragment"),
        (r#"@patch:fragment("/f")"#, "fragment", "PATCH", "fragment"),
        (
            r#"@delete:fragment("/f")"#,
            "fragment",
            "DELETE",
            "fragment",
        ),
        (r#"@post:command("/c")"#, "command", "POST", "command"),
        (r#"@put:command("/c")"#, "command", "PUT", "command"),
        (r#"@patch:command("/c")"#, "command", "PATCH", "command"),
        (r#"@delete:command("/c")"#, "command", "DELETE", "command"),
    ];
    for (header, head, method, role) in cases {
        let src = wrap_handler(header);
        let out = compile_ok(&src);
        let ast = format_ast(&src, &out.document);
        assert!(
            ast.contains(&format!("({head} {method} ")),
            "{header}\n{ast}"
        );
        let handlers = inspect_handlers(&out.document);
        assert_eq!(handlers.len(), 1, "{header}");
        assert_eq!(handlers[0].method, method, "{header}");
        assert_eq!(handlers[0].role, role, "{header}");
        assert_eq!(ast, format_ast(&src, &out.document));
    }
}

#[test]
fn accepts_many_distinct_live_paths() {
    let src = r#"
@get:live("/streams/dashboard") { Html.text("dashboard") }
@get:live("/streams/notifications") = |state, request| { Html.text("notifications") }
"#;
    let out = compile_ok(src);
    let live: Vec<_> = inspect_handlers(&out.document)
        .into_iter()
        .filter(|handler| handler.role == "live")
        .collect();
    assert_eq!(live.len(), 2);
    assert_eq!(live[0].path, "/streams/dashboard");
    assert_eq!(live[1].path, "/streams/notifications");
}

#[test]
fn rejects_illegal_method_role_pairs_in_validation() {
    let cases = [
        (
            r#"@get:command("/x") { {} }"#,
            "GET accepts view, fragment, or live",
        ),
        (
            r#"@post:view("/x") { Html.text("x") }"#,
            "mutation methods accept fragment or command",
        ),
        (
            r#"@put:live("/x") { Html.text("x") }"#,
            "mutation methods accept fragment or command",
        ),
        (
            r#"@delete:view("/x") { Html.text("x") }"#,
            "mutation methods accept fragment or command",
        ),
    ];
    for (src, needle) in cases {
        let errors = compile_err(src);
        assert_eq!(
            errors
                .iter()
                .filter(|msg| msg.contains("RC2009") && msg.contains("illegal handler pair"))
                .count(),
            1,
            "{src}: {errors:?}"
        );
        assert!(
            errors.iter().any(|msg| msg.contains(needle)),
            "{src}: {errors:?}"
        );
    }
}

#[test]
fn diagnoses_malformed_method_first_headers() {
    let cases = [
        (
            r#"@get("/x") { Html.text("x") }"#,
            "RC1001: missing `:` and response role",
        ),
        (
            r#"@get:("/x") { Html.text("x") }"#,
            "RC1001: expected a response role",
        ),
        (
            r#"@get:stream("/x") { Html.text("x") }"#,
            "RC1003: unknown handler role `stream`",
        ),
        (
            r#"@head:view("/x") { Html.text("x") }"#,
            "RC2008: unknown HTTP method `head`",
        ),
        (
            r#"@get:fragment(path) { Html.text("x") }"#,
            "RC1001: expected a string literal path",
        ),
        (
            r#"@get:live("") { Html.text("x") }"#,
            "RC2010: `@get:live` requires a non-empty literal path",
        ),
        (
            r#"@get:fragment("/x") json { Html.text("x") }"#,
            "RC1003: `json` is not a response selector",
        ),
        (
            r#"@post:fragment[html]("/x") { Html.text("x") }"#,
            "RC1003: selector brackets are not part",
        ),
        (
            r#"@post:fragment("/x") -> html { Html.text("x") }"#,
            "RC1003: `->` does not select a response",
        ),
    ];
    for (src, needle) in cases {
        let errors = compile_err(src);
        assert!(
            errors.iter().any(|msg| msg.contains(needle)),
            "{src}: {errors:?}"
        );
    }
}

#[test]
fn rejects_every_role_first_form_with_a_canonical_rewrite() {
    let cases = [
        (r#"@view("/") { Html.text("x") }"#, "@get:view"),
        (r#"@patch("/x") { Html.text("x") }"#, "@post:fragment"),
        (r#"@patch:put("/x") { Html.text("x") }"#, "@put:fragment"),
        (
            r#"@patch:patch("/x") { Html.text("x") }"#,
            "@patch:fragment",
        ),
        (
            r#"@patch:delete("/x") { Html.text("x") }"#,
            "@delete:fragment",
        ),
        (r#"@command("/x") { {} }"#, "@post:command"),
        (r#"@command:put("/x") { {} }"#, "@put:command"),
        (r#"@command:patch("/x") { {} }"#, "@patch:command"),
        (r#"@command:delete("/x") { {} }"#, "@delete:command"),
        (r#"@live { Html.text("x") }"#, "@get:live(\"/sse\")"),
        (
            r#"@live("/custom") { Html.text("x") }"#,
            "@get:live(\"/custom\")",
        ),
    ];
    for (src, rewrite) in cases {
        let errors = compile_err(src);
        assert!(
            errors
                .iter()
                .any(|msg| msg.contains("RC1004") && msg.contains("role-first syntax was removed")),
            "{src}: {errors:?}"
        );
        assert!(
            errors.iter().any(|msg| msg.contains(rewrite)),
            "{src}: {errors:?}"
        );
    }
}

#[test]
fn retained_on_and_action_removals_point_to_final_syntax() {
    let cases = [
        (r#"@on:get("/") { Html.text("x") }"#, "@get:view"),
        (r#"@on:post("/x") { Html.text("x") }"#, "@post:fragment"),
        (r#"@on:delete("/x") json { {} }"#, "@delete:command"),
        (
            r#"@action[patch]:delete("/x") { Html.text("x") }"#,
            "@method:fragment",
        ),
    ];
    for (src, rewrite) in cases {
        let errors = compile_err(src);
        assert!(
            errors
                .iter()
                .any(|msg| msg.contains("RC1004") && msg.contains(rewrite)),
            "{src}: {errors:?}"
        );
    }
}

#[test]
fn duplicate_routes_and_generated_names_are_rejected() {
    let duplicate = compile_err(
        r#"
@get:view("/x") { Html.text("a") }
@get:fragment("/x") { Html.text("b") }
"#,
    );
    assert!(
        duplicate.iter().any(|msg| {
            msg.contains("RC2011") && msg.contains("duplicate") && msg.contains("@get:fragment")
        }),
        "{duplicate:?}"
    );

    let names = compile_err(
        r#"
@get:fragment("/a-b") { Html.text("a") }
@get:fragment("/a_b") { Html.text("b") }
"#,
    );
    assert!(
        names
            .iter()
            .any(|msg| msg.contains("RC2012") && msg.contains("both generate Roc handler")),
        "{names:?}"
    );
}

#[test]
fn malformed_and_unclosed_routes_recover_to_later_declarations() {
    let src = r#"
@get:fragment("/broken"
@css { .x { color: red; } }

@post:live("/illegal") { Html.text("bad") }

@get:live("/unclosed") {
    Html.text("unclosed")

helper = || 1

@get:view("/") { Html.text("ok") }
@post:command("/c") { {} }

@component Survives = |{}| { <p>yes</p> }
"#;
    let out = compile(SourceFile::new("test.rocci", src), &LowerOptions::default());
    assert!(out.has_errors());
    assert!(
        out.components
            .iter()
            .any(|component| component.name == "survives")
    );
    let handlers = inspect_handlers(&out.document);
    assert!(
        handlers
            .iter()
            .any(|handler| handler.method == "GET" && handler.path == "/")
    );
    assert!(
        handlers
            .iter()
            .any(|handler| handler.method == "POST" && handler.path == "/c")
    );
}

#[test]
fn inspects_final_kind_method_path_and_role() {
    let src = r#"
@get:view("/") { Html.text("v") }
@get:fragment("/search") { Html.text("s") }
@patch:fragment("/p") { Html.text("p") }
@delete:command("/c") { {} }
@get:live("/streams/main") { Html.text("l") }
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
            "fragment GET \"/search\" fragment",
            "fragment PATCH \"/p\" fragment",
            "command DELETE \"/c\" command",
            "live GET \"/streams/main\" live",
        ]
    );
}
