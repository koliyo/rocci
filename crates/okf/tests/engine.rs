use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use okf::{
    InspectKind, KnowledgeFilter, LoadOptions, ParseCache, Profile, TrustTier, build, check,
    inspect_filtered, load, load_timed, load_with_cache, search,
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
fn invalid_yaml_still_captures_markdown_body() {
    let root = temp("yaml-recovery");
    fs::write(
        root.join("broken.md"),
        "---\ntype: Note\ntitle: [unclosed\n---\n\n# Broken\n\nRecovered body.\n",
    )
    .unwrap();

    let bundle = load(&root, Profile::Base).expect("load");
    assert!(bundle.has_errors(), "{:?}", bundle.diagnostics);
    assert_eq!(bundle.concepts.len(), 1);
    assert!(
        bundle.concepts[0].article_html.contains("Recovered body"),
        "{}",
        bundle.concepts[0].article_html
    );
    assert!(
        bundle
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "OKF1003")
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

#[test]
fn resolve_preview_path_opens_bundle_and_concept() {
    let root = temp("preview");
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
    )
    .unwrap();
    let plans = root.join("plans");
    fs::create_dir(&plans).unwrap();
    fs::write(plans.join("index.md"), "# Plans\n").unwrap();
    fs::write(
        plans.join("cli-entry-points.md"),
        "---\ntype: Implementation Plan\ntitle: CLI\nauthority: exploratory\n---\n\n# CLI\n",
    )
    .unwrap();

    let bundle = okf::resolve_preview_path(&root).unwrap();
    assert_eq!(bundle.open_path, "/");
    assert_eq!(bundle.root, fs::canonicalize(&root).unwrap());

    let home = okf::resolve_preview_path(&root.join("index.md")).unwrap();
    assert_eq!(home.open_path, "/");

    let concept = okf::resolve_preview_path(&plans.join("cli-entry-points.md")).unwrap();
    assert_eq!(concept.open_path, "/plans/cli-entry-points/");
    assert_eq!(concept.root, fs::canonicalize(&root).unwrap());

    let collection = okf::resolve_preview_path(&plans.join("index.md")).unwrap_err();
    assert!(collection.to_string().contains("collection index"));

    let outside = temp("preview-outside");
    fs::write(outside.join("notes.md"), "# Notes\n").unwrap();
    let missing = okf::resolve_preview_path(&outside.join("notes.md")).unwrap_err();
    assert!(missing.to_string().contains("not inside an OKF bundle"));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

#[test]
fn article_html_rewrites_bundle_root_links() {
    let root = temp("rewrite-links");
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n\n* [Architecture](architecture/)\n",
    )
    .unwrap();
    let architecture = root.join("architecture");
    fs::create_dir(&architecture).unwrap();
    fs::write(architecture.join("index.md"), "# Architecture\n").unwrap();
    fs::write(
        architecture.join("overview.md"),
        valid_rocci_concept(
            "Overview",
            "",
            "See the [decision](/decisions/choice.md#context) and [relative](../decisions/choice.md).\n\nThe [matrix](../migration-matrix.tsv) stays a file link.\n",
        ),
    )
    .unwrap();
    let decisions = root.join("decisions");
    fs::create_dir(&decisions).unwrap();
    fs::write(decisions.join("index.md"), "# Decisions\n").unwrap();
    fs::write(
        decisions.join("choice.md"),
        valid_rocci_concept("Choice", "", "## Context\n\nAccepted.\n"),
    )
    .unwrap();
    fs::write(root.join("migration-matrix.tsv"), "id\tpath\n").unwrap();

    let bundle = load(&root, Profile::Rocci).expect("load bundle");
    let overview = bundle
        .concepts
        .iter()
        .find(|concept| concept.id == "architecture/overview")
        .expect("overview concept");
    assert!(
        overview
            .links
            .iter()
            .any(|link| link.url == "/decisions/choice.md#context"),
        "authored bundle-root href should stay on concept.links"
    );
    assert!(
        overview
            .links
            .iter()
            .any(|link| link.url == "../decisions/choice.md"),
        "authored relative href should stay on concept.links"
    );
    assert!(
        overview
            .article_html
            .contains("href=\"/decisions/choice/#context\"")
    );
    assert!(
        overview
            .article_html
            .contains("href=\"/decisions/choice/\"")
    );
    assert!(
        !overview
            .article_html
            .contains("href=\"/decisions/choice.md")
    );
    assert!(
        overview
            .article_html
            .contains("href=\"../migration-matrix.tsv\"")
    );

    let home = bundle
        .indexes
        .iter()
        .find(|index| index.path == "index.md")
        .expect("root index");
    assert!(home.article_html.contains("href=\"/architecture/\""));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_timed_records_nonzero_parse_on_tiny_fixture() {
    let root = temp("load-timings");
    fs::write(
        root.join("note.md"),
        "---\ntype: Note\ntitle: Timing\n---\n\n# Timing\n\nBody text for parse timing.\n",
    )
    .unwrap();

    let loaded = load_timed(&root, LoadOptions::new(Profile::Base)).expect("load timed base");
    assert!(
        loaded.timings.parse > std::time::Duration::ZERO,
        "parse span should be non-zero even on a tiny fixture"
    );
    assert_eq!(loaded.bundle.concepts.len(), 1);
    assert_eq!(loaded.timings.provenance, None);

    let rocci = load_timed(&root, LoadOptions::new(Profile::Rocci)).expect("load timed rocci");
    assert!(rocci.timings.provenance.is_some());

    let preview = load_timed(
        &root,
        LoadOptions::new(Profile::Rocci).with_provenance(false),
    )
    .expect("load timed rocci without provenance");
    assert_eq!(preview.timings.provenance, Some(std::time::Duration::ZERO));
    assert!(
        preview.bundle.has_errors(),
        "Rocci schema should still reject a minimal record when provenance is off"
    );
    assert!(!preview.bundle.diagnostics.iter().any(|diagnostic| {
        diagnostic.code.starts_with("OKF4006")
            || diagnostic.code == "OKF4007"
            || diagnostic.code == "OKF4008"
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn parse_cache_skips_unchanged_files_on_second_load() {
    let root = temp("parse-cache");
    fs::write(
        root.join("a.md"),
        valid_rocci_concept("A", "", "Alpha body.\n"),
    )
    .unwrap();
    fs::write(
        root.join("b.md"),
        valid_rocci_concept("B", "", "Beta body.\n"),
    )
    .unwrap();
    fs::write(
        root.join("c.md"),
        valid_rocci_concept("C", "", "Gamma body.\n"),
    )
    .unwrap();

    let mut cache = ParseCache::new();
    let options = LoadOptions::new(Profile::Base);
    let first = load_with_cache(&root, options, Some(&mut cache)).expect("first load");
    assert_eq!(first.timings.parse_cache_hits, 0);
    assert!(first.timings.parse_cache_misses >= 3);

    let second = load_with_cache(&root, options, Some(&mut cache)).expect("second load");
    assert!(
        second.timings.parse_cache_hits >= 3,
        "unchanged files should be cache hits: {:?}",
        second.timings
    );
    assert_eq!(second.timings.parse_cache_misses, 0);

    fs::write(
        root.join("b.md"),
        valid_rocci_concept("B", "", "Beta body changed.\n"),
    )
    .unwrap();
    let third = load_with_cache(&root, options, Some(&mut cache)).expect("third load");
    assert_eq!(third.timings.parse_cache_misses, 1);
    assert!(third.timings.parse_cache_hits >= 2);
    assert!(
        third
            .bundle
            .concepts
            .iter()
            .any(|concept| concept.article_html.contains("Beta body changed"))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn parallel_load_is_deterministic_across_runs() {
    let root = temp("parallel-parse");
    for name in ["a", "b", "c", "d", "e"] {
        fs::write(
            root.join(format!("{name}.md")),
            valid_rocci_concept(name, "", &format!("{name} body.\n")),
        )
        .unwrap();
    }

    let first = load(&root, Profile::Base).expect("first load");
    let second = load(&root, Profile::Base).expect("second load");
    let ids: Vec<_> = first
        .concepts
        .iter()
        .map(|concept| concept.id.as_str())
        .collect();
    let ids_again: Vec<_> = second
        .concepts
        .iter()
        .map(|concept| concept.id.as_str())
        .collect();
    assert_eq!(ids, ids_again);
    let codes: Vec<_> = first
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.path.as_str(), diagnostic.code))
        .collect();
    let codes_again: Vec<_> = second
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.path.as_str(), diagnostic.code))
        .collect();
    assert_eq!(codes, codes_again);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn parse_cache_roundtrips_through_a_directory() {
    let root = temp("parse-cache-disk-root");
    fs::write(
        root.join("a.md"),
        valid_rocci_concept("A", "", "Alpha body.\n"),
    )
    .unwrap();
    fs::write(
        root.join("b.md"),
        valid_rocci_concept("B", "", "Beta body.\n"),
    )
    .unwrap();
    fs::write(
        root.join("c.md"),
        valid_rocci_concept("C", "", "Gamma body.\n"),
    )
    .unwrap();

    let store = temp("parse-cache-disk-store");
    let mut first = ParseCache::new();
    let options = LoadOptions::new(Profile::Base);
    let loaded = load_with_cache(&root, options, Some(&mut first)).expect("populate cache");
    assert_eq!(loaded.timings.parse_cache_hits, 0);
    first.save_dir(&store).expect("save parse cache");

    let mut restored = ParseCache::load_dir(&store, Profile::Base);
    let second = load_with_cache(&root, options, Some(&mut restored)).expect("restored cache");
    assert!(
        second.timings.parse_cache_hits >= 3,
        "directory cache should hit unchanged files: {:?}",
        second.timings
    );
    assert_eq!(second.timings.parse_cache_misses, 0);
    assert_eq!(second.bundle.concepts.len(), loaded.bundle.concepts.len());

    fs::write(
        root.join("b.md"),
        valid_rocci_concept("B", "", "Beta body changed.\n"),
    )
    .unwrap();
    let third = load_with_cache(&root, options, Some(&mut restored)).expect("edited file");
    assert_eq!(third.timings.parse_cache_misses, 1);
    assert!(third.timings.parse_cache_hits >= 2);

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(store);
}

fn write_bundle_root(root: &std::path::Path) {
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
    )
    .unwrap();
}

#[test]
fn nested_concept_loads_and_preview_opens() {
    let root = temp("nested-concept");
    write_bundle_root(&root);
    let area = root.join("plans").join("okf");
    fs::create_dir_all(&area).unwrap();
    fs::write(
        root.join("plans").join("index.md"),
        "# Plans\n\n* [OKF](okf/)\n",
    )
    .unwrap();
    fs::write(
        area.join("index.md"),
        "# OKF\n\n* [Nested collections](nested-collections.md)\n",
    )
    .unwrap();
    fs::write(
        area.join("nested-collections.md"),
        valid_rocci_concept("Nested", "", "Nested body.\n"),
    )
    .unwrap();

    let bundle = load(&root, Profile::Base).expect("load nested");
    assert!(
        bundle
            .concepts
            .iter()
            .any(|concept| concept.id == "plans/okf/nested-collections")
    );
    let preview = okf::resolve_preview_path(&area.join("nested-collections.md")).unwrap();
    assert_eq!(preview.open_path, "/plans/okf/nested-collections/");

    let json = inspect_filtered(
        &root,
        InspectKind::Concept,
        Some("nested-collections"),
        Profile::Base,
        &KnowledgeFilter::default(),
    )
    .expect("inspect stem");
    assert!(json.contains("plans/okf/nested-collections"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn concept_id_colliding_with_collection_is_an_error() {
    let root = temp("route-collision");
    write_bundle_root(&root);
    let plans = root.join("plans");
    fs::create_dir_all(plans.join("rocci")).unwrap();
    fs::write(plans.join("index.md"), "# Plans\n").unwrap();
    fs::write(plans.join("rocci").join("index.md"), "# Rocci\n").unwrap();
    fs::write(
        plans.join("rocci.md"),
        "---\ntype: Note\ntitle: Rocci\n---\n\n# Rocci\n",
    )
    .unwrap();

    let bundle = load(&root, Profile::Base).expect("load collision");
    assert!(
        bundle
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "OKF3005"),
        "{:?}",
        bundle.diagnostics
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rocci_profile_warns_when_nearest_index_omits_a_concept() {
    let root = temp("index-membership");
    write_bundle_root(&root);
    let area = root.join("plans").join("okf");
    fs::create_dir_all(&area).unwrap();
    fs::write(root.join("plans").join("index.md"), "# Plans\n").unwrap();
    fs::write(area.join("index.md"), "# OKF\n").unwrap();
    fs::write(
        area.join("nested-collections.md"),
        valid_rocci_concept("Nested", "", "Body.\n"),
    )
    .unwrap();

    let bundle = load_timed(
        &root,
        LoadOptions::new(Profile::Rocci).with_provenance(false),
    )
    .expect("load membership")
    .bundle;
    assert!(
        bundle.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "OKF2010" && diagnostic.path == "plans/okf/nested-collections.md"
        }),
        "{:?}",
        bundle.diagnostics
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn okf_scheme_links_are_not_bundle_paths() {
    let root = temp("okf-scheme");
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
    )
    .unwrap();
    fs::write(
        root.join("overview.md"),
        valid_rocci_concept(
            "Overview",
            "",
            "See [notes](okf:notes/plans/okf/nested-collections.md#goal).\n",
        ),
    )
    .unwrap();

    let bundle = load(&root, Profile::Rocci).expect("load");
    let overview = bundle
        .concepts
        .iter()
        .find(|concept| concept.id == "overview")
        .expect("overview");
    assert!(
        overview
            .links
            .iter()
            .any(|link| link.url == "okf:notes/plans/okf/nested-collections.md#goal"),
        "{:?}",
        overview.links
    );
    assert!(
        !bundle
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "OKF3001" || diagnostic.code == "OKF3002"),
        "{:?}",
        bundle.diagnostics
    );
    assert!(
        overview
            .article_html
            .contains("href=\"okf:notes/plans/okf/nested-collections.md#goal\"")
    );

    let _ = fs::remove_dir_all(root);
}
