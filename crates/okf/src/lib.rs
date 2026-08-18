//! Portable Open Knowledge Format (OKF) parsing, validation, search, graph, and artifact engine.

pub mod artifact;
pub mod ast;
pub mod benchmark;
pub mod diagnostic;
pub mod frontmatter;
pub mod graph;
pub mod markdown;
pub mod preview;
pub mod review;
pub mod search;
pub mod validate;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;

pub use artifact::{
    ConceptInspect, absolute, build_artifacts, commit_output, llms_text, unique_temp,
};
pub use ast::{
    BuildSummary, Bundle, CheckReport, Concept, Edge, Heading, HeadingSection, Index, InspectKind,
    KnowledgeFilter, Link, Log, Profile, Span, TrustTier,
};
pub use benchmark::{
    RetrievalBenchmark, RetrievalQuestion, RetrievalQuestionResult, RetrievalReport, run_benchmark,
};
pub use diagnostic::{Diagnostic, Severity, SourceLocation};
pub use frontmatter::{
    Frontmatter, lines_with_offsets, location, parse_yaml_mapping, split_frontmatter,
};
pub use graph::{resolve_bundle_path, resolve_graph, split_fragment};
pub use markdown::{
    MarkdownOutput, footnote_labels, parse_markdown_body, reject_declarations, slugify,
};
pub use preview::{PreviewTarget, resolve_preview_path};
pub use review::{ActionKind, ConceptAction, classify_concept_action};
pub use search::{SearchChunk, matching_search_chunks, normalize_search_text, search_index};
pub use validate::{
    PROFILE_TYPES, STANDARD_FIELDS, collect_source_ids, current_utc_date, external_url,
    filesystem_modified_at, git_last_modified, git_path_dirty, git_repository_root, is_date,
    latest_human_verification, metadata_string_array, parse_timestamp, repository_source_path,
    string_field, validate_lifecycle_and_sources, validate_metadata, validate_optional_string,
    validate_unique_ids,
};

pub fn check(root: &Path, profile: Profile) -> Result<CheckReport> {
    let bundle = load(root, profile)?;
    Ok(CheckReport {
        diagnostics: bundle.diagnostics,
    })
}

pub fn load(root: &Path, profile: Profile) -> Result<Bundle> {
    let root = absolute(root)?;
    if !root.is_dir() {
        bail!("knowledge bundle {} is not a directory", root.display());
    }
    let mut paths = Vec::new();
    discover_markdown(&root, &mut paths)?;
    paths.sort();

    let mut concepts = Vec::new();
    let mut indexes = Vec::new();
    let mut logs = Vec::new();
    let mut diagnostics = Vec::new();

    for path in paths {
        let relative = relative_path(&root, &path);
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let source = match String::from_utf8(bytes) {
            Ok(source) => source,
            Err(_) => {
                diagnostics.push(Diagnostic::error(
                    "OKF1001",
                    relative,
                    None,
                    "document is not valid UTF-8",
                ));
                continue;
            }
        };
        match path.file_name().and_then(|name| name.to_str()) {
            Some("index.md") => {
                parse_index(&root, &relative, &source, &mut indexes, &mut diagnostics)
            }
            Some("log.md") => parse_log(&relative, &source, &mut logs, &mut diagnostics),
            _ => parse_concept(&relative, &source, profile, &mut concepts, &mut diagnostics),
        }
    }

    concepts.sort_by(|a, b| a.id.cmp(&b.id));
    indexes.sort_by(|a, b| a.path.cmp(&b.path));
    logs.sort_by(|a, b| a.path.cmp(&b.path));
    validate_unique_ids(&concepts, &mut diagnostics);
    let graph = resolve_graph(&concepts, &indexes, &mut diagnostics);
    if profile == Profile::Rocci {
        validate_lifecycle_and_sources(&root, &concepts, &mut diagnostics);
    }
    diagnostics.sort_by(|a, b| {
        (&a.path, a.location.as_ref().map(|span| span.start), a.code).cmp(&(
            &b.path,
            b.location.as_ref().map(|span| span.start),
            b.code,
        ))
    });
    let version = indexes
        .iter()
        .find(|index| index.path == "index.md")
        .and_then(|index| index.version.clone());

    Ok(Bundle {
        root,
        version,
        concepts,
        indexes,
        logs,
        graph,
        diagnostics,
    })
}

