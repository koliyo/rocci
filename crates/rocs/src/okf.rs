use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use rocci_rocdown::{
    Document, Item, MarkdownBodyOptions, MdNode, SourceFile, Span, parse_markdown_body,
};
use serde::Serialize;
use serde_json::{Map, Value};
use yaml_rust::{Yaml, YamlLoader};

use crate::article::render_document;
use crate::catalog::Severity;

const STANDARD_FIELDS: &[&str] = &[
    "type",
    "title",
    "description",
    "resource",
    "tags",
    "sources",
    "usage_window",
    "generated",
    "verified",
    "status",
    "stale_after",
    "authority",
    "owners",
];

const PROFILE_TYPES: &[&str] = &[
    "Architecture",
    "Decision",
    "Specification",
    "Status",
    "Implementation Plan",
    "Research Report",
    "Audit",
    "Case Study",
    "Reference",
    "Design Standard",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Base,
    Rocci,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectKind {
    Catalog,
    Concept,
    Graph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLocation {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    pub message: String,
}

impl Diagnostic {
    fn error(
        code: &'static str,
        path: impl Into<String>,
        location: Option<SourceLocation>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            path: path.into(),
            location,
            message: message.into(),
        }
    }

    fn warning(
        code: &'static str,
        path: impl Into<String>,
        location: Option<SourceLocation>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            path: path.into(),
            location,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        if let Some(location) = &self.location {
            write!(
                f,
                "{} {severity} {}:{}:{}: {}",
                self.code, self.path, location.line, location.column, self.message
            )
        } else {
            write!(
                f,
                "{} {severity} {}: {}",
                self.code, self.path, self.message
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Link {
    pub url: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub raw: String,
    pub broken: bool,
}

#[derive(Debug, Clone)]
pub struct Concept {
    pub id: String,
    pub path: String,
    pub metadata: BTreeMap<String, Value>,
    pub body_span: Span,
    pub body_location: SourceLocation,
    pub document: Document,
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub source_ids: BTreeSet<String>,
    pub footnote_ids: BTreeSet<String>,
    pub article_html: String,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub path: String,
    pub version: Option<String>,
    pub body_span: Span,
    pub document: Document,
    pub article_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Log {
    pub path: String,
    pub body_span: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct Bundle {
    pub root: PathBuf,
    pub version: Option<String>,
    pub concepts: Vec<Concept>,
    pub indexes: Vec<Index>,
    pub logs: Vec<Log>,
    pub graph: Vec<Edge>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Bundle {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

pub struct CheckReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn terminal(&self) -> String {
        self.diagnostics
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.diagnostics)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildSummary {
    pub concepts: usize,
    pub indexes: usize,
    pub output: String,
}

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
    target: Option<&str>,
    profile: Profile,
) -> Result<String> {
    let bundle = load(root, profile)?;
    match kind {
        InspectKind::Catalog => {
            let catalog = bundle
                .concepts
                .iter()
                .map(ConceptInspect::from)
                .collect::<Vec<_>>();
            Ok(serde_json::to_string_pretty(&catalog)?)
        }
        InspectKind::Concept => {
            let target = target.filter(|target| !target.is_empty()).ok_or_else(|| {
                anyhow::anyhow!("knowledge inspect concept requires a concept id")
            })?;
            let target = target.strip_suffix(".md").unwrap_or(target);
            let concept = bundle
                .concepts
                .iter()
                .find(|concept| concept.id == target)
                .ok_or_else(|| anyhow::anyhow!("unknown knowledge concept `{target}`"))?;
            Ok(serde_json::to_string_pretty(&ConceptInspect::from(
                concept,
            ))?)
        }
        InspectKind::Graph => Ok(serde_json::to_string_pretty(&bundle.graph)?),
    }
}

pub fn build(root: &Path, output: &Path, profile: Profile) -> Result<BuildSummary> {
    let bundle = load(root, profile)?;
    if bundle.has_errors() {
        bail!("knowledge bundle has validation errors");
    }
    let output = absolute(output)?;
    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create {}", output.display()))?;
    let site = output.join("site");
    fs::create_dir_all(&site).with_context(|| format!("failed to create {}", site.display()))?;

    for concept in &bundle.concepts {
        let destination = site.join(&concept.id).join("index.html");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let title = string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        fs::write(&destination, html_page(title, &concept.article_html))
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }
    if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
        fs::write(
            site.join("index.html"),
            html_page("Knowledge", &index.article_html),
        )
        .context("failed to write knowledge index")?;
    }
    let catalog = bundle
        .concepts
        .iter()
        .map(ConceptInspect::from)
        .collect::<Vec<_>>();
    fs::write(
        output.join("catalog.json"),
        format!("{}\n", serde_json::to_string_pretty(&catalog)?),
    )
    .context("failed to write knowledge catalog")?;
    fs::write(
        output.join("validation.json"),
        format!("{}\n", serde_json::to_string_pretty(&bundle.diagnostics)?),
    )
    .context("failed to write knowledge validation report")?;

    Ok(BuildSummary {
        concepts: bundle.concepts.len(),
        indexes: bundle.indexes.len(),
        output: output.to_string_lossy().into_owned(),
    })
}

#[derive(Serialize)]
struct ConceptInspect<'a> {
    id: &'a str,
    path: &'a str,
    metadata: &'a BTreeMap<String, Value>,
    body_span: SourceLocation,
    headings: &'a [Heading],
    links: &'a [Link],
    source_ids: &'a BTreeSet<String>,
    footnote_ids: &'a BTreeSet<String>,
}

impl<'a> From<&'a Concept> for ConceptInspect<'a> {
    fn from(concept: &'a Concept) -> Self {
        Self {
            id: &concept.id,
            path: &concept.path,
            metadata: &concept.metadata,
            body_span: concept.body_location.clone(),
            headings: &concept.headings,
            links: &concept.links,
            source_ids: &concept.source_ids,
            footnote_ids: &concept.footnote_ids,
        }
    }
}

fn parse_concept(
    relative: &str,
    source: &str,
    profile: Profile,
    concepts: &mut Vec<Concept>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let file = SourceFile::new(relative, source);
    let frontmatter = match split_frontmatter(source, true) {
        Ok(Some(frontmatter)) => frontmatter,
        Ok(None) => unreachable!(),
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1002",
                relative,
                Some(location(file, Span::point(0))),
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
                Some(location(file, frontmatter.yaml)),
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
    reject_declarations(relative, source, frontmatter.body, diagnostics);

    let parsed = parse_markdown_body(
        file,
        frontmatter.body,
        MarkdownBodyOptions {
            raw_html: false,
            footnotes: true,
        },
    );
    for diagnostic in &parsed.diagnostics {
        diagnostics.push(Diagnostic::error(
            "OKF2008",
            relative,
            Some(location(file, diagnostic.span)),
            diagnostic.message.clone(),
        ));
    }
    let headings = parsed
        .headings
        .iter()
        .map(|heading| Heading {
            level: heading.level,
            id: heading.id.clone(),
            text: heading.text.clone(),
            location: location(file, heading.span),
        })
        .collect::<Vec<_>>();
    let links = parsed
        .links
        .iter()
        .map(|link| Link {
            url: link.url.clone(),
            location: location(file, link.span),
        })
        .collect::<Vec<_>>();
    let (footnote_ids, defined_footnotes) = collect_footnotes(&parsed.document);
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
                format!("footnote `{footnote}` has no definition"),
            ));
        }
    }
    for source_id in source_ids.difference(&footnote_ids) {
        diagnostics.push(Diagnostic::warning(
            "OKF4002",
            relative,
            None,
            format!("source id `{source_id}` is not used by a body footnote"),
        ));
    }

    let id = relative.strip_suffix(".md").unwrap_or(relative).to_string();
    let article_html = render_document(&parsed.document);
    concepts.push(Concept {
        id,
        path: relative.to_string(),
        metadata,
        body_span: frontmatter.body,
        body_location: location(file, frontmatter.body),
        document: parsed.document,
        headings,
        links,
        source_ids,
        footnote_ids,
        article_html,
    });
}

fn parse_index(
    root: &Path,
    relative: &str,
    source: &str,
    indexes: &mut Vec<Index>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let file = SourceFile::new(relative, source);
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
                                Some(location(file, frontmatter.yaml)),
                                format!("root index frontmatter may only contain `okf_version`, found `{key}`"),
                            ));
                        }
                    }
                    match metadata.get("okf_version").and_then(Value::as_str) {
                        Some("0.2") => version = Some("0.2".to_string()),
                        Some(other) => diagnostics.push(Diagnostic::error(
                            "OKF1012",
                            relative,
                            Some(location(file, frontmatter.yaml)),
                            format!("unsupported okf_version `{other}`"),
                        )),
                        None => diagnostics.push(Diagnostic::error(
                            "OKF1012",
                            relative,
                            Some(location(file, frontmatter.yaml)),
                            "okf_version must be a string",
                        )),
                    }
                }
                Err(message) => diagnostics.push(Diagnostic::error(
                    "OKF1003",
                    relative,
                    Some(location(file, frontmatter.yaml)),
                    message,
                )),
            }
            frontmatter.body
        }
        Ok(Some(frontmatter)) => {
            diagnostics.push(Diagnostic::error(
                "OKF1011",
                relative,
                Some(location(file, frontmatter.yaml)),
                "non-root index.md must not contain frontmatter",
            ));
            frontmatter.body
        }
        Ok(None) => Span::new(0, source.len()),
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1002",
                relative,
                Some(location(file, Span::point(0))),
                message,
            ));
            return;
        }
    };
    let parsed = parse_markdown_body(
        file,
        body,
        MarkdownBodyOptions {
            raw_html: false,
            footnotes: true,
        },
    );
    for diagnostic in &parsed.diagnostics {
        diagnostics.push(Diagnostic::error(
            "OKF2008",
            relative,
            Some(location(file, diagnostic.span)),
            diagnostic.message.clone(),
        ));
    }
    indexes.push(Index {
        path: relative.to_string(),
        version,
        body_span: body,
        article_html: render_document(&parsed.document),
        document: parsed.document,
    });
}

