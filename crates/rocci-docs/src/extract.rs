use rocci_template::{
    CommandDecl, ComponentDecl, ContextDecl, CssDecl, Document, FixtureDecl, Ident, InitDecl,
    LeadingComments, LiveDecl, ModuleItem, PatchDecl, SourceFile, ViewDecl, parse,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclDoc {
    pub heading: String,
    pub body: String,
}

pub fn documented_declarations(src: &str) -> Vec<DeclDoc> {
    let parsed = parse(SourceFile::new("module.rocci", src));
    collect_docs(src, &parsed.document)
}

fn collect_docs(src: &str, document: &Document) -> Vec<DeclDoc> {
    let mut out = Vec::new();
    for item in &document.items {
        if let Some(decl) = item_doc(src, item) {
            out.push(decl);
        }
    }
    out
}

fn item_doc(src: &str, item: &ModuleItem) -> Option<DeclDoc> {
    match item {
        ModuleItem::Roc { .. } => None,
        ModuleItem::Component(item) => named_doc(src, &item.leading, component_heading(item)),
        ModuleItem::Fixture(item) => named_doc(src, &item.leading, fixture_heading(item)),
        ModuleItem::Css(item) => named_doc(src, &item.leading, css_heading(item)),
        ModuleItem::Context(item) => named_doc(src, &item.leading, context_heading(item)),
        ModuleItem::Init(item) => named_doc(src, &item.leading, init_heading(item)),
        ModuleItem::Live(item) => named_doc(src, &item.leading, live_heading(item)),
        ModuleItem::View(item) => named_doc(src, &item.leading, view_heading(item)),
        ModuleItem::Patch(item) => named_doc(src, &item.leading, patch_heading(item)),
        ModuleItem::Command(item) => named_doc(src, &item.leading, command_heading(item)),
    }
}

fn named_doc(src: &str, leading: &Option<LeadingComments>, heading: String) -> Option<DeclDoc> {
    let body = docs_body(src, leading)?;
    Some(DeclDoc { heading, body })
}

fn docs_body(src: &str, leading: &Option<LeadingComments>) -> Option<String> {
    let leading = leading.as_ref()?;
    if leading.docs.is_empty() {
        return None;
    }
    Some(
        leading
            .docs
            .iter()
            .map(|span| strip_doc_line(span.of(src)))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn strip_doc_line(line: &str) -> &str {
    let line = line
        .trim_end()
        .strip_suffix('\r')
        .unwrap_or(line.trim_end());
    if line == "##" {
        ""
    } else if let Some(rest) = line.strip_prefix("## ") {
        rest
    } else {
        line
    }
}

fn component_heading(item: &ComponentDecl) -> String {
    format!("@component {}", item.name.name)
}

fn fixture_heading(item: &FixtureDecl) -> String {
    format!("@fixture {}", item.name.name)
}

fn css_heading(_item: &CssDecl) -> String {
    "@css".to_string()
}

fn context_heading(_item: &ContextDecl) -> String {
    "@context".to_string()
}

fn init_heading(_item: &InitDecl) -> String {
    "@init".to_string()
}

fn live_heading(_item: &LiveDecl) -> String {
    "@live".to_string()
}

fn view_heading(item: &ViewDecl) -> String {
    format!("@view(\"{}\")", item.path)
}

fn patch_heading(item: &PatchDecl) -> String {
    handler_heading("patch", item.method.as_ref(), &item.path)
}

fn command_heading(item: &CommandDecl) -> String {
    handler_heading("command", item.method.as_ref(), &item.path)
}

fn handler_heading(kind: &str, method: Option<&Ident>, path: &str) -> String {
    match method {
        Some(method) => format!("@{}:{}(\"{}\")", kind, method.name, path),
        None => format!("@{}(\"{}\")", kind, path),
    }
}

pub fn declarations_markdown(src: &str) -> String {
    let decls = documented_declarations(src);
    if decls.is_empty() {
        return String::new();
    }
    let mut md = String::from("## Declarations\n\n");
    for decl in decls {
        md.push_str("### `");
        md.push_str(&decl.heading);
        md.push_str("`\n\n");
        md.push_str(&escape_rocdown_prose(&decl.body));
        md.push_str("\n\n");
    }
    md
}

fn escape_rocdown_prose(body: &str) -> String {
    body.lines()
        .map(|line| {
            if line.starts_with('@') {
                format!("@{line}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
