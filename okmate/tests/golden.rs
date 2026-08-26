use std::fs;
use std::path::Path;

use okmate::views::{Document, NavNode, ReviewRow, TocEntry};

fn golden(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden")
            .join(name),
    )
    .unwrap()
}

fn assert_contains_golden(html: &str, fixture: &str) {
    for line in golden(fixture).lines() {
        let needle = line.trim();
        if needle.is_empty() {
            continue;
        }
        assert!(
            html.contains(needle),
            "missing {needle} from {fixture}: {html}"
        );
    }
}

fn sample() -> Document {
    Document {
        title: "Hello".into(),
        page_kind: "page".into(),
        nav: vec![NavNode {
            href: "/".into(),
            title: "Dashboard".into(),
            current: true,
            open: false,
            children: Vec::new(),
        }],
        toc: vec![TocEntry {
            id: "section".into(),
            text: "Section".into(),
            level: 2,
        }],
        article_html: "<h1>Hello</h1>".into(),
        concept_type: "Architecture".into(),
        status: "draft".into(),
        authority: "descriptive".into(),
        review_rows: vec![ReviewRow {
            href: "/hello/".into(),
            title: "Hello".into(),
            id: "hello".into(),
            status: "draft".into(),
            action: "Clean".into(),
        }],
        message: String::new(),
        config_path: "~/.okmate/config.toml".into(),
        settings_roots: Vec::new(),
    }
}

#[test]
fn shell_landmarks_match_golden() {
    let html = sample().render_page().unwrap();
    assert_contains_golden(&html, "shell-landmarks.txt");
}

#[test]
fn settings_patch_matches_golden() {
    let mut document = sample();
    document.page_kind = "settings".into();
    let html = document.render_settings_fragment().unwrap();
    assert_contains_golden(&html, "settings-patch.txt");
    assert!(!html.to_ascii_lowercase().contains("<html"));
}

#[test]
fn queue_region_matches_golden() {
    let mut document = sample();
    document.page_kind = "review".into();
    let html = document.render_queue_fragment().unwrap();
    assert_contains_golden(&html, "queue-region.txt");
    assert!(!html.to_ascii_lowercase().contains("<html"));
}