fn parse_log(relative: &str, source: &str, logs: &mut Vec<Log>, diagnostics: &mut Vec<Diagnostic>) {
    let file = SourceFile::new(relative, source);
    if source.starts_with("---\n") || source.starts_with("---\r\n") {
        diagnostics.push(Diagnostic::error(
            "OKF1021",
            relative,
            Some(location(file, Span::point(0))),
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
                Some(location(file, Span::new(offset, offset + line.len()))),
                "log date headings must use YYYY-MM-DD",
            ));
        }
    }
    logs.push(Log {
        path: relative.to_string(),
        body_span: location(file, Span::new(0, source.len())),
    });
}

fn validate_metadata(
    relative: &str,
    source: &str,
    span: Span,
    metadata: &BTreeMap<String, Value>,
    profile: Profile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let file = SourceFile::new(relative, source);
    let at = Some(location(file, span));
    match metadata.get("type").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => {
            if profile == Profile::Rocci && !PROFILE_TYPES.contains(&value) {
                diagnostics.push(Diagnostic::warning(
                    "OKF2002",
                    relative,
                    at.clone(),
                    format!("unknown Rocci concept type `{value}`"),
                ));
            }
        }
        _ => diagnostics.push(Diagnostic::error(
            "OKF1004",
            relative,
            at.clone(),
            "concept frontmatter requires a non-empty string `type`",
        )),
    }
    validate_optional_string(relative, metadata, "title", at.clone(), diagnostics);
    validate_optional_string(relative, metadata, "description", at.clone(), diagnostics);
    validate_optional_string(relative, metadata, "resource", at.clone(), diagnostics);
    if let Some(status) = metadata.get("status") {
        match status.as_str() {
            Some("draft" | "stable" | "deprecated") => {}
            _ => diagnostics.push(Diagnostic::error(
                "OKF1005",
                relative,
                at.clone(),
                "status must be draft, stable, or deprecated",
            )),
        }
    }
    if let Some(stale_after) = metadata.get("stale_after")
        && !stale_after.as_str().is_some_and(is_date)
    {
        diagnostics.push(Diagnostic::error(
            "OKF1006",
            relative,
            at.clone(),
            "stale_after must use YYYY-MM-DD",
        ));
    }
    if let Some(tags) = metadata.get("tags")
        && !string_array(tags)
    {
        diagnostics.push(Diagnostic::error(
            "OKF1007",
            relative,
            at.clone(),
            "tags must be a list of strings",
        ));
    }
    if let Some(generated) = metadata.get("generated")
        && !generated.as_object().is_some_and(|object| {
            object.get("by").is_some_and(Value::is_string)
                && object.get("at").is_some_and(Value::is_string)
        })
    {
        diagnostics.push(Diagnostic::error(
            "OKF1008",
            relative,
            at.clone(),
            "generated must be a mapping with string `by` and `at`",
        ));
    }
    for key in metadata.keys() {
        if !STANDARD_FIELDS.contains(&key.as_str()) {
            diagnostics.push(Diagnostic::warning(
                "OKF2001",
                relative,
                at.clone(),
                format!("unknown metadata field `{key}` is preserved"),
            ));
        }
    }
    if profile == Profile::Rocci {
        for required in ["title", "description", "status", "generated"] {
            if !metadata.contains_key(required) {
                diagnostics.push(Diagnostic::error(
                    "OKF2003",
                    relative,
                    at.clone(),
                    format!("Rocci profile requires `{required}`"),
                ));
            }
        }
        let tags = metadata.get("tags").and_then(Value::as_array);
        if !tags.is_some_and(|tags| {
            !tags.is_empty()
                && tags
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|tag| tag.starts_with("domain/"))
        }) {
            diagnostics.push(Diagnostic::error(
                "OKF2004",
                relative,
                at.clone(),
                "Rocci profile requires tags with at least one domain/* value",
            ));
        }
        if let Some(tags) = tags {
            for tag in tags.iter().filter_map(Value::as_str) {
                let valid_prefix = ["domain/", "integration/", "concern/", "audience/"]
                    .iter()
                    .any(|prefix| tag.starts_with(prefix));
                if !valid_prefix {
                    diagnostics.push(Diagnostic::error(
                        "OKF2005",
                        relative,
                        at.clone(),
                        format!("unknown tag prefix in `{tag}`"),
                    ));
                }
            }
        }
        match metadata.get("authority").and_then(Value::as_str) {
            Some("normative" | "descriptive" | "exploratory" | "historical") => {}
            Some(_) => diagnostics.push(Diagnostic::error(
                "OKF2006",
                relative,
                at.clone(),
                "authority must be normative, descriptive, exploratory, or historical",
            )),
            None => diagnostics.push(Diagnostic::error(
                "OKF2003",
                relative,
                at.clone(),
                "Rocci profile requires `authority`",
            )),
        }
        if !metadata.get("owners").is_some_and(|owners| {
            owners
                .as_array()
                .is_some_and(|owners| !owners.is_empty() && owners.iter().all(Value::is_string))
        }) {
            diagnostics.push(Diagnostic::error(
                "OKF2003",
                relative,
                at,
                "Rocci profile requires string-list `owners`",
            ));
        }
    }
}

