use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::ast::{Concept, Index, Profile, Span};
use crate::diagnostic::{Diagnostic, SourceLocation};
use crate::frontmatter::location;
use crate::graph::{resolve_bundle_path, split_fragment};

pub const STANDARD_FIELDS: &[&str] = &[
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

pub const PROFILE_TYPES: &[&str] = &[
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

pub fn validate_metadata(
    relative: &str,
    source: &str,
    span: Span,
    metadata: &BTreeMap<String, Value>,
    profile: Profile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let at = Some(location(source, span));
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
                && object
                    .get("at")
                    .and_then(Value::as_str)
                    .is_some_and(|value| parse_timestamp(value).is_some())
        })
    {
        diagnostics.push(Diagnostic::error(
            "OKF1008",
            relative,
            at.clone(),
            "generated must be a mapping with string `by` and RFC 3339 `at`",
        ));
    }
    if let Some(verified) = metadata.get("verified")
        && !verified.as_array().is_some_and(|events| {
            events.iter().all(|event| {
                event.as_object().is_some_and(|object| {
                    object.get("by").is_some_and(Value::is_string)
                        && object
                            .get("at")
                            .and_then(Value::as_str)
                            .is_some_and(|value| parse_timestamp(value).is_some())
                })
            })
        })
    {
        diagnostics.push(Diagnostic::error(
            "OKF1010",
            relative,
            at.clone(),
            "verified must be a list of mappings with string `by` and RFC 3339 `at`",
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

pub fn collect_source_ids(
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

pub fn validate_unique_ids(concepts: &[Concept], diagnostics: &mut Vec<Diagnostic>) {
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

pub fn validate_route_collisions(
    concepts: &[Concept],
    indexes: &[Index],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut collections = BTreeMap::new();
    for index in indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        collections.insert(collection.to_ascii_lowercase(), index.path.clone());
    }
    for concept in concepts {
        let folded = concept.id.to_ascii_lowercase();
        if let Some(index_path) = collections.get(&folded) {
            diagnostics.push(Diagnostic::error(
                "OKF3005",
                &concept.path,
                None,
                format!(
                    "concept id `{id}` collides with collection `{index_path}`",
                    id = concept.id
                ),
            ));
        }
    }
}

pub fn validate_index_membership(
    concepts: &[Concept],
    indexes: &[Index],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let indexes_by_path: BTreeMap<&str, &Index> = indexes
        .iter()
        .map(|index| (index.path.as_str(), index))
        .collect();
    for concept in concepts {
        let Some((dir, _)) = concept.path.rsplit_once('/') else {
            continue;
        };
        let index_path = format!("{dir}/index.md");
        let Some(index) = indexes_by_path.get(index_path.as_str()) else {
            diagnostics.push(Diagnostic::warning(
                "OKF2010",
                &concept.path,
                None,
                format!("concept is not listed because `{index_path}` is missing"),
            ));
            continue;
        };
        if !index_lists_concept(index, concept) {
            diagnostics.push(Diagnostic::warning(
                "OKF2010",
                &concept.path,
                None,
                format!("concept is not listed in `{index_path}`"),
            ));
        }
    }
}

fn index_lists_concept(index: &Index, concept: &Concept) -> bool {
    index.links.iter().any(|link| {
        if external_url(&link.url) || link.url.starts_with('#') {
            return false;
        }
        let (path, _) = split_fragment(&link.url);
        let Some(resolved) = resolve_bundle_path(&index.path, path) else {
            return false;
        };
        resolved == concept.path
            || resolved == concept.id
            || resolved == format!("{}.md", concept.id)
    })
}

static GIT_INVOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, Default)]
struct GitProvenance {
    last_modified: BTreeMap<String, GitModification>,
    dirty: BTreeSet<String>,
}

impl GitProvenance {
    fn modification(&self, relative: &str) -> GitModification {
        self.last_modified
            .get(relative)
            .copied()
            .unwrap_or(GitModification::Untracked)
    }

    fn is_dirty(&self, relative: &str) -> bool {
        self.dirty.contains(relative)
    }
}

pub fn validate_lifecycle_and_sources(
    root: &Path,
    concepts: &[Concept],
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_lifecycle_and_sources_with(root, concepts, diagnostics, true);
}

pub fn validate_lifecycle_and_sources_with(
    root: &Path,
    concepts: &[Concept],
    diagnostics: &mut Vec<Diagnostic>,
    check_git: bool,
) {
    let today = current_utc_date();
    let repository = check_git.then(|| git_repository_root(root)).flatten();
    let mut relative_paths = BTreeSet::new();
    let mut source_refs = Vec::new();

    for concept in concepts {
        if let (Some(today), Some(stale_after)) = (
            today.as_deref(),
            string_field(&concept.metadata, "stale_after"),
        ) && is_date(stale_after)
            && stale_after < today
        {
            diagnostics.push(Diagnostic::warning(
                "OKF4004",
                &concept.path,
                None,
                format!("record is stale: stale_after was {stale_after}"),
            ));
        }

        let generated_at = concept
            .metadata
            .get("generated")
            .and_then(Value::as_object)
            .and_then(|generated| generated.get("at"))
            .and_then(Value::as_str)
            .and_then(parse_timestamp);
        let human_verification = latest_human_verification(&concept.metadata);
        if let (Some(generated_at), Some((verified_at, _))) =
            (generated_at, human_verification.as_ref())
            && *verified_at < generated_at
        {
            diagnostics.push(Diagnostic::warning(
                "OKF4005",
                &concept.path,
                None,
                "latest human verification is older than generated.at",
            ));
        }

        if !check_git {
            continue;
        }
        let Some(repository) = repository.as_deref() else {
            continue;
        };
        let Some(sources) = concept.metadata.get("sources").and_then(Value::as_array) else {
            continue;
        };
        for source in sources {
            let Some(source) = source.as_object() else {
                continue;
            };
            let Some(resource) = source.get("resource").and_then(Value::as_str) else {
                continue;
            };
            if external_url(resource) || Path::new(resource).is_absolute() {
                continue;
            }
            let source_id = source
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(resource)
                .to_string();
            let Some(path) = repository_source_path(root, repository, &concept.path, resource)
            else {
                continue;
            };
            let Some(relative) = path.strip_prefix(repository).ok() else {
                continue;
            };
            let relative_display = git_path_key(relative);
            relative_paths.insert(relative_display.clone());
            source_refs.push((
                concept.path.clone(),
                source_id,
                relative_display,
                path,
                human_verification.map(|(timestamp, label)| (timestamp, label.to_string())),
            ));
        }
    }

    if !check_git {
        return;
    }
    let Some(repository) = repository.as_deref() else {
        return;
    };
    let git_state = load_git_provenance(repository, &relative_paths);
    for (concept_path, source_id, relative_display, path, human_verification) in source_refs {
        match git_state.modification(&relative_display) {
            GitModification::Tracked(modified_at) => {
                if let Some((verified_at, verified_label)) = human_verification.as_ref()
                    && modified_at > *verified_at
                {
                    diagnostics.push(Diagnostic::warning(
                        "OKF4006",
                        &concept_path,
                        None,
                        format!(
                            "source `{source_id}` ({relative_display}) changed after human verification at {verified_label}"
                        ),
                    ));
                }
                if let Some((verified_at, _)) = human_verification.as_ref()
                    && git_state.is_dirty(&relative_display)
                    && filesystem_modified_at(&path)
                        .is_none_or(|modified_at| modified_at > *verified_at)
                {
                    diagnostics.push(Diagnostic::warning(
                        "OKF4008",
                        &concept_path,
                        None,
                        format!(
                            "source `{source_id}` ({relative_display}) has uncommitted changes and cannot be matched to its human verification"
                        ),
                    ));
                }
            }
            GitModification::Untracked if path.exists() => {
                diagnostics.push(Diagnostic::warning(
                    "OKF4007",
                    &concept_path,
                    None,
                    format!(
                        "source `{source_id}` ({relative_display}) is untracked and has no git provenance"
                    ),
                ));
            }
            GitModification::Unknown | GitModification::Untracked => {}
        }
    }
}

pub fn latest_human_verification(metadata: &BTreeMap<String, Value>) -> Option<(i64, &str)> {
    metadata
        .get("verified")?
        .as_array()?
        .iter()
        .filter_map(Value::as_object)
        .filter(|event| {
            event
                .get("by")
                .and_then(Value::as_str)
                .is_some_and(|actor| actor.starts_with("human:"))
        })
        .filter_map(|event| {
            let at = event.get("at")?.as_str()?;
            Some((parse_timestamp(at)?, at))
        })
        .max_by_key(|(timestamp, _)| *timestamp)
}

pub fn repository_source_path(
    root: &Path,
    repository: &Path,
    concept_path: &str,
    resource: &str,
) -> Option<PathBuf> {
    let parent = root.join(concept_path).parent()?.to_path_buf();
    let joined = parent.join(resource);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    let normalized = normalized.canonicalize().unwrap_or(normalized);
    normalized.starts_with(repository).then_some(normalized)
}

pub fn git_repository_root(root: &Path) -> Option<PathBuf> {
    let output = git_command(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    output.status.success().then(|| {
        let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
        path.canonicalize().unwrap_or(path)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitModification {
    Tracked(i64),
    Untracked,
    Unknown,
}

pub fn git_last_modified(repository: &Path, relative: &Path) -> GitModification {
    load_git_provenance(repository, &BTreeSet::from([git_path_key(relative)]))
        .modification(&git_path_key(relative))
}

pub fn git_path_dirty(repository: &Path, relative: &Path) -> bool {
    load_git_provenance(repository, &BTreeSet::from([git_path_key(relative)]))
        .is_dirty(&git_path_key(relative))
}

fn git_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn git_command(dir: &Path) -> Command {
    GIT_INVOCATIONS.fetch_add(1, Ordering::Relaxed);
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir);
    cmd
}

fn load_git_provenance(repository: &Path, relatives: &BTreeSet<String>) -> GitProvenance {
    if relatives.is_empty() {
        return GitProvenance::default();
    }
    let dirty = git_dirty_paths(repository);
    let timestamps = git_last_modified_paths(repository, relatives);
    let last_modified = relatives
        .iter()
        .map(|relative| {
            let modification = match &timestamps {
                None => GitModification::Unknown,
                Some(found) => found
                    .get(relative)
                    .copied()
                    .map(GitModification::Tracked)
                    .unwrap_or(GitModification::Untracked),
            };
            (relative.clone(), modification)
        })
        .collect();
    GitProvenance {
        last_modified,
        dirty,
    }
}

fn git_dirty_paths(repository: &Path) -> BTreeSet<String> {
    let output = git_command(repository)
        .args(["status", "--porcelain", "-z", "--untracked-files=no"])
        .output();
    let Ok(output) = output else {
        return BTreeSet::new();
    };
    if !output.status.success() {
        return BTreeSet::new();
    }
    parse_porcelain_z(&output.stdout)
}

fn parse_porcelain_z(stdout: &[u8]) -> BTreeSet<String> {
    let mut dirty = BTreeSet::new();
    for record in stdout.split(|byte| *byte == 0) {
        if record.len() < 4 || record[2] != b' ' {
            continue;
        }
        let xy = &record[..2];
        if xy == b"??" || xy == b"!!" {
            continue;
        }
        let path = String::from_utf8_lossy(&record[3..]).replace('\\', "/");
        if !path.is_empty() {
            dirty.insert(path);
        }
    }
    dirty
}

fn git_last_modified_paths(
    repository: &Path,
    relatives: &BTreeSet<String>,
) -> Option<BTreeMap<String, i64>> {
    let mut cmd = git_command(repository);
    cmd.args(["log", "--format=%cI", "--name-only", "--"]);
    for relative in relatives {
        cmd.arg(relative);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_git_log_name_only(
        &String::from_utf8_lossy(&output.stdout),
        relatives,
    ))
}

fn parse_git_log_name_only(stdout: &str, wanted: &BTreeSet<String>) -> BTreeMap<String, i64> {
    let mut current_ts = None;
    let mut found = BTreeMap::new();
    for line in stdout.lines() {
        if found.len() == wanted.len() {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(timestamp) = parse_timestamp(line) {
            current_ts = Some(timestamp);
            continue;
        }
        let key = line.replace('\\', "/");
        if wanted.contains(&key)
            && let Some(timestamp) = current_ts
        {
            found.entry(key).or_insert(timestamp);
        }
    }
    found
}

pub fn filesystem_modified_at(path: &Path) -> Option<i64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let seconds = modified.duration_since(UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(seconds).ok()
}

pub fn validate_optional_string(
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

pub fn string_array(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|values| values.iter().all(Value::is_string))
}

pub fn string_field<'a>(metadata: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    metadata.get(key).and_then(Value::as_str)
}

pub fn is_date(value: &str) -> bool {
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

pub fn parse_timestamp(value: &str) -> Option<i64> {
    if !value.is_ascii()
        || value.len() < 20
        || &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
    {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let hour = value[11..13].parse::<i64>().ok()?;
    let minute = value[14..16].parse::<i64>().ok()?;
    let second = value[17..19].parse::<i64>().ok()?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=60).contains(&second)
    {
        return None;
    }
    let mut timezone = &value[19..];
    if let Some(fraction_and_timezone) = timezone.strip_prefix('.') {
        let timezone_start = fraction_and_timezone.find(['Z', '+', '-'])?;
        let fraction = &fraction_and_timezone[..timezone_start];
        if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        timezone = &fraction_and_timezone[timezone_start..];
    }
    let offset = if timezone == "Z" {
        0
    } else if timezone.len() == 6
        && (&timezone[0..1] == "+" || &timezone[0..1] == "-")
        && &timezone[3..4] == ":"
    {
        let hours = timezone[1..3].parse::<i64>().ok()?;
        let minutes = timezone[4..6].parse::<i64>().ok()?;
        if hours > 23 || minutes > 59 {
            return None;
        }
        let seconds = hours * 3_600 + minutes * 60;
        if &timezone[0..1] == "+" {
            seconds
        } else {
            -seconds
        }
    } else {
        return None;
    };
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second - offset)
}

pub fn days_from_civil(mut year: i64, month: i64, day: i64) -> i64 {
    year -= i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

pub fn current_utc_date() -> Option<String> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let days = (seconds / 86_400) as i64 + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

pub fn external_url(url: &str) -> bool {
    url.contains("://")
        || url.starts_with("mailto:")
        || url.starts_with("tel:")
        || url.starts_with("data:")
        || url.starts_with("okf:")
}

pub fn metadata_string_array(metadata: &BTreeMap<String, Value>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Profile, load};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("okf-provenance-{name}-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn git(repo: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    fn git_commit(repo: &Path, message: &str, date: &str) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["commit", "-m", message])
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .status()
            .unwrap();
        assert!(status.success(), "git commit {message} failed");
    }

    fn concept(sources: &str) -> String {
        format!(
            "---\ntype: Architecture\ntitle: Provenance\ndescription: Provenance fixture.\ntags: [domain/okf, concern/validation]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-07-01T00:00:00Z }}\nverified:\n  - {{ by: human:nils, at: 2026-08-05T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\nsources:\n{sources}---\n\n# Provenance\n\nBody.[^tracked]\n\n[^tracked]: Tracked source.\n"
        )
    }

    #[test]
    fn parse_git_log_keeps_first_timestamp_per_path() {
        let wanted = BTreeSet::from(["a.rs".to_string(), "b.rs".to_string()]);
        let found = parse_git_log_name_only(
            "2026-08-10T00:00:00Z\n\na.rs\n\n2026-08-01T00:00:00Z\n\na.rs\nb.rs\n",
            &wanted,
        );
        assert_eq!(
            found["a.rs"],
            parse_timestamp("2026-08-10T00:00:00Z").unwrap()
        );
        assert_eq!(
            found["b.rs"],
            parse_timestamp("2026-08-01T00:00:00Z").unwrap()
        );
    }

    #[test]
    fn parse_porcelain_z_lists_dirty_tracked_paths() {
        let stdout = b" M tracked.txt\0?? untracked.txt\0A  staged.txt\0";
        let dirty = parse_porcelain_z(stdout);
        assert!(dirty.contains("tracked.txt"));
        assert!(dirty.contains("staged.txt"));
        assert!(!dirty.contains("untracked.txt"));
    }

    #[test]
    fn batched_provenance_emits_stable_codes_with_constant_git_calls() {
        let root = temp("batch");
        git(&root, &["init", "--initial-branch=main"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        git(&root, &["config", "commit.gpgsign", "false"]);

        fs::write(root.join("tracked.txt"), "tracked v1\n").unwrap();
        fs::write(root.join("dirty.txt"), "dirty v1\n").unwrap();
        git(&root, &["add", "tracked.txt", "dirty.txt"]);
        git_commit(&root, "initial", "2026-08-01T00:00:00Z");

        fs::write(root.join("tracked.txt"), "tracked v2\n").unwrap();
        git(&root, &["add", "tracked.txt"]);
        git_commit(&root, "update tracked", "2026-08-10T00:00:00Z");
        fs::write(root.join("dirty.txt"), "dirty v2\n").unwrap();
        fs::write(root.join("untracked.txt"), "new\n").unwrap();

        fs::write(
            root.join("note.md"),
            concept(
                "  - id: tracked\n    resource: tracked.txt\n    title: Tracked\n  - id: dirty\n    resource: dirty.txt\n    title: Dirty\n  - id: untracked\n    resource: untracked.txt\n    title: Untracked\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("copy.md"),
            concept("  - id: tracked\n    resource: tracked.txt\n    title: Tracked again\n"),
        )
        .unwrap();

        GIT_INVOCATIONS.store(0, Ordering::SeqCst);
        let bundle = load(&root, Profile::Rocci).expect("load rocci fixture");
        let git_calls = GIT_INVOCATIONS.load(Ordering::SeqCst);
        assert!(
            git_calls <= 3,
            "expected constant git invocations, got {git_calls}"
        );

        let codes: Vec<_> = bundle
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.message.as_str()))
            .collect();
        assert!(
            codes.iter().any(|(code, message)| {
                *code == "OKF4006" && message.contains("source `tracked`")
            }),
            "missing OKF4006: {codes:?}"
        );
        assert!(
            codes.iter().any(|(code, message)| {
                *code == "OKF4008" && message.contains("source `dirty`")
            }),
            "missing OKF4008: {codes:?}"
        );
        assert!(
            codes.iter().any(|(code, message)| {
                *code == "OKF4007" && message.contains("source `untracked`")
            }),
            "missing OKF4007: {codes:?}"
        );

        let _ = fs::remove_dir_all(root);
    }
}
