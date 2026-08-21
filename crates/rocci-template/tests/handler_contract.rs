//! Frozen contract for semantic `@view` / `@patch` / `@command` / `@live`.
//! Phase 0 records the intended language; later phases implement it. This file
//! does not parse new syntax.

struct HandlerRole {
    noun: &'static str,
    header: &'static str,
    default_method: &'static str,
    allowed_overrides: &'static [&'static str],
    success_value: &'static str,
    generated_response: &'static str,
}

const ROLES: &[HandlerRole] = &[
    HandlerRole {
        noun: "view",
        header: "@view(path)",
        default_method: "GET",
        allowed_overrides: &[],
        success_value: "Html",
        generated_response: "document HTML",
    },
    HandlerRole {
        noun: "patch",
        header: "@patch(path)",
        default_method: "POST",
        allowed_overrides: &["PUT", "PATCH", "DELETE"],
        success_value: "Html",
        generated_response: "one-shot patch-elements SSE",
    },
    HandlerRole {
        noun: "command",
        header: "@command(path)",
        default_method: "POST",
        allowed_overrides: &["PUT", "PATCH", "DELETE"],
        success_value: "JSON-encodable data",
        generated_response: "Datastar 204; otherwise encoded JSON",
    },
    HandlerRole {
        noun: "live",
        header: "@live",
        default_method: "GET",
        allowed_overrides: &[],
        success_value: "Html",
        generated_response: "long-lived patch-elements SSE",
    },
];

const ACCEPTED_HEADERS: &[&str] = &[
    "@view(path)",
    "@patch(path)",
    "@patch:put(path)",
    "@patch:patch(path)",
    "@patch:delete(path)",
    "@command(path)",
    "@command:put(path)",
    "@command:patch(path)",
    "@command:delete(path)",
    "@live",
];

const REJECTED_NEAR_MISSES: &[&str] = &[
    "@patch:post(path)",
    "@command:post(path)",
    "@patch:get(path)",
    "@command:get(path)",
    "@view:get(path)",
    "@on:get(path)",
    "@on:post(path)",
    "@on:post(path) json",
    "@action[patch]:delete(path)",
    "@action:delete[patch](path)",
    "@action(path) -> patch",
];

const PROPOSED_COMMAND_EXAMPLE: &str = r#"
@command("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    { count }
}
"#;

const COMMAND_ENCODER: &str = "Encoding.Json.to_str_try";
const TOTAL_ENCODER: &str = "Encoding.Json.to_str";
const NAMING_GATE_PATCH: &str = "@patch";
const NAMING_GATE_SUBSTITUTE: &str = "@fragment";

#[test]
fn freezes_four_declaration_roles() {
    assert_eq!(
        ROLES.iter().map(|role| role.noun).collect::<Vec<_>>(),
        ["view", "patch", "command", "live"]
    );
    let view = role("view");
    assert_eq!(view.default_method, "GET");
    assert!(view.allowed_overrides.is_empty());
    assert_eq!(view.success_value, "Html");

    let patch = role("patch");
    assert_eq!(patch.default_method, "POST");
    assert_eq!(patch.allowed_overrides, ["PUT", "PATCH", "DELETE"]);
    assert_eq!(patch.success_value, "Html");

    let command = role("command");
    assert_eq!(command.default_method, "POST");
    assert_eq!(command.allowed_overrides, ["PUT", "PATCH", "DELETE"]);
    assert_eq!(command.success_value, "JSON-encodable data");
    assert!(command.generated_response.contains("204"));
    assert!(command.generated_response.contains("JSON"));

    let live = role("live");
    assert_eq!(live.header, "@live");
    assert_eq!(live.default_method, "GET");
    assert_eq!(live.success_value, "Html");
}

#[test]
fn freezes_accepted_headers_and_post_omission() {
    for header in ACCEPTED_HEADERS {
        assert!(!header.contains(":post"), "POST is omitted: {header}");
    }
    assert!(ACCEPTED_HEADERS.contains(&"@patch:patch(path)"));
    assert!(ACCEPTED_HEADERS.contains(&"@command:delete(path)"));
    assert!(!ACCEPTED_HEADERS.iter().any(|header| header.contains("@on")));
}

#[test]
fn freezes_clean_cut_removal_and_rejected_near_misses() {
    for near_miss in REJECTED_NEAR_MISSES {
        assert!(
            !ACCEPTED_HEADERS.contains(near_miss),
            "{near_miss} must not be an accepted header"
        );
    }
    assert!(
        REJECTED_NEAR_MISSES
            .iter()
            .any(|header| header.starts_with("@on:")),
        "removed @on declarations stay rejected, not aliases"
    );
}

#[test]
fn freezes_patch_versus_fragment_naming_gate() {
    assert_eq!(NAMING_GATE_PATCH, "@patch");
    assert_eq!(NAMING_GATE_SUBSTITUTE, "@fragment");
    assert!(
        ACCEPTED_HEADERS
            .iter()
            .any(|header| header.starts_with("@patch")),
        "trial syntax keeps @patch"
    );
    assert!(
        !ACCEPTED_HEADERS
            .iter()
            .any(|header| header.contains("fragment")),
        "do not substitute @fragment unless complete-example testing shows method confusion"
    );
}

#[test]
fn proposed_examples_do_not_interpolate_json() {
    assert!(!PROPOSED_COMMAND_EXAMPLE.contains("Json.to_str"));
    assert!(!PROPOSED_COMMAND_EXAMPLE.contains("{\\\""));
    assert!(!PROPOSED_COMMAND_EXAMPLE.contains("${count.to_str()}"));
    assert!(PROPOSED_COMMAND_EXAMPLE.contains("{ count }"));
}

#[test]
fn freezes_host_json_encoder_api() {
    assert_eq!(COMMAND_ENCODER, "Encoding.Json.to_str_try");
    assert_eq!(TOTAL_ENCODER, "Encoding.Json.to_str");
}

fn role(noun: &str) -> &'static HandlerRole {
    ROLES
        .iter()
        .find(|role| role.noun == noun)
        .unwrap_or_else(|| panic!("missing role {noun}"))
}
