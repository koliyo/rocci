use crate::ast::Ident;

pub fn is_ambiguous_pascal(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && chars.next().is_some_and(|ch| ch.is_ascii_uppercase())
}

pub fn pascal_to_camel(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.extend(first.to_lowercase());
    out.extend(chars);
    out
}

pub fn component_roc_name(parts: &[Ident]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    if parts.len() == 1 {
        let name = &parts[0].name;
        if name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            return pascal_to_camel(name);
        }
        return name.clone();
    }
    let mut roc = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            roc.push('.');
        }
        if i == parts.len() - 1 {
            roc.push_str(&pascal_to_camel(&part.name));
        } else {
            roc.push_str(&part.name);
        }
    }
    roc
}
