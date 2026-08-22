use rocci_docs::documented_declarations;

#[test]
fn extracts_attached_docs_and_skips_blank_separated() {
    let src = r#"
## Shared count card.
## Morphs `#counter`.
@component Card = |{ count }|
    <output>{count}</output>

## skipped because of the blank line

@put:fragment("/actions/put") = |_| {
    Card
}

## GET `/`.
@get:view("/") = || {
    <html><body></body></html>
}
"#;
    let decls = documented_declarations(src);
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[0].heading, "@component Card");
    assert_eq!(decls[0].body, "Shared count card.\nMorphs `#counter`.");
    assert_eq!(decls[0].line, 4);
    assert_eq!(decls[1].heading, "@get:view(\"/\")");
    assert_eq!(decls[1].body, "GET `/`.");
    assert_eq!(decls[1].line, 14);
}

#[test]
fn declaration_line_targets_decl_not_doc_comment() {
    let src = "## Card used by the listing fixture.\n@component Card = |{ title }| {\n    <div>{title}</div>\n}\n";
    let decls = documented_declarations(src);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].line, 2);
}

#[test]
fn extracts_verb_first_fragment_docs() {
    let src = r#"
## Replace the fragment.
@put:fragment("/actions/put-frag") = |_| {
    <p/>
}
"#;
    let decls = documented_declarations(src);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].heading, "@put:fragment(\"/actions/put-frag\")");
    assert_eq!(decls[0].body, "Replace the fragment.");
}
