use rocci_highlight::composite::{
    collect_command, collect_component, collect_context, collect_css, collect_embedded_regions,
    collect_fixture, collect_fragment, collect_init, collect_items, collect_keyword, collect_live,
    collect_view, heading_marker,
};
use rocci_highlight::language::LanguageId;
use rocci_highlight::regions::{RegionBuilder, RegionContext, RegionPurpose, RegionTree};
use rocci_highlight::token::{HighlightKind, HighlightSpan, resolve_and_sort_spans};
use rocci_template::{Cursor, SourceFile, Span};

use crate::ast::{BlockCall, BlockContent, Document, HeadingInfo, Item, MdNode};

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

fn is_atx_heading_sugar(src: &str, call: &BlockCall) -> bool {
    crate::registry::heading_level(&call.name).is_some()
        && !call.is_colon(src)
        && src
            .get(call.span.start as usize..)
            .unwrap_or("")
            .trim_start_matches([' ', '\t'])
            .starts_with('#')
}

fn is_markdown_image_sugar(src: &str, call: &BlockCall) -> bool {
    call.name == "img" && !call.is_colon(src)
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
                collect_record_fields(src, collector, page.body);
            }
            Item::Roc(roc) => {
                collect_keyword(src, collector, roc.span, roc.body.start, "@roc");
            }
            Item::Render(render) => {
                collect_keyword(src, collector, render.span, render.args.start, "@render");
            }
            Item::Component(component) => {
                collect_component(src, collector, component);
            }
            Item::Fixture(fixture) => collect_fixture(src, collector, fixture),
            Item::Css(css) => collect_css(src, collector, css),
            Item::Context(context) => collect_context(src, collector, context),
            Item::Init(init) => collect_init(src, collector, init),
            Item::Live(live) => collect_live(src, collector, live),
            Item::View(view) => collect_view(src, collector, view),
            Item::Fragment(fragment) => collect_fragment(src, collector, fragment),
            Item::Command(command) => collect_command(src, collector, command),
            Item::Use(used) => {
                collect_keyword(src, collector, used.span, used.path_span.start, "@use");
            }
            Item::Template(item) => {
                collect_items(src, collector, std::slice::from_ref(item));
            }
            Item::Block(call) if is_atx_heading_sugar(src, call) => {}
            Item::Block(call) if is_markdown_image_sugar(src, call) => {}
            Item::Block(call) => {
                collect_docs_block(src, collector, call);
            }
        }
    }
}

fn collect_docs_block(src: &str, collector: &mut Vec<HighlightSpan>, call: &BlockCall) {
    if call.is_colon(src) {
        let colon_start = (call.name_span.start as usize).saturating_sub(1);
        if src.as_bytes().get(colon_start) == Some(&b':') {
            collector.push(HighlightSpan::new(
                Span::new(colon_start, call.name_span.end as usize),
                HighlightKind::Keyword,
                0,
                55,
            ));
        }
    }
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
        } else if val_str == "True" || val_str == "False" {
            collector.push(HighlightSpan::new(
                field.value,
                HighlightKind::Keyword,
                0,
                50,
            ));
        }
    }
    if let Some(BlockContent::End(section)) = &call.content {
        collector.push(HighlightSpan::new(
            section.marker.span,
            HighlightKind::Keyword,
            0,
            55,
        ));
        let marker = section.marker.span.of(src);
        if let Some(rest) = marker.strip_prefix(':')
            && let Some(dot) = rest.find('.')
        {
            let kind_start = section.marker.span.start as usize + 1;
            let kind_end = kind_start + dot;
            if kind_end <= section.marker.span.end as usize {
                collector.push(HighlightSpan::new(
                    Span::new(kind_start, kind_end),
                    HighlightKind::Type,
                    0,
                    56,
                ));
            }
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
                        enclosing_braces(text, page.body),
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
                if !render.args.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Expression,
                        RegionPurpose::Executable,
                        render.args,
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
            Item::Live(live) => {
                let live_id = builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    live.span,
                    Some(parent_id),
                    10,
                );
                if let Some(params) = live.params
                    && !params.is_empty()
                {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Params,
                        RegionPurpose::Executable,
                        params,
                        Some(live_id),
                        20,
                    );
                }
                if !live.body.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Body,
                        RegionPurpose::Executable,
                        live.body,
                        Some(live_id),
                        20,
                    );
                }
            }
            Item::View(view) => {
                add_handler_regions(builder, parent_id, view.span, view.params, view.body)
            }
            Item::Fragment(fragment) => add_handler_regions(
                builder,
                parent_id,
                fragment.span,
                fragment.params,
                fragment.body,
            ),
            Item::Command(command) => add_handler_regions(
                builder,
                parent_id,
                command.span,
                command.params,
                command.body,
            ),
            Item::Use(used) => {
                builder.add(
                    LanguageId::Rocdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    used.span,
                    Some(parent_id),
                    10,
                );
            }
            Item::Template(item) => {
                collect_template_item_regions(builder, std::slice::from_ref(item), parent_id);
            }
            Item::Block(call) if is_atx_heading_sugar(text, call) => {}
            Item::Block(call) if is_markdown_image_sugar(text, call) => {}
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

