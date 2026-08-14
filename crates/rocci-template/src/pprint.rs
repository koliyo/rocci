use crate::ast::{
    Attr, AttrValue, ComponentCall, ComponentDecl, Document, Element, FixtureDecl, ForDirective,
    Fragment, IfDirective, MatchDirective, ModuleItem, TemplateBlock, TemplateItem,
};
use crate::span::Span;

pub fn format_ast(src: &str, document: &Document) -> String {
    let mut out = String::new();
    let mut w = Writer::new(&mut out);
    w.open("module", &[]);
    for item in &document.items {
        match item {
            ModuleItem::Roc { span } => write_roc(&mut w, span.of(src)),
            ModuleItem::Component(component) => write_component(&mut w, src, component),
            ModuleItem::Fixture(fixture) => write_fixture(&mut w, src, fixture),
        }
    }
    w.close();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
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

    fn atom_line(&mut self, atom: &str) {
        self.break_line();
        self.out.push_str(atom);
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

fn write_roc(w: &mut Writer<'_>, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    let lines: Vec<&str> = trimmed
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.len() == 1 {
        w.leaf("roc", &[string_atom(lines[0])]);
        return;
    }
    w.open("roc", &[]);
    for line in lines {
        w.atom_line(&string_atom(line));
    }
    w.close();
}

fn write_component(w: &mut Writer<'_>, src: &str, component: &ComponentDecl) {
    w.open("component", &[atom(&component.name.name)]);
    w.leaf("params", &[atom(component.params.of(src).trim())]);
    write_block(w, src, &component.body);
    w.close();
}

fn write_fixture(w: &mut Writer<'_>, src: &str, fixture: &FixtureDecl) {
    w.open(
        "fixture",
        &[
            atom(&fixture.name.name),
            format!("target:{}", fixture.target.roc_name),
        ],
    );
    write_roc(w, fixture.value.of(src));
    w.close();
}

fn write_block(w: &mut Writer<'_>, src: &str, block: &TemplateBlock) {
    for item in &block.items {
        write_item(w, src, item);
    }
}

fn write_item(w: &mut Writer<'_>, src: &str, item: &TemplateItem) {
    match item {
        TemplateItem::Element(el) => write_element(w, src, el),
        TemplateItem::ComponentCall(call) => write_call(w, src, call),
        TemplateItem::Fragment(frag) => write_fragment(w, src, frag),
        TemplateItem::Text(text) => w.leaf("text", &[string_atom(&text.value)]),
        TemplateItem::Interpolation(interp) => w.leaf("interp", &[roc_atom(src, interp.expr)]),
        TemplateItem::If(dir) => write_if(w, src, dir),
        TemplateItem::For(dir) => write_for(w, src, dir),
        TemplateItem::Match(dir) => write_match(w, src, dir),
        TemplateItem::Let(dir) => w.leaf("let", &[atom(&dir.binder.name), roc_atom(src, dir.expr)]),
    }
}

fn write_element(w: &mut Writer<'_>, src: &str, el: &Element) {
    let mut head = vec![atom(&el.name.name)];
    if el.self_closing {
        head.push("self-closing".to_string());
    }
    if el.attrs.is_empty() && el.children.is_empty() {
        w.leaf("element", &head);
        return;
    }
    w.open("element", &head);
    write_attrs(w, src, &el.attrs);
    for child in &el.children {
        write_item(w, src, child);
    }
    w.close();
}

fn write_call(w: &mut Writer<'_>, src: &str, call: &ComponentCall) {
    let mut head = vec![atom(&path_source(call))];
    let children = call.children.as_deref().unwrap_or(&[]);
    if call.children.is_none() {
        head.push("self-closing".to_string());
    }
    if call.attrs.is_empty() && children.is_empty() {
        w.leaf("call", &head);
        return;
    }
    w.open("call", &head);
    write_attrs(w, src, &call.attrs);
    for child in children {
        write_item(w, src, child);
    }
    w.close();
}

fn write_fragment(w: &mut Writer<'_>, src: &str, frag: &Fragment) {
    if frag.children.is_empty() {
        w.leaf("fragment", &[]);
        return;
    }
    w.open("fragment", &[]);
    for child in &frag.children {
        write_item(w, src, child);
    }
    w.close();
}

fn write_attrs(w: &mut Writer<'_>, src: &str, attrs: &[Attr]) {
    for attr in attrs {
        match &attr.value {
            AttrValue::Boolean => w.leaf("attr", &[atom(&attr.name.name)]),
            AttrValue::Static { value, .. } => {
                w.leaf("attr", &[atom(&attr.name.name), string_atom(value)]);
            }
            AttrValue::Expr { expr } => {
                w.leaf("attr", &[atom(&attr.name.name), roc_atom(src, *expr)]);
            }
        }
    }
}

fn write_if(w: &mut Writer<'_>, src: &str, dir: &IfDirective) {
    w.open("if", &[roc_atom(src, dir.condition)]);
    write_block(w, src, &dir.then_body);
    for (cond, body) in &dir.else_ifs {
        w.open("else-if", &[roc_atom(src, *cond)]);
        write_block(w, src, body);
        w.close();
    }
    if let Some(body) = &dir.else_body {
        w.open("else", &[]);
        write_block(w, src, body);
        w.close();
    }
    w.close();
}

fn write_for(w: &mut Writer<'_>, src: &str, dir: &ForDirective) {
    w.open(
        "for",
        &[atom(&dir.binder.name), roc_atom(src, dir.collection)],
    );
    write_block(w, src, &dir.body);
    w.close();
}

fn write_match(w: &mut Writer<'_>, src: &str, dir: &MatchDirective) {
    w.open("match", &[roc_atom(src, dir.scrutinee)]);
    for arm in &dir.arms {
        w.open("arm", &[roc_atom(src, arm.pattern)]);
        write_item(w, src, &arm.value);
        w.close();
    }
    w.close();
}

fn path_source(call: &ComponentCall) -> String {
    call.path
        .parts
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn roc_atom(src: &str, span: Span) -> String {
    atom(span.of(src).trim())
}

fn atom(text: &str) -> String {
    if text.is_empty() {
        return "\"\"".to_string();
    }
    if is_bare_atom(text) {
        text.to_string()
    } else {
        string_atom(text)
    }
}

fn is_bare_atom(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '-'))
}

fn string_atom(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}
