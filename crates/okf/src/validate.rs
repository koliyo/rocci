use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::ast::{Concept, Profile, Span};
use crate::diagnostic::{Diagnostic, SourceLocation};
use crate::frontmatter::location;

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

pub fn validate_lifecycle_and_sources(
    root: &Path,
    concepts: &[Concept],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let today = current_utc_date();
    let repository = git_repository_root(root);

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
            let source_id = source.get("id").and_then(Value::as_str).unwrap_or(resource);
            let Some(path) = repository_source_path(root, repository, &concept.path, resource)
            else {
                continue;
            };
            let Some(relative) = path.strip_prefix(repository).ok() else {
                continue;
            };
            let relative_display = relative.to_string_lossy().replace('\\', "/");
            match git_last_modified(repository, relative) {
                GitModification::Tracked(modified_at) => {
                    if let Some((verified_at, verified_label)) = human_verification.as_ref()
                        && modified_at > *verified_at
                    {
                        diagnostics.push(Diagnostic::warning(
                            "OKF4006",
                            &concept.path,
                            None,
                            format!(
                                "source `{source_id}` ({relative_display}) changed after human verification at {verified_label}"
                            ),
                        ));
                    }
                    if let Some((verified_at, _)) = human_verification.as_ref()
                        && git_path_dirty(repository, relative)
                        && filesystem_modified_at(&path)
                            .is_none_or(|modified_at| modified_at > *verified_at)
                    {
                        diagnostics.push(Diagnostic::warning(
                            "OKF4008",
                            &concept.path,
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
                        &concept.path,
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
    let output = Command::new("git")
        .args(["-C", root.to_str()?, "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    output.status.success().then(|| {
        let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
        path.canonicalize().unwrap_or(path)
    })
}

pub enum GitModification {
    Tracked(i64),
    Untracked,
    Unknown,
}

pub fn git_last_modified(repository: &Path, relative: &Path) -> GitModification {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["log", "-1", "--format=%cI", "--"])
        .arg(relative)
        .output();
    let Ok(output) = output else {
        return GitModification::Unknown;
    };
    if !output.status.success() {
        return GitModification::Unknown;
    }
    let timestamp = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if timestamp.is_empty() {
        GitModification::Untracked
    } else if let Some(timestamp) = parse_timestamp(&timestamp) {
        GitModification::Tracked(timestamp)
    } else {
        GitModification::Unknown
    }
}

pub fn git_path_dirty(repository: &Path, relative: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["status", "--porcelain", "--untracked-files=no", "--"])
        .arg(relative)
        .output()
        .is_ok_and(|output| output.status.success() && !output.stdout.is_empty())
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