fn add_handler_regions(
    builder: &mut RegionBuilder,
    parent_id: usize,
    span: Span,
    params: Option<Span>,
    body: Span,
) {
    let host_id = builder.add(
        LanguageId::Rocdown,
        RegionContext::Body,
        RegionPurpose::HostStructure,
        span,
        Some(parent_id),
        10,
    );
    if let Some(params) = params
        && !params.is_empty()
    {
        builder.add(
            LanguageId::Roc,
            RegionContext::Params,
            RegionPurpose::Executable,
            params,
            Some(host_id),
            20,
        );
    }
    if !body.is_empty() {
        builder.add(
            LanguageId::Roc,
            RegionContext::Body,
            RegionPurpose::Executable,
            body,
            Some(host_id),
            20,
        );
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

fn enclosing_braces(src: &str, inner: Span) -> Span {
    let bytes = src.as_bytes();
    let mut start = inner.start as usize;
    while start > 0 && bytes[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    if start > 0 && bytes[start - 1] == b'{' {
        start -= 1;
    }
    let mut end = (inner.end as usize).min(src.len());
    while end < src.len() && bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    if end < src.len() && bytes[end] == b'}' {
        end += 1;
    }
    Span::new(start, end)
}

fn collect_record_fields(src: &str, collector: &mut Vec<HighlightSpan>, span: Span) {
    let mut cur = Cursor::at(src, span.start as usize);
    let end = (span.end as usize).min(src.len());
    while cur.pos < end && !cur.is_eof() {
        let before = cur.pos;
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        match cur.peek() {
            Some('{' | '}' | ',' | ':') => {
                let start = cur.pos;
                cur.bump();
                collector.push(HighlightSpan::new(
                    Span::new(start, cur.pos),
                    HighlightKind::Punctuation,
                    0,
                    50,
                ));
            }
            Some('"') => {
                let start = cur.pos;
                cur.bump();
                while let Some(ch) = cur.peek() {
                    if ch == '"' {
                        cur.bump();
                        break;
                    }
                    if ch == '\\' {
                        cur.bump();
                        cur.bump();
                        continue;
                    }
                    if ch == '\n' {
                        break;
                    }
                    cur.bump();
                }
                collector.push(HighlightSpan::new(
                    Span::new(start, cur.pos.min(end)),
                    HighlightKind::String,
                    0,
                    56,
                ));
            }
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                let Some(name_span) = cur.scan_ident() else {
                    cur.bump();
                    continue;
                };
                let saved = cur.pos;
                cur.skip_trivia();
                if cur.peek() == Some(':') && cur.pos <= end {
                    collector.push(HighlightSpan::new(
                        name_span,
                        HighlightKind::Property,
                        0,
                        56,
                    ));
                    cur.pos = saved;
                } else {
                    cur.pos = saved;
                    let name = name_span.of(src);
                    let kind = if name == "True" || name == "False" {
                        HighlightKind::Keyword
                    } else if name.starts_with(|ch: char| ch.is_ascii_uppercase()) {
                        HighlightKind::EnumMember
                    } else {
                        HighlightKind::Variable
                    };
                    collector.push(HighlightSpan::new(name_span, kind, 0, 50));
                }
            }
            _ => {
                cur.bump();
            }
        }
        if cur.pos <= before {
            cur.bump();
        }
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
