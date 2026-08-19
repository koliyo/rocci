use std::collections::BTreeSet;
use std::str::FromStr;

use ungrammar::Grammar;

use crate::dialect::Error;
use crate::sidecar::Sidecar;

pub fn check_inspect(ungram_names: &[String], sidecar: &Sidecar) -> Result<(), Error> {
    check_inspect_keys(ungram_names, sidecar)?;
    check_generated_coverage(ungram_names, sidecar)?;
    Ok(())
}

pub fn format_inspect_mapping(ungram: &str, sidecar_toml: &str) -> Result<String, Error> {
    let sidecar = Sidecar::parse(sidecar_toml)?;
    let grammar = Grammar::from_str(ungram).map_err(|err| Error::Ungram(err.to_string()))?;
    let names: Vec<String> = grammar
        .iter()
        .map(|node| grammar[node].name.clone())
        .collect();
    check_inspect(&names, &sidecar)?;
    Ok(render_mapping(&names, &sidecar))
}

pub fn format_appendix_table(ungram: &str, sidecar_toml: &str) -> Result<String, Error> {
    let sidecar = Sidecar::parse(sidecar_toml)?;
    let grammar = Grammar::from_str(ungram).map_err(|err| Error::Ungram(err.to_string()))?;
    let names: Vec<String> = grammar
        .iter()
        .map(|node| grammar[node].name.clone())
        .collect();
    check_inspect(&names, &sidecar)?;
    Ok(render_appendix_table(&names, &sidecar))
}

fn check_inspect_keys(ungram_names: &[String], sidecar: &Sidecar) -> Result<(), Error> {
    let names: BTreeSet<&str> = ungram_names.iter().map(String::as_str).collect();
    let inspect = &sidecar.inspect;
    let mut seen = BTreeSet::new();
    for (section, keys) in [
        ("tags", inspect.tags.keys()),
        ("omit", inspect.omit.keys()),
        ("fallback", inspect.fallback.keys()),
    ] {
        for key in keys {
            if !seen.insert(key) {
                return Err(Error::Dialect(format!(
                    "inspect key {key} appears in more than one of tags, omit, and fallback"
                )));
            }
            let prefix = key.split('.').next().unwrap_or(key);
            if !names.contains(prefix) {
                return Err(Error::Dialect(format!(
                    "inspect.{section} key {key} does not name an ungram production"
                )));
            }
        }
    }
    for key in inspect.overlay.keys() {
        let prefix = key.split('.').next().unwrap_or(key);
        if !names.contains(prefix) {
            return Err(Error::Dialect(format!(
                "inspect.overlay key {key} does not name an ungram production"
            )));
        }
    }
    for key in sidecar.highlight.omit.keys() {
        if !names.contains(key.as_str()) {
            return Err(Error::Dialect(format!(
                "highlight.omit key {key} does not name an ungram production"
            )));
        }
        if !sidecar.generated.contains(key) {
            return Err(Error::Dialect(format!(
                "highlight.omit key {key} is not a generated production"
            )));
        }
    }
    Ok(())
}

fn check_generated_coverage(ungram_names: &[String], sidecar: &Sidecar) -> Result<(), Error> {
    let mut missing = Vec::new();
    for name in ungram_names {
        if sidecar.generated.contains(name) && !sidecar.inspect.covers(name) {
            missing.push(name.clone());
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(Error::Dialect(format!(
            "generated production(s) have no inspect tag, omit, or fallback: {}",
            missing.join(", ")
        )))
    }
}

fn render_mapping(ungram_names: &[String], sidecar: &Sidecar) -> String {
    let mut lines = Vec::new();
    lines.push("# production\tkind\tvalue".to_string());
    let mut keys = BTreeSet::new();
    keys.extend(ungram_names.iter().cloned());
    keys.extend(sidecar.inspect.tags.keys().cloned());
    keys.extend(sidecar.inspect.omit.keys().cloned());
    keys.extend(sidecar.inspect.fallback.keys().cloned());
    for key in keys {
        let (kind, value) = mapping_row(&key, sidecar);
        lines.push(format!("{key}\t{kind}\t{value}"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn render_appendix_table(ungram_names: &[String], sidecar: &Sidecar) -> String {
    let mut lines = Vec::new();
    lines.push("| Production | Inspect | Classification |".to_string());
    lines.push("| --- | --- | --- |".to_string());
    let mut keys = BTreeSet::new();
    keys.extend(ungram_names.iter().cloned());
    keys.extend(sidecar.inspect.tags.keys().cloned());
    keys.extend(sidecar.inspect.omit.keys().cloned());
    keys.extend(sidecar.inspect.fallback.keys().cloned());
    for key in keys {
        let inspect = appendix_inspect(&key, sidecar);
        let class = classification(&key, sidecar);
        lines.push(format!("| `{key}` | {inspect} | {class} |"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn appendix_inspect(key: &str, sidecar: &Sidecar) -> String {
    let (kind, value) = mapping_row(key, sidecar);
    match kind {
        "tag" => format!("`{value}`"),
        "fallback" => format!("`{value}` (fallback)"),
        "omit" => "omit".into(),
        _ => "—".into(),
    }
}

fn classification(key: &str, sidecar: &Sidecar) -> &'static str {
    let prefix = key.split('.').next().unwrap_or(key);
    if sidecar.generated.contains(prefix) {
        "generated"
    } else if sidecar.foreign.contains_key(prefix) {
        "foreign"
    } else if sidecar.opaque.contains_key(prefix) {
        "opaque"
    } else if sidecar.doc_only.contains(prefix) {
        "doc-only"
    } else if sidecar.inline.contains_key(prefix) {
        "inline"
    } else if sidecar.leaves.contains_key(prefix) {
        "leaf"
    } else {
        "unclassified"
    }
}

fn mapping_row(key: &str, sidecar: &Sidecar) -> (&'static str, String) {
    if let Some(tag) = sidecar.inspect.tags.get(key) {
        return ("tag", tag.clone());
    }
    if let Some(reason) = sidecar.inspect.omit.get(key) {
        return ("omit", reason.clone());
    }
    if let Some(tag) = sidecar.inspect.fallback.get(key) {
        return ("fallback", tag.clone());
    }
    if sidecar.doc_only.contains(key) {
        return ("doc_only", "—".into());
    }
    if sidecar.foreign.contains_key(key) {
        return ("foreign", "—".into());
    }
    if sidecar.opaque.contains_key(key) {
        return ("opaque", "—".into());
    }
    if sidecar.inline.contains_key(key) {
        return ("inline", "—".into());
    }
    if sidecar.leaves.contains_key(key) {
        return ("leaf", "—".into());
    }
    if sidecar.generated.contains(key) {
        return ("generated", "uncovered".into());
    }
    ("unmapped", "—".into())
}
