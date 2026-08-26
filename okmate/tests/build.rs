mod common;

use std::fs;
use std::process::Command;

use common::{okmate_bin, temp_dir, valid_rocci_concept, write_index};

#[test]
fn build_writes_engine_catalog_html_landmarks_and_pages_json() {
    let root = temp_dir("build-src");
    write_index(&root);
    fs::write(
        root.join("hello.md"),
        valid_rocci_concept(
            "Hello",
            "",
            "Intro paragraph.\n\n## Details\n\nMore about the concept.\n",
        ),
    )
    .unwrap();
    let output = temp_dir("build-out");

    let status = Command::new(okmate_bin())
        .arg("build")
        .arg(&root)
        .arg("-o")
        .arg(&output)
        .arg("--profile")
        .arg("rocci")
        .status()
        .unwrap();
    assert!(status.success());

    assert!(output.join("catalog.json").is_file());
    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("catalog.json")).unwrap()).unwrap();
    assert!(catalog.is_array());
    assert_eq!(catalog.as_array().unwrap().len(), 1);

    let home = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(home.contains("id=\"okmate-nav\""), "{home}");
    assert!(home.contains("id=\"okmate-main\""), "{home}");

    let concept = fs::read_to_string(output.join("hello").join("index.html")).unwrap();
    assert!(concept.contains("id=\"okmate-nav\""), "{concept}");
    assert!(concept.contains("id=\"okmate-main\""), "{concept}");
    assert!(concept.contains("id=\"okmate-toc\""), "{concept}");
    assert!(concept.contains("Details"));

    let review = fs::read_to_string(output.join("review").join("index.html")).unwrap();
    assert!(review.contains("id=\"okmate-queue\""));

    let settings = fs::read_to_string(output.join("settings").join("index.html")).unwrap();
    assert!(settings.contains("id=\"okmate-settings\""));

    assert!(output.join("__okmate").join("app.css").is_file());
    let pages: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
    let routes: Vec<&str> = pages
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page["route"].as_str().unwrap())
        .collect();
    assert!(routes.contains(&"/"));
    assert!(routes.contains(&"/hello/"));
    assert!(routes.contains(&"/review/"));
    assert!(routes.contains(&"/settings/"));
}
