use super::*;
use crate::{CompileOptions, compile};
use std::time::Duration;

fn compile_src(src: &str) -> crate::CompileOutput {
    compile(
        SourceFile::new("guide.rocdown", src),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    )
}

fn load(src: &str) -> (PageDocs, Vec<CatalogDiagnostic>) {
    let compiled = compile_src(src);
    assert!(!compiled.has_errors(), "{:?}", compiled.diagnostics);
    let mut diagnostics = Vec::new();
    let docs = load_page_docs(
        SourceFile::new("guide.rocdown", src),
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root: Path::new("."),
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    (docs, diagnostics)
}

#[test]
fn note_projects_markdown_and_search() {
    let (docs, diagnostics) =
        load("# Guide\n\n:note[title: \"Watch\"] {{\n    See the [next](/next/).\n}}\n");
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    assert!(collect_links(&docs.article).contains(&"/next/".to_string()));
    let markdown = markdown_fragment(&docs.article);
    assert!(markdown.contains("**Watch:**"), "{markdown}");
    assert!(search_text(&docs.article).contains("next"));
}

#[test]
fn unknown_kind_is_rd2401() {
    let compiled = compile_src(":widget Hi\n");
    assert!(
        compiled
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("unknown article kind `:widget`")),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn authored_h2_is_a_valid_sugar_kind() {
    let (docs, diagnostics) = load(":h2 Title\n");
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    assert!(
        collect_headings(&docs.article)
            .iter()
            .any(|heading| heading.text == "Title")
    );
}

#[test]
fn details_requires_summary_from_registry() {
    let compiled = compile_src(":details Body\n");
    assert!(
        compiled.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error() && diagnostic.message.contains("`:details` requires `summary`")
        }),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn step_requires_steps_parent_from_registry() {
    let compiled = compile_src(":step[title: \"One\"] Do it.\n");
    assert!(
        compiled.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("`:step` is only valid inside `:steps`")
        }),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn tabs_reject_stray_markdown_from_child_policy() {
    let compiled = compile_src(
        ":tabs.begin[group: \"os\", kind: \"platform\"]\n    A stray paragraph.\n    :tab[id: \"mac\", label: \"macOS\"] Hello.\n:tabs.end\n",
    );
    assert!(
        compiled.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("`:tabs` cannot contain Markdown")
        }),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn aside_forbids_tabs_from_child_policy() {
    let compiled = compile_src(
        ":note.begin\n    :tabs.begin[group: \"os\", kind: \"platform\"]\n        :tab[id: \"mac\", label: \"macOS\"] Hello.\n    :tabs.end\n:note.end\n",
    );
    assert!(
        compiled.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("`:note` cannot contain `:tabs`")
        }),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn card_grid_requires_link_card_from_child_policy() {
    let compiled = compile_src(":card-grid.begin\n:card-grid.end\n");
    assert!(!compiled.has_errors(), "{:?}", compiled.diagnostics);
    let mut diagnostics = Vec::new();
    let _docs = load_page_docs(
        SourceFile::new("guide.rocdown", ":card-grid.begin\n:card-grid.end\n"),
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root: Path::new("."),
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("`:card-grid` requires `:link-card` children")
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn card_grid_rejects_non_card_children() {
    let compiled = compile_src(
        ":card-grid.begin\n    :note Aside.\n    :link-card[href: \"https://example.com\"]\n:card-grid.end\n",
    );
    assert!(
        compiled.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("`:card-grid` cannot contain `:note`")
        }),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn api_operation_is_rd2406() {
    let compiled = compile_src(":api-operation[id: \"get\"]\n");
    assert!(
        compiled.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic
                    .message
                    .contains("`:api-operation` is not an authorable article kind")
        }),
        "{:?}",
        compiled.diagnostics
    );
}

#[test]
fn untested_example_warns() {
    let (_docs, diagnostics) =
        load(":example[language: \"sh\"] {{\n    ```sh\n    echo hi\n    ```\n}}\n");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "RD2601")
    );
    assert!(!diagnostics.iter().any(CatalogDiagnostic::is_error));
}

#[test]
fn tabs_project_all_panels_and_omit_outline_headings() {
    let (docs, diagnostics) = load(
        "# Guide\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] {{\n        ## Inside\n\n        Mac panel.\n    }}\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n",
    );
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    let markdown = markdown_fragment(&docs.article);
    assert!(markdown.contains("### macOS"), "{markdown}");
    assert!(markdown.contains("Linux panel"), "{markdown}");
    assert!(
        !collect_headings(&docs.article)
            .iter()
            .any(|heading| heading.text == "Inside")
    );
}

