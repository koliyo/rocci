use crate::ast::{
    Attr, AttrValue, CommandDecl, ComponentCall, ComponentDecl, ContextDecl, CssDecl, Document,
    Element, FixtureDecl, ForDirective, Fragment, FragmentDecl, IfDirective, InitDecl,
    Interpolation, LeadingComments, LetDirective, LiveDecl, MatchDirective, ModuleItem, RouteDecl,
    TemplateBlock, TemplateItem, TextNode, ViewDecl,
};
use crate::span::Span;

// Inspect heads live in Rocci.AST.toml [inspect]. This file owns Writer and atom policy.

pub fn format_ast(src: &str, document: &Document) -> String {
    let mut out = String::new();
    let mut w = Writer::new(&mut out);
    write_document(&mut w, src, document);
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

fn write_roc_region(w: &mut Writer<'_>, src: &str, span: &Span) {
    write_roc(w, span.of(src));
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

fn write_leading(w: &mut Writer<'_>, src: &str, leading: &LeadingComments) {
    w.open("leading", &[]);
    for span in &leading.comments {
        w.leaf("comment", &[string_atom(span.of(src).trim_end())]);
    }
    for span in &leading.docs {
        w.leaf("docs", &[string_atom(span.of(src).trim_end())]);
    }
    w.close();
}

fn write_optional_leading(w: &mut Writer<'_>, src: &str, leading: &Option<LeadingComments>) {
    if let Some(leading) = leading {
        write_leading(w, src, leading);
    }
}

fn write_component(w: &mut Writer<'_>, src: &str, component: &ComponentDecl) {
    w.open("component", &[atom(&component.name.name)]);
    write_optional_leading(w, src, &component.leading);
    w.leaf("params", &[atom(component.params.of(src).trim())]);
    write_template_block(w, src, &component.body);
    w.close();
}

fn write_fixture(w: &mut Writer<'_>, src: &str, fixture: &FixtureDecl) {
    w.open(
        "fixture",
        &[
            atom(&fixture.name.name),
            format!("target:{}", fixture.target.source_name()),
        ],
    );
    write_optional_leading(w, src, &fixture.leading);
    write_roc(w, fixture.value.of(src));
    w.close();
}

fn write_text(w: &mut Writer<'_>, _src: &str, text: &TextNode) {
    w.leaf("text", &[string_atom(&text.value)]);
}

fn write_interp(w: &mut Writer<'_>, src: &str, interp: &Interpolation) {
    w.leaf("interp", &[roc_atom(src, interp.expr)]);
}

fn write_let(w: &mut Writer<'_>, src: &str, dir: &LetDirective) {
    w.leaf("let", &[atom(&dir.binder.name), roc_atom(src, dir.expr)]);
}

fn write_css(w: &mut Writer<'_>, src: &str, css: &CssDecl) {
    if css.leading.is_some() {
        w.open("css", &[]);
        write_optional_leading(w, src, &css.leading);
        w.leaf("body", &[string_atom(css.body.of(src).trim())]);
        w.close();
    } else {
        w.leaf("css", &[string_atom(css.body.of(src).trim())]);
    }
}

fn write_context(w: &mut Writer<'_>, src: &str, context: &ContextDecl) {
    if context.leading.is_some() {
        w.open("context", &[]);
        write_optional_leading(w, src, &context.leading);
        w.leaf("ty", &[roc_atom(src, context.ty)]);
        w.close();
    } else {
        w.leaf("context", &[roc_atom(src, context.ty)]);
    }
}

fn write_init(w: &mut Writer<'_>, src: &str, init: &InitDecl) {
    w.open("init", &[]);
    write_optional_leading(w, src, &init.leading);
    write_roc(w, init.body.of(src));
    w.close();
}

fn write_live(w: &mut Writer<'_>, src: &str, live: &LiveDecl) {
    let mut atoms = vec![
        atom(&live.method.name.to_ascii_uppercase()),
        string_atom(&live.path),
    ];
    if let Some(params) = live.params {
        atoms.push(atom(params.of(src).trim()));
    }
    w.open("live", &atoms);
    write_optional_leading(w, src, &live.leading);
    write_roc(w, live.body.of(src));
    w.close();
}

fn write_view(w: &mut Writer<'_>, src: &str, view: &ViewDecl) {
    let mut atoms = vec![
        atom(&view.method.name.to_ascii_uppercase()),
        string_atom(&view.path),
    ];
    if let Some(params) = view.params {
        atoms.push(atom(params.of(src).trim()));
    }
    w.open("view", &atoms);
    write_optional_leading(w, src, &view.leading);
    write_roc(w, view.body.of(src));
    w.close();
}

fn write_fragment_decl(w: &mut Writer<'_>, src: &str, fragment: &FragmentDecl) {
    write_mutation(
        w,
        src,
        Mutation {
            head: "fragment",
            method: &fragment.method,
            path: &fragment.path,
            params: fragment.params,
            body: fragment.body,
            leading: &fragment.leading,
        },
    );
}

fn write_command(w: &mut Writer<'_>, src: &str, command: &CommandDecl) {
    write_mutation(
        w,
        src,
        Mutation {
            head: "command",
            method: &command.method,
            path: &command.path,
            params: command.params,
            body: command.body,
            leading: &command.leading,
        },
    );
}

struct Mutation<'a> {
    head: &'a str,
    method: &'a crate::ast::Ident,
    path: &'a str,
    params: Option<Span>,
    body: Span,
    leading: &'a Option<LeadingComments>,
}

fn write_mutation(w: &mut Writer<'_>, src: &str, mutation: Mutation<'_>) {
    let mut atoms = vec![atom(&mutation.method.name.to_ascii_uppercase())];
    atoms.push(string_atom(mutation.path));
    if let Some(params) = mutation.params {
        atoms.push(atom(params.of(src).trim()));
    }
    w.open(mutation.head, &atoms);
    write_optional_leading(w, src, mutation.leading);
    write_roc(w, mutation.body.of(src));
    w.close();
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
        write_template_item(w, src, child);
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
        write_template_item(w, src, child);
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
        write_template_item(w, src, child);
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
            AttrValue::Action { name, args } => {
                w.leaf(
                    "attr",
                    &[
                        atom(&attr.name.name),
                        atom(&format!("@{}", name.name)),
                        roc_atom(src, *args),
                    ],
                );
            }
        }
    }
}

