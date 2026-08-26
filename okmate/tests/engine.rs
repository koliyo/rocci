mod common;

use std::fs;
use std::process::Command;

use common::{okmate_bin, temp_dir, valid_rocci_concept, write_index};
use okf::{InspectKind, KnowledgeFilter, Profile};

fn write_two_concepts() -> std::path::PathBuf {
    let root = temp_dir("engine");
    write_index(&root);
    fs::write(
        root.join("alpha.md"),
        valid_rocci_concept(
            "Alpha",
            "verified:\n  - { by: human:nils, at: 2026-08-17T00:00:00Z }\nstale_after: 2099-01-01\n",
            "Explains template lowering and routing algorithm.\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("beta.md"),
        valid_rocci_concept(
            "Beta",
            "stale_after: 2000-01-01\n",
            "Explains parser recovery behavior.\n",
        ),
    )
    .unwrap();
    root
}

fn run_ok(args: &[&str]) -> String {
    let output = Command::new(okmate_bin()).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn parse_json(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).unwrap()
}

#[test]
fn inspect_catalog_concept_and_graph_match_okf() {
    let root = write_two_concepts();
    let root_s = root.to_str().unwrap();

    let catalog = run_ok(&["inspect", "--profile", "rocci", "catalog", root_s]);
    let expected = okf::inspect(&root, InspectKind::Catalog, None, Profile::Rocci).unwrap();
    assert_eq!(parse_json(&catalog), parse_json(&expected));

    let concept = run_ok(&["inspect", "concept", "alpha", root_s]);
    let expected =
        okf::inspect(&root, InspectKind::Concept, Some("alpha"), Profile::Rocci).unwrap();
    assert_eq!(parse_json(&concept), parse_json(&expected));

    let graph = run_ok(&["inspect", "graph", root_s]);
    let expected = okf::inspect(&root, InspectKind::Graph, None, Profile::Rocci).unwrap();
    assert_eq!(parse_json(&graph), parse_json(&expected));
}

#[test]
fn inspect_catalog_filters_match_okf() {
    let root = write_two_concepts();
    let root_s = root.to_str().unwrap();
    let stdout = run_ok(&[
        "inspect",
        "catalog",
        root_s,
        "--trust-tier",
        "human-reviewed",
        "--stale",
        "false",
    ]);
    let filter = KnowledgeFilter {
        trust_tiers: vec![okf::TrustTier::HumanReviewed],
        stale: Some(false),
        ..KnowledgeFilter::default()
    };
    let expected =
        okf::inspect_filtered(&root, InspectKind::Catalog, None, Profile::Rocci, &filter).unwrap();
    assert_eq!(parse_json(&stdout), parse_json(&expected));
    let catalog = parse_json(&stdout);
    let ids: Vec<&str> = catalog
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"alpha"));
    assert!(!ids.contains(&"beta"));
}

#[test]
fn search_matches_okf_json() {
    let root = write_two_concepts();
    let root_s = root.to_str().unwrap();
    let stdout = run_ok(&["search", "routing algorithm", root_s, "--profile", "rocci"]);
    let expected = okf::search(
        &root,
        "routing algorithm",
        Profile::Rocci,
        &KnowledgeFilter::default(),
    )
    .unwrap();
    assert_eq!(parse_json(&stdout), parse_json(&expected));
    assert!(stdout.contains("Alpha"));
}

#[test]
fn benchmark_matches_okf_report() {
    let root = write_two_concepts();
    let bench = root.join("bench.toml");
    fs::write(
        &bench,
        r#"version = 1
top_k = 5
minimum_hit_rate = 1.0

[[questions]]
id = "routing"
question = "Where is routing described?"
query = "routing algorithm"
expected_concepts = ["alpha"]
"#,
    )
    .unwrap();
    let stdout = run_ok(&[
        "benchmark",
        bench.to_str().unwrap(),
        root.to_str().unwrap(),
        "--profile",
        "rocci",
    ]);
    let expected = okf::benchmark_retrieval(&root, &bench, Profile::Rocci).unwrap();
    assert_eq!(
        parse_json(&stdout),
        parse_json(&serde_json::to_string_pretty(&expected).unwrap())
    );
    assert!(parse_json(&stdout)["threshold_met"].as_bool().unwrap());
}