#[test]
fn sugar_and_colon_headings_share_outline() {
    let (docs, diagnostics) =
        load("# Guide\n\n## Install\n\n:h2[id: \"from-source\"] Building from source\n");
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    let headings = collect_headings(&docs.article);
    let ids: Vec<_> = headings.iter().map(|heading| heading.id.as_str()).collect();
    assert_eq!(ids, ["guide", "install", "from-source"]);
    let html = render_article(&docs.article);
    assert!(html.contains("id=\"install\""), "{html}");
    assert!(html.contains("id=\"from-source\""), "{html}");
    assert!(html.contains("<h2 class=\"rd-header-2\""), "{html}");
    assert!(
        !html.contains("<p class=\"rd-paragraph\">Building from source"),
        "{html}"
    );
}

#[test]
fn figure_with_markdown_image_does_not_require_figure_alt() {
    let (_docs, diagnostics) = load(":figure {{\n    ![x](/x.png)\n}}\n");
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
}

#[test]
fn figure_level_alt_is_unknown() {
    let (_docs, diagnostics) = load(":figure[alt: \"Diagram\"] {{\n    ![x](/x.png)\n}}\n");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "RD2402" && diagnostic.message.contains("unknown")
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn include_source_code_has_line_anchors() {
    let root =
        std::env::temp_dir().join(format!("rocdown-docs-line-anchor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("snippet.rocci"),
        "@component Card = || { <p/> }\n",
    )
    .unwrap();
    let src = ":include[path: \"snippet.rocci\"]\n";
    let compiled = compile(
        SourceFile::new("guide.rocdown", src),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    let mut diagnostics = Vec::new();
    let docs = load_page_docs(
        SourceFile::new(root.join("guide.rocdown").to_str().unwrap(), src),
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root: &root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    let html = render_article(&docs.article);
    assert!(html.contains("rd-source-code"), "{html}");
    assert!(html.contains("id=\"L1\""), "{html}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn declaration_heading_links_to_included_source_line() {
    let root = std::env::temp_dir().join(format!("rocdown-docs-decl-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("snippet.rocci"),
        "@component Card = || { <p/> }\n",
    )
    .unwrap();
    let src = "@page { layout: \"docs\" }\n\n## Declarations\n\n### `@component Card` · [#L1](#L1)\n\n:include[path: \"snippet.rocci\"]\n";
    let compiled = compile(
        SourceFile::new("guide.rocdown", src),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    let mut diagnostics = Vec::new();
    let docs = load_page_docs(
        SourceFile::new(root.join("guide.rocdown").to_str().unwrap(), src),
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root: &root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    let html = render_article(&docs.article);
    assert!(html.contains("href=\"#L1\""), "{html}");
    assert!(html.contains("id=\"component-card-l1\""), "{html}");
    assert!(
        html.contains("<span class=\"rd-source-line\" id=\"L1\">"),
        "{html}"
    );
    assert!(
        !html.contains("<h3 class=\"rd-header-3\" id=\"L1\">"),
        "{html}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn include_reads_region_and_warns_on_line_range() {
    let root = std::env::temp_dir().join(format!("rocdown-docs-include-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("snippet.rs"),
        "// docs-region: install\nfn install() {}\n// docs-region-end: install\nfn other() {}\n",
    )
    .unwrap();
    let src = ":include[path: \"snippet.rs\", region: \"install\"]\n";
    let compiled = compile(
        SourceFile::new("guide.rocdown", src),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    let mut diagnostics = Vec::new();
    let docs = load_page_docs(
        SourceFile::new(root.join("guide.rocdown").to_str().unwrap(), src),
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root: &root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
    let markdown = markdown_fragment(&docs.article);
    assert!(markdown.contains("fn install()"), "{markdown}");
    assert!(!markdown.contains("fn other()"), "{markdown}");
    assert!(markdown.contains("Source: snippet.rs"), "{markdown}");

    let ranged = ":include[path: \"snippet.rs\", start: 1, end: 2]\n";
    let compiled = compile(
        SourceFile::new("guide.rocdown", ranged),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    let mut diagnostics = Vec::new();
    load_page_docs(
        SourceFile::new(root.join("guide.rocdown").to_str().unwrap(), ranged),
        &compiled.document,
        "guide.rocdown",
        IncludeOptions {
            root: &root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "RD2504")
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn cyclic_include_is_rd2505() {
    let root = std::env::temp_dir().join(format!("rocdown-docs-cycle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.rocdown"), ":include[path: \"b.rocdown\"]\n").unwrap();
    std::fs::write(root.join("b.rocdown"), ":include[path: \"a.rocdown\"]\n").unwrap();
    let src = std::fs::read_to_string(root.join("a.rocdown")).unwrap();
    let compiled = compile(
        SourceFile::new("a.rocdown", &src),
        &CompileOptions {
            resolve_links: false,
            ..CompileOptions::default()
        },
    );
    let mut diagnostics = Vec::new();
    load_page_docs(
        SourceFile::new(root.join("a.rocdown").to_str().unwrap(), &src),
        &compiled.document,
        "a.rocdown",
        IncludeOptions {
            root: &root,
            snippet_roots: &[],
        },
        &mut diagnostics,
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "RD2505")
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn echo_example_matches_expect() {
    let examples = [ExampleRecord {
        id: "echo".into(),
        language: "sh".into(),
        test: vec!["/bin/echo".into(), "hello".into()],
        expect: Some("hello".into()),
        origin: IncludeOrigin {
            source_path: "guide.rocdown".into(),
            ..IncludeOrigin::default()
        },
        line: 1,
        ..ExampleRecord::default()
    }];
    let diagnostics = run_examples(
        &examples,
        &ExampleTestOptions {
            root: PathBuf::from("."),
            timeout: Duration::from_secs(5),
            allow_network: false,
            update: false,
        },
    );
    assert!(
        !diagnostics.iter().any(CatalogDiagnostic::is_error),
        "{diagnostics:?}"
    );
}

#[test]
fn body_only_docs_edit_keeps_segment_paths() {
    let first = load("# Guide\n\n:note[title: \"Watch\"] First.\n").0;
    let second = load("# Guide\n\n:note[title: \"Watch\"] Second paragraph.\n").0;
    let rewrite = BTreeMap::new();
    let (first_segs, first_files) = plan_segments("Page", &first.article, &rewrite);
    let (second_segs, second_files) = plan_segments("Page", &second.article, &rewrite);
    let paths = |segs: &[PlannedNode]| {
        segs.iter()
            .map(|seg| match seg {
                PlannedNode::Html { path } => ("html".to_string(), String::new(), path.clone()),
                PlannedNode::Widget(widget) => (
                    "widget".to_string(),
                    widget.kind.clone(),
                    widget.str_prop("title").unwrap_or_default().to_string(),
                ),
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(paths(&first_segs), paths(&second_segs));
    assert_ne!(first_files, second_files);
}

#[test]
fn footnotes_render_in_accessible_section() {
    let (docs, diags) = load("Claim.[^source]\n\n[^source]: Evidence.\n");
    assert!(!diags.iter().any(CatalogDiagnostic::is_error), "{diags:?}");
    let html = render_article(&docs.article);
    assert!(html.contains("data-footnote-ref"), "{html}");
    assert!(html.contains("aria-label=\"Footnotes\""), "{html}");
    assert!(html.contains("data-footnote-backref"), "{html}");
    assert!(html.contains("id=\"fn-source\""), "{html}");
    assert!(html.contains("id=\"fnref-source\""), "{html}");
    assert!(html.contains("Evidence."), "{html}");
}

#[test]
fn img_decl_preserves_all_fields() {
    let (docs, diags) = load(
        "# Guide\n\n:img[src: \"img/banner.png\", alt: \"Banner\", title: \"Hero\", width: \"300px\", height: \"120px\", class: \"hero\", loading: \"lazy\", decoding: \"async\"]\n",
    );
    assert!(!diags.iter().any(CatalogDiagnostic::is_error), "{diags:?}");
    assert_eq!(collect_images(&docs.article), vec!["img/banner.png"]);
    let html = render_article(&docs.article);
    assert!(html.contains("src=\"img/banner.png\""), "{html}");
    assert!(html.contains("alt=\"Banner\""), "{html}");
    assert!(html.contains("title=\"Hero\""), "{html}");
    assert!(html.contains("width=\"300px\""), "{html}");
    assert!(html.contains("height=\"120px\""), "{html}");
    assert!(html.contains("class=\"rd-image hero\""), "{html}");
    assert!(html.contains("loading=\"lazy\""), "{html}");
    assert!(html.contains("decoding=\"async\""), "{html}");
}

#[test]
fn figure_preserves_caption_credit_and_nested_img_fields() {
    let (docs, diags) = load(
        ":figure[caption: \"Architecture\", credit: \"Rocci docs\"] {{\n    :img[src: \"diagram.png\", alt: \"Diagram\", width: \"400px\", loading: \"lazy\", decoding: \"async\"]\n}}\n",
    );
    assert!(!diags.iter().any(CatalogDiagnostic::is_error), "{diags:?}");
    let (segments, _fragments) = plan_segments("fig", &docs.article, &Default::default());
    let figure = segments
        .iter()
        .find_map(|node| match node {
            PlannedNode::Widget(widget) if widget.kind == "figure" => Some(widget),
            _ => None,
        })
        .expect("missing figure widget");
    assert_eq!(figure.str_prop("caption"), Some("Architecture"));
    assert_eq!(figure.str_prop("credit"), Some("Rocci docs"));
    let html = render_article(&docs.article);
    assert!(html.contains("src=\"diagram.png\""), "{html}");
    assert!(html.contains("alt=\"Diagram\""), "{html}");
    assert!(html.contains("width=\"400px\""), "{html}");
    assert!(html.contains("loading=\"lazy\""), "{html}");
    assert!(html.contains("decoding=\"async\""), "{html}");
}