pub fn inspect(
    root: &Path,
    kind: InspectKind,
    concept_id: Option<&str>,
    profile: Profile,
) -> Result<String> {
    inspect_filtered(root, kind, concept_id, profile, &KnowledgeFilter::default())
}

pub fn inspect_filtered(
    root: &Path,
    kind: InspectKind,
    concept_id: Option<&str>,
    profile: Profile,
    filter: &KnowledgeFilter,
) -> Result<String> {
    let bundle = load(root, profile)?;
    match kind {
        InspectKind::Catalog => {
            let filtered = bundle
                .concepts
                .iter()
                .filter(|concept| filter.matches(concept))
                .map(ConceptInspect::from)
                .collect::<Vec<_>>();
            Ok(serde_json::to_string_pretty(&filtered)?)
        }
        InspectKind::Concept => {
            let Some(id) = concept_id else {
                bail!("inspect concept requires a concept id");
            };
            let Some(concept) = bundle.concepts.iter().find(|concept| concept.id == id) else {
                bail!("unknown concept `{id}`");
            };
            Ok(serde_json::to_string_pretty(&ConceptInspect::from(
                concept,
            ))?)
        }
        InspectKind::Graph => Ok(serde_json::to_string_pretty(&bundle.graph)?),
    }
}

pub fn search(
    root: &Path,
    query: &str,
    profile: Profile,
    filter: &KnowledgeFilter,
) -> Result<String> {
    let bundle = load(root, profile)?;
    Ok(serde_json::to_string_pretty(&matching_search_chunks(
        &bundle, query, filter,
    ))?)
}

pub fn benchmark_retrieval(
    root: &Path,
    benchmark_path: &Path,
    profile: Profile,
) -> Result<RetrievalReport> {
    let bundle = load(root, profile)?;
    run_benchmark(&bundle, benchmark_path)
}

pub fn build(root: &Path, output: &Path, profile: Profile) -> Result<BuildSummary> {
    let bundle = load(root, profile)?;
    if bundle.has_errors() {
        bail!("knowledge bundle has validation errors");
    }
    build_artifacts(&bundle, output)
}

fn parse_concept(
    relative: &str,
    source: &str,
    profile: Profile,
    concepts: &mut Vec<Concept>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let frontmatter = match split_frontmatter(source, true) {
        Ok(Some(frontmatter)) => frontmatter,
        Ok(None) => return,
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1002",
                relative,
                Some(location(source, Span::point(0))),
                message,
            ));
            return;
        }
    };
    let metadata = match parse_yaml_mapping(frontmatter.yaml.of(source)) {
        Ok(metadata) => metadata,
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1003",
                relative,
                Some(location(source, frontmatter.yaml)),
                message,
            ));
            return;
        }
    };
    validate_metadata(
        relative,
        source,
        frontmatter.yaml,
        &metadata,
        profile,
        diagnostics,
    );

    let parsed = parse_markdown_body(relative, source, frontmatter.body, diagnostics);

    let (footnote_ids, defined_footnotes) = (parsed.footnote_ids, parsed.defined_footnotes);
    let source_ids = collect_source_ids(relative, &metadata, diagnostics);
    for footnote in &footnote_ids {
        if !source_ids.contains(footnote) {
            diagnostics.push(Diagnostic::error(
                "OKF4001",
                relative,
                None,
                format!("footnote `{footnote}` has no matching sources[].id"),
            ));
        }
        if !defined_footnotes.contains(footnote) {
            diagnostics.push(Diagnostic::error(
                "OKF4003",
                relative,
                None,
                format!("footnote `{footnote}` has no definition in the body"),
            ));
        }
    }
    for source_id in &source_ids {
        if !footnote_ids.contains(source_id) {
            diagnostics.push(Diagnostic::warning(
                "OKF4002",
                relative,
                None,
                format!("source id `{source_id}` is not used by a body footnote"),
            ));
        }
    }

    let id = relative.strip_suffix(".md").unwrap_or(relative).to_string();
    concepts.push(Concept {
        id,
        path: relative.to_string(),
        metadata,
        body_span: frontmatter.body,
        body_location: location(source, frontmatter.body),
        headings: parsed.headings,
        heading_sections: parsed.heading_sections,
        links: parsed.links,
        source_ids,
        footnote_ids,
        article_html: parsed.article_html,
    });
}

