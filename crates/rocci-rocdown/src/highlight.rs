use rocci_highlight::composite::{
    collect_component, collect_context, collect_css, collect_embedded_regions, collect_fixture,
    collect_init, collect_items, collect_keyword, collect_on, heading_marker,
};
use rocci_highlight::language::LanguageId;
use rocci_highlight::regions::{RegionBuilder, RegionContext, RegionPurpose, RegionTree};
use rocci_highlight::token::{HighlightKind, HighlightSpan, resolve_and_sort_spans};
use rocci_template::{Cursor, SourceFile, Span};

use crate::ast::{BlockCall, DocsDecl, Document, HeadingInfo, ImgDecl, Item, MdNode};

pub fn highlight_rocdown(source: &str) -> Vec<HighlightSpan> {
    let sf = SourceFile::new("snippet.rocdown", source);
    let parsed = crate::parse(sf, false);
    highlight_rocdown_document(source, &parsed.document, &parsed.headings)
}

pub fn highlight_rocdown_document(
    source: &str,
    document: &Document,
    headings: &[HeadingInfo],
) -> Vec<HighlightSpan> {
    let regions = extract_rocdown_regions("snippet.rocdown", source, document, headings);
    let mut raw_tokens = Vec::new();
    collect_rocdown(source, &mut raw_tokens, document, headings);
    collect_embedded_regions(source, &mut raw_tokens, &regions);
    resolve_and_sort_spans(source, &raw_tokens)
}

pub fn collect_rocdown(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    document: &Document,
    headings: &[HeadingInfo],
) {
    for heading in headings {
        if let Some(span) = heading_marker(src, heading.span, heading.level) {
            collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
        }
    }
    for item in &document.items {
        match item {
            Item::Markdown(md_node) => {
                let mut md_tokens = Vec::new();
                collect_markdown_node(src, md_node, &mut md_tokens);
                for tok in md_tokens {
                    collector.push(HighlightSpan::new(
                        tok.span,
                        tok.kind,
                        tok.modifiers,
                        tok.priority,
                    ));
                }
            }
            Item::Page(page) => {
                collect_keyword(src, collector, page.span, page.body.start, "@page");
            }
            Item::Roc(roc) => {
                collect_keyword(src, collector, roc.span, roc.body.start, "@roc");
            }
            Item::Render(render) => {
                collect_keyword(src, collector, render.span, render.expr.start, "@render");
            }
            Item::Component(component) => {
                collect_component(src, collector, component);
            }
            Item::Fixture(fixture) => collect_fixture(src, collector, fixture),
            Item::Css(css) => collect_css(src, collector, css),
            Item::Context(context) => collect_context(src, collector, context),
            Item::Init(init) => collect_init(src, collector, init),
            Item::On(on) => collect_on(src, collector, on),
            Item::Template(item) => {
                collect_items(src, collector, std::slice::from_ref(item));
            }
            Item::Docs(docs) => {
                collect_docs(src, collector, docs);
            }
            Item::Img(img) => {
                collect_img(src, collector, img);
            }
            Item::Block(call) if call.is_legacy_img(src) => {
                collect_img_block(src, collector, call);
            }
            Item::Block(call) => {
                collect_docs_block(src, collector, call);
            }
        }
    }
}

pub fn collect_img(src: &str, collector: &mut Vec<HighlightSpan>, img: &ImgDecl) {
    collect_img_body(src, collector, img.span, img.body);
}

fn collect_img_block(src: &str, collector: &mut Vec<HighlightSpan>, call: &BlockCall) {
    let body = call
        .params
        .as_ref()
        .map(|params| params.span)
        .unwrap_or(call.span);
    collect_img_body(src, collector, call.span, body);
}

