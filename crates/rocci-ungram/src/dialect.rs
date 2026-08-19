use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use ungrammar::{Grammar, Node, Rule};

use crate::sidecar::Sidecar;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse ungrammar: {0}")]
    Ungram(String),
    #[error("{0}")]
    Dialect(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Ir {
    pub uses: Vec<String>,
    pub span_method: std::collections::BTreeSet<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum Variant {
    Unit { name: String },
    Tuple { name: String, ty: Ty },
    Struct { name: String, fields: Vec<Field> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Named(String),
    Vec(Box<Ty>),
    Option(Box<Ty>),
    Box(Box<Ty>),
    Tuple(Vec<Ty>),
    Raw(String),
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => write!(f, "{name}"),
            Self::Vec(inner) => write!(f, "Vec<{inner}>"),
            Self::Option(inner) => write!(f, "Option<{inner}>"),
            Self::Box(inner) => write!(f, "Box<{inner}>"),
            Self::Tuple(parts) => {
                write!(f, "(")?;
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{part}")?;
                }
                write!(f, ")")
            }
            Self::Raw(raw) => write!(f, "{raw}"),
        }
    }
}

impl Ty {
    fn named_by_value(&self) -> Vec<&str> {
        match self {
            Self::Named(name) if !is_primitive(name) => vec![name.as_str()],
            Self::Option(inner) | Self::Box(inner) => inner.named_by_value(),
            Self::Tuple(parts) => parts.iter().flat_map(Self::named_by_value).collect(),
            Self::Vec(_) | Self::Raw(_) | Self::Named(_) => Vec::new(),
        }
    }

    fn box_named(self, target: &str) -> Self {
        match self {
            Self::Named(name) if name == target => Self::Box(Box::new(Self::Named(name))),
            Self::Option(inner) => Self::Option(Box::new(inner.box_named(target))),
            Self::Tuple(parts) => Self::Tuple(
                parts
                    .into_iter()
                    .map(|part| part.box_named(target))
                    .collect(),
            ),
            other => other,
        }
    }
}

pub fn lower(ungram: &str, sidecar: &Sidecar) -> Result<Ir, Error> {
    let grammar = Grammar::from_str(ungram).map_err(|err| Error::Ungram(err.to_string()))?;
    let names = production_names(&grammar);
    for name in &names {
        if !sidecar.classified(name) {
            return Err(Error::Dialect(format!("unclassified production {name}")));
        }
    }

    let mut items = Vec::new();
    for node in grammar.iter() {
        let name = grammar[node].name.clone();
        check_rule_shape(&grammar, node)?;
        if sidecar.doc_only.contains(&name)
            || sidecar.foreign.contains_key(&name)
            || sidecar.opaque.contains_key(&name)
            || sidecar.inline.contains_key(&name)
        {
            continue;
        }
        if !sidecar.generated.contains(&name) {
            continue;
        }
        items.push(lower_node(&grammar, sidecar, node)?);
    }
    box_cycles(&mut items);
    crate::inspect::check_inspect(&names, sidecar)?;
    Ok(Ir {
        uses: sidecar.uses.clone(),
        span_method: sidecar.span_method.clone(),
        items,
    })
}

fn production_names(grammar: &Grammar) -> Vec<String> {
    grammar
        .iter()
        .map(|node| grammar[node].name.clone())
        .collect()
}

fn check_rule_shape(grammar: &Grammar, node: Node) -> Result<(), Error> {
    let name = &grammar[node].name;
    check_rule(grammar, name, &grammar[node].rule, false)
}

