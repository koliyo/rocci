use rocci_template::{Diagnostic, OriginKind, SourceFile, Span};

use crate::ast::{BlockCall, Item, MdNode, ParamValue};
use crate::docs::{
    docs_fields_from_params, extract_lines, extract_region, field_bool, field_string,
    resolve_include_path,
};
use crate::parse_fragment;

use super::emitter::Emitter;

pub(crate) fn illegal_docs_item(item: &Item) -> Option<&'static str> {
    match item {
        Item::Markdown(_) | Item::Block(_) => None,
        Item::Page(_) => Some("page"),
        Item::Roc(_) => Some("roc"),
        Item::Render(_) => Some("render"),
        Item::Component(_) => Some("component"),
        Item::Fixture(_) => Some("fixture"),
        Item::Css(_) => Some("css"),
        Item::Context(_) => Some("context"),
        Item::Init(_) => Some("init"),
        Item::Live(_) => Some("live"),
        Item::View(_) => Some("view"),
        Item::Fragment(_) => Some("fragment"),
        Item::Command(_) => Some("command"),
        Item::Use(_) => Some("use"),
        Item::Template(_) => Some("template"),
    }
}

pub(crate) fn is_heading_sugar(call: &BlockCall, src: &str) -> bool {
    crate::registry::heading_level(&call.name).is_some()
        && (call.is_colon(src)
            || src
                .get(call.span.start as usize..)
                .unwrap_or("")
                .trim_start_matches([' ', '\t'])
                .starts_with('#'))
}

pub(crate) fn heading_id_from_params(call: &BlockCall) -> Option<String> {
    call.params.as_ref().and_then(|params| {
        params
            .fields
            .iter()
            .find(|field| field.name == "id")
            .and_then(|field| match &field.value {
                ParamValue::StringLit { value, .. } => Some(value.clone()),
                ParamValue::Ident { name, .. } => Some(name.clone()),
                _ => None,
            })
    })
}

pub(crate) fn heading_inline_nodes(items: &[Item]) -> Vec<MdNode> {
    let mut nodes = Vec::new();
    for item in items {
        match item {
            Item::Markdown(MdNode::Paragraph { children, .. }) => {
                nodes.extend(children.iter().cloned());
            }
            Item::Markdown(node) => nodes.push(node.clone()),
            _ => {}
        }
    }
    nodes
}

