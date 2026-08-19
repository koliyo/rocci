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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde_json::Value;

pub use artifact::{
    ConceptInspect, absolute, build_artifacts, commit_output, llms_text, unique_temp,
};
pub use ast::{
    BuildSummary, Bundle, CheckReport, Concept, Edge, Heading, HeadingSection, Index, InspectKind,
    KnowledgeFilter, Link, LoadOptions, Log, Profile, Span, TrustTier,
};
pub use benchmark::{
    RetrievalBenchmark, RetrievalQuestion, RetrievalQuestionResult, RetrievalReport, run_benchmark,
};
pub use diagnostic::{Diagnostic, Severity, SourceLocation};
pub use frontmatter::{
    Frontmatter, lines_with_offsets, location, parse_yaml_mapping, split_frontmatter,
};
pub use graph::{published_href, resolve_bundle_path, resolve_graph, split_fragment};
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
    string_field, validate_lifecycle_and_sources, validate_lifecycle_and_sources_with,
    validate_metadata, validate_optional_string, validate_unique_ids,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoadTimings {
    pub discover: Duration,
    pub parse: Duration,
    pub graph: Duration,
    pub provenance: Option<Duration>,
    pub parse_cache_hits: u32,
    pub parse_cache_misses: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileFingerprint {
    modified_secs: u64,
    modified_nanos: u32,
    len: u64,
}

#[derive(Clone, Debug)]
enum CachedDocument {
    Concept {
        concept: Concept,
        diagnostics: Vec<Diagnostic>,
    },
    Index {
        index: Index,
        diagnostics: Vec<Diagnostic>,
    },
    Log {
        log: Log,
        diagnostics: Vec<Diagnostic>,
    },
    Diagnostics(Vec<Diagnostic>),
}

#[derive(Clone, Debug)]
struct CacheEntry {
    fingerprint: FileFingerprint,
    document: CachedDocument,
}

#[derive(Clone, Debug, Default)]
pub struct ParseCache {
    profile: Option<Profile>,
    entries: BTreeMap<String, CacheEntry>,
}

impl ParseCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn begin(&mut self, profile: Profile) {
        if self.profile != Some(profile) {
            self.entries.clear();
            self.profile = Some(profile);
        }
    }

    fn get(&self, relative: &str, fingerprint: FileFingerprint) -> Option<&CachedDocument> {
        self.entries
            .get(relative)
            .and_then(|entry| (entry.fingerprint == fingerprint).then_some(&entry.document))
    }

    fn insert(&mut self, relative: String, fingerprint: FileFingerprint, document: CachedDocument) {
        self.entries.insert(
            relative,
            CacheEntry {
                fingerprint,
                document,
            },
        );
    }

    fn retain_paths(&mut self, live: &BTreeSet<String>) {
        self.entries.retain(|path, _| live.contains(path));
    }
}

#[derive(Debug)]
pub struct LoadResult {
    pub bundle: Bundle,
    pub timings: LoadTimings,
}

pub fn check(root: &Path, profile: Profile) -> Result<CheckReport> {
    let bundle = load(root, profile)?;
    Ok(CheckReport {
        diagnostics: bundle.diagnostics,
    })
}

pub fn load(root: &Path, profile: Profile) -> Result<Bundle> {
    Ok(load_timed(root, LoadOptions::new(profile))?.bundle)
}

pub fn load_timed(root: &Path, options: LoadOptions) -> Result<LoadResult> {
    load_with_cache(root, options, None)
}

