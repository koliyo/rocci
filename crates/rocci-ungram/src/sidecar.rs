use std::collections::{BTreeMap, BTreeSet};

use crate::dialect::Error;

#[derive(Debug, Clone)]
pub struct Sidecar {
    pub uses: Vec<String>,
    pub generated: BTreeSet<String>,
    pub foreign: BTreeMap<String, String>,
    pub opaque: BTreeMap<String, String>,
    pub doc_only: BTreeSet<String>,
    pub inline: BTreeMap<String, String>,
    pub leaves: BTreeMap<String, String>,
    pub add_fields: BTreeMap<String, String>,
    pub wrap: BTreeMap<String, String>,
    pub flatten: BTreeMap<String, String>,
    pub omit_span: BTreeSet<String>,
    pub span_method: BTreeSet<String>,
    pub variants: BTreeMap<String, BTreeMap<String, String>>,
    pub inspect: InspectSpec,
    pub highlight: HighlightSpec,
}

#[derive(Debug, Clone, Default)]
pub struct HighlightSpec {
    pub omit: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct InspectSpec {
    pub tags: BTreeMap<String, String>,
    pub omit: BTreeMap<String, String>,
    pub fallback: BTreeMap<String, String>,
    pub overlay: BTreeMap<String, String>,
}

impl Sidecar {
    pub fn parse(src: &str) -> Result<Self, Error> {
        let value: toml::Value = toml::from_str(src)
            .map_err(|err| Error::Dialect(format!("invalid sidecar TOML: {err}")))?;
        let table = value
            .as_table()
            .ok_or_else(|| Error::Dialect("sidecar must be a TOML table".into()))?;

        let mut sidecar = Self {
            uses: Vec::new(),
            generated: BTreeSet::new(),
            foreign: BTreeMap::new(),
            opaque: BTreeMap::new(),
            doc_only: BTreeSet::new(),
            inline: BTreeMap::new(),
            leaves: BTreeMap::new(),
            add_fields: BTreeMap::new(),
            wrap: BTreeMap::new(),
            flatten: BTreeMap::new(),
            omit_span: BTreeSet::new(),
            span_method: BTreeSet::new(),
            variants: BTreeMap::new(),
            inspect: InspectSpec::default(),
            highlight: HighlightSpec::default(),
        };

        if let Some(meta) = table.get("meta").and_then(|v| v.as_table()) {
            sidecar.uses = string_list(meta.get("uses"));
        }
        sidecar.generated = table_keys(table.get("generated"));
        sidecar.foreign = string_map(table.get("foreign"))?;
        sidecar.opaque = string_map(table.get("opaque"))?;
        sidecar.doc_only = table_keys(table.get("doc_only"));
        sidecar.inline = string_map(table.get("inline"))?;
        sidecar.leaves = string_map(table.get("leaves"))?;
        sidecar.add_fields = string_map(table.get("add_fields"))?;
        sidecar.wrap = string_map(table.get("wrap"))?;
        sidecar.flatten = string_map(table.get("flatten"))?;
        sidecar.omit_span = bool_or_key_set(table.get("omit_span"))?;
        sidecar.span_method = bool_or_key_set(table.get("span_method"))?;
        sidecar.inspect = InspectSpec::parse(table.get("inspect"))?;
        sidecar.highlight = HighlightSpec::parse(table.get("highlight"))?;

        for (key, value) in table {
            if let Some(stem) = key.strip_suffix("_variants") {
                sidecar
                    .variants
                    .insert(snake_to_pascal(stem), string_map(Some(value))?);
            }
        }

        Ok(sidecar)
    }

    pub fn classified(&self, name: &str) -> bool {
        self.generated.contains(name)
            || self.foreign.contains_key(name)
            || self.opaque.contains_key(name)
            || self.doc_only.contains(name)
            || self.inline.contains_key(name)
            || self.leaves.contains_key(name)
    }

    pub fn variant_name(&self, enum_name: &str, alt: &str) -> String {
        self.variants
            .get(enum_name)
            .and_then(|map| map.get(alt))
            .cloned()
            .unwrap_or_else(|| alt.to_string())
    }

    pub fn rust_name(&self, production: &str) -> String {
        if let Some(path) = self.foreign.get(production) {
            return path.rsplit("::").next().unwrap_or(path).to_string();
        }
        if let Some(path) = self.opaque.get(production) {
            return path.rsplit("::").next().unwrap_or(path).to_string();
        }
        production.to_string()
    }
}

impl InspectSpec {
    fn parse(value: Option<&toml::Value>) -> Result<Self, Error> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let table = value
            .as_table()
            .ok_or_else(|| Error::Dialect("[inspect] must be a TOML table".into()))?;
        for key in table.keys() {
            if !matches!(key.as_str(), "tags" | "omit" | "fallback" | "overlay") {
                return Err(Error::Dialect(format!(
                    "unknown [inspect] table {key}; expected tags, omit, fallback, or overlay"
                )));
            }
        }
        Ok(Self {
            tags: string_map(table.get("tags"))?,
            omit: string_map(table.get("omit"))?,
            fallback: string_map(table.get("fallback"))?,
            overlay: string_map(table.get("overlay"))?,
        })
    }

    pub fn covers(&self, production: &str) -> bool {
        self.tags.contains_key(production)
            || self.omit.contains_key(production)
            || self.fallback.contains_key(production)
    }
}

impl HighlightSpec {
    fn parse(value: Option<&toml::Value>) -> Result<Self, Error> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let table = value
            .as_table()
            .ok_or_else(|| Error::Dialect("[highlight] must be a TOML table".into()))?;
        for key in table.keys() {
            if key.as_str() != "omit" {
                return Err(Error::Dialect(format!(
                    "unknown [highlight] table {key}; expected omit"
                )));
            }
        }
        Ok(Self {
            omit: string_map(table.get("omit"))?,
        })
    }
}

fn table_keys(value: Option<&toml::Value>) -> BTreeSet<String> {
    match value.and_then(|v| v.as_table()) {
        Some(table) => table.keys().cloned().collect(),
        None => BTreeSet::new(),
    }
}

fn string_map(value: Option<&toml::Value>) -> Result<BTreeMap<String, String>, Error> {
    let Some(table) = value.and_then(|v| v.as_table()) else {
        return Ok(BTreeMap::new());
    };
    let mut map = BTreeMap::new();
    for (key, value) in table {
        let Some(text) = value.as_str() else {
            return Err(Error::Dialect(format!(
                "sidecar key {key} must be a string"
            )));
        };
        map.insert(key.clone(), text.to_string());
    }
    Ok(map)
}

fn string_list(value: Option<&toml::Value>) -> Vec<String> {
    match value {
        Some(toml::Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(toml::Value::String(text)) => vec![text.clone()],
        _ => Vec::new(),
    }
}

fn bool_or_key_set(value: Option<&toml::Value>) -> Result<BTreeSet<String>, Error> {
    let Some(value) = value else {
        return Ok(BTreeSet::new());
    };
    if let Some(table) = value.as_table() {
        return Ok(table
            .iter()
            .filter(|(_, value)| value.as_bool() != Some(false))
            .map(|(key, _)| key.clone())
            .collect());
    }
    if let Some(array) = value.as_array() {
        return Ok(array
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect());
    }
    Err(Error::Dialect(
        "omit_span and span_method must be tables or arrays".into(),
    ))
}

fn snake_to_pascal(stem: &str) -> String {
    stem.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
