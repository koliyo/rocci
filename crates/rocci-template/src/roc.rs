use std::path::Path;

pub fn type_name_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("View");
    let mut out = String::new();
    let mut cap_next = true;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            if cap_next {
                out.extend(ch.to_uppercase());
                cap_next = false;
            } else {
                out.push(ch);
            }
        } else {
            cap_next = true;
        }
    }
    if out.is_empty() {
        "View".to_string()
    } else {
        out
    }
}

pub fn wrap_type_module(src: &str, type_name: &str) -> String {
    let mut imports = Vec::new();
    let mut body = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") && trimmed.contains(" exposing ") {
            continue;
        }
        if line.starts_with("import ") {
            imports.push(line);
        } else {
            body.push(line);
        }
    }
    while body.first().is_some_and(|line| line.trim().is_empty()) {
        body.remove(0);
    }
    while body.last().is_some_and(|line| line.trim().is_empty()) {
        body.pop();
    }
    let indented = body
        .iter()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut out = String::new();
    if !imports.is_empty() {
        out.push_str(&imports.join("\n"));
        out.push_str("\n\n");
    }
    out.push_str(type_name);
    out.push_str(" := [].{\n");
    out.push_str(&indented);
    out.push_str("\n}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_name_pascal_cases_file_stem() {
        assert_eq!(type_name_from_path(Path::new("foo.rocci")), "Foo");
        assert_eq!(type_name_from_path(Path::new("Counter.rocci")), "Counter");
        assert_eq!(type_name_from_path(Path::new("foo-bar.rocci")), "FooBar");
    }

    #[test]
    fn wrap_type_module_strips_header_and_keeps_imports() {
        let src = "\
module CounterPage exposing [hello]

import Html

hello = |{ name }| {
    Html.text(name)
}
";
        let wrapped = wrap_type_module(src, "Foo");
        assert!(!wrapped.contains("module CounterPage"));
        assert!(wrapped.starts_with("import Html\n\nFoo := [].{\n"));
        assert!(wrapped.contains("    hello = |{ name }| {"));
        assert!(wrapped.ends_with("}\n"));
    }
}