fn check_rule(grammar: &Grammar, name: &str, rule: &Rule, in_alt: bool) -> Result<(), Error> {
    match rule {
        Rule::Alt(alts) => {
            if in_alt {
                return Err(Error::Dialect(format!(
                    "nested anonymous alternative in {name}"
                )));
            }
            let mut saw_node = false;
            let mut saw_token = false;
            for alt in alts {
                match alt {
                    Rule::Node(_) => saw_node = true,
                    Rule::Token(_) => saw_token = true,
                    Rule::Alt(_) => {
                        return Err(Error::Dialect(format!(
                            "nested anonymous alternative in {name}"
                        )));
                    }
                    other => check_rule(grammar, name, other, true)?,
                }
            }
            if saw_node && saw_token {
                return Err(Error::Dialect(format!(
                    "mixed token and node alternatives in {name}"
                )));
            }
            Ok(())
        }
        Rule::Rep(inner) => check_rule(grammar, name, inner, in_alt),
        Rule::Seq(rules) => {
            for rule in rules {
                check_rule(grammar, name, rule, in_alt)?;
            }
            Ok(())
        }
        Rule::Opt(inner) | Rule::Labeled { rule: inner, .. } => {
            check_rule(grammar, name, inner, in_alt)
        }
        Rule::Node(_) | Rule::Token(_) => Ok(()),
    }
}

fn lower_node(grammar: &Grammar, sidecar: &Sidecar, node: Node) -> Result<Item, Error> {
    let name = grammar[node].name.clone();
    let rule = &grammar[node].rule;
    if let Rule::Alt(_) = rule {
        if is_enum_rule(rule) {
            return lower_enum(grammar, sidecar, &name, rule);
        }
        return Err(Error::Dialect(format!(
            "alternatives in {name} must be node names only"
        )));
    }
    lower_struct(grammar, sidecar, &name, rule)
}

fn is_enum_rule(rule: &Rule) -> bool {
    matches!(rule, Rule::Alt(alts) if alts.iter().all(|alt| matches!(alt, Rule::Node(_))))
}

fn lower_enum(
    grammar: &Grammar,
    sidecar: &Sidecar,
    name: &str,
    rule: &Rule,
) -> Result<Item, Error> {
    let Rule::Alt(alts) = rule else {
        return Err(Error::Dialect(format!("{name} is not an enum")));
    };
    let mut variants = Vec::new();
    for alt in alts {
        let Rule::Node(node) = alt else {
            return Err(Error::Dialect(format!(
                "mixed token and node alternatives in {name}"
            )));
        };
        let alt_name = grammar[*node].name.clone();
        variants.push(lower_variant(grammar, sidecar, name, &alt_name)?);
    }
    Ok(Item::Enum {
        name: name.to_string(),
        variants,
    })
}

fn lower_variant(
    grammar: &Grammar,
    sidecar: &Sidecar,
    enum_name: &str,
    alt_name: &str,
) -> Result<Variant, Error> {
    let variant_name = sidecar.variant_name(enum_name, alt_name);
    if let Some(inline) = sidecar.inline.get(alt_name) {
        return parse_inline_variant(&variant_name, inline);
    }
    if sidecar.generated.contains(alt_name)
        || sidecar.foreign.contains_key(alt_name)
        || sidecar.opaque.contains_key(alt_name)
    {
        return Ok(Variant::Tuple {
            name: variant_name,
            ty: Ty::Named(sidecar.rust_name(alt_name)),
        });
    }
    if let Some(leaf) = sidecar.leaves.get(alt_name) {
        if leaf == "Span" {
            return Ok(Variant::Struct {
                name: variant_name,
                fields: vec![Field {
                    name: "span".into(),
                    ty: Ty::Named("Span".into()),
                }],
            });
        }
        let fields = parse_named_fields(leaf)?;
        if fields.is_empty() {
            return Ok(Variant::Unit { name: variant_name });
        }
        return Ok(Variant::Struct {
            name: variant_name,
            fields,
        });
    }
    if is_token_leaf(grammar, alt_name) {
        return Err(Error::Dialect(format!(
            "leaf {alt_name} has no [leaves] rust type"
        )));
    }
    Err(Error::Dialect(format!(
        "cannot lower {enum_name} alternative {alt_name}"
    )))
}

