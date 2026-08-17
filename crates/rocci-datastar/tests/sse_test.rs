use rocci_datastar::sse::*;

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
    assert!(formatted.contains("event: datastar-patch-signals\n"));
    assert!(formatted.contains("data: onlyIfMissing true\n"));
    assert!(formatted.contains("data: signals {\"count\": 10}\n"));
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