fn collect_source_ids(
    relative: &str,
    metadata: &BTreeMap<String, Value>,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(sources) = metadata.get("sources") else {
        return ids;
    };
    let Some(sources) = sources.as_array() else {
        diagnostics.push(Diagnostic::error(
            "OKF1009",
            relative,
            None,
            "sources must be a list",
        ));
        return ids;
    };
    for (index, source) in sources.iter().enumerate() {
        let Some(source) = source.as_object() else {
            diagnostics.push(Diagnostic::error(
                "OKF1009",
                relative,
                None,
                format!("sources[{index}] must be a mapping"),
            ));
            continue;
        };
        if !source.get("resource").is_some_and(Value::is_string) {
            diagnostics.push(Diagnostic::error(
                "OKF1009",
                relative,
                None,
                format!("sources[{index}] requires string `resource`"),
            ));
        }
        if let Some(id) = source.get("id") {
            match id.as_str() {
                Some(id) if !id.is_empty() => {
                    if !ids.insert(id.to_string()) {
                        diagnostics.push(Diagnostic::error(
                            "OKF4010",
                            relative,
                            None,
                            format!("duplicate source id `{id}`"),
                        ));
                    }
                }
                _ => diagnostics.push(Diagnostic::error(
                    "OKF1009",
                    relative,
                    None,
                    format!("sources[{index}].id must be a non-empty string"),
                )),
            }
        }
    }
    ids
}

