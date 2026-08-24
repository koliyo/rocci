use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::ast::{Concept, Index, Log, Profile};
use crate::diagnostic::{Diagnostic, Severity, SourceLocation, intern_diagnostic_code};

pub const PARSE_CACHE_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileFingerprint {
    pub modified_secs: u64,
    pub modified_nanos: u32,
    pub len: u64,
}

#[derive(Clone, Debug)]
pub(crate) enum CachedDocument {
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
pub(crate) struct CacheEntry {
    pub fingerprint: FileFingerprint,
    pub document: CachedDocument,
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

    pub fn load_dir(path: &Path, profile: Profile) -> Self {
        let mut cache = Self {
            profile: Some(profile),
            entries: BTreeMap::new(),
        };
        let stored = match fs::read_to_string(path.join("entries.json")) {
            Ok(raw) => raw,
            Err(_) => return cache,
        };
        let Ok(stored) = serde_json::from_str::<StoredCache>(&stored) else {
            return cache;
        };
        if stored.version != PARSE_CACHE_VERSION {
            return cache;
        }
        if stored.profile != profile_name(profile) {
            return cache;
        }
        for (relative, entry) in stored.entries {
            if let Some(entry) = entry.into_runtime() {
                cache.entries.insert(relative, entry);
            }
        }
        cache
    }

    pub fn save_dir(&self, path: &Path) -> Result<()> {
        let Some(profile) = self.profile else {
            return Ok(());
        };
        fs::create_dir_all(path)
            .with_context(|| format!("failed to create parse cache {}", path.display()))?;
        let stored = StoredCache {
            version: PARSE_CACHE_VERSION,
            profile: profile_name(profile).to_string(),
            entries: self
                .entries
                .iter()
                .map(|(relative, entry)| (relative.clone(), StoredEntry::from_runtime(entry)))
                .collect(),
        };
        let json = serde_json::to_string(&stored).context("failed to serialize parse cache")?;
        let tmp = path.join("entries.json.tmp");
        let dest = path.join("entries.json");
        fs::write(&tmp, json).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &dest).with_context(|| format!("failed to replace {}", dest.display()))?;
        Ok(())
    }

    pub(crate) fn begin(&mut self, profile: Profile) {
        if self.profile != Some(profile) {
            self.entries.clear();
            self.profile = Some(profile);
        }
    }

    pub(crate) fn get(
        &self,
        relative: &str,
        fingerprint: FileFingerprint,
    ) -> Option<&CachedDocument> {
        self.entries
            .get(relative)
            .and_then(|entry| (entry.fingerprint == fingerprint).then_some(&entry.document))
    }

    pub(crate) fn insert(
        &mut self,
        relative: String,
        fingerprint: FileFingerprint,
        document: CachedDocument,
    ) {
        self.entries.insert(
            relative,
            CacheEntry {
                fingerprint,
                document,
            },
        );
    }

    pub(crate) fn retain_paths(&mut self, live: &BTreeSet<String>) {
        self.entries.retain(|path, _| live.contains(path));
    }
}

pub(crate) fn file_fingerprint(path: &Path) -> Option<FileFingerprint> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some(FileFingerprint {
        modified_secs: modified.as_secs(),
        modified_nanos: modified.subsec_nanos(),
        len: meta.len(),
    })
}

pub(crate) fn apply_cached(
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

pub(crate) fn capture_cached(
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

fn profile_name(profile: Profile) -> &'static str {
    match profile {
        Profile::Base => "base",
        Profile::Rocci => "rocci",
    }
}

#[derive(Serialize, Deserialize)]
struct StoredCache {
    version: u32,
    profile: String,
    entries: BTreeMap<String, StoredEntry>,
}

#[derive(Serialize, Deserialize)]
struct StoredEntry {
    fingerprint: FileFingerprint,
    document: StoredDocument,
}

impl StoredEntry {
    fn from_runtime(entry: &CacheEntry) -> Self {
        Self {
            fingerprint: entry.fingerprint,
            document: StoredDocument::from_runtime(&entry.document),
        }
    }

    fn into_runtime(self) -> Option<CacheEntry> {
        Some(CacheEntry {
            fingerprint: self.fingerprint,
            document: self.document.into_runtime()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum StoredDocument {
    Concept {
        concept: Concept,
        diagnostics: Vec<StoredDiagnostic>,
    },
    Index {
        index: Index,
        diagnostics: Vec<StoredDiagnostic>,
    },
    Log {
        log: Log,
        diagnostics: Vec<StoredDiagnostic>,
    },
    Diagnostics {
        diagnostics: Vec<StoredDiagnostic>,
    },
}

impl StoredDocument {
    fn from_runtime(document: &CachedDocument) -> Self {
        match document {
            CachedDocument::Concept {
                concept,
                diagnostics,
            } => Self::Concept {
                concept: concept.clone(),
                diagnostics: diagnostics
                    .iter()
                    .map(StoredDiagnostic::from_runtime)
                    .collect(),
            },
            CachedDocument::Index { index, diagnostics } => Self::Index {
                index: index.clone(),
                diagnostics: diagnostics
                    .iter()
                    .map(StoredDiagnostic::from_runtime)
                    .collect(),
            },
            CachedDocument::Log { log, diagnostics } => Self::Log {
                log: log.clone(),
                diagnostics: diagnostics
                    .iter()
                    .map(StoredDiagnostic::from_runtime)
                    .collect(),
            },
            CachedDocument::Diagnostics(diagnostics) => Self::Diagnostics {
                diagnostics: diagnostics
                    .iter()
                    .map(StoredDiagnostic::from_runtime)
                    .collect(),
            },
        }
    }

    fn into_runtime(self) -> Option<CachedDocument> {
        Some(match self {
            Self::Concept {
                concept,
                diagnostics,
            } => CachedDocument::Concept {
                concept,
                diagnostics: intern_diagnostics(diagnostics)?,
            },
            Self::Index { index, diagnostics } => CachedDocument::Index {
                index,
                diagnostics: intern_diagnostics(diagnostics)?,
            },
            Self::Log { log, diagnostics } => CachedDocument::Log {
                log,
                diagnostics: intern_diagnostics(diagnostics)?,
            },
            Self::Diagnostics { diagnostics } => {
                CachedDocument::Diagnostics(intern_diagnostics(diagnostics)?)
            }
        })
    }
}

#[derive(Serialize, Deserialize)]
struct StoredDiagnostic {
    code: String,
    severity: Severity,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<SourceLocation>,
    message: String,
}

impl StoredDiagnostic {
    fn from_runtime(diagnostic: &Diagnostic) -> Self {
        Self {
            code: diagnostic.code.to_string(),
            severity: diagnostic.severity,
            path: diagnostic.path.clone(),
            location: diagnostic.location.clone(),
            message: diagnostic.message.clone(),
        }
    }

    fn into_runtime(self) -> Option<Diagnostic> {
        Some(Diagnostic {
            code: intern_diagnostic_code(&self.code)?,
            severity: self.severity,
            path: self.path,
            location: self.location,
            message: self.message,
        })
    }
}

fn intern_diagnostics(stored: Vec<StoredDiagnostic>) -> Option<Vec<Diagnostic>> {
    stored
        .into_iter()
        .map(StoredDiagnostic::into_runtime)
        .collect()
}
