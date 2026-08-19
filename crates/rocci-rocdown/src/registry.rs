//! Closed v1 article-block kind schema.
//!
//! Kind names are data, not parser keywords. Unknown article kinds, parent/child
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
pub enum PaintType {
    Str,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintField {
    pub prop: &'static str,
    pub attr: &'static str,
    pub ty: PaintType,
}

impl PaintField {
    pub const fn str(name: &'static str) -> Self {
        Self {
            prop: name,
            attr: name,
            ty: PaintType::Str,
        }
    }

    pub const fn bool(name: &'static str) -> Self {
        Self {
            prop: name,
            attr: name,
            ty: PaintType::Bool,
        }
    }

    pub const fn map_str(prop: &'static str, attr: &'static str) -> Self {
        Self {
            prop,
            attr,
            ty: PaintType::Str,
        }
    }
}

const TITLE: &[PaintField] = &[PaintField::str("title")];
const DETAILS: &[PaintField] = &[PaintField::str("summary"), PaintField::bool("open")];
const STEP: &[PaintField] = &[PaintField::str("title"), PaintField::bool("verify")];
const FIGURE: &[PaintField] = &[PaintField::str("caption"), PaintField::str("credit")];
const DEFINITION: &[PaintField] = &[PaintField::map_str("title", "term")];
const TAB: &[PaintField] = &[PaintField::str("label")];
const BADGE: &[PaintField] = &[PaintField::str("label")];
const LINK_CARD: &[PaintField] = &[
    PaintField::str("href"),
    PaintField::str("title"),
    PaintField::str("summary"),
];
const CAPTION: &[PaintField] = &[PaintField::str("caption")];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildPredicate {
    None,
    StepsXorList,
    FigureOneImage,
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
    pub accepts: &'static [&'static str],
    pub accepts_markdown: bool,
    pub requires: &'static [&'static str],
    pub forbids: &'static [&'static str],
    pub child_predicate: ChildPredicate,
    pub required_one_of: &'static [&'static [&'static str]],
}

impl KindSpec {
    pub fn paints_as_widget(self) -> bool {
        matches!(
            self.family,
            KindFamily::Aside | KindFamily::Structure | KindFamily::Chrome | KindFamily::Tooling
        ) && self.name != "playground"
    }

    pub fn paint_content(self) -> bool {
        self.paints_as_widget() && !matches!(self.name, "badge" | "link-card")
    }

    pub fn paint_fields(self) -> &'static [PaintField] {
        match self.name {
            "note" | "tip" | "caution" | "danger" | "deprecated" => TITLE,
            "details" => DETAILS,
            "step" => STEP,
            "figure" => FIGURE,
            "definition" => DEFINITION,
            "tab" => TAB,
            "badge" => BADGE,
            "link-card" => LINK_CARD,
            "compatibility" => CAPTION,
            _ => &[],
        }
    }

    pub fn completion_fields(self) -> impl Iterator<Item = &'static str> {
        self.required_fields
            .iter()
            .copied()
            .chain(self.optional_fields.iter().copied())
    }

    pub fn accepts_block_child(self, child: &str) -> bool {
        !self.forbids.contains(&child) && (self.accepts.is_empty() || self.accepts.contains(&child))
    }

    pub fn rejects_markdown(self) -> bool {
        !self.accepts_markdown && self.child_predicate == ChildPredicate::None
    }
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &["step"],
        accepts_markdown: false,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::StepsXorList,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::FigureOneImage,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &["tab"],
        accepts_markdown: false,
        requires: &["tab"],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &["link-card"],
        accepts_markdown: false,
        requires: &["link-card"],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &["tabs"],
        child_predicate: ChildPredicate::None,
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
        accepts: &[],
        accepts_markdown: true,
        requires: &[],
        forbids: &[],
        child_predicate: ChildPredicate::None,
        required_one_of: &[],
    }
}

pub fn lookup(name: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|kind| kind.name == name)
}

pub fn authorable_kinds() -> impl Iterator<Item = &'static KindSpec> {
    KINDS.iter().filter(|kind| kind.authorable)
}

pub fn field_enum_values(kind: &str, field: &str) -> Option<&'static [&'static str]> {
    match (kind, field) {
        ("tabs", "kind") => Some(TAB_KIND_VALUES),
        ("badge", "tone") => Some(BADGE_TONE_VALUES),
        _ => None,
    }
}

pub fn is_bool_field(kind: &str, field: &str) -> bool {
    matches!(field, "open" | "verify" | "decorative" | "allow_network")
        || lookup(kind).is_some_and(|spec| {
            spec.paint_fields()
                .iter()
                .any(|paint| paint.attr == field && paint.ty == PaintType::Bool)
        })
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
    spec.parents.is_empty() || parent_kind.is_some_and(|parent| spec.parents.contains(&parent))
}

pub fn child_completion_allowed(spec: &KindSpec, parent_kind: Option<&str>) -> bool {
    if !parent_allowed(spec, parent_kind) {
        return false;
    }
    match parent_kind.and_then(lookup) {
        Some(parent) => parent.accepts_block_child(spec.name),
        None => true,
    }
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
            | "use"
            | "if"
            | "for"
            | "match"
            | "let"
    )
}

