use std::{fs, path::Path, time::Instant};

use rocci_ungram::{check_languages, find_workspace_root, format_inspect_mapping, generate_source};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn snapshot_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name)
}

fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_path(name);
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        expected,
        actual,
        "snapshot mismatch for {}\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
        path.display()
    );
}

fn generate_err(ungram: &str, sidecar: &str) -> String {
    generate_source(&fixture(ungram), &fixture(sidecar))
        .unwrap_err()
        .to_string()
}

#[test]
fn parses_and_snapshots_mini_ungram() {
    let source = generate_source(&fixture("mini.ungram"), &fixture("mini.toml")).unwrap();
    assert_snapshot("mini.generated.rs", &source);
}

#[test]
fn mini_generated_compiles() {
    let _ = std::any::type_name::<mini_ast::Document>();
    let _ = std::any::type_name::<mini_ast::Node>();
}

mod mini_ast {
    #![allow(dead_code)]

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Span {
        pub start: u32,
        pub end: u32,
    }

    include!("snapshots/mini.generated.rs");
}

#[test]
fn parses_and_snapshots_language_ungrams() {
    let root = find_workspace_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let rocci = generate_source(
        &fs::read_to_string(root.join(rocci_ungram::ROCCI_UNGRAM)).unwrap(),
        &fs::read_to_string(root.join(rocci_ungram::ROCCI_TOML)).unwrap(),
    )
    .unwrap();
    let rocdown = generate_source(
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_UNGRAM)).unwrap(),
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_TOML)).unwrap(),
    )
    .unwrap();
    assert!(rocci.contains("pub struct Document"));
    assert!(rocci.contains("pub enum ModuleItem"));
    assert!(rocdown.contains("pub enum Item"));
    assert!(rocdown.contains("Markdown(MdNode)"));
    assert!(!rocci.contains("fn parse"));
    assert!(!rocdown.contains("struct MdNode"));
    assert!(!rocdown.contains("struct ComponentDecl"));
    let md = generate_source(
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_MARKDOWN_UNGRAM)).unwrap(),
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_MARKDOWN_TOML)).unwrap(),
    )
    .unwrap();
    assert!(md.contains("pub enum MdNode"));
    assert!(md.contains("Heading {"));
    assert!(!md.contains("fn parse"));
    let committed_md = fs::read_to_string(root.join(rocci_ungram::ROCDOWN_MD_GENERATED)).unwrap();
    assert_eq!(
        committed_md,
        md,
        "committed {} is stale",
        rocci_ungram::ROCDOWN_MD_GENERATED
    );
    let committed_rocci = fs::read_to_string(root.join(rocci_ungram::ROCCI_GENERATED)).unwrap();
    assert_eq!(
        committed_rocci,
        rocci,
        "committed {} is stale",
        rocci_ungram::ROCCI_GENERATED
    );
    let committed_rocdown = fs::read_to_string(root.join(rocci_ungram::ROCDOWN_GENERATED)).unwrap();
    assert_eq!(
        committed_rocdown,
        rocdown,
        "committed {} is stale",
        rocci_ungram::ROCDOWN_GENERATED
    );
    let rocci_pprint = fs::read_to_string(root.join(rocci_ungram::ROCCI_PPRINT)).unwrap();
    let rocdown_pprint = fs::read_to_string(root.join(rocci_ungram::ROCDOWN_PPRINT)).unwrap();
    assert!(rocci_pprint.contains("fn write_module_item"));
    assert!(rocci_pprint.contains("match item"));
    assert!(rocdown_pprint.contains("fn write_item"));
    assert!(rocdown_pprint.contains("Item::Markdown"));
    let rocci_tree = fs::read_to_string(root.join(rocci_ungram::ROCCI_TREE_APPENDIX)).unwrap();
    let rocdown_tree = fs::read_to_string(root.join(rocci_ungram::ROCDOWN_TREE_APPENDIX)).unwrap();
    assert!(rocci_tree.contains("| `Document` | `module` | generated |"));
    assert!(rocdown_tree.contains("| `Document` | `rocdown` | generated |"));
    assert!(rocdown_tree.contains("| `Heading` | `h` | inline |"));
    assert!(rocdown_tree.contains("Markdown projection"));
}

#[test]
fn check_matches_committed_snapshots() {
    let root = find_workspace_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    check_languages(&root).unwrap();
}

#[test]
fn snapshots_inspect_mappings() {
    let root = find_workspace_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let rocci = format_inspect_mapping(
        &fs::read_to_string(root.join(rocci_ungram::ROCCI_UNGRAM)).unwrap(),
        &fs::read_to_string(root.join(rocci_ungram::ROCCI_TOML)).unwrap(),
    )
    .unwrap();
    let rocdown = format_inspect_mapping(
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_UNGRAM)).unwrap(),
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_TOML)).unwrap(),
    )
    .unwrap();
    assert_snapshot("rocci.inspect.txt", &rocci);
    assert_snapshot("rocdown.inspect.txt", &rocdown);
    let markdown = format_inspect_mapping(
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_MARKDOWN_UNGRAM)).unwrap(),
        &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_MARKDOWN_TOML)).unwrap(),
    )
    .unwrap();
    assert_snapshot("markdown.inspect.txt", &markdown);
}

#[test]
fn rejects_generated_production_without_inspect_mapping() {
    let err = generate_source(
        "Foo = 'foo'\n",
        "[generated]\nFoo = \"Foo\"\n[leaves]\nFoo = \"Span\"\n",
    )
    .unwrap_err()
    .to_string();
    assert!(
        err.contains("no inspect tag, omit, or fallback") && err.contains("Foo"),
        "{err}"
    );
}

#[test]
fn rejects_nested_anonymous_alternatives() {
    let err = generate_err("illegal_nested_alt.ungram", "illegal_nested_alt.toml");
    assert!(
        err.contains("nested anonymous alternative") || err.contains("alternatives in Item"),
        "{err}"
    );
}

#[test]
fn rejects_mixed_token_and_node_enums() {
    let err = generate_err("illegal_mixed.ungram", "illegal_mixed.toml");
    assert!(
        err.contains("mixed token and node") || err.contains("alternatives in Item"),
        "{err}"
    );
}

#[test]
fn rejects_unlabeled_repeated_tokens() {
    let err = generate_err("illegal_repeat.ungram", "illegal_repeat.toml");
    assert!(err.contains("unlabeled repeated token"), "{err}");
}

#[test]
fn rejects_leaf_without_sidecar_type() {
    let err = generate_err("illegal_leaf.ungram", "illegal_leaf.toml");
    assert!(err.contains("no [leaves] rust type"), "{err}");
}

#[test]
fn rejects_unclassified_production() {
    let err = generate_source("Foo = 'foo'\n", "[generated]\n").unwrap_err();
    assert!(err.to_string().contains("unclassified production Foo"));
}

#[test]
fn generate_is_subsecond() {
    let root = find_workspace_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let start = Instant::now();
    for _ in 0..20 {
        generate_source(
            &fs::read_to_string(root.join(rocci_ungram::ROCCI_UNGRAM)).unwrap(),
            &fs::read_to_string(root.join(rocci_ungram::ROCCI_TOML)).unwrap(),
        )
        .unwrap();
        generate_source(
            &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_UNGRAM)).unwrap(),
            &fs::read_to_string(root.join(rocci_ungram::ROCDOWN_TOML)).unwrap(),
        )
        .unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 800,
        "generating both ungrams 20 times took {elapsed:?}"
    );
}