fn collect_footnotes(document: &Document) -> (BTreeSet<String>, BTreeSet<String>) {
    fn walk(node: &MdNode, references: &mut BTreeSet<String>, definitions: &mut BTreeSet<String>) {
        match node {
            MdNode::FootnoteReference { name, .. } => {
                references.insert(name.clone());
            }
            MdNode::FootnoteDefinition { name, children, .. } => {
                definitions.insert(name.clone());
                for child in children {
                    walk(child, references, definitions);
                }
            }
            MdNode::Text { value, .. } => {
                references.extend(footnote_labels(value));
            }
            _ => {
                for child in node_children(node) {
                    walk(child, references, definitions);
                }
            }
        }
    }
    let mut references = BTreeSet::new();
    let mut definitions = BTreeSet::new();
    for item in &document.items {
        if let Item::Markdown(node) = item {
            walk(node, &mut references, &mut definitions);
        }
    }
    (references, definitions)
}

fn node_children(node: &MdNode) -> &[MdNode] {
    match node {
        MdNode::Heading { children, .. }
        | MdNode::Paragraph { children, .. }
        | MdNode::BlockQuote { children, .. }
        | MdNode::List { children, .. }
        | MdNode::Item { children, .. }
        | MdNode::TaskItem { children, .. }
        | MdNode::Table { children, .. }
        | MdNode::TableRow { children, .. }
        | MdNode::TableCell { children, .. }
        | MdNode::Emph { children, .. }
        | MdNode::Strong { children, .. }
        | MdNode::Strikethrough { children, .. }
        | MdNode::FootnoteDefinition { children, .. }
        | MdNode::Link { children, .. } => children,
        _ => &[],
    }
}