#[cfg(test)]
fn widget_kinds() -> impl Iterator<Item = &'static KindSpec> {
    KINDS.iter().filter(|kind| kind.paints_as_widget())
}

#[cfg(test)]
fn article_node_type_roc() -> String {
    let mut out = String::from("ArticleNode : [\n    HtmlFile({ path : Str }),\n");
    for spec in widget_kinds() {
        out.push_str("    ");
        out.push_str(spec.component);
        out.push_str("({ ");
        let mut parts = Vec::new();
        for field in spec.paint_fields() {
            let ty = match field.ty {
                PaintType::Str => "Str",
                PaintType::Bool => "Bool",
            };
            parts.push(format!("{} : {ty}", field.prop));
        }
        if spec.paint_content() {
            parts.push("child_count : U64".into());
        }
        out.push_str(&parts.join(", "));
        out.push_str(" }),\n");
    }
    out.push_str("]\n\n");
    out
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
        for spec in widget_kinds() {
            assert!(
                lookup(spec.name).is_some(),
                "widget kind `{}` missing from registry",
                spec.name
            );
            let needle = format!("@component {}", spec.component);
            assert!(
                src.contains(&needle),
                "DocsComponents.rocci missing painter `{needle}` for kind `{}`",
                spec.name
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
            !src.contains("@component Render"),
            "Render dispatcher should be gone once typed props land"
        );
        assert!(
            !src.contains("@match segment.kind"),
            "kind matcher should not remain as the paint contract"
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
    fn child_policy_is_data_on_the_spec() {
        let tabs = lookup("tabs").unwrap();
        assert_eq!(tabs.accepts, &["tab"]);
        assert!(!tabs.accepts_markdown);
        assert_eq!(tabs.requires, &["tab"]);
        assert!(tabs.rejects_markdown());
        assert!(tabs.accepts_block_child("tab"));
        assert!(!tabs.accepts_block_child("note"));

        let grid = lookup("card-grid").unwrap();
        assert_eq!(grid.accepts, &["link-card"]);
        assert_eq!(grid.requires, &["link-card"]);
        assert!(!grid.accepts_markdown);
        assert!(grid.accepts_block_child("link-card"));
        assert!(!grid.accepts_block_child("note"));

        let note = lookup("note").unwrap();
        assert_eq!(note.forbids, &["tabs"]);
        assert!(note.accepts_markdown);
        assert!(!note.accepts_block_child("tabs"));
        assert!(note.accepts_block_child("details"));

        let steps = lookup("steps").unwrap();
        assert_eq!(steps.child_predicate, ChildPredicate::StepsXorList);
        assert_eq!(steps.accepts, &["step"]);
        assert!(!steps.rejects_markdown());

        let figure = lookup("figure").unwrap();
        assert_eq!(figure.child_predicate, ChildPredicate::FigureOneImage);
        assert!(figure.accepts_markdown);

        let tab = lookup("tab").unwrap();
        assert!(child_completion_allowed(tab, Some("tabs")));
        assert!(!child_completion_allowed(tab, None));
        let note_spec = lookup("note").unwrap();
        assert!(!child_completion_allowed(note_spec, Some("tabs")));
        assert!(child_completion_allowed(note_spec, None));
        let tabs_spec = lookup("tabs").unwrap();
        assert!(!child_completion_allowed(tabs_spec, Some("note")));
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

    #[test]
    fn widget_paint_fields_are_named() {
        for spec in widget_kinds() {
            for field in spec.paint_fields() {
                assert!(!field.prop.is_empty(), "{}", spec.name);
                assert!(!field.attr.is_empty(), "{}", spec.name);
            }
        }
        let note = lookup("note").unwrap();
        assert!(note.paints_as_widget());
        assert!(note.paint_content());
        assert_eq!(note.paint_fields()[0].prop, "title");
        let badge = lookup("badge").unwrap();
        assert!(!badge.paint_content());
        let heading = lookup("h2").unwrap();
        assert!(!heading.paints_as_widget());
    }

    #[test]
    fn article_node_type_lists_widget_tags() {
        let src = article_node_type_roc();
        assert!(src.contains("HtmlFile({ path : Str })"), "{src}");
        assert!(src.contains("ArticleNode : ["), "{src}");
        assert!(src.contains("child_count : U64"), "{src}");
        for spec in widget_kinds() {
            assert!(
                src.contains(&format!("{}({{ ", spec.component)),
                "type missing `{}`: {src}",
                spec.component
            );
        }
    }

    #[test]
    fn rocdown_build_calls_each_widget_component() {
        let src = include_str!("../runtime/RocdownBuild.roc");
        assert!(
            !src.contains("DocsComponents.render"),
            "apply host should call per-kind components"
        );
        assert!(src.contains("child_count"), "{src}");
        assert!(src.contains("HtmlFile"), "{src}");
        for spec in widget_kinds() {
            let mut chars = spec.component.chars();
            let first = chars.next().unwrap();
            let roc_name: String = first.to_lowercase().chain(chars).collect();
            let call = format!("DocsComponents.{roc_name}");
            assert!(
                src.contains(&call),
                "RocdownBuild.roc missing `{call}` for kind `{}`",
                spec.name
            );
            assert!(
                src.contains(spec.component),
                "RocdownBuild.roc missing tag `{}`",
                spec.component
            );
        }
    }
}
