use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use okf::{
    InspectKind, KnowledgeFilter, Profile, TrustTier, build, check, inspect_filtered, load, search,
};

fn temp(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("okf-engine-test-{name}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn valid_rocci_concept(id: &str, extra_yaml: &str, body: &str) -> String {
    format!(
        "---\ntype: Architecture\ntitle: {id}\ndescription: Test concept {id}.\ntags: [domain/rocci, concern/architecture]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-08-17T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\n{extra_yaml}---\n\n# {id}\n\n{body}\n"
    )
}

#[test]
fn test_okf_profile_matrix() {
    let root = temp("profiles");

    fs::write(
        root.join("minimal.md"),
        "---\ntype: Note\ntitle: Minimal\n---\n\n# Minimal\n\nBody text.\n",
    )
    .unwrap();

    let base_bundle = load(&root, Profile::Base).expect("load base");
    assert!(
        !base_bundle.has_errors(),
        "Base profile should accept minimal record"
    );

    let rocci_bundle = load(&root, Profile::Rocci).expect("load rocci");
    assert!(
        rocci_bundle.has_errors(),
        "Rocci profile should reject minimal record lacking tags/owners"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_okf_unknown_metadata_and_body_offsets() {
    let root = temp("metadata");
    fs::write(
        root.join("concept.md"),
        valid_rocci_concept(
            "Concept",
            "custom_tool_metadata:\n  accuracy: 0.99\n  reviewed_by: \"bot\"\n",
            "This is the body content.\n",
        ),
    )
    .unwrap();

    let bundle = load(&root, Profile::Rocci).expect("load bundle");
    assert_eq!(bundle.concepts.len(), 1);
    let concept = &bundle.concepts[0];

    // Verify custom YAML metadata is preserved
    assert_eq!(concept.metadata["custom_tool_metadata"]["accuracy"], 0.99);
    assert_eq!(
        concept.metadata["custom_tool_metadata"]["reviewed_by"],
        "bot"
    );

    // Verify body offsets
    assert!(concept.body_span.start > 0);
    assert!(concept.body_span.end > concept.body_span.start);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_okf_filter_permutations_and_search() {
    let root = temp("filters");
    fs::write(
        root.join("concept_a.md"),
        valid_rocci_concept(
            "Alpha",
            "verified:\n  - { by: human:nils, at: 2026-08-17T00:00:00Z }\nstale_after: 2099-01-01\n",
            "Explains template lowering and routing algorithm.\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("concept_b.md"),
        valid_rocci_concept(
            "Beta",
            "stale_after: 2000-01-01\n",
            "Explains parser recovery behavior.\n",
        ),
    )
    .unwrap();

    // Filter by human-reviewed trust tier and non-stale
    let filter = KnowledgeFilter {
        types: vec!["Architecture".into()],
        tags: vec!["domain/rocci".into()],
        statuses: vec!["draft".into()],
        authorities: vec!["descriptive".into()],
        trust_tiers: vec![TrustTier::HumanReviewed],
        stale: Some(false),
    };

    let catalog = inspect_filtered(&root, InspectKind::Catalog, None, Profile::Rocci, &filter)
        .expect("inspect filtered");
    assert!(catalog.contains("Alpha"), "catalog should contain Alpha");
    assert!(
        !catalog.contains("Beta"),
        "catalog should exclude stale/unverified Beta"
    );

    // Search query with filter
    let results = search(&root, "routing algorithm", Profile::Rocci, &filter).expect("search");
    assert!(results.contains("Alpha"), "search should find Alpha");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_okf_deterministic_build_artifacts() {
    let root = temp("build-artifacts");
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge Base\n",
    )
    .unwrap();
    fs::write(
        root.join("record.md"),
        valid_rocci_concept("Record", "", "Detailed architecture decisions.\n"),
    )
    .unwrap();

    let dist = root.join("dist");
    let summary = build(&root, &dist, Profile::Rocci).expect("build okf");
    assert_eq!(summary.concepts, 1);

    assert!(dist.join("catalog.json").is_file());
    assert!(dist.join("search.json").is_file());
    assert!(dist.join("llms.txt").is_file());
    assert!(dist.join("validation.json").is_file());

    let catalog_1 = fs::read_to_string(dist.join("catalog.json")).unwrap();
    let search_1 = fs::read_to_string(dist.join("search.json")).unwrap();

    // Repeat build and verify exact byte-identity
    build(&root, &dist, Profile::Rocci).expect("rebuild okf");
    let catalog_2 = fs::read_to_string(dist.join("catalog.json")).unwrap();
    let search_2 = fs::read_to_string(dist.join("search.json")).unwrap();

    assert_eq!(catalog_1, catalog_2, "catalog.json must be deterministic");
    assert_eq!(search_1, search_2, "search.json must be deterministic");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_okf_rejects_declarations_and_raw_html() {
    let root = temp("rejects");
    fs::write(
        root.join("bad_decl.md"),
        valid_rocci_concept(
            "BadDecl",
            "",
            "Some text\n\n@render {\n  Html.text(\"forbidden\")\n}\n",
        ),
    )
    .unwrap();

    let report = check(&root, Profile::Rocci).expect("check");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "OKF2007" && d.message.contains("@render")),
        "should reject @render declaration"
    );

    let _ = fs::remove_dir_all(root);
}
