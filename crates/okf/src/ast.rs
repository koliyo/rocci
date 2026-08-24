use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::diagnostic::{Diagnostic, Severity, SourceLocation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start as u32,
            end: end as u32,
        }
    }

    pub fn point(pos: usize) -> Self {
        Self {
            start: pos as u32,
            end: pos as u32,
        }
    }

    pub fn of<'a>(&self, src: &'a str) -> &'a str {
        let start = (self.start as usize).min(src.len());
        let end = (self.end as usize).min(src.len());
        if start <= end { &src[start..end] } else { "" }
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Base,
    Rocci,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadOptions {
    pub profile: Profile,
    pub provenance: bool,
}

impl LoadOptions {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile,
            provenance: profile == Profile::Rocci,
        }
    }

    pub fn with_provenance(mut self, provenance: bool) -> Self {
        self.provenance = provenance;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustTier {
    HumanReviewed,
    Generated,
    Unverified,
}

impl TrustTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HumanReviewed => "human-reviewed",
            Self::Generated => "generated",
            Self::Unverified => "unverified",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnowledgeFilter {
    pub types: Vec<String>,
    pub tags: Vec<String>,
    pub statuses: Vec<String>,
    pub authorities: Vec<String>,
    pub trust_tiers: Vec<TrustTier>,
    pub stale: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectKind {
    Catalog,
    Concept,
    Graph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingSection {
    pub id: String,
    pub heading_text: String,
    pub body_texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub path: String,
    pub metadata: BTreeMap<String, Value>,
    pub body_span: Span,
    pub body_location: SourceLocation,
    pub headings: Vec<Heading>,
    pub heading_sections: Vec<HeadingSection>,
    pub links: Vec<Link>,
    pub source_ids: BTreeSet<String>,
    pub footnote_ids: BTreeSet<String>,
    pub article_html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub path: String,
    pub version: Option<String>,
    pub body_span: Span,
    #[serde(default)]
    pub headings: Vec<Heading>,
    #[serde(default)]
    pub links: Vec<Link>,
    pub article_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
