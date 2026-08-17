use rocci_template::TemplateItem;

use crate::ast::{Document, Item, MdNode};

pub fn format_ast(src: &str, document: &Document) -> String {
    let mut out = String::new();
    let mut w = Writer::new(&mut out);
    w.open("rocdown", &[]);
    for item in &document.items {
        match item {
            Item::Markdown(node) => write_md(&mut w, node),
            Item::Page(page) => w.leaf("page", &[atom(page.body.of(src).trim())]),
            Item::Roc(roc) => w.leaf("roc", &[atom(roc.body.of(src).trim())]),
            Item::Render(render) => w.leaf("render", &[atom(render.expr.of(src).trim())]),
            Item::Component(component) => {
                w.leaf("component", std::slice::from_ref(&component.name.name));
            }
            Item::Fixture(fixture) => w.leaf("fixture", std::slice::from_ref(&fixture.name.name)),
            Item::Css(_) => w.leaf("css", &[]),
            Item::Context(_) => w.leaf("context", &[]),
            Item::Init(_) => w.leaf("init", &[]),
            Item::On(on) => w.leaf("on", &[format!("{}:{}", on.method.name, on.path)]),
            Item::Docs(docs) => w.leaf("docs", std::slice::from_ref(&docs.kind)),
            Item::Img(img) => w.leaf("img", &[atom(img.body.of(src).trim())]),
            Item::Template(item) => match item {
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
            },
        }
    }
    w.close();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn write_md(w: &mut Writer<'_>, node: &MdNode) {
    match node {
        MdNode::Heading {
            level,
            id,
            children,
            ..
        } => {
            w.open("h", &[level.to_string(), id.clone()]);
            for child in children {
                write_md(w, child);
            }
            w.close();
        }
        MdNode::Paragraph { children, .. } => {
            w.open("p", &[]);
            for child in children {
                write_md(w, child);
            }
            w.close();
        }
        MdNode::Text { value, .. } => w.leaf("text", &[atom(value)]),
        MdNode::Code { value, .. } => w.leaf("code", &[atom(value)]),
        MdNode::CodeBlock { info, literal, .. } => {
            w.leaf("fence", &[info.clone(), atom(literal)]);
        }
        MdNode::Link { url, children, .. } => {
            w.open("a", &[atom(url)]);
            for child in children {
                write_md(w, child);
            }
            w.close();
        }
        MdNode::List {
            ordered, children, ..
        } => {
            w.open(if *ordered { "ol" } else { "ul" }, &[]);
            for child in children {
                write_md(w, child);
            }
            w.close();
        }
        MdNode::Item { children, .. } => {
            w.open("li", &[]);
            for child in children {
                write_md(w, child);
            }
            w.close();
        }
        MdNode::Emph { children, .. } => {
            w.open("em", &[]);
            for child in children {
                write_md(w, child);
            }
            w.close();
        }
        MdNode::Strong { children, .. } => {
            w.open("strong", &[]);
            for child in children {
                write_md(w, child);
            }
            w.close();
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
