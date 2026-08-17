pub mod actions;
pub mod attributes;
pub mod modifiers;

pub use actions::{ACTIONS, ActionSpec, lookup_action};
pub use attributes::{ATTRIBUTES, AttributeSpec, lookup_attribute};
pub use modifiers::{MODIFIERS, ModifierSpec, lookup_modifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedAttribute {
    pub directive: &'static str,
    pub key: Option<String>,
    pub modifiers: Vec<ParsedModifier>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedModifier {
    pub name: String,
    pub argument: Option<String>,
}

pub fn is_datastar_attribute(name: &str) -> bool {
    lookup_attribute(name).is_some()
}

pub fn parse_attribute(attr_name: &str) -> Option<ParsedAttribute> {
    let spec = lookup_attribute(attr_name)?;
    let remainder = attr_name.strip_prefix(spec.name)?;

    let mut key = None;
    let mut modifiers = Vec::new();

    let parts: Vec<&str> = remainder.split("__").collect();
    if let Some(first) = parts.first()
        && let Some(k) = first.strip_prefix(':')
        && !k.is_empty()
    {
        key = Some(k.to_string());
    }

    for mod_part in parts.iter().skip(1) {
        if mod_part.is_empty() {
            continue;
        }
        let (name, arg) = match mod_part.split_once('.') {
            Some((name, arg)) => (name.to_string(), Some(arg.to_string())),
            None => (mod_part.to_string(), None),
        };
        modifiers.push(ParsedModifier {
            name,
            argument: arg,
        });
    }

    Some(ParsedAttribute {
        directive: spec.name,
        key,
        modifiers,
    })
}