fn collect_img_body(src: &str, collector: &mut Vec<HighlightSpan>, span: Span, body: Span) {
    collect_keyword(src, collector, span, body.start, "@img");
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;
    while cur.pos < end && !cur.is_eof() {
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let Some(name_span) = cur.scan_ident() else {
            break;
        };
        collector.push(HighlightSpan::new(
            name_span,
            HighlightKind::Property,
            0,
            50,
        ));
        cur.skip_trivia();
        if !cur.eat(':') {
            break;
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        if cur.peek() == Some('"') {
            cur.skip_string();
            let value_span = Span::new(value_start, cur.pos.min(end));
            collector.push(HighlightSpan::new(value_span, HighlightKind::String, 0, 50));
        }
    }
}

pub fn collect_docs(src: &str, collector: &mut Vec<HighlightSpan>, docs: &DocsDecl) {
    let call = crate::docs::block_from_docs_decl(src, docs.clone());
    collect_docs_block(src, collector, &call);
}

fn collect_docs_block(src: &str, collector: &mut Vec<HighlightSpan>, call: &BlockCall) {
    collect_keyword(src, collector, call.span, call.name_span.start, "@docs");
    collector.push(HighlightSpan::new(
        call.name_span,
        HighlightKind::Type,
        0,
        55,
    ));
    let fields = crate::docs::docs_fields_from_params(call.params.as_ref());
    for field in fields {
        collector.push(HighlightSpan::new(
            field.name_span,
            HighlightKind::Property,
            0,
            50,
        ));
        let val_str = field.value.of(src).trim();
        if val_str.starts_with('"') {
            collector.push(HighlightSpan::new(
                field.value,
                HighlightKind::String,
                0,
                50,
            ));
        } else if val_str == "true"
            || val_str == "false"
            || val_str == "Bool.true"
            || val_str == "Bool.false"
        {
            collector.push(HighlightSpan::new(
                field.value,
                HighlightKind::Keyword,
                0,
                50,
            ));
        }
    }
    if let Some(content) = call.content_span()
        && !content.is_empty()
        && (content.start as usize) < src.len()
    {
        let source = SourceFile::new("docs", src);
        let parsed = crate::parse_fragment(source, content, false);
        collect_rocdown(src, collector, &parsed.document, &parsed.headings);
    }
}

pub fn collect_markdown_node(src: &str, node: &MdNode, tokens: &mut Vec<HighlightSpan>) {
    match node {
        MdNode::Heading {
            level,
            span,
            children,
            ..
        } => {
            if let Some(marker) = heading_marker(src, *span, *level) {
                tokens.push(HighlightSpan::new(marker, HighlightKind::Keyword, 0, 50));
            }
            for child in children {
                collect_markdown_node(src, child, tokens);
            }
        }
        MdNode::Code { span, .. } => {
            if !span.is_empty() {
                tokens.push(HighlightSpan::new(*span, HighlightKind::String, 0, 40));
            }
        }
        MdNode::Link {
            children,
            span,
            url,
            ..
        } => {
            for child in children {
                collect_markdown_node(src, child, tokens);
            }
            if !url.is_empty() {
                let text = span.of(src);
                if let Some(idx) = text.rfind(url) {
                    let start = span.start as usize + idx;
                    tokens.push(HighlightSpan::new(
                        Span::new(start, start + url.len()),
                        HighlightKind::String,
                        0,
                        40,
                    ));
                }
            }
        }
        MdNode::Image { span, url, .. } => {
            if !url.is_empty() {
                let text = span.of(src);
                if let Some(idx) = text.rfind(url) {
                    let start = span.start as usize + idx;
                    tokens.push(HighlightSpan::new(
                        Span::new(start, start + url.len()),
                        HighlightKind::String,
                        0,
                        40,
                    ));
                }
            }
        }
        MdNode::ThematicBreak { span } => {
            if !span.is_empty() {
                tokens.push(HighlightSpan::new(*span, HighlightKind::Operator, 0, 40));
            }
        }
        MdNode::Paragraph { children, .. }
        | MdNode::BlockQuote { children, .. }
        | MdNode::List { children, .. }
        | MdNode::Item { children, .. }
        | MdNode::TaskItem { children, .. }
        | MdNode::Table { children, .. }
        | MdNode::TableRow { children, .. }
        | MdNode::TableCell { children, .. }
        | MdNode::Emph { children, .. }
        | MdNode::Strong { children, .. }
        | MdNode::Strikethrough { children, .. }
        | MdNode::FootnoteDefinition { children, .. } => {
            for child in children {
                collect_markdown_node(src, child, tokens);
            }
        }
        MdNode::CodeBlock { .. }
        | MdNode::Text { .. }
        | MdNode::SoftBreak { .. }
        | MdNode::LineBreak { .. }
        | MdNode::FootnoteReference { .. }
        | MdNode::RawHtml { .. } => {}
    }
}

pub fn extract_rocdown_regions(
    _name: &str,
    text: &str,
    doc: &Document,
    _headings: &[HeadingInfo],
) -> RegionTree {
    let mut builder = RegionBuilder::new();
    let root = builder.add(
        LanguageId::Markdown,
        RegionContext::Document,
        RegionPurpose::HostStructure,
        Span::new(0, text.len()),
        None,
        0,
    );

    collect_rocdown_items(&mut builder, text, &doc.items, root);

    RegionTree::new(builder.regions)
}

fn collect_rocdown_items(
    builder: &mut RegionBuilder,
    text: &str,
    items: &[Item],
    parent_id: usize,
) {
    for item in items {
        match item {
            Item::Markdown(md_node) => {
                collect_md_node_regions(builder, text, md_node, parent_id);
            }
            Item::Page(page) => {
                let page_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::Metadata,
                    page.span,
                    Some(parent_id),
                    10,
                );
                if !page.body.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Body,
                        RegionPurpose::Metadata,
                        page.body,
                        Some(page_id),
                        20,
                    );
                }
            }
            Item::Roc(roc) => {
                let roc_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    roc.span,
                    Some(parent_id),
                    10,
                );
                if !roc.body.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Module,
                        RegionPurpose::Executable,
                        roc.body,
                        Some(roc_id),
                        20,
                    );
                }
            }
            Item::Render(render) => {
                let render_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    render.span,
                    Some(parent_id),
                    10,
                );
                if !render.expr.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        render.expr,
                        Some(render_id),
                        20,
                    );
                }
            }
            Item::Component(c) => {
                let comp_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    c.span,
                    Some(parent_id),
                    10,
                );
                if !c.params.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Params,
                        RegionPurpose::Executable,
                        c.params,
                        Some(comp_id),
                        20,
                    );
                }
                collect_template_item_regions(builder, &c.body.items, comp_id);
            }
            Item::Fixture(f) => {
                let fix_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    f.span,
                    Some(parent_id),
                    10,
                );
                if !f.value.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        f.value,
                        Some(fix_id),
                        20,
                    );
                }
            }
            Item::Css(css) => {
                let css_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    css.span,
                    Some(parent_id),
                    10,
                );
                if !css.body.is_empty() {
                    builder.add(
                        LanguageId::Css,
                        RegionContext::Stylesheet,
                        RegionPurpose::HostStructure,
                        css.body,
                        Some(css_id),
                        20,
                    );
                }
            }
            Item::Context(ctx) => {
                let ctx_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    ctx.span,
                    Some(parent_id),
                    10,
                );
                if !ctx.ty.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Type,
                        RegionPurpose::Executable,
                        ctx.ty,
                        Some(ctx_id),
                        20,
                    );
                }
            }
            Item::Init(init) => {
                let init_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    init.span,
                    Some(parent_id),
                    10,
                );
                if !init.body.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Body,
                        RegionPurpose::Executable,
                        init.body,
                        Some(init_id),
                        20,
                    );
                }
            }
            Item::On(on) => {
                let on_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    on.span,
                    Some(parent_id),
                    10,
                );
                if let Some(params) = on.params
                    && !params.is_empty()
                {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Params,
                        RegionPurpose::Executable,
                        params,
                        Some(on_id),
                        20,
                    );
                }
                if !on.body.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Body,
                        RegionPurpose::Executable,
                        on.body,
                        Some(on_id),
                        20,
                    );
                }
            }
            Item::Template(item) => {
                collect_template_item_regions(builder, std::slice::from_ref(item), parent_id);
            }
            Item::Docs(docs) => {
                let docs_id = builder.add(
                    LanguageId::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    docs.span,
                    Some(parent_id),
                    10,
                );
                let (_fields, content) = crate::split_docs_body(text, docs.body);
                if !content.is_empty() && (content.start as usize) < text.len() {
                    let source = SourceFile::new("docs", text);
                    let parsed = crate::parse_fragment(source, content, false);
                    collect_rocdown_items(builder, text, &parsed.document.items, docs_id);
                }
            }
            Item::Img(img) => {
                builder.add(
                    LanguageId::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    img.span,
                    Some(parent_id),
                    10,
                );
            }
            Item::Block(call) if call.is_legacy_img(text) => {
                builder.add(
                    LanguageId::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    call.span,
                    Some(parent_id),
                    10,
                );
            }
            Item::Block(call) => {
                let docs_id = builder.add(
                    LanguageId::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    call.span,
                    Some(parent_id),
                    10,
                );
                if let Some(content) = call.content_span()
                    && !content.is_empty()
                    && (content.start as usize) < text.len()
                {
                    let source = SourceFile::new("docs", text);
                    let parsed = crate::parse_fragment(source, content, false);
                    collect_rocdown_items(builder, text, &parsed.document.items, docs_id);
                }
            }
        }
    }
}