fn validate_unique_ids(concepts: &[Concept], diagnostics: &mut Vec<Diagnostic>) {
    let mut ids = BTreeMap::new();
    for concept in concepts {
        let folded = concept.id.to_ascii_lowercase();
        if let Some(previous) = ids.insert(folded, concept.path.clone()) {
            diagnostics.push(Diagnostic::error(
                "OKF3003",
                &concept.path,
                None,
                format!("concept id conflicts case-insensitively with `{previous}`"),
            ));
        }
    }
}

fn resolve_graph(
    concepts: &[Concept],
    indexes: &[Index],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Edge> {
    let concept_paths = concepts
        .iter()
        .map(|concept| (concept.path.as_str(), concept))
        .collect::<BTreeMap<_, _>>();
    let index_paths = indexes
        .iter()
        .map(|index| index.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for concept in concepts {
        for link in &concept.links {
            if external_url(&link.url) || link.url.starts_with('#') {
                continue;
            }
            let (path, fragment) = split_fragment(&link.url);
            let resolved = resolve_bundle_path(&concept.path, path);
            let Some(resolved) = resolved else {
                diagnostics.push(Diagnostic::error(
                    "OKF3001",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("link `{}` escapes the knowledge bundle", link.url),
                ));
                continue;
            };
            let directory_index = if resolved.ends_with('/') {
                format!("{resolved}index.md")
            } else {
                String::new()
            };
            let target = concept_paths.get(resolved.as_str()).copied();
            let valid_index = index_paths.contains(resolved.as_str())
                || (!directory_index.is_empty() && index_paths.contains(directory_index.as_str()));
            let broken = target.is_none() && !valid_index;
            if broken {
                diagnostics.push(Diagnostic::warning(
                    "OKF3002",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("broken concept link `{}`", link.url),
                ));
            } else if let (Some(target), Some(fragment)) = (target, fragment)
                && !target.headings.iter().any(|heading| heading.id == fragment)
            {
                diagnostics.push(Diagnostic::warning(
                    "OKF3004",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("unknown heading `{fragment}` in `{resolved}`"),
                ));
            }
            edges.push(Edge {
                from: concept.id.clone(),
                to: resolved
                    .strip_suffix(".md")
                    .unwrap_or(&resolved)
                    .to_string(),
                raw: link.url.clone(),
                broken,
            });
        }
    }
    edges.sort_by(|a, b| (&a.from, &a.to, &a.raw).cmp(&(&b.from, &b.to, &b.raw)));
    edges
}

fn reject_declarations(
    relative: &str,
    source: &str,
    body: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    const DECLARATIONS: &[&str] = &[
        "page",
        "roc",
        "render",
        "component",
        "fixture",
        "css",
        "context",
        "init",
        "on",
    ];
    let file = SourceFile::new(relative, source);
    let mut fence: Option<char> = None;
    for (relative_offset, line) in lines_with_offsets(body.of(source)) {
        let trimmed = line.trim_start();
        let fence_marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        if let Some(marker) = fence_marker {
            match fence {
                Some(active) if active == marker => fence = None,
                None => fence = Some(marker),
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix('@') else {
            continue;
        };
        let name = rest
            .split(|character: char| character.is_whitespace() || character == '{')
            .next()
            .unwrap_or("");
        if DECLARATIONS.contains(&name) {
            let indent = line.len() - trimmed.len();
            let start = body.start as usize + relative_offset + indent;
            diagnostics.push(Diagnostic::error(
                "OKF2007",
                relative,
                Some(location(file, Span::new(start, start + name.len() + 1))),
                format!("Rocdown declaration `@{name}` is forbidden in knowledge records"),
            ));
        }
    }
}

#[derive(Clone, Copy)]
struct Frontmatter {
    yaml: Span,
    body: Span,
}

fn split_frontmatter(
    source: &str,
    required: bool,
) -> std::result::Result<Option<Frontmatter>, String> {
    let mut lines = lines_with_offsets(source).into_iter();
    let Some((_, first)) = lines.next() else {
        return if required {
            Err("concept requires YAML frontmatter".into())
        } else {
            Ok(None)
        };
    };
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return if required {
            Err("concept must start with `---` YAML frontmatter".into())
        } else {
            Ok(None)
        };
    }
    let yaml_start = first.len();
    for (offset, line) in lines {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Ok(Some(Frontmatter {
                yaml: Span::new(yaml_start, offset),
                body: Span::new(offset + line.len(), source.len()),
            }));
        }
    }
    Err("frontmatter is missing its closing `---` delimiter".into())
}

fn parse_yaml_mapping(source: &str) -> std::result::Result<BTreeMap<String, Value>, String> {
    let documents = YamlLoader::load_from_str(source).map_err(|error| error.to_string())?;
    if documents.len() != 1 {
        return Err("frontmatter must contain exactly one YAML document".into());
    }
    let Some(mapping) = documents[0].as_hash() else {
        return Err("frontmatter must be a YAML mapping".into());
    };
    let mut out = BTreeMap::new();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err("frontmatter keys must be strings".into());
        };
        out.insert(key.to_string(), yaml_to_json(value)?);
    }
    Ok(out)
}

