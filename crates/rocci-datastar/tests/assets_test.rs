use rocci_datastar::assets::*;

#[test]
fn test_parse_valid_version() {
    assert_eq!(parse_version("1.0.2").unwrap(), "1.0.2");
    assert_eq!(parse_version("v1.0.2").unwrap(), "1.0.2");
    assert_eq!(parse_version("  1.2.3  ").unwrap(), "1.2.3");
}

#[test]
fn test_parse_invalid_version() {
    assert!(parse_version("1.0").is_err());
    assert!(parse_version("1.0.0.0").is_err());
    assert!(parse_version("alpha").is_err());
    assert!(parse_version("").is_err());
}

#[test]
fn test_tag_name() {
    assert_eq!(tag_name("1.0.2"), "v1.0.2");
    assert_eq!(tag_name("v1.0.2"), "v1.0.2");
}

#[test]
fn test_looks_like_datastar_js() {
    assert!(looks_like_datastar_js(
        b"// Datastar v1.0.2\nconsole.log('datastar');"
    ));
    assert!(!looks_like_datastar_js(b"alert('other library');"));
}

#[test]
fn test_parse_version_comment() {
    let js = b"// Datastar v1.0.2\nconst x = 1;";
    assert_eq!(parse_version_comment(js), Some("1.0.2".to_string()));
}