fn collect_template_item_regions(
    builder: &mut RegionBuilder,
    items: &[rocci_template::TemplateItem],
    parent_id: usize,
) {
    for item in items {
        match item {
            rocci_template::TemplateItem::Element(el) => {
                let el_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    el.span,
                    Some(parent_id),
                    10,
                );
                for attr in &el.attrs {
                    match &attr.value {
                        rocci_template::AttrValue::Expr { expr } => {
                            if !expr.is_empty() {
                                builder.add(
                                    LanguageId::Roc,
                                    RegionContext::Expression,
                                    RegionPurpose::Executable,
                                    *expr,
                                    Some(el_id),
                                    20,
                                );
                            }
                        }
                        rocci_template::AttrValue::Action { args, .. } => {
                            if !args.is_empty() {
                                builder.add(
                                    LanguageId::Roc,
                                    RegionContext::Expression,
                                    RegionPurpose::Executable,
                                    *args,
                                    Some(el_id),
                                    20,
                                );
                            }
                        }
                        rocci_template::AttrValue::Static { .. }
                        | rocci_template::AttrValue::Boolean => {}
                    }
                }
                collect_template_item_regions(builder, &el.children, el_id);
            }
            rocci_template::TemplateItem::ComponentCall(call) => {
                let call_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    call.span,
                    Some(parent_id),
                    10,
                );
                for attr in &call.attrs {
                    match &attr.value {
                        rocci_template::AttrValue::Expr { expr } => {
                            if !expr.is_empty() {
                                builder.add(
                                    LanguageId::Roc,
                                    RegionContext::Expression,
                                    RegionPurpose::Executable,
                                    *expr,
                                    Some(call_id),
                                    20,
                                );
                            }
                        }
                        rocci_template::AttrValue::Action { args, .. } => {
                            if !args.is_empty() {
                                builder.add(
                                    LanguageId::Roc,
                                    RegionContext::Expression,
                                    RegionPurpose::Executable,
                                    *args,
                                    Some(call_id),
                                    20,
                                );
                            }
                        }
                        rocci_template::AttrValue::Static { .. }
                        | rocci_template::AttrValue::Boolean => {}
                    }
                }
                if let Some(children) = &call.children {
                    collect_template_item_regions(builder, children, call_id);
                }
            }
            rocci_template::TemplateItem::Fragment(frag) => {
                let frag_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    frag.span,
                    Some(parent_id),
                    10,
                );
                collect_template_item_regions(builder, &frag.children, frag_id);
            }
            rocci_template::TemplateItem::Interpolation(interp) => {
                if !interp.expr.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        interp.expr,
                        Some(parent_id),
                        20,
                    );
                }
            }
            rocci_template::TemplateItem::If(dir) => {
                let if_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.condition.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        dir.condition,
                        Some(if_id),
                        20,
                    );
                }
                collect_template_item_regions(builder, &dir.then_body.items, if_id);
                for (cond, body) in &dir.else_ifs {
                    if !cond.is_empty() {
                        builder.add(
                            LanguageId::Roc,
                            RegionContext::Expression,
                            RegionPurpose::Executable,
                            *cond,
                            Some(if_id),
                            20,
                        );
                    }
                    collect_template_item_regions(builder, &body.items, if_id);
                }
                if let Some(body) = &dir.else_body {
                    collect_template_item_regions(builder, &body.items, if_id);
                }
            }
            rocci_template::TemplateItem::For(dir) => {
                let for_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.binder.span.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Pattern,
                        RegionPurpose::Executable,
                        dir.binder.span,
                        Some(for_id),
                        20,
                    );
                }
                if !dir.collection.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        dir.collection,
                        Some(for_id),
                        20,
                    );
                }
                collect_template_item_regions(builder, &dir.body.items, for_id);
            }
            rocci_template::TemplateItem::Match(dir) => {
                let match_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.scrutinee.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        dir.scrutinee,
                        Some(match_id),
                        20,
                    );
                }
                for arm in &dir.arms {
                    if !arm.pattern.is_empty() {
                        builder.add(
                            LanguageId::Roc,
                            RegionContext::Pattern,
                            RegionPurpose::Executable,
                            arm.pattern,
                            Some(match_id),
                            20,
                        );
                    }
                    collect_template_item_regions(
                        builder,
                        std::slice::from_ref(&*arm.value),
                        match_id,
                    );
                }
            }
            rocci_template::TemplateItem::Let(dir) => {
                let let_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.binder.span.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Pattern,
                        RegionPurpose::Executable,
                        dir.binder.span,
                        Some(let_id),
                        20,
                    );
                }
                if !dir.expr.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        dir.expr,
                        Some(let_id),
                        20,
                    );
                }
            }
            rocci_template::TemplateItem::Css(css) => {
                let css_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    css.span,
                    Some(parent_id),
                    10,
                );
                if !css.body.is_empty() {
                    builder.add(
                        LanguageId::Css,
                        RegionContext::Stylesheet,
                        RegionPurpose::HostStructure,
                        css.body,
                        Some(css_id),
                        20,
                    );
                }
            }
            rocci_template::TemplateItem::Text(_) => {}
        }
    }
}