impl<'a> Emitter<'a> {
    pub(crate) fn lower_docs(&mut self, call: &BlockCall) {
        if let Some(imported) = self.imported_kinds.get(&call.name).cloned() {
            self.lower_imported_block(call, &imported);
            return;
        }
        let src = self.source.src;
        let fields = docs_fields_from_params(call.params.as_ref());
        let content = call
            .content_span()
            .unwrap_or_else(|| Span::point(call.span.end as usize));
        let title = fields
            .iter()
            .find(|field| field.name == "title")
            .or_else(|| fields.iter().find(|field| field.name == "term"))
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let summary = fields
            .iter()
            .find(|field| field.name == "summary")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let label = fields
            .iter()
            .find(|field| field.name == "label")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let open = fields
            .iter()
            .find(|field| field.name == "open")
            .and_then(|field| field_bool(src, field))
            .unwrap_or(false);
        let caption = fields
            .iter()
            .find(|field| field.name == "caption")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let credit = fields
            .iter()
            .find(|field| field.name == "credit")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        if call.name == "include" {
            self.lower_docs_include(call, &fields);
            return;
        }
        let parsed = parse_fragment(self.source, content, false);
        self.diagnostics.extend(parsed.diagnostics);
        for item in &parsed.document.items {
            if let Some(kind) = illegal_docs_item(item) {
                self.diagnostics.push(Diagnostic::error(
                    item.span(),
                    format!("`@{kind}` is not allowed inside an article block"),
                ));
            }
        }
        let class = if crate::registry::is_aside(&call.name) {
            format!("rd-docs-aside rd-docs-block rd-docs-{}", call.name)
        } else {
            format!("rd-docs-{} rd-docs-block", call.name)
        };
        let tag = if crate::registry::is_aside(&call.name) {
            "aside"
        } else {
            match call.name.as_str() {
                "details" => "details",
                "figure" => "figure",
                "badge" => "p",
                _ => "section",
            }
        };
        let label_text = if crate::registry::is_aside(&call.name) {
            match call.name.as_str() {
                "note" => "Note",
                "tip" => "Tip",
                "caution" => "Caution",
                "danger" => "Danger",
                "deprecated" => "Deprecated",
                _ => "Note",
            }
        } else {
            ""
        };
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(tag, call.span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        let mut attrs = vec![
            ("class", class.as_str()),
            ("data-rocci-docs", call.name.as_str()),
        ];
        let aria = if call.name == "deprecated" {
            "Deprecated"
        } else if call.name == "file-tree" {
            "File tree"
        } else if call.name == "tab" && !label.is_empty() {
            label.as_str()
        } else {
            ""
        };
        if !aria.is_empty() {
            attrs.push(("aria-label", aria));
        }
        if call.name == "details" && open {
            attrs.push(("open", "open"));
        }
        self.emit_attrs(&attrs, call.span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        if call.name == "details" {
            self.push_indent();
            self.emit_text_element("summary", "rd-docs-summary", &summary, call.span);
            self.emit(",\n");
        } else if !label_text.is_empty() {
            self.push_indent();
            self.emit_text_element("p", "rd-docs-label", label_text, call.span);
            self.emit(",\n");
        }
        if call.name == "tab" && !label.is_empty() {
            self.push_indent();
            self.emit_text_element("h3", "rd-docs-tab-label", &label, call.span);
            self.emit(",\n");
        }
        if !title.is_empty() && call.name != "details" {
            self.push_indent();
            self.emit_text_element("p", "rd-docs-title", &title, call.span);
            self.emit(",\n");
        }
        if call.name == "badge" {
            self.push_indent();
            self.emit_text_element(
                "span",
                "rd-docs-badge-label",
                if label.is_empty() { &title } else { &label },
                call.span,
            );
            self.emit(",\n");
        }
        self.lower_docs_items(&parsed.document.items);
        if call.name == "figure" {
            if !caption.is_empty() {
                self.push_indent();
                self.emit_text_element("figcaption", "rd-docs-caption", &caption, call.span);
                self.emit(",\n");
            }
            if !credit.is_empty() {
                self.push_indent();
                self.emit_text_element("p", "rd-docs-credit", &credit, call.span);
                self.emit(",\n");
            }
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_imported_block(
        &mut self,
        call: &BlockCall,
        imported: &crate::imports::ImportedKind,
    ) {
        let content = call
            .content_span()
            .unwrap_or_else(|| Span::point(call.span.end as usize));
        let parsed = parse_fragment(self.source, content, false);
        self.diagnostics.extend(parsed.diagnostics);
        for item in &parsed.document.items {
            if let Some(kind) = illegal_docs_item(item) {
                self.diagnostics.push(Diagnostic::error(
                    item.span(),
                    format!("`@{kind}` is not allowed inside an article block"),
                ));
            }
        }
        self.emit(&imported.roc_name);
        self.emit("(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_imported_props(call, &imported.roc_name);
        self.emit(",\n");
        self.push_indent();
        self.emit_html(".fragment([\n");
        self.indent += 1;
        self.lower_docs_items(&parsed.document.items);
        self.indent -= 1;
        self.push_indent();
        self.emit("]),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn emit_imported_props(&mut self, call: &BlockCall, roc_name: &str) {
        let fields = call
            .params
            .as_ref()
            .map(|record| record.fields.as_slice())
            .unwrap_or(&[]);
        let missing_defaults: Vec<(String, String)> = self
            .field_defaults
            .get(roc_name)
            .into_iter()
            .flatten()
            .filter(|(name, _)| !fields.iter().any(|field| field.name == *name))
            .cloned()
            .collect();
        if fields.is_empty() && missing_defaults.is_empty() {
            self.emit("{}");
            return;
        }
        self.emit("{ ");
        let mut first = true;
        for field in fields {
            if !first {
                self.emit(", ");
            }
            first = false;
            self.emit(&field.name);
            self.emit(": ");
            self.emit_param_value(&field.value);
        }
        for (name, default) in missing_defaults {
            if !first {
                self.emit(", ");
            }
            first = false;
            self.emit(&name);
            self.emit(": ");
            self.emit(&default);
        }
        self.emit(" }");
    }

    pub(crate) fn emit_param_value(&mut self, value: &ParamValue) {
        match value {
            ParamValue::StringLit { value, span } => {
                self.emit_string(value, *span, OriginKind::StaticMarkup);
            }
            ParamValue::BoolLit { value, span } => {
                self.emit_mapped(
                    if *value { "True" } else { "False" },
                    *span,
                    OriginKind::StaticMarkup,
                );
            }
            ParamValue::NumberLit { value, span } => {
                self.emit_mapped(value, *span, OriginKind::StaticMarkup);
            }
            ParamValue::Ident { name, span } => {
                self.emit_string(name, *span, OriginKind::StaticMarkup);
            }
            ParamValue::Record(record) => {
                if record.fields.is_empty() {
                    self.emit("{}");
                    return;
                }
                self.emit("{ ");
                for (index, field) in record.fields.iter().enumerate() {
                    if index > 0 {
                        self.emit(", ");
                    }
                    self.emit(&field.name);
                    self.emit(": ");
                    self.emit_param_value(&field.value);
                }
                self.emit(" }");
            }
            ParamValue::List(list) => {
                self.emit("[");
                for (index, item) in list.items.iter().enumerate() {
                    if index > 0 {
                        self.emit(", ");
                    }
                    self.emit_param_value(item);
                }
                self.emit("]");
            }
        }
    }

    pub(crate) fn lower_docs_items(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Markdown(node) => {
                    self.push_indent();
                    self.lower_md(node);
                    self.emit(",\n");
                }
                Item::Block(nested) if is_heading_sugar(nested, self.source.src) => {
                    self.push_indent();
                    self.lower_heading_sugar(nested);
                    self.emit(",\n");
                }
                Item::Block(nested) if nested.name == "img" => {
                    self.push_indent();
                    self.lower_img(nested);
                    self.emit(",\n");
                }
                Item::Block(nested) => {
                    self.push_indent();
                    self.lower_docs(nested);
                    self.emit(",\n");
                }
                _ => {}
            }
        }
    }

    pub(crate) fn lower_heading_sugar(&mut self, call: &BlockCall) {
        let level = crate::registry::heading_level(&call.name).unwrap_or(1);
        let id = heading_id_from_params(call).unwrap_or_default();
        let tag = format!("h{level}");
        let class = format!("rd-header-{level}");
        let content = call
            .content_span()
            .unwrap_or_else(|| Span::point(call.span.end as usize));
        let parsed = parse_fragment(self.source, content, false);
        self.diagnostics.extend(parsed.diagnostics);
        let mut children = heading_inline_nodes(&parsed.document.items);
        crate::markdown::restore_interpolations(
            self.source.src,
            &mut children,
            &mut Vec::new(),
            "Markdown interpolation `@{` is not allowed in headings",
        );
        self.emit_element(
            &tag,
            &[("class", class.as_str()), ("id", id.as_str())],
            &children,
            false,
            call.span,
        );
    }

    pub(crate) fn lower_img(&mut self, call: &BlockCall) {
        let body = call
            .params
            .as_ref()
            .map(|params| params.span)
            .unwrap_or(call.span);
        let fields =
            crate::img::img_fields_from_params(call.params.as_ref(), body, self.diagnostics);
        let image = crate::img::StaticImage::from_fields(&fields, call.span);
        let attrs = image.html_attrs();

        self.emit_html(".void_element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("img", call.span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_img_attrs(&attrs, call.span);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn emit_img_attrs(&mut self, attrs: &[crate::img::ImgHtmlAttr], decl_span: Span) {
        if attrs.is_empty() && self.css_stamp.is_none() {
            self.emit("[]");
            return;
        }
        self.emit("[\n");
        self.indent += 1;
        for attr in attrs {
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string(attr.name, attr.span, OriginKind::MarkdownStructure);
            self.emit(", ");
            self.emit_string(&attr.value, attr.span, OriginKind::MarkdownText);
            self.emit("),\n");
        }
        if let Some(stamp) = &self.css_stamp.clone() {
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("data-rocci-css", decl_span, OriginKind::Scaffolding);
            self.emit(", ");
            self.emit_string(stamp, decl_span, OriginKind::Scaffolding);
            self.emit("),\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    pub(crate) fn lower_docs_include(
        &mut self,
        call: &BlockCall,
        fields: &[crate::docs::DocsField],
    ) {
        if !self.resolve_includes {
            self.emit_html(".empty");
            return;
        }
        let src = self.source.src;
        let path = fields
            .iter()
            .find(|field| field.name == "path")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let region = fields
            .iter()
            .find(|field| field.name == "region")
            .and_then(|field| field_string(src, field));
        let start = fields
            .iter()
            .find(|field| field.name == "start")
            .and_then(|field| field.value.of(src).trim().parse::<u32>().ok());
        let end = fields
            .iter()
            .find(|field| field.name == "end")
            .and_then(|field| field.value.of(src).trim().parse::<u32>().ok());
        let language = fields
            .iter()
            .find(|field| field.name == "language")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let resolved = match resolve_include_path(self.source.name, &path) {
            Ok(path) => path,
            Err(err) => {
                self.diagnostics.push(Diagnostic::error(call.span, err));
                self.emit_html(".empty");
                return;
            }
        };
        let contents = match std::fs::read_to_string(&resolved) {
            Ok(contents) => contents,
            Err(_) => {
                self.diagnostics.push(Diagnostic::error(
                    call.span,
                    format!("could not read include `{}`", resolved.display()),
                ));
                self.emit_html(".empty");
                return;
            }
        };
        let excerpt = if let Some(region) = region.as_deref() {
            match extract_region(&contents, region) {
                Ok((excerpt, _, _)) => excerpt,
                Err(err) => {
                    self.diagnostics.push(Diagnostic::error(call.span, err));
                    self.emit_html(".empty");
                    return;
                }
            }
        } else if let (Some(start), Some(end)) = (start, end) {
            match extract_lines(&contents, start, end) {
                Ok((excerpt, _, _)) => excerpt,
                Err(err) => {
                    self.diagnostics.push(Diagnostic::error(call.span, err));
                    self.emit_html(".empty");
                    return;
                }
            }
        } else {
            contents
        };
        let is_doc = matches!(
            resolved.extension().and_then(|ext| ext.to_str()),
            Some("rocdown" | "md" | "markdown")
        );
        if is_doc {
            let included = crate::parse(
                SourceFile::new(&resolved.to_string_lossy(), &excerpt),
                false,
            );
            self.diagnostics.extend(included.diagnostics);
            for item in &included.document.items {
                if let Some(kind) = illegal_docs_item(item) {
                    self.diagnostics.push(Diagnostic::error(
                        item.span(),
                        format!("`@{kind}` is not allowed inside `:include`"),
                    ));
                }
            }
            self.emit_html(".fragment([\n");
            self.indent += 1;
            self.lower_docs_items(&included.document.items);
            self.indent -= 1;
            self.push_indent();
            self.emit("])");
            return;
        }
        let info = if language.is_empty() {
            resolved
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_string()
        } else {
            language
        };
        self.lower_md(&MdNode::CodeBlock {
            info,
            literal: excerpt,
            span: call.span,
        });
    }
}