fn parse_index(
    root: &Path,
    relative: &str,
    source: &str,
    indexes: &mut Vec<Index>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let is_root = root.join(relative) == root.join("index.md");
    let mut version = None;
    let body = match split_frontmatter(source, false) {
        Ok(Some(frontmatter)) if is_root => {
            match parse_yaml_mapping(frontmatter.yaml.of(source)) {
                Ok(metadata) => {
                    for key in metadata.keys() {
                        if key != "okf_version" {
                            diagnostics.push(Diagnostic::error(
                                "OKF1011",
                                relative,
                                Some(location(source, frontmatter.yaml)),
                                format!("root index frontmatter may only contain `okf_version`, found `{key}`"),
                            ));
                        }
                    }
                    match metadata.get("okf_version").and_then(Value::as_str) {
                        Some("0.2") => version = Some("0.2".to_string()),
                        Some(other) => diagnostics.push(Diagnostic::error(
                            "OKF1012",
                            relative,
                            Some(location(source, frontmatter.yaml)),
                            format!("unsupported okf_version `{other}`"),
                        )),
                        None => diagnostics.push(Diagnostic::error(
                            "OKF1012",
                            relative,
                            Some(location(source, frontmatter.yaml)),
                            "okf_version must be a string",
                        )),
                    }
                }
                Err(message) => diagnostics.push(Diagnostic::error(
                    "OKF1003",
                    relative,
                    Some(location(source, frontmatter.yaml)),
                    message,
                )),
            }
            frontmatter.body
        }
        Ok(Some(frontmatter)) => {
            diagnostics.push(Diagnostic::error(
                "OKF1011",
                relative,
                Some(location(source, frontmatter.yaml)),
                "non-root index.md must not contain frontmatter",
            ));
            frontmatter.body
        }
        Ok(None) => Span::new(0, source.len()),
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1002",
                relative,
                Some(location(source, Span::point(0))),
                message,
            ));
            return;
        }
    };
    let parsed = parse_markdown_body(relative, source, body, diagnostics);
    indexes.push(Index {
        path: relative.to_string(),
        version,
        body_span: body,
        article_html: parsed.article_html,
    });
}

fn parse_log(relative: &str, source: &str, logs: &mut Vec<Log>, diagnostics: &mut Vec<Diagnostic>) {
    if source.starts_with("---\n") || source.starts_with("---\r\n") {
        diagnostics.push(Diagnostic::error(
            "OKF1021",
            relative,
            Some(location(source, Span::point(0))),
            "log.md must not contain frontmatter",
        ));
    }
    for (offset, line) in lines_with_offsets(source) {
        if let Some(date) = line.trim_end_matches(['\r', '\n']).strip_prefix("## ")
            && !is_date(date)
        {
            diagnostics.push(Diagnostic::error(
                "OKF1022",
                relative,
                Some(location(source, Span::new(offset, offset + line.len()))),
                "log date headings must use YYYY-MM-DD",
            ));
        }
    }
    logs.push(Log {
        path: relative.to_string(),
        body_span: location(source, Span::new(0, source.len())),
    });
}

fn discover_markdown(directory: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_name().as_encoded_bytes().starts_with(b".") {
            continue;
        }
        if path.is_dir() {
            discover_markdown(&path, out)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
