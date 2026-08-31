use rocci_template::{
    LoweredTemplate, OriginKind, SourceFile, Span, TemplateItem, TemplateValueCtx,
    lower_template_items, pascal_to_camel,
};

use crate::ast::{Document, Item, RenderDecl};
use crate::page::{import_local_name, roc_name_appears, roc_rest_name};

use super::emitter::Emitter;

pub(crate) fn filter_snapshot_roc(
    source: SourceFile<'_>,
    document: &Document,
    imports: Vec<Span>,
    rest: Vec<Span>,
) -> (Vec<Span>, Vec<Span>) {
    let mut kept_text = island_used_text(source.src, document);
    let names: Vec<Option<String>> = rest
        .iter()
        .map(|span| roc_rest_name(source.src, *span))
        .collect();
    let mut keep = vec![false; rest.len()];
    loop {
        let mut changed = false;
        for (i, span) in rest.iter().enumerate() {
            if keep[i] {
                continue;
            }
            let take = match names[i].as_deref() {
                None => true,
                Some(name) => roc_name_appears(name, &kept_text),
            };
            if take {
                keep[i] = true;
                kept_text.push('\n');
                kept_text.push_str(span.of(source.src));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let kept_rest: Vec<Span> = rest
        .iter()
        .zip(keep.iter())
        .filter_map(|(span, keep)| keep.then_some(*span))
        .collect();
    let kept_imports: Vec<Span> = imports
        .into_iter()
        .filter(|span| match import_local_name(source.src, *span) {
            None => true,
            Some(name) => roc_name_appears(&name, &kept_text),
        })
        .collect();
    (kept_imports, kept_rest)
}

pub(crate) fn island_used_text(src: &str, document: &Document) -> String {
    let mut text = String::new();
    for item in &document.items {
        match item {
            Item::Render(render) => {
                text.push('\n');
                text.push_str(render.span.of(src));
            }
            Item::Template(template) => {
                text.push('\n');
                text.push_str(template.span().of(src));
            }
            _ => {}
        }
    }
    let components: Vec<_> = document
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Component(decl) => Some(decl),
            _ => None,
        })
        .collect();
    let mut included = vec![false; components.len()];
    loop {
        let mut changed = false;
        for (i, component) in components.iter().enumerate() {
            if included[i] {
                continue;
            }
            let pascal = component.name.name.as_str();
            let camel = pascal_to_camel(pascal);
            if roc_name_appears(pascal, &text) || roc_name_appears(&camel, &text) {
                included[i] = true;
                text.push('\n');
                text.push_str(component.span.of(src));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    text
}

pub(crate) fn island_item_count(document: &Document) -> usize {
    document
        .items
        .iter()
        .filter(|item| is_island_item(item))
        .count()
}

pub(crate) fn is_island_item(item: &Item) -> bool {
    match item {
        Item::Render(_) => true,
        Item::Template(TemplateItem::Let(_)) => false,
        Item::Template(_) => true,
        _ => false,
    }
}

impl<'a> Emitter<'a> {
    pub(crate) fn lower_render_call(&mut self, render: &RenderDecl) {
        if render.path.roc_name.is_empty() {
            self.emit_html(".empty");
            return;
        }
        self.emit_mapped(
            &render.path.roc_name,
            render.path.span,
            OriginKind::RenderRoc,
        );
        self.emit("(");
        let args = render.args.of(self.source.src).trim();
        if !args.is_empty() {
            self.emit_mapped(args, render.args, OriginKind::RenderRoc);
        }
        self.emit(")");
    }

    pub(crate) fn emit_island_list(&mut self, document: &Document) {
        self.emit("[\n");
        self.indent += 1;
        for item in &document.items {
            match item {
                Item::Render(render) => {
                    self.push_indent();
                    self.lower_render_call(render);
                    self.emit(",\n");
                }
                Item::Template(TemplateItem::Let(_)) => {}
                Item::Template(item) if matches!(item, TemplateItem::For(_)) => {
                    self.push_indent();
                    self.emit_html(".fragment(\n");
                    self.indent += 1;
                    self.push_indent();
                    self.splice_template(std::slice::from_ref(item), TemplateValueCtx::List);
                    self.emit("\n");
                    self.indent -= 1;
                    self.push_indent();
                    self.emit("),\n");
                }
                Item::Template(item) => {
                    self.push_indent();
                    self.splice_template(std::slice::from_ref(item), TemplateValueCtx::Node);
                    self.emit(",\n");
                }
                _ => {}
            }
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }
}

impl<'a> Emitter<'a> {
    pub(crate) fn splice_template(&mut self, items: &[TemplateItem], ctx: TemplateValueCtx) {
        let lowered: LoweredTemplate = lower_template_items(
            self.source,
            items,
            self.options,
            &self.field_defaults,
            self.indent,
            ctx,
            self.css_stamp.clone(),
        );
        let start = self.roc.len();
        self.roc.push_str(&lowered.roc);
        self.at_line_start = lowered.roc.ends_with('\n');
        for mut segment in lowered.segments {
            segment.generated.start += start as u32;
            segment.generated.end += start as u32;
            self.segments.push(segment);
        }
    }
}