fn parse_inline_variant(variant_name: &str, inline: &str) -> Result<Variant, Error> {
    let inline = inline.trim();
    if !inline.contains('{')
        && let Some((_, rhs)) = inline.split_once(':')
        && rhs.contains('<')
    {
        return Err(Error::Dialect(format!(
            "inline field type {inline} is not a variant"
        )));
    }
    let payload = inline
        .rsplit_once("::")
        .map(|(_, rest)| rest)
        .unwrap_or(inline);
    if let Some((head, fields)) = payload.split_once('{') {
        let head = head.trim();
        let name = head
            .rsplit_once("::")
            .map(|(_, name)| name)
            .unwrap_or(head)
            .trim();
        let fields_src = fields.trim().trim_end_matches('}').trim();
        if fields_src.is_empty() {
            return Ok(Variant::Unit {
                name: name.to_string(),
            });
        }
        let mut fields = Vec::new();
        for part in fields_src.split(',') {
            let part = part.trim();
            if part.is_empty() || part == ".." {
                continue;
            }
            if let Some((fname, ty)) = part.split_once(':') {
                fields.push(Field {
                    name: fname.trim().to_string(),
                    ty: parse_ty(ty.trim())?,
                });
            } else {
                fields.push(Field {
                    name: part.to_string(),
                    ty: inferred_inline_type(part),
                });
            }
        }
        return Ok(Variant::Struct {
            name: name.to_string(),
            fields,
        });
    }
    let name = payload
        .rsplit_once("::")
        .map(|(_, name)| name)
        .unwrap_or(payload)
        .trim();
    if name == variant_name || sidecar_unit_variant(name) {
        return Ok(Variant::Unit {
            name: name.to_string(),
        });
    }
    Ok(Variant::Unit {
        name: variant_name.to_string(),
    })
}

fn sidecar_unit_variant(name: &str) -> bool {
    !name.contains('(') && !name.contains('{')
}

fn inferred_inline_type(field: &str) -> Ty {
    match field {
        "span" | "expr" | "ty" | "condition" | "scrutinee" | "pattern" | "body" | "params" => {
            Ty::Named("Span".into())
        }
        "value" => Ty::Named("String".into()),
        "name" => Ty::Named("Ident".into()),
        "args" => Ty::Named("Span".into()),
        _ => Ty::Named("Span".into()),
    }
}

fn lower_struct(
    grammar: &Grammar,
    sidecar: &Sidecar,
    name: &str,
    rule: &Rule,
) -> Result<Item, Error> {
    if let Some(leaf) = sidecar.leaves.get(name)
        && is_token_leaf_rule(rule)
    {
        return Ok(Item::Struct {
            name: name.to_string(),
            fields: leaf_struct_fields(leaf)?,
        });
    }
    if is_token_leaf_rule(rule) && sidecar.leaves.get(name).is_none() {
        return Err(Error::Dialect(format!(
            "leaf {name} has no [leaves] rust type"
        )));
    }

    let mut fields = Vec::new();
    for part in flatten_seq(rule) {
        match part {
            Rule::Token(_) => {}
            Rule::Labeled { label, rule: inner } => {
                fields.extend(lower_field(grammar, sidecar, name, label, inner)?);
            }
            Rule::Node(node) => {
                return Err(Error::Dialect(format!(
                    "unlabeled node field {} in {name}",
                    grammar[*node].name
                )));
            }
            Rule::Rep(inner) if matches!(inner.as_ref(), Rule::Token(_)) => {
                return Err(Error::Dialect(format!(
                    "unlabeled repeated token in {name}"
                )));
            }
            other => {
                return Err(Error::Dialect(format!(
                    "unsupported rule in {name}: {other:?}"
                )));
            }
        }
    }

    let prefix = format!("{name}.");
    for (key, rust_ty) in &sidecar.add_fields {
        if let Some(field_name) = key.strip_prefix(&prefix) {
            fields.push(Field {
                name: field_name.to_string(),
                ty: parse_ty(rust_ty)?,
            });
        }
    }

    if !sidecar.omit_span.contains(name) && !fields.iter().any(|field| field.name == "span") {
        fields.push(Field {
            name: "span".into(),
            ty: Ty::Named("Span".into()),
        });
    }

    Ok(Item::Struct {
        name: name.to_string(),
        fields,
    })
}

