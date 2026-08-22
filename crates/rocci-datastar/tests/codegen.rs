use rocci_datastar::DATASTAR_ROC_TEMPLATE;

#[test]
fn generated_and_authored_runtime_helpers_are_in_sync() {
    let cli = include_str!("../../rocci-cli/runtime/Datastar.roc");
    assert_eq!(DATASTAR_ROC_TEMPLATE, cli);
    let authored = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/rocci/custom/datastar/Datastar.roc");
    if let Ok(src) = std::fs::read_to_string(&authored) {
        assert_eq!(DATASTAR_ROC_TEMPLATE, src);
    }
}

#[test]
fn roc_patch_signals_surface_uses_typed_options_and_canonical_sse_fields() {
    assert!(DATASTAR_ROC_TEMPLATE.contains("PatchSignalsOpt : [OnlyIfMissing(Bool)]"));
    assert!(DATASTAR_ROC_TEMPLATE.contains("patch_signals = patch_signals_event"));
    assert!(DATASTAR_ROC_TEMPLATE.contains("patch_signals_with = patch_signals_event_with"));
    assert!(DATASTAR_ROC_TEMPLATE.contains("Sse.Event.named("));
    assert!(DATASTAR_ROC_TEMPLATE.contains("\"datastar-patch-signals\""));
    assert!(DATASTAR_ROC_TEMPLATE.contains("\"onlyIfMissing true\""));
    assert!(DATASTAR_ROC_TEMPLATE.contains("\"signals ${line}\""));
    assert!(!DATASTAR_ROC_TEMPLATE.contains("execute_script"));
}