fn yaml_to_json(value: &Yaml) -> std::result::Result<Value, String> {
    match value {
        Yaml::Real(value) => value
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| format!("invalid YAML number `{value}`")),
        Yaml::Integer(value) => Ok(Value::Number((*value).into())),
        Yaml::String(value) => Ok(Value::String(value.clone())),
        Yaml::Boolean(value) => Ok(Value::Bool(*value)),
        Yaml::Array(values) => values
            .iter()
            .map(yaml_to_json)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map(Value::Array),
        Yaml::Hash(values) => {
            let mut object = Map::new();
            for (key, value) in values {
                let Some(key) = key.as_str() else {
                    return Err("nested YAML mapping keys must be strings".into());
                };
                object.insert(key.to_string(), yaml_to_json(value)?);
            }
            Ok(Value::Object(object))
        }
        Yaml::Null | Yaml::BadValue => Ok(Value::Null),
        Yaml::Alias(_) => Err("unresolved YAML aliases are not supported".into()),
    }
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

fn resolve_bundle_path(source_path: &str, raw: &str) -> Option<String> {
    let raw = raw.replace('\\', "/");
    let directory = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let joined = if let Some(absolute) = raw.strip_prefix('/') {
        PathBuf::from(absolute)
    } else {
        directory.join(&raw)
    };
    let trailing_slash = raw.ends_with('/');
    let mut parts = Vec::new();
    for component in joined.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    let mut path = parts.join("/");
    if trailing_slash && !path.is_empty() {
        path.push('/');
    }
    Some(path)
}

fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}

fn external_url(url: &str) -> bool {
    url.contains("://")
        || url.starts_with("mailto:")
        || url.starts_with("tel:")
        || url.starts_with("data:")
}

fn validate_optional_string(
    relative: &str,
    metadata: &BTreeMap<String, Value>,
    field: &str,
    location: Option<SourceLocation>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if metadata.get(field).is_some_and(|value| !value.is_string()) {
        diagnostics.push(Diagnostic::error(
            "OKF1005",
            relative,
            location,
            format!("{field} must be a string"),
        ));
    }
}

fn string_array(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|values| values.iter().all(Value::is_string))
}

fn string_field<'a>(metadata: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    metadata.get(key).and_then(Value::as_str)
}

fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit()))
    {
        return false;
    }
    let month = value[5..7].parse::<u8>().unwrap_or(0);
    let day = value[8..10].parse::<u8>().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn footnote_labels(text: &str) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index + 3 < bytes.len() {
        if bytes[index] == b'['
            && bytes[index + 1] == b'^'
            && (index == 0 || bytes[index - 1] != b'\\')
            && let Some(end) = text[index + 2..].find(']')
        {
            let end = index + 2 + end;
            let label = &text[index + 2..end];
            if !label.is_empty() && !label.chars().any(char::is_whitespace) {
                labels.insert(label.to_string());
            }
            index = end + 1;
        } else {
            index += 1;
        }
    }
    labels
}

fn lines_with_offsets(source: &str) -> Vec<(usize, &str)> {
    let mut offset = 0;
    let mut lines = Vec::new();
    for line in source.split_inclusive('\n') {
        lines.push((offset, line));
        offset += line.len();
    }
    if source.is_empty() {
        return lines;
    }
    if !source.ends_with('\n') && lines.is_empty() {
        lines.push((0, source));
    }
    lines
}