fn leaf_struct_fields(leaf: &str) -> Result<Vec<Field>, Error> {
    if leaf == "Span" {
        return Ok(vec![Field {
            name: "span".into(),
            ty: Ty::Named("Span".into()),
        }]);
    }
    parse_named_fields(leaf)
}

fn lower_field(
    grammar: &Grammar,
    sidecar: &Sidecar,
    struct_name: &str,
    label: &str,
    rule: &Rule,
) -> Result<Vec<Field>, Error> {
    let flatten_key = format!("{struct_name}.{label}");
    if let Some(spec) = sidecar.flatten.get(&flatten_key) {
        return parse_named_fields(spec);
    }

    let (inner, multiplicity) = peel(rule);
    let Rule::Node(node) = inner else {
        return Err(Error::Dialect(format!(
            "labeled field {struct_name}.{label} must name a node"
        )));
    };
    let prod = &grammar[*node].name;
    if sidecar.doc_only.contains(prod) {
        return Err(Error::Dialect(format!(
            "production {prod} used as a field but is doc-only"
        )));
    }

    if let Some(inline) = sidecar.inline.get(prod)
        && let Some(ty) = inline_field_type(inline)
    {
        return Ok(vec![Field {
            name: label.to_string(),
            ty,
        }]);
    }

    if sidecar.leaves.contains_key(prod)
        && !sidecar.generated.contains(prod)
        && !sidecar.foreign.contains_key(prod)
        && !sidecar.opaque.contains_key(prod)
    {
        let leaf = &sidecar.leaves[prod];
        if leaf == "Span" {
            let ty = apply_multiplicity(Ty::Named("Span".into()), multiplicity);
            return Ok(vec![Field {
                name: label.to_string(),
                ty: apply_wrap(sidecar, &flatten_key, ty),
            }]);
        }
        return Err(Error::Dialect(format!(
            "{struct_name}.{label}: compound leaf {prod} needs [flatten]"
        )));
    }

    let mut ty = apply_multiplicity(Ty::Named(sidecar.rust_name(prod)), multiplicity);
    ty = apply_wrap(sidecar, &flatten_key, ty);
    Ok(vec![Field {
        name: label.to_string(),
        ty,
    }])
}

fn apply_wrap(sidecar: &Sidecar, key: &str, ty: Ty) -> Ty {
    match sidecar.wrap.get(key).map(String::as_str) {
        Some("Option") => Ty::Option(Box::new(ty)),
        Some("Box") => match ty {
            Ty::Box(_) => ty,
            other => Ty::Box(Box::new(other)),
        },
        _ => ty,
    }
}

fn apply_multiplicity(ty: Ty, multiplicity: Multiplicity) -> Ty {
    match multiplicity {
        Multiplicity::One => ty,
        Multiplicity::Opt => Ty::Option(Box::new(ty)),
        Multiplicity::Rep => Ty::Vec(Box::new(ty)),
    }
}

enum Multiplicity {
    One,
    Opt,
    Rep,
}

fn peel(rule: &Rule) -> (&Rule, Multiplicity) {
    match rule {
        Rule::Opt(inner) => (inner, Multiplicity::Opt),
        Rule::Rep(inner) => (inner, Multiplicity::Rep),
        other => (other, Multiplicity::One),
    }
}

fn flatten_seq(rule: &Rule) -> Vec<&Rule> {
    match rule {
        Rule::Seq(rules) => rules.iter().collect(),
        other => vec![other],
    }
}

fn is_token_leaf_rule(rule: &Rule) -> bool {
    matches!(rule, Rule::Token(_))
}

fn is_token_leaf(grammar: &Grammar, name: &str) -> bool {
    grammar
        .iter()
        .any(|node| grammar[node].name == name && matches!(grammar[node].rule, Rule::Token(_)))
}

fn inline_field_type(inline: &str) -> Option<Ty> {
    let (_, rhs) = inline.split_once(':')?;
    let rhs = rhs.trim();
    if rhs.is_empty() || rhs.contains("::") && !rhs.contains('<') {
        return None;
    }
    parse_ty(rhs).ok()
}