pub fn load_with_cache(
    root: &Path,
    options: LoadOptions,
    mut cache: Option<&mut ParseCache>,
) -> Result<LoadResult> {
    let root = absolute(root)?;
    if !root.is_dir() {
        bail!("knowledge bundle {} is not a directory", root.display());
    }

    let discover_started = Instant::now();
    let mut paths = Vec::new();
    discover_markdown(&root, &mut paths)?;
    paths.sort();
    let discover = discover_started.elapsed();

    let parse_started = Instant::now();
    let mut concepts = Vec::new();
    let mut indexes = Vec::new();
    let mut logs = Vec::new();
    let mut diagnostics = Vec::new();
    let mut parse_cache_hits = 0;
    let mut parse_cache_misses = 0;
    let mut live_paths = BTreeSet::new();
    if let Some(cache) = cache.as_mut() {
        cache.begin(options.profile);
    }

    for path in paths {
        let relative = relative_path(&root, &path);
        live_paths.insert(relative.clone());
        let fingerprint = file_fingerprint(&path);
        if let Some(document) = cache
            .as_ref()
            .and_then(|cache| fingerprint.and_then(|fingerprint| cache.get(&relative, fingerprint)))
            .cloned()
        {
            apply_cached(
                &document,
                &mut concepts,
                &mut indexes,
                &mut logs,
                &mut diagnostics,
            );
            parse_cache_hits += 1;
            continue;
        }
        parse_cache_misses += 1;
        let diagnostics_before = diagnostics.len();
        let concepts_before = concepts.len();
        let indexes_before = indexes.len();
        let logs_before = logs.len();
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let source = match String::from_utf8(bytes) {
            Ok(source) => source,
            Err(_) => {
                diagnostics.push(Diagnostic::error(
                    "OKF1001",
                    relative.clone(),
                    None,
                    "document is not valid UTF-8",
                ));
                if let (Some(cache), Some(fingerprint)) = (cache.as_mut(), fingerprint) {
                    cache.insert(
                        relative,
                        fingerprint,
                        CachedDocument::Diagnostics(diagnostics[diagnostics_before..].to_vec()),
                    );
                }
                continue;
            }
        };
        match path.file_name().and_then(|name| name.to_str()) {
            Some("index.md") => {
                parse_index(&root, &relative, &source, &mut indexes, &mut diagnostics)
            }
            Some("log.md") => parse_log(&relative, &source, &mut logs, &mut diagnostics),
            _ => parse_concept(
                &relative,
                &source,
                options.profile,
                &mut concepts,
                &mut diagnostics,
            ),
        }
        if let (Some(cache), Some(fingerprint)) = (cache.as_mut(), fingerprint) {
            let document = capture_cached(
                &concepts[concepts_before..],
                &indexes[indexes_before..],
                &logs[logs_before..],
                &diagnostics[diagnostics_before..],
            );
            cache.insert(relative, fingerprint, document);
        }
    }
    if let Some(cache) = cache.as_mut() {
        cache.retain_paths(&live_paths);
    }

    concepts.sort_by(|a, b| a.id.cmp(&b.id));
    indexes.sort_by(|a, b| a.path.cmp(&b.path));
    logs.sort_by(|a, b| a.path.cmp(&b.path));
    validate_unique_ids(&concepts, &mut diagnostics);
    let parse = parse_started.elapsed();

    let graph_started = Instant::now();
    let graph = resolve_graph(&concepts, &indexes, &mut diagnostics);
    let graph_duration = graph_started.elapsed();

    let provenance = if options.provenance {
        let provenance_started = Instant::now();
        validate_lifecycle_and_sources_with(&root, &concepts, &mut diagnostics, true);
        Some(provenance_started.elapsed())
    } else if options.profile == Profile::Rocci {
        validate_lifecycle_and_sources_with(&root, &concepts, &mut diagnostics, false);
        Some(Duration::ZERO)
    } else {
        None
    };

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

    Ok(LoadResult {
        bundle: Bundle {
            root,
            version,
            concepts,
            indexes,
            logs,
            graph,
            diagnostics,
        },
        timings: LoadTimings {
            discover,
            parse,
            graph: graph_duration,
            provenance,
            parse_cache_hits,
            parse_cache_misses,
        },
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

fn file_fingerprint(path: &Path) -> Option<FileFingerprint> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some(FileFingerprint {
        modified_secs: modified.as_secs(),
        modified_nanos: modified.subsec_nanos(),
        len: meta.len(),
    })
}

fn apply_cached(
    document: &CachedDocument,
    concepts: &mut Vec<Concept>,
    indexes: &mut Vec<Index>,
    logs: &mut Vec<Log>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match document {
        CachedDocument::Concept {
            concept,
            diagnostics: cached,
        } => {
            concepts.push(concept.clone());
            diagnostics.extend(cached.iter().cloned());
        }
        CachedDocument::Index {
            index,
            diagnostics: cached,
        } => {
            indexes.push(index.clone());
            diagnostics.extend(cached.iter().cloned());
        }
        CachedDocument::Log {
            log,
            diagnostics: cached,
        } => {
            logs.push(log.clone());
            diagnostics.extend(cached.iter().cloned());
        }
        CachedDocument::Diagnostics(cached) => diagnostics.extend(cached.iter().cloned()),
    }
}

fn capture_cached(
    concepts: &[Concept],
    indexes: &[Index],
    logs: &[Log],
    diagnostics: &[Diagnostic],
) -> CachedDocument {
    let diagnostics = diagnostics.to_vec();
    if let Some(concept) = concepts.first() {
        CachedDocument::Concept {
            concept: concept.clone(),
            diagnostics,
        }
    } else if let Some(index) = indexes.first() {
        CachedDocument::Index {
            index: index.clone(),
            diagnostics,
        }
    } else if let Some(log) = logs.first() {
        CachedDocument::Log {
            log: log.clone(),
            diagnostics,
        }
    } else {
        CachedDocument::Diagnostics(diagnostics)
    }
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
