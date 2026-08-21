use rocci_template::{
    CommandDecl, ComponentDecl, ContextDecl, CssDecl, FixtureDecl, InitDecl, LiveDecl, PatchDecl,
    TemplateItem, ViewDecl,
};

use crate::ast::{
    BlockCall, Document, Item, MdNode, PageDecl, ParamValue, RenderDecl, RocDecl, UseDecl,
};
use crate::parse::nested_items;

// Inspect heads live in Rocdown.AST.toml [inspect]. This file owns Writer, 40-character
// truncation, nested parse_fragment, and leftover MdNode fallback.

pub fn format_ast(src: &str, document: &Document) -> String {
    let mut out = String::new();
    let mut w = Writer::new(&mut out);
    write_document(&mut w, src, document);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn write_page(w: &mut Writer<'_>, src: &str, page: &PageDecl) {
    w.leaf("page", &[atom(page.body.of(src).trim())]);
}

fn write_roc_decl(w: &mut Writer<'_>, src: &str, roc: &RocDecl) {
    w.leaf("roc", &[atom(roc.body.of(src).trim())]);
}

fn write_render(w: &mut Writer<'_>, _src: &str, render: &RenderDecl) {
    let name = render
        .path
        .parts
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".");
    if name.is_empty() {
        w.leaf("render", &[]);
    } else {
        w.leaf("render", std::slice::from_ref(&name));
    }
}

fn write_component_leaf(w: &mut Writer<'_>, _src: &str, component: &ComponentDecl) {
    w.leaf("component", std::slice::from_ref(&component.name.name));
}

fn write_fixture_leaf(w: &mut Writer<'_>, _src: &str, fixture: &FixtureDecl) {
    w.leaf("fixture", std::slice::from_ref(&fixture.name.name));
}

fn write_css_leaf(w: &mut Writer<'_>, _src: &str, _css: &CssDecl) {
    w.leaf("css", &[]);
}

fn write_context_leaf(w: &mut Writer<'_>, _src: &str, _context: &ContextDecl) {
    w.leaf("context", &[]);
}

fn write_init_leaf(w: &mut Writer<'_>, _src: &str, _init: &InitDecl) {
    w.leaf("init", &[]);
}

fn write_live_leaf(w: &mut Writer<'_>, _src: &str, _live: &LiveDecl) {
    w.leaf("live", &[]);
}

fn write_view_leaf(w: &mut Writer<'_>, _src: &str, view: &ViewDecl) {
    w.leaf("view", &[format!("GET:{}", view.path)]);
}

fn write_patch_leaf(w: &mut Writer<'_>, _src: &str, patch: &PatchDecl) {
    let method = patch
        .method
        .as_ref()
        .map(|ident| ident.name.as_str())
        .unwrap_or("post");
    w.leaf("patch", &[format!("{}:{}", method, patch.path)]);
}

fn write_command_leaf(w: &mut Writer<'_>, _src: &str, command: &CommandDecl) {
    let method = command
        .method
        .as_ref()
        .map(|ident| ident.name.as_str())
        .unwrap_or("post");
    w.leaf("command", &[format!("{}:{}", method, command.path)]);
}

fn write_use(w: &mut Writer<'_>, _src: &str, used: &UseDecl) {
    w.leaf("use", std::slice::from_ref(&used.path));
}

fn write_template_island(w: &mut Writer<'_>, _src: &str, item: &TemplateItem) {
    match item {
        TemplateItem::If(_) => w.leaf("if", &[]),
        TemplateItem::For(dir) => w.leaf("for", std::slice::from_ref(&dir.binder.name)),
        TemplateItem::Match(_) => w.leaf("match", &[]),
        TemplateItem::Let(dir) => w.leaf("let", std::slice::from_ref(&dir.binder.name)),
        TemplateItem::ComponentCall(call) => {
            w.leaf("call", std::slice::from_ref(&call.path.roc_name))
        }
        TemplateItem::Element(el) => w.leaf("element", std::slice::from_ref(&el.name.name)),
        TemplateItem::Fragment(_) => w.leaf("fragment", &[]),
        _ => w.leaf("template", &[]),
    }
}

fn write_block(w: &mut Writer<'_>, src: &str, call: &BlockCall) {
    let mut atoms = vec![call.name.clone()];
    if let Some(content) = &call.content {
        atoms.push(content.scope_name().to_string());
    }
    if let Some(params) = &call.params {
        for field in &params.fields {
            atoms.push(field.name.clone());
            atoms.push(atom(&param_display(src, &field.value)));
        }
    }
    let children = nested_items(src, call);
    if children.is_empty() {
        w.leaf("block", &atoms);
        return;
    }
    w.open("block", &atoms);
    for child in &children {
        write_item(w, src, child);
    }
    w.close();
}

fn param_display(src: &str, value: &ParamValue) -> String {
    match value {
        ParamValue::StringLit { value, .. } => value.clone(),
        ParamValue::BoolLit { value, .. } => {
            if *value {
                "True".into()
            } else {
                "False".into()
            }
        }
        ParamValue::NumberLit { value, .. } => value.clone(),
        ParamValue::Ident { name, .. } => name.clone(),
        ParamValue::Record(record) => record.span.of(src).to_string(),
        ParamValue::List(list) => list.span.of(src).to_string(),
    }
}

fn write_md(w: &mut Writer<'_>, src: &str, node: &MdNode) {
    match node {
        MdNode::Heading {
            level,
            id,
            children,
            ..
        } => {
            w.open("h", &[level.to_string(), id.clone()]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::Paragraph { children, .. } => {
            w.open("p", &[]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::Text { value, .. } => w.leaf("text", &[atom(value)]),
        MdNode::Code { value, .. } => w.leaf("code", &[atom(value)]),
        MdNode::Link { url, children, .. } => {
            w.open("a", &[atom(url)]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::List {
            ordered, children, ..
        } => {
            w.open(if *ordered { "ol" } else { "ul" }, &[]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::Item { children, .. } => {
            w.open("li", &[]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::Emph { children, .. } => {
            w.open("em", &[]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::Strong { children, .. } => {
            w.open("strong", &[]);
            for child in children {
                write_md(w, src, child);
            }
            w.close();
        }
        MdNode::CodeBlock { info, literal, .. } => {
            w.leaf("fence", &[info.clone(), atom(literal)]);
        }
        _ => w.leaf("md", &[]),
    }
}

fn atom(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.len() > 40 {
        format!("{}…", compact.chars().take(40).collect::<String>())
    } else {
        compact
    }
}

struct Writer<'a> {
    out: &'a mut String,
    indent: usize,
}

impl<'a> Writer<'a> {
    fn new(out: &'a mut String) -> Self {
        Self { out, indent: 0 }
    }

    fn open(&mut self, head: &str, atoms: &[String]) {
        self.break_line();
        self.out.push('(');
        self.out.push_str(head);
        for atom in atoms {
            self.out.push(' ');
            self.out.push_str(atom);
        }
        self.indent += 1;
    }

    fn leaf(&mut self, head: &str, atoms: &[String]) {
        self.break_line();
        self.out.push('(');
        self.out.push_str(head);
        for atom in atoms {
            self.out.push(' ');
            self.out.push_str(atom);
        }
        self.out.push(')');
    }

    fn close(&mut self) {
        self.indent -= 1;
        self.out.push(')');
    }

    fn break_line(&mut self) {
        if !self.out.is_empty() {
            self.out.push('\n');
            for _ in 0..self.indent {
                self.out.push_str("  ");
            }
        }
    }
}

include!("pprint.generated.rs");
