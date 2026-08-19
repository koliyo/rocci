//! Closed v1 article-block kind schema.
//!
//! Kind names are data, not parser keywords. Unknown `@docs` kinds, parent/child
//! placement, and simple required-field diagnostics are driven from this table.

const LINK_CARD_TARGET: &[&str] = &["page", "href"];

pub const TAB_KIND_VALUES: &[&str] = &["language", "platform", "tool"];
pub const BADGE_TONE_VALUES: &[&str] = &["stable", "beta", "preview", "deprecated", "removed"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindFamily {
    Aside,
    Structure,
    Chrome,
    Tooling,
    Sugar,
    Reserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindSpec {
    pub name: &'static str,
    pub component: &'static str,
    pub family: KindFamily,
    pub authorable: bool,
    pub diagnostic_code: &'static str,
    pub required_fields: &'static [&'static str],
    pub optional_fields: &'static [&'static str],
    pub parents: &'static [&'static str],
    pub child_kinds: &'static [&'static str],
    pub required_child_kinds: &'static [&'static str],
    pub forbidden_children: &'static [&'static str],
    pub required_one_of: &'static [&'static [&'static str]],
}

pub const KINDS: &[KindSpec] = &[
    aside("note", "Note"),
    aside("tip", "Tip"),
    aside("caution", "Caution"),
    aside("danger", "Danger"),
    aside("deprecated", "Deprecated"),
    KindSpec {
        name: "details",
        component: "Details",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &["summary"],
        optional_fields: &["open"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "steps",
        component: "Steps",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &[],
        parents: &[],
        child_kinds: &["step"],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "step",
        component: "Step",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["title", "verify"],
        parents: &["steps"],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "figure",
        component: "Figure",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["caption", "credit"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "definition",
        component: "Definition",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &["term"],
        optional_fields: &[],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "tabs",
        component: "Tabs",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2405",
        required_fields: &["group", "kind"],
        optional_fields: &[],
        parents: &[],
        child_kinds: &["tab"],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "tab",
        component: "Tab",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2405",
        required_fields: &["id", "label"],
        optional_fields: &[],
        parents: &["tabs"],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "badge",
        component: "Badge",
        family: KindFamily::Structure,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &["label"],
        optional_fields: &["tone"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "link-card",
        component: "LinkCard",
        family: KindFamily::Chrome,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["title", "summary", "page", "href"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[LINK_CARD_TARGET],
    },
    KindSpec {
        name: "card-grid",
        component: "CardGrid",
        family: KindFamily::Chrome,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &[],
        parents: &[],
        child_kinds: &["link-card"],
        required_child_kinds: &["link-card"],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "file-tree",
        component: "FileTree",
        family: KindFamily::Chrome,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &[],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "compatibility",
        component: "Compatibility",
        family: KindFamily::Chrome,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["caption"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "include",
        component: "Include",
        family: KindFamily::Tooling,
        authorable: true,
        diagnostic_code: "RD2501",
        required_fields: &["path"],
        optional_fields: &["region", "start", "end"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "example",
        component: "Example",
        family: KindFamily::Tooling,
        authorable: true,
        diagnostic_code: "RD2602",
        required_fields: &[],
        optional_fields: &[
            "path",
            "language",
            "test",
            "expect",
            "allow_network",
            "region",
        ],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "playground",
        component: "Playground",
        family: KindFamily::Tooling,
        authorable: false,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["id"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    sugar("h1", "H1"),
    sugar("h2", "H2"),
    sugar("h3", "H3"),
    sugar("h4", "H4"),
    sugar("h5", "H5"),
    sugar("h6", "H6"),
    KindSpec {
        name: "img",
        component: "Img",
        family: KindFamily::Sugar,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &["src"],
        optional_fields: &[
            "alt",
            "decorative",
            "title",
            "width",
            "height",
            "class",
            "loading",
            "decoding",
        ],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
    KindSpec {
        name: "api-operation",
        component: "ApiOperation",
        family: KindFamily::Reserved,
        authorable: false,
        diagnostic_code: "RD2406",
        required_fields: &[],
        optional_fields: &["id"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    },
];

const fn aside(name: &'static str, component: &'static str) -> KindSpec {
    KindSpec {
        name,
        component,
        family: KindFamily::Aside,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["title"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &["tabs"],
        required_one_of: &[],
    }
}

const fn sugar(name: &'static str, component: &'static str) -> KindSpec {
    KindSpec {
        name,
        component,
        family: KindFamily::Sugar,
        authorable: true,
        diagnostic_code: "RD2402",
        required_fields: &[],
        optional_fields: &["id"],
        parents: &[],
        child_kinds: &[],
        required_child_kinds: &[],
        forbidden_children: &[],
        required_one_of: &[],
    }
}

pub fn lookup(name: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|kind| kind.name == name)
}

pub fn is_aside(name: &str) -> bool {
    lookup(name).is_some_and(|kind| kind.family == KindFamily::Aside)
}

pub fn is_reserved(name: &str) -> bool {
    lookup(name).is_some_and(|kind| kind.family == KindFamily::Reserved)
}

pub fn is_docs_kind(name: &str) -> bool {
    lookup(name).is_some_and(|kind| {
        matches!(
            kind.family,
            KindFamily::Aside | KindFamily::Structure | KindFamily::Chrome | KindFamily::Tooling
        )
    })
}

pub fn heading_level(name: &str) -> Option<u8> {
    name.strip_prefix('h')
        .and_then(|rest| rest.parse::<u8>().ok())
        .filter(|level| (1..=6).contains(level))
}

pub fn parent_allowed(spec: &KindSpec, parent_kind: Option<&str>) -> bool {
    spec.parents.is_empty()
        || parent_kind.is_some_and(|parent| spec.parents.iter().any(|want| *want == parent))
}

pub fn module_collision(name: &str) -> bool {
    matches!(
        name,
        "page"
            | "roc"
            | "render"
            | "component"
            | "fixture"
            | "css"
            | "context"
            | "init"
            | "on"
            | "if"
            | "for"
            | "match"
            | "let"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;

    #[test]
    fn render_kinds_have_registry_rows() {
        let src = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/DocsComponents.rocci"
        ))
        .expect("DocsComponents.rocci");
        for kind in render_match_kinds(&src) {
            assert!(
                lookup(&kind).is_some(),
                "DocsComponents.Render matches `{kind}` but the registry has no row"
            );
        }
    }

    #[test]
    fn authorable_docs_kinds_have_named_components() {
        let src = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/DocsComponents.rocci"
        ))
        .expect("DocsComponents.rocci");
        for kind in KINDS {
            if !kind.authorable || kind.family == KindFamily::Sugar {
                continue;
            }
            let needle = format!("@component {}", kind.component);
            assert!(
                src.contains(&needle),
                "DocsComponents.rocci missing painter `{needle}` for kind `{}`",
                kind.name
            );
        }
        assert!(
            src.contains("@component Render"),
            "thin Render adapter should remain until typed props land"
        );
        assert!(
            !src.contains("@component DocsAside"),
            "DocsAside should not remain as the public aside painter"
        );
    }

    #[test]
    fn api_operation_is_reserved_and_not_authorable() {
        let spec = lookup("api-operation").expect("api-operation row");
        assert_eq!(spec.family, KindFamily::Reserved);
        assert!(!spec.authorable);
        assert!(is_reserved("api-operation"));
        assert!(!is_docs_kind("api-operation"));
    }

    #[test]
    fn unknown_widget_is_absent() {
        assert!(lookup("widget").is_none());
        assert!(!is_docs_kind("widget"));
        assert!(!is_reserved("widget"));
    }

    #[test]
    fn playground_stays_a_docs_kind() {
        let spec = lookup("playground").expect("playground row");
        assert!(!spec.authorable);
        assert!(is_docs_kind("playground"));
    }

    #[test]
    fn sugar_kinds_are_not_docs_kinds() {
        for name in ["h1", "h2", "h3", "h4", "h5", "h6", "img"] {
            let spec = lookup(name).expect(name);
            assert_eq!(spec.family, KindFamily::Sugar);
            assert!(!is_docs_kind(name));
        }
    }

    #[test]
    fn registry_names_are_unique() {
        let mut seen = BTreeSet::new();
        for kind in KINDS {
            assert!(
                seen.insert(kind.name),
                "duplicate registry kind `{}`",
                kind.name
            );
            assert!(!kind.component.is_empty());
        }
    }

    fn render_match_kinds(src: &str) -> Vec<String> {
        let start = src
            .find("@match segment.kind")
            .expect("Render kind matcher");
        let block = &src[start..];
        let end = block.find("_ =>").expect("wildcard arm");
        let block = &block[..end];
        let mut kinds = Vec::new();
        let bytes = block.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'"' {
                let rest = &block[i + 1..];
                if let Some(close) = rest.find('"') {
                    let token = &rest[..close];
                    let after = rest.get(close + 1..).unwrap_or("");
                    let trimmed = after.trim_start();
                    if trimmed.starts_with("=>")
                        && token
                            .chars()
                            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                        && !token.is_empty()
                    {
                        kinds.push(token.to_string());
                    }
                    i += close + 2;
                    continue;
                }
            }
            i += 1;
        }
        kinds
    }
}