fn parse_named_fields(spec: &str) -> Result<Vec<Field>, Error> {
    let spec = spec
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if spec.is_empty() {
        return Ok(Vec::new());
    }
    let mut fields = Vec::new();
    for part in split_top_level(spec, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((name, ty)) = part.split_once(':') else {
            return Err(Error::Dialect(format!(
                "expected name:Type in field list, got {part}"
            )));
        };
        fields.push(Field {
            name: name.trim().to_string(),
            ty: parse_ty(ty.trim())?,
        });
    }
    Ok(fields)
}

fn parse_ty(src: &str) -> Result<Ty, Error> {
    let src = src.trim();
    if let Some(inner) = src.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
        return Ok(Ty::Vec(Box::new(parse_ty(inner)?)));
    }
    if let Some(inner) = src
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return Ok(Ty::Option(Box::new(parse_ty(inner)?)));
    }
    if let Some(inner) = src.strip_prefix("Box<").and_then(|s| s.strip_suffix('>')) {
        return Ok(Ty::Box(Box::new(parse_ty(inner)?)));
    }
    if let Some(inner) = src.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        let parts = split_top_level(inner, ',')
            .into_iter()
            .map(parse_ty)
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Ty::Tuple(parts));
    }
    if src
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Ok(Ty::Named(src.to_string()));
    }
    Ok(Ty::Raw(src.to_string()))
}

fn split_top_level(src: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, ch) in src.char_indices() {
        match ch {
            '<' | '(' | '{' => depth += 1,
            '>' | ')' | '}' => depth -= 1,
            c if c == sep && depth == 0 => {
                parts.push(&src[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&src[start..]);
    parts
}

fn box_cycles(items: &mut [Item]) {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for item in items.iter() {
        let (name, names) = match item {
            Item::Struct { name, fields } => (
                name.clone(),
                fields
                    .iter()
                    .flat_map(|field| field.ty.named_by_value().into_iter().map(str::to_string))
                    .collect::<Vec<_>>(),
            ),
            Item::Enum { name, variants } => (
                name.clone(),
                variants
                    .iter()
                    .flat_map(variant_named_by_value)
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
            ),
        };
        graph.insert(name, names);
    }

    let mut to_box: HashSet<(String, String)> = HashSet::new();
    let type_names: Vec<String> = graph.keys().cloned().collect();
    for item in items.iter() {
        if let Item::Struct { name, fields } = item {
            for field in fields {
                for target in field.ty.named_by_value() {
                    if can_reach(&graph, target, name, &type_names) {
                        to_box.insert((name.clone(), target.to_string()));
                    }
                }
            }
        }
    }

    for item in items.iter_mut() {
        if let Item::Struct { name, fields } = item {
            for field in fields.iter_mut() {
                for (_, target) in to_box.iter().filter(|(owner, _)| owner == name) {
                    let current = std::mem::replace(&mut field.ty, Ty::Named(String::new()));
                    field.ty = current.box_named(target);
                }
            }
        }
    }
}

fn variant_named_by_value(variant: &Variant) -> Vec<&str> {
    match variant {
        Variant::Unit { .. } => Vec::new(),
        Variant::Tuple { ty, .. } => ty.named_by_value(),
        Variant::Struct { fields, .. } => fields
            .iter()
            .flat_map(|field| field.ty.named_by_value())
            .collect(),
    }
}

fn can_reach(graph: &HashMap<String, Vec<String>>, from: &str, to: &str, all: &[String]) -> bool {
    if from == to {
        return true;
    }
    let mut seen = HashSet::new();
    let mut stack = vec![from.to_string()];
    while let Some(cur) = stack.pop() {
        if !seen.insert(cur.clone()) {
            continue;
        }
        let Some(edges) = graph.get(&cur) else {
            continue;
        };
        for next in edges {
            if next == to {
                return true;
            }
            if all.iter().any(|name| name == next) {
                stack.push(next.clone());
            }
        }
    }
    false
}

fn is_primitive(name: &str) -> bool {
    matches!(
        name,
        "Span" | "String" | "bool" | "u8" | "u32" | "u64" | "i32"
    )
}
