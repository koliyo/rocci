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

pub fn camel_to_pascal(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.extend(first.to_uppercase());
    out.extend(chars);
    out
}

pub fn component_matches(source_name: &str, query: &str) -> bool {
    source_name == query || pascal_to_camel(source_name) == query
}

pub fn component_name_error(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    if is_ambiguous_pascal(name) {
        return Some(format!(
            "ambiguous component name `{name}`; write `@component HtmlShell` rather than `@component HTMLShell`"
        ));
    }
    if !name
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        return Some(format!(
            "component names must be PascalCase; write `@component {}`",
            camel_to_pascal(name)
        ));
    }
    None
}

pub fn fixture_target_name_error(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    if is_ambiguous_pascal(name) {
        return Some(format!(
            "ambiguous fixture target `{name}`; write `HtmlShell` rather than `HTMLShell`"
        ));
    }
    if !name
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        return Some(format!(
            "fixture targets must be PascalCase; write `{}`",
            camel_to_pascal(name)
        ));
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_pascal_and_camel() {
        assert_eq!(pascal_to_camel("Hello"), "hello");
        assert_eq!(pascal_to_camel("CounterCard"), "counterCard");
        assert_eq!(pascal_to_camel("HtmlShell"), "htmlShell");
        assert_eq!(camel_to_pascal("hello"), "Hello");
        assert_eq!(camel_to_pascal("counterCard"), "CounterCard");
        assert!(is_ambiguous_pascal("HTMLShell"));
        assert!(!is_ambiguous_pascal("HtmlShell"));
        assert!(!is_ambiguous_pascal("Hello"));
        assert!(component_matches("Hello", "hello"));
        assert!(component_matches("Hello", "Hello"));
        assert!(!component_matches("Hello", "Badge"));
        assert!(
            component_name_error("hello")
                .unwrap()
                .contains("PascalCase")
        );
        assert!(
            component_name_error("HTMLShell")
                .unwrap()
                .contains("ambiguous")
        );
        assert!(component_name_error("Hello").is_none());
    }
}