fn write_if(w: &mut Writer<'_>, src: &str, dir: &IfDirective) {
    w.open("if", &[roc_atom(src, dir.condition)]);
    write_template_block(w, src, &dir.then_body);
    for (cond, body) in &dir.else_ifs {
        w.open("else-if", &[roc_atom(src, *cond)]);
        write_template_block(w, src, body);
        w.close();
    }
    if let Some(body) = &dir.else_body {
        w.open("else", &[]);
        write_template_block(w, src, body);
        w.close();
    }
    w.close();
}

fn write_for(w: &mut Writer<'_>, src: &str, dir: &ForDirective) {
    w.open(
        "for",
        &[atom(&dir.binder.name), roc_atom(src, dir.collection)],
    );
    write_template_block(w, src, &dir.body);
    w.close();
}

fn write_match(w: &mut Writer<'_>, src: &str, dir: &MatchDirective) {
    w.open("match", &[roc_atom(src, dir.scrutinee)]);
    for arm in &dir.arms {
        w.open("arm", &[roc_atom(src, arm.pattern)]);
        write_template_item(w, src, &arm.value);
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandlerInspect {
    pub kind: &'static str,
    pub method: String,
    pub path: String,
    pub role: &'static str,
}

impl HandlerInspect {
    pub fn line(&self) -> String {
        format!(
            "{} {} \"{}\" {}",
            self.kind, self.method, self.path, self.role
        )
    }
}

pub fn inspect_handlers(document: &Document) -> Vec<HandlerInspect> {
    document
        .items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Route(route) => Some(match route {
                RouteDecl::View(view) => HandlerInspect {
                    kind: "view",
                    method: mutation_method(&view.method),
                    path: view.path.clone(),
                    role: "document",
                },
                RouteDecl::Fragment(fragment) => HandlerInspect {
                    kind: "fragment",
                    method: mutation_method(&fragment.method),
                    path: fragment.path.clone(),
                    role: "fragment",
                },
                RouteDecl::Command(command) => HandlerInspect {
                    kind: "command",
                    method: mutation_method(&command.method),
                    path: command.path.clone(),
                    role: "command",
                },
                RouteDecl::Live(live) => HandlerInspect {
                    kind: "live",
                    method: mutation_method(&live.method),
                    path: live.path.clone(),
                    role: "live",
                },
            }),
            _ => None,
        })
        .collect()
}

fn mutation_method(method: &crate::ast::Ident) -> String {
    method.name.to_ascii_uppercase()
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

include!("pprint.generated.rs");
