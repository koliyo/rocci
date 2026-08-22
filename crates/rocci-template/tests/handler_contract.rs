//! Frozen contract for the approved verb-first handler and path-addressed live cutover.
//! Phase 0 records the intended language; later phases implement it. This file
//! deliberately does not parse the new syntax.

struct HandlerPair {
    method: &'static str,
    role: &'static str,
    header: &'static str,
    success_value: &'static str,
    generated_response: &'static str,
}

const ACCEPTED_PAIRS: &[HandlerPair] = &[
    pair("GET", "view", "@get:view(path)", "Html", "document HTML"),
    pair(
        "GET",
        "fragment",
        "@get:fragment(path)",
        "Html",
        "one-shot element morph",
    ),
    pair(
        "GET",
        "live",
        "@get:live(path)",
        "Html",
        "long-lived element morph stream",
    ),
    pair(
        "POST",
        "fragment",
        "@post:fragment(path)",
        "Html",
        "one-shot element morph",
    ),
    pair(
        "PUT",
        "fragment",
        "@put:fragment(path)",
        "Html",
        "one-shot element morph",
    ),
    pair(
        "PATCH",
        "fragment",
        "@patch:fragment(path)",
        "Html",
        "one-shot element morph",
    ),
    pair(
        "DELETE",
        "fragment",
        "@delete:fragment(path)",
        "Html",
        "one-shot element morph",
    ),
    pair(
        "POST",
        "command",
        "@post:command(path)",
        "{}",
        "no success representation",
    ),
    pair(
        "PUT",
        "command",
        "@put:command(path)",
        "{}",
        "no success representation",
    ),
    pair(
        "PATCH",
        "command",
        "@patch:command(path)",
        "{}",
        "no success representation",
    ),
    pair(
        "DELETE",
        "command",
        "@delete:command(path)",
        "{}",
        "no success representation",
    ),
];

const REJECTED_NEAR_MISSES: &[&str] = &[
    "@get(path)",
    "@post(path)",
    "@get:command(path)",
    "@post:view(path)",
    "@post:live(path)",
    "@post:json(path)",
    "@get:signals(path)",
    "@get:stream(path)",
    "@fragment:get(path)",
    "@view(path)",
    "@patch:patch(path)",
    "@command:delete(path)",
    "@live",
    "@live(path)",
    "@on:get(path)",
];

const COMPLETE_EXAMPLE: &str = r#"
@get:view("/") = |state| { page(state) }
@get:fragment("/search") = |state, request| { search_results(state, request) }
@post:fragment("/actions/validate") = |state, request| { validation(state, request) }
@patch:fragment("/actions/items/42") = |state, request| { item_row(state, request) }
@post:command("/actions/increment") = |state| { increment!(state)? }
@get:live("/streams/dashboard") = |state| { dashboard_regions(state) }
"#;

const SINGLETON_INJECTION: &str = "inject authored path with OpenWhenHidden(True)";
const MULTIPLE_INJECTION: &str = "inject nothing; subscriptions are explicit";
const AUTHORED_DATA_INIT: &str = "preserve exactly";
const COMMAND_WIRE_POLICY: &str = "Datastar: empty SSE; ordinary caller: 204";
const POLL_INTERVAL_MS: u64 = 100;

const fn pair(
    method: &'static str,
    role: &'static str,
    header: &'static str,
    success_value: &'static str,
    generated_response: &'static str,
) -> HandlerPair {
    HandlerPair {
        method,
        role,
        header,
        success_value,
        generated_response,
    }
}

#[test]
fn freezes_closed_method_role_matrix() {
    assert_eq!(ACCEPTED_PAIRS.len(), 11);
    assert!(accepted("GET", "view"));
    assert!(accepted("GET", "fragment"));
    assert!(accepted("GET", "live"));
    for method in ["POST", "PUT", "PATCH", "DELETE"] {
        assert!(accepted(method, "fragment"));
        assert!(accepted(method, "command"));
        assert!(!accepted(method, "view"));
        assert!(!accepted(method, "live"));
    }
    assert!(!accepted("GET", "command"));
}

#[test]
fn freezes_verb_first_fragment_naming_and_clean_cut() {
    assert!(ACCEPTED_PAIRS.iter().all(|pair| {
        pair.header
            .starts_with(&format!("@{}:", pair.method.to_ascii_lowercase()))
    }));
    assert!(ACCEPTED_PAIRS.iter().any(|pair| pair.role == "fragment"));
    assert!(!ACCEPTED_PAIRS.iter().any(|pair| {
        pair.header.starts_with("@view")
            || pair.header.starts_with("@patch(")
            || pair.header.starts_with("@command")
            || pair.header.starts_with("@live")
    }));
    for near_miss in REJECTED_NEAR_MISSES {
        assert!(!ACCEPTED_PAIRS.iter().any(|pair| pair.header == *near_miss));
    }
}

#[test]
fn freezes_representation_free_commands() {
    for command in ACCEPTED_PAIRS.iter().filter(|pair| pair.role == "command") {
        assert_eq!(command.success_value, "{}");
        assert_eq!(command.generated_response, "no success representation");
    }
    assert_eq!(
        COMMAND_WIRE_POLICY,
        "Datastar: empty SSE; ordinary caller: 204"
    );
    assert!(!COMPLETE_EXAMPLE.contains("Json.to_str"));
    assert!(!COMPLETE_EXAMPLE.contains("{ count"));
}

#[test]
fn freezes_plural_live_paths_and_injection_policy() {
    let live = ACCEPTED_PAIRS
        .iter()
        .find(|pair| pair.role == "live")
        .expect("GET live pair");
    assert_eq!(live.method, "GET");
    assert_eq!(live.header, "@get:live(path)");
    assert!(COMPLETE_EXAMPLE.contains("@get:live(\"/streams/dashboard\")"));
    assert_eq!(
        SINGLETON_INJECTION,
        "inject authored path with OpenWhenHidden(True)"
    );
    assert_eq!(
        MULTIPLE_INJECTION,
        "inject nothing; subscriptions are explicit"
    );
    assert_eq!(AUTHORED_DATA_INIT, "preserve exactly");
}

#[test]
fn records_linear_polling_cost_model() {
    assert_eq!(POLL_INTERVAL_MS, 100);
    let streams = 2;
    let tabs = 2;
    assert_eq!(
        streams * tabs,
        4,
        "one independent response and poll loop per stream subscription"
    );
}

fn accepted(method: &str, role: &str) -> bool {
    ACCEPTED_PAIRS
        .iter()
        .any(|pair| pair.method == method && pair.role == role)
}
