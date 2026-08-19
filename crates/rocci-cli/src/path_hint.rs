use std::fs;
use std::path::Path;

pub fn looks_like_okf_file(path: &Path) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    looks_like_okf_markdown(&source)
}

pub fn looks_like_okf_bundle(dir: &Path) -> bool {
    looks_like_okf_file(&dir.join("index.md"))
}

pub fn looks_like_okf_markdown(source: &str) -> bool {
    let Some(yaml) = leading_yaml_frontmatter(source) else {
        return false;
    };
    let keys = top_level_yaml_keys(yaml);
    if keys.is_empty() {
        return false;
    }
    (keys.contains(&"type") && keys.contains(&"authority"))
        || keys.iter().all(|key| *key == "okf_version")
}

fn leading_yaml_frontmatter(source: &str) -> Option<&str> {
    let rest = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))?;
    let end = rest.find("\n---").or_else(|| rest.find("\r\n---"))?;
    Some(&rest[..end])
}

fn top_level_yaml_keys(yaml: &str) -> Vec<&str> {
    let mut keys = Vec::new();
    for line in yaml.lines() {
        if line.starts_with(' ') || line.starts_with('\t') || line.starts_with('#') {
            continue;
        }
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let Some(colon) = line.find(':') else {
            continue;
        };
        let key = &line[..colon];
        if !key.is_empty()
            && key
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            keys.push(key);
        }
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_concept_frontmatter() {
        let source = "---\ntype: Implementation Plan\ntitle: Example\nauthority: exploratory\n---\n\n# Example\n";
        assert!(looks_like_okf_markdown(source));
    }

    #[test]
    fn detects_bundle_root_index() {
        let source = "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n";
        assert!(looks_like_okf_markdown(source));
    }

    #[test]
    fn ignores_ordinary_markdown_and_partial_yaml() {
        assert!(!looks_like_okf_markdown("# Plan\n\nBody.\n"));
        assert!(!looks_like_okf_markdown(
            "---\ntitle: Plan\ndescription: A document\n---\n\n# Plan\n"
        ));
        assert!(!looks_like_okf_markdown(
            "---\ntype: Note\ntitle: Missing authority\n---\n\n# Note\n"
        ));
    }
}
