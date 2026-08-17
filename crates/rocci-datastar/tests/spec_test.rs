use rocci_datastar::spec::*;

#[test]
fn test_lookup_attributes() {
    let bind = lookup_attribute("data-bind").expect("data-bind exists");
    assert!(bind.supports_custom_key);
    assert!(bind.supports_modifiers);

    let on = lookup_attribute("data-on:click").expect("data-on matches prefix");
    assert_eq!(on.name, "data-on");

    assert!(is_datastar_attribute("data-signals"));
    assert!(!is_datastar_attribute("data-unknown-custom"));
}

#[test]
fn test_parse_attribute_with_modifiers() {
    let parsed = parse_attribute("data-on:input__debounce.500ms__passive").expect("valid parse");
    assert_eq!(parsed.directive, "data-on");
    assert_eq!(parsed.key.as_deref(), Some("input"));
    assert_eq!(parsed.modifiers.len(), 2);
    assert_eq!(parsed.modifiers[0].name, "debounce");
    assert_eq!(parsed.modifiers[0].argument.as_deref(), Some("500ms"));
    assert_eq!(parsed.modifiers[1].name, "passive");
    assert_eq!(parsed.modifiers[1].argument, None);
}

#[test]
fn test_lookup_actions() {
    let get = lookup_action("@get").expect("@get exists");
    assert_eq!(get.name, "@get");
    assert!(lookup_action("@unknown").is_none());
}
