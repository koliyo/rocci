use rocci_template::{
    Attr, AttrValue, CommandDecl, ComponentCall, ComponentDecl, ComponentPath, ContextDecl,
    CssDecl, Document, Element, FixtureDecl, ForDirective, Fragment, Ident, IfDirective, InitDecl,
    Interpolation, LetDirective, LiveDecl, MatchArm, MatchDirective, ModuleItem, PatchDecl,
    TemplateBlock, TemplateItem, TextNode, ViewDecl,
};

fn ungram_productions(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let Some((name, _)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
            && name.chars().all(|ch| ch.is_ascii_alphanumeric())
        {
            names.push(name.to_string());
        }
    }
    names
}

fn toml_table_keys(src: &str, heading: &str) -> Vec<String> {
    let header = format!("[{heading}]");
    let mut keys = Vec::new();
    let mut in_section = false;
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == header;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, _)) = trimmed.split_once('=') else {
            continue;
        };
        keys.push(key.trim().trim_matches('"').to_string());
    }
    keys
}

fn classify_module_item(item: &ModuleItem) -> &'static str {
    match item {
        ModuleItem::Roc { .. } => "roc",
        ModuleItem::Component(_) => "component",
        ModuleItem::Fixture(_) => "fixture",
        ModuleItem::Css(_) => "css",
        ModuleItem::Context(_) => "context",
        ModuleItem::Init(_) => "init",
        ModuleItem::Live(_) => "live",
        ModuleItem::View(_) => "view",
        ModuleItem::Patch(_) => "patch",
        ModuleItem::Command(_) => "command",
    }
}

fn classify_template_item(item: &TemplateItem) -> &'static str {
    match item {
        TemplateItem::Element(_) => "element",
        TemplateItem::ComponentCall(_) => "component-call",
        TemplateItem::Fragment(_) => "fragment",
        TemplateItem::Text(_) => "text",
        TemplateItem::Interpolation(_) => "interpolation",
        TemplateItem::If(_) => "if",
        TemplateItem::For(_) => "for",
        TemplateItem::Match(_) => "match",
        TemplateItem::Let(_) => "let",
        TemplateItem::Css(_) => "css",
    }
}

#[test]
fn ungram_productions_are_classified_in_sidecar() {
    let ungram = include_str!("../Rocci.AST.ungram");
    let sidecar = include_str!("../Rocci.AST.toml");
    let productions = ungram_productions(ungram);
    assert!(
        productions.iter().any(|name| name == "Document"),
        "Rocci ungram must define Document"
    );

    let mut classified = Vec::new();
    for section in [
        "generated",
        "foreign",
        "opaque",
        "doc_only",
        "inline",
        "leaves",
    ] {
        classified.extend(toml_table_keys(sidecar, section));
    }
    for name in &productions {
        assert!(
            classified.iter().any(|key| key == name),
            "unclassified Rocci production {name}"
        );
    }
    for key in &classified {
        assert!(
            productions.iter().any(|name| name == key),
            "sidecar key {key} is not a Rocci ungram production"
        );
    }
}

#[test]
fn ungram_generated_productions_exist_as_rust_types() {
    let sidecar = include_str!("../Rocci.AST.toml");
    for name in toml_table_keys(sidecar, "generated") {
        assert!(
            [
                "Document",
                "ModuleItem",
                "ComponentDecl",
                "FixtureDecl",
                "CssDecl",
                "ContextDecl",
                "InitDecl",
                "LiveDecl",
                "ViewDecl",
                "PatchDecl",
                "CommandDecl",
                "TemplateBlock",
                "TemplateItem",
                "Element",
                "ComponentCall",
                "ComponentPath",
                "Fragment",
                "TextNode",
                "Interpolation",
                "Attr",
                "AttrValue",
                "IfDirective",
                "ForDirective",
                "MatchDirective",
                "MatchArm",
                "LetDirective",
                "Ident",
            ]
            .contains(&name.as_str()),
            "unexpected generated Rocci production {name}"
        );
    }
    let _ = std::any::type_name::<Document>();
    let _ = std::any::type_name::<ModuleItem>();
    let _ = std::any::type_name::<ComponentDecl>();
    let _ = std::any::type_name::<FixtureDecl>();
    let _ = std::any::type_name::<CssDecl>();
    let _ = std::any::type_name::<ContextDecl>();
    let _ = std::any::type_name::<InitDecl>();
    let _ = std::any::type_name::<LiveDecl>();
    let _ = std::any::type_name::<ViewDecl>();
    let _ = std::any::type_name::<PatchDecl>();
    let _ = std::any::type_name::<CommandDecl>();
    let _ = std::any::type_name::<TemplateBlock>();
    let _ = std::any::type_name::<TemplateItem>();
    let _ = std::any::type_name::<Element>();
    let _ = std::any::type_name::<ComponentCall>();
    let _ = std::any::type_name::<ComponentPath>();
    let _ = std::any::type_name::<Fragment>();
    let _ = std::any::type_name::<TextNode>();
    let _ = std::any::type_name::<Interpolation>();
    let _ = std::any::type_name::<Attr>();
    let _ = std::any::type_name::<AttrValue>();
    let _ = std::any::type_name::<IfDirective>();
    let _ = std::any::type_name::<ForDirective>();
    let _ = std::any::type_name::<MatchDirective>();
    let _ = std::any::type_name::<MatchArm>();
    let _ = std::any::type_name::<LetDirective>();
    let _ = std::any::type_name::<Ident>();
}

#[test]
fn module_and_template_enums_match_shipped_variants() {
    let _ = classify_module_item as fn(&ModuleItem) -> &'static str;
    let _ = classify_template_item as fn(&TemplateItem) -> &'static str;
}