fn collect_md_node_regions(
    builder: &mut RegionBuilder,
    text: &str,
    node: &MdNode,
    parent_id: usize,
) {
    match node {
        MdNode::CodeBlock { info, span, .. } => {
            if !span.is_empty() {
                let lang = LanguageId::parse(info);
                let inner_span = code_block_inner_span(text, *span);
                if !inner_span.is_empty() {
                    builder.add(
                        lang,
                        RegionContext::Fence,
                        RegionPurpose::DisplayOnly,
                        inner_span,
                        Some(parent_id),
                        20,
                    );
                }
            }
        }
        MdNode::Heading { children, .. }
        | MdNode::Paragraph { children, .. }
        | MdNode::BlockQuote { children, .. }
        | MdNode::List { children, .. }
        | MdNode::Item { children, .. }
        | MdNode::TaskItem { children, .. }
        | MdNode::Table { children, .. }
        | MdNode::TableRow { children, .. }
        | MdNode::TableCell { children, .. }
        | MdNode::Emph { children, .. }
        | MdNode::Strong { children, .. }
        | MdNode::Strikethrough { children, .. }
        | MdNode::FootnoteDefinition { children, .. }
        | MdNode::Link { children, .. } => {
            for child in children {
                collect_md_node_regions(builder, text, child, parent_id);
            }
        }
        MdNode::ThematicBreak { .. }
        | MdNode::Text { .. }
        | MdNode::SoftBreak { .. }
        | MdNode::LineBreak { .. }
        | MdNode::Code { .. }
        | MdNode::FootnoteReference { .. }
        | MdNode::Image { .. }
        | MdNode::RawHtml { .. } => {}
    }
}

fn code_block_inner_span(src: &str, span: Span) -> Span {
    let start = (span.start as usize).min(src.len());
    let end = (span.end as usize).min(src.len());
    if start >= end {
        return span;
    }
    let text = &src[start..end];
    let start_offset = if let Some(idx) = text.find('\n') {
        idx + 1
    } else {
        0
    };
    let inner_text = &text[start_offset..];
    let end_offset = if let Some(idx) = inner_text.rfind("```") {
        start_offset + idx
    } else if let Some(idx) = inner_text.rfind("~~~") {
        start_offset + idx
    } else {
        text.len()
    };
    Span::new(start + start_offset, start + end_offset)
}
