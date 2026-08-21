use rocci_docs::documented_declarations;

#[test]
fn extracts_attached_docs_and_skips_blank_separated() {
    let src = r#"
## Shared count card.
## Morphs `#counter`.
@component Card = |{ count }|
    <output>{count}</output>

## skipped because of the blank line

@patch:put("/actions/put") = |_| {
    Card
}

## GET `/`.
@view("/") = || {
    <html><body></body></html>
}
"#;
    let decls = documented_declarations(src);
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[0].heading, "@component Card");
    assert_eq!(decls[0].body, "Shared count card.\nMorphs `#counter`.");
    assert_eq!(decls[1].heading, "@view(\"/\")");
    assert_eq!(decls[1].body, "GET `/`.");
}

#[test]
fn extracts_methoded_patch_docs() {
    let src = r#"
## Replace the fragment.
@patch:put("/actions/put-frag") = |_| {
    <p/>
}
"#;
    let decls = documented_declarations(src);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].heading, "@patch:put(\"/actions/put-frag\")");
    assert_eq!(decls[0].body, "Replace the fragment.");
}