fn location(source: SourceFile<'_>, span: Span) -> SourceLocation {
    let (line, column) = source.line_col(span.start);
    SourceLocation {
        start: span.start,
        end: span.end,
        line,
        column,
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn html_page(title: &str, article: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title></head><body><main class=\"rd-document\">{article}</main></body></html>\n",
        escape(title)
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocs-okf-{name}-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn concept(extra: &str, body: &str) -> String {
        format!(
            "---\ntype: Architecture\ntitle: Test\ndescription: A test concept.\ntags: [domain/rocs]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-08-16T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\n{extra}---\n\n{body}"
        )
    }

    #[test]
    fn preserves_unknown_metadata_and_original_body_offsets() {
        let root = temp("unknown");
        fs::write(
            root.join("test.md"),
            concept("extension_data: { nested: true }\n", "# Héading\n"),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        let concept = &bundle.concepts[0];
        assert_eq!(concept.metadata["extension_data"]["nested"], true);
        assert!(concept.body_span.start > 0);
        assert!(concept.headings[0].location.start >= concept.body_span.start);
        assert!(
            bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF2001")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keyed_footnotes_render_accessibly_and_validate_sources() {
        let root = temp("footnotes");
        fs::write(
            root.join("test.md"),
            concept(
                "sources:\n  - id: parser\n    resource: ../parser.rs\n",
                "# Test\n\nA claim.[^parser]\n\n[^parser]: Parser evidence.\n",
            ),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        assert!(!bundle.has_errors(), "{:?}", bundle.diagnostics);
        let html = &bundle.concepts[0].article_html;
        assert!(html.contains("data-footnote-ref"), "{html}");
        assert!(html.contains("aria-label=\"Footnotes\""), "{html}");
        assert!(html.contains("data-footnote-backref"), "{html}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recognizes_reserved_files_and_warns_for_broken_links() {
        let root = temp("reserved");
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(
            root.join("log.md"),
            "# Log\n\n## 2026-08-16\n\n* Created.\n",
        )
        .unwrap();
        fs::write(
            root.join("test.md"),
            concept("", "# Test\n\n[Missing](/missing.md)\n"),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        assert_eq!(bundle.version.as_deref(), Some("0.2"));
        assert_eq!(bundle.indexes.len(), 1);
        assert_eq!(bundle.logs.len(), 1);
        assert!(
            bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF3002"
                    && diagnostic.severity == Severity::Warning)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn base_and_rocci_profiles_are_separate() {
        let root = temp("profiles");
        fs::write(root.join("minimal.md"), "---\ntype: Note\n---\n\n# Note\n").unwrap();
        assert!(!load(&root, Profile::Base).unwrap().has_errors());
        assert!(load(&root, Profile::Rocci).unwrap().has_errors());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unresolved_keyed_footnote_is_an_error_but_fenced_declarations_are_inert() {
        let root = temp("unresolved-footnote");
        fs::write(
            root.join("test.md"),
            concept(
                "sources:\n  - id: missing\n    resource: ../missing.rs\n",
                "# Test\n\nClaim.[^missing]\n\n```rocdown\n@roc { x = 1 }\n```\n",
            ),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        assert!(
            bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF4003")
        );
        assert!(
            !bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF2007")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn build_emits_catalog_validation_and_site() {
        let root = temp("build");
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(root.join("test.md"), concept("", "# Test\n")).unwrap();
        let output = root.join("dist");
        let summary = build(&root, &output, Profile::Rocci).unwrap();
        assert_eq!(summary.concepts, 1);
        assert!(output.join("catalog.json").is_file());
        assert!(output.join("validation.json").is_file());
        assert!(output.join("site/test/index.html").is_file());
        let first_catalog = fs::read(output.join("catalog.json")).unwrap();
        let first_page = fs::read(output.join("site/test/index.html")).unwrap();
        build(&root, &output, Profile::Rocci).unwrap();
        assert_eq!(
            first_catalog,
            fs::read(output.join("catalog.json")).unwrap()
        );
        assert_eq!(
            first_page,
            fs::read(output.join("site/test/index.html")).unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
