use rocci_template::{
    CommandDecl, ComponentDecl, ContextDecl, CssDecl, Document, FixtureDecl, FragmentDecl,
    InitDecl, LeadingComments, LiveDecl, ModuleItem, RouteDecl, SourceFile, ViewDecl, parse,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclDoc {
    pub heading: String,
    pub body: String,
    pub line: u32,
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
    let line = declaration_line(src, item);
    match item {
        ModuleItem::Roc { .. } => None,
        ModuleItem::Component(item) => named_doc(src, &item.leading, component_heading(item), line),
        ModuleItem::Fixture(item) => named_doc(src, &item.leading, fixture_heading(item), line),
        ModuleItem::Css(item) => named_doc(src, &item.leading, css_heading(item), line),
        ModuleItem::Context(item) => named_doc(src, &item.leading, context_heading(item), line),
        ModuleItem::Init(item) => named_doc(src, &item.leading, init_heading(item), line),
        ModuleItem::Route(route) => match route {
            RouteDecl::Live(item) => named_doc(src, &item.leading, live_heading(item), line),
            RouteDecl::View(item) => named_doc(src, &item.leading, view_heading(item), line),
            RouteDecl::Fragment(item) => {
                named_doc(src, &item.leading, fragment_heading(item), line)
            }
            RouteDecl::Command(item) => named_doc(src, &item.leading, command_heading(item), line),
        },
    }
}

fn named_doc(
    src: &str,
    leading: &Option<LeadingComments>,
    heading: String,
    line: u32,
) -> Option<DeclDoc> {
    let body = docs_body(src, leading)?;
    Some(DeclDoc {
        heading,
        body,
        line,
    })
}

fn declaration_line(src: &str, item: &ModuleItem) -> u32 {
    line_number(src, declaration_offset(src, item))
}

fn declaration_offset(src: &str, item: &ModuleItem) -> usize {
    match item {
        ModuleItem::Roc { span } => span.start as usize,
        ModuleItem::Component(item) => leading_or_decl_start(src, &item.leading, item.span.start),
        ModuleItem::Fixture(item) => leading_or_decl_start(src, &item.leading, item.span.start),
        ModuleItem::Css(item) => leading_or_decl_start(src, &item.leading, item.span.start),
        ModuleItem::Context(item) => leading_or_decl_start(src, &item.leading, item.span.start),
        ModuleItem::Init(item) => leading_or_decl_start(src, &item.leading, item.span.start),
        ModuleItem::Route(route) => match route {
            RouteDecl::Live(item) => leading_or_decl_start(src, &item.leading, item.span.start),
            RouteDecl::View(item) => leading_or_decl_start(src, &item.leading, item.span.start),
            RouteDecl::Fragment(item) => leading_or_decl_start(src, &item.leading, item.span.start),
            RouteDecl::Command(item) => leading_or_decl_start(src, &item.leading, item.span.start),
        },
    }
}

fn leading_or_decl_start(src: &str, leading: &Option<LeadingComments>, decl_start: u32) -> usize {
    if let Some(leading) = leading {
        let end = leading.span.end as usize;
        if end < src.len() && src.as_bytes()[end] == b'\n' {
            return end + 1;
        }
        return end;
    }
    decl_start as usize
}

fn line_number(src: &str, offset: usize) -> u32 {
    src.get(..offset.min(src.len()))
        .unwrap_or("")
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1
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

fn live_heading(item: &LiveDecl) -> String {
    handler_heading(&item.method.name, "live", &item.path)
}

fn view_heading(item: &ViewDecl) -> String {
    handler_heading(&item.method.name, "view", &item.path)
}

fn fragment_heading(item: &FragmentDecl) -> String {
    handler_heading(&item.method.name, "fragment", &item.path)
}

fn command_heading(item: &CommandDecl) -> String {
    handler_heading(&item.method.name, "command", &item.path)
}

fn handler_heading(method: &str, role: &str, path: &str) -> String {
    format!("@{method}:{role}(\"{path}\")")
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
        md.push_str(&format!("` · [#L{line}](#L{line})\n\n", line = decl.line));
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
