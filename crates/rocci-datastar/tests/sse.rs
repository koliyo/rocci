use rocci_datastar::sse::*;

#[test]
fn strip_style_elements_drops_embedded_css_and_keeps_id_roots() {
    let html = concat!(
        "<style>.card{color:navy}</style>",
        "<section id=\"counter\">3</section>",
        "<style>.feed{margin:0}</style>",
        "<div id=\"counter-feed\"></div>",
    );
    assert_eq!(
        strip_style_elements(html),
        "<section id=\"counter\">3</section><div id=\"counter-feed\"></div>"
    );
}

#[test]
fn strip_style_elements_leaves_unclosed_style_in_place() {
    assert_eq!(
        strip_style_elements("<style>.x{color:red}<div id=\"n\">1</div>"),
        "<style>.x{color:red}<div id=\"n\">1</div>"
    );
}

#[test]
fn test_patch_elements_sse() {
    let patch = PatchElements::new("<div id=\"counter\">42</div>")
        .mode(PatchMode::Inner)
        .selector("#counter")
        .view_transition(true)
        .settle_duration(300);

    let formatted = patch.format_sse();
    assert!(formatted.contains("event: datastar-patch-elements\n"));
    assert!(formatted.contains("data: selector #counter\n"));
    assert!(formatted.contains("data: mode inner\n"));
    assert!(formatted.contains("data: useViewTransition true\n"));
    assert!(formatted.contains("data: settleDuration 300\n"));
    assert!(formatted.contains("data: elements <div id=\"counter\">42</div>\n"));
}

#[test]
fn test_patch_signals_sse() {
    let signals = PatchSignals::new("{\"count\": 10}").only_if_missing(true);
    let formatted = signals.format_sse();
    assert_eq!(
        formatted,
        "event: datastar-patch-signals\ndata: onlyIfMissing true\ndata: signals {\"count\": 10}\n\n"
    );
}

#[test]
fn patch_signals_normalizes_every_logical_line_like_the_roc_sse_builder() {
    let formatted = PatchSignals::new("{\r\n  \"count\": 10\r}\n").format_sse();
    assert_eq!(
        formatted,
        "event: datastar-patch-signals\ndata: signals {\ndata: signals   \"count\": 10\ndata: signals }\ndata: signals \n\n"
    );
}

#[test]
fn test_remove_fragments_sse() {
    let remove = RemoveFragments::new("#old-toast").view_transition(true);
    let formatted = remove.format_sse();
    assert!(formatted.contains("event: datastar-remove-fragments\n"));
    assert!(formatted.contains("data: selector #old-toast\n"));
    assert!(formatted.contains("data: useViewTransition true\n"));
}

#[test]
fn test_execute_script_sse() {
    let exec = ExecuteScript::new("console.log(\"hello\");");
    let formatted = exec.format_sse();
    assert!(formatted.contains("event: datastar-execute-script\n"));
    assert!(formatted.contains("data: script console.log(\"hello\");\n"));
}
