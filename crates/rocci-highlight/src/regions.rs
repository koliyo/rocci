use rocci_template::{
    AttrValue, Document as RocciDocument, ModuleItem, SourceFile, Span, TemplateItem,
};
use serde::{Deserialize, Serialize};

use crate::language::LanguageId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegionContext {
    Document,
    Module,
    Expression,
    Pattern,
    Type,
    Params,
    Stylesheet,
    Body,
    Fence,
}

impl RegionContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Module => "module",
            Self::Expression => "expression",
            Self::Pattern => "pattern",
            Self::Type => "type",
            Self::Params => "params",
            Self::Stylesheet => "stylesheet",
            Self::Body => "body",
            Self::Fence => "fence",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegionPurpose {
    Executable,
    HostStructure,
    DisplayOnly,
    Metadata,
}

impl RegionPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::HostStructure => "hostStructure",
            Self::DisplayOnly => "displayOnly",
            Self::Metadata => "metadata",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionSpan {
    pub start: u32,
    pub end: u32,
}

impl From<Span> for RegionSpan {
    fn from(s: Span) -> Self {
        Self {
            start: s.start,
            end: s.end,
        }
    }
}

impl From<RegionSpan> for Span {
    fn from(s: RegionSpan) -> Self {
        Span::new(s.start as usize, s.end as usize)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub id: usize,
    pub language: LanguageId,
    pub context: RegionContext,
    pub purpose: RegionPurpose,
    pub span: Span,
    pub parent: Option<usize>,
    pub priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionTree {
    pub regions: Vec<Region>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegionValidationError {
    OutOfBounds { id: usize, span: Span, len: usize },
    NotMonotonicOrder { prev_id: usize, curr_id: usize },
    ChildExceedsParent { child_id: usize, parent_id: usize },
    InvalidParent { child_id: usize, parent_id: usize },
}

impl RegionTree {
    pub fn new(regions: Vec<Region>) -> Self {
        Self { regions }
    }

    pub fn find_at(&self, offset: usize) -> Option<&Region> {
        let pos = offset as u32;
        let mut best: Option<&Region> = None;
        for region in &self.regions {
            if region.span.start <= pos && pos < region.span.end {
                match best {
                    None => best = Some(region),
                    Some(curr) => {
                        let curr_len = curr.span.end.saturating_sub(curr.span.start);
                        let reg_len = region.span.end.saturating_sub(region.span.start);
                        if reg_len < curr_len
                            || (reg_len == curr_len && region.priority >= curr.priority)
                        {
                            best = Some(region);
                        }
                    }
                }
            }
        }
        best
    }

    pub fn overlapping(&self, span: Span) -> Vec<&Region> {
        self.regions
            .iter()
            .filter(|r| r.span.start < span.end && r.span.end > span.start)
            .collect()
    }

    pub fn validate(&self, src_len: usize) -> Result<(), RegionValidationError> {
        for (i, region) in self.regions.iter().enumerate() {
            if (region.span.end as usize) > src_len || region.span.start > region.span.end {
                return Err(RegionValidationError::OutOfBounds {
                    id: region.id,
                    span: region.span,
                    len: src_len,
                });
            }
            if let Some(parent_id) = region.parent {
                let Some(parent) = self.regions.get(parent_id) else {
                    return Err(RegionValidationError::InvalidParent {
                        child_id: region.id,
                        parent_id,
                    });
                };
                if region.span.start < parent.span.start || region.span.end > parent.span.end {
                    return Err(RegionValidationError::ChildExceedsParent {
                        child_id: region.id,
                        parent_id,
                    });
                }
            }
            if i > 0 && region.id <= self.regions[i - 1].id {
                return Err(RegionValidationError::NotMonotonicOrder {
                    prev_id: self.regions[i - 1].id,
                    curr_id: region.id,
                });
            }
        }
        Ok(())
    }
}

pub struct RegionBuilder {
    pub regions: Vec<Region>,
}

impl RegionBuilder {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        language: LanguageId,
        context: RegionContext,
        purpose: RegionPurpose,
        span: Span,
        parent: Option<usize>,
        priority: u32,
    ) -> usize {
        let id = self.regions.len();
        self.regions.push(Region {
            id,
            language,
            context,
            purpose,
            span,
            parent,
            priority,
        });
        id
    }
}

impl Default for RegionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn extract_rocci_regions(_name: &str, text: &str, doc: &RocciDocument) -> RegionTree {
    let mut builder = RegionBuilder::new();
    let root = builder.add(
        LanguageId::Rocci,
        RegionContext::Document,
        RegionPurpose::HostStructure,
        Span::new(0, text.len()),
        None,
        0,
    );

    for item in &doc.items {
        match item {
            ModuleItem::Roc { span } => {
                if !span.is_empty() {
                    builder.add(
                        LanguageId::Roc,
                        RegionContext::Module,
                        RegionPurpose::Executable,
                        *span,
                        Some(root),
                        20,
                    );
                }
            }
            ModuleItem::Component(c) => {
                let comp_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    c.span,
                    Some(root),
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
                collect_template_items(&mut builder, &c.body.items, comp_id);
            }
            ModuleItem::Fixture(f) => {
                let fix_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    f.span,
                    Some(root),
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
            ModuleItem::Css(css) => {
                let css_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    css.span,
                    Some(root),
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
            ModuleItem::Context(ctx) => {
                let ctx_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    ctx.span,
                    Some(root),
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
            ModuleItem::Init(init) => {
                let init_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    init.span,
                    Some(root),
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
            ModuleItem::On(on) => {
                let on_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    on.span,
                    Some(root),
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
        }
    }

    RegionTree::new(builder.regions)
}

fn collect_template_items(builder: &mut RegionBuilder, items: &[TemplateItem], parent_id: usize) {
    for item in items {
        match item {
            TemplateItem::Element(el) => {
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
                        AttrValue::Expr { expr } => {
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
                        AttrValue::Action { args, .. } => {
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
                        AttrValue::Static { .. } | AttrValue::Boolean => {}
                    }
                }
                collect_template_items(builder, &el.children, el_id);
            }
            TemplateItem::ComponentCall(call) => {
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
                        AttrValue::Expr { expr } => {
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
                        AttrValue::Action { args, .. } => {
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
                        AttrValue::Static { .. } | AttrValue::Boolean => {}
                    }
                }
                if let Some(children) = &call.children {
                    collect_template_items(builder, children, call_id);
                }
            }
            TemplateItem::Fragment(frag) => {
                let frag_id = builder.add(
                    LanguageId::Rocci,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    frag.span,
                    Some(parent_id),
                    10,
                );
                collect_template_items(builder, &frag.children, frag_id);
            }
            TemplateItem::Interpolation(interp) => {
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
            TemplateItem::If(dir) => {
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
                collect_template_items(builder, &dir.then_body.items, if_id);
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
                    collect_template_items(builder, &body.items, if_id);
                }
                if let Some(body) = &dir.else_body {
                    collect_template_items(builder, &body.items, if_id);
                }
            }
            TemplateItem::For(dir) => {
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
                collect_template_items(builder, &dir.body.items, for_id);
            }
            TemplateItem::Match(dir) => {
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
                    collect_template_items(builder, std::slice::from_ref(&*arm.value), match_id);
                }
            }
            TemplateItem::Let(dir) => {
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
            TemplateItem::Css(css) => {
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
            TemplateItem::Text(_) => {}
        }
    }
}

pub fn extract_rocdown_regions(
    _name: &str,
    text: &str,
    doc: &rocci_rocdown::Document,
    _headings: &[rocci_rocdown::HeadingInfo],
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
    items: &[rocci_rocdown::Item],
    parent_id: usize,
) {
    for item in items {
        match item {
            rocci_rocdown::Item::Markdown(md_node) => {
                collect_md_node(builder, text, md_node, parent_id);
            }
            rocci_rocdown::Item::Page(page) => {
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
            rocci_rocdown::Item::Roc(roc) => {
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
            rocci_rocdown::Item::Render(render) => {
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
            rocci_rocdown::Item::Component(c) => {
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
                collect_template_items(builder, &c.body.items, comp_id);
            }
            rocci_rocdown::Item::Fixture(f) => {
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
            rocci_rocdown::Item::Css(css) => {
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
            rocci_rocdown::Item::Context(ctx) => {
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
            rocci_rocdown::Item::Init(init) => {
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
            rocci_rocdown::Item::On(on) => {
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
            rocci_rocdown::Item::Template(item) => {
                collect_template_items(builder, std::slice::from_ref(item), parent_id);
            }
            rocci_rocdown::Item::Docs(docs) => {
                let docs_id = builder.add(
                    LanguageId::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    docs.span,
                    Some(parent_id),
                    10,
                );
                let (_fields, content) = rocci_rocdown::split_docs_body(text, docs.body);
                if !content.is_empty() && (content.start as usize) < text.len() {
                    let source = SourceFile::new("docs", text);
                    let parsed = rocci_rocdown::parse_fragment(source, content, false);
                    collect_rocdown_items(builder, text, &parsed.document.items, docs_id);
                }
            }
            rocci_rocdown::Item::Img(img) => {
                builder.add(
                    LanguageId::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    img.span,
                    Some(parent_id),
                    10,
                );
            }
        }
    }
}

fn collect_md_node(
    builder: &mut RegionBuilder,
    text: &str,
    node: &rocci_rocdown::MdNode,
    parent_id: usize,
) {
    match node {
        rocci_rocdown::MdNode::CodeBlock { info, span, .. } => {
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
        rocci_rocdown::MdNode::Heading { children, .. }
        | rocci_rocdown::MdNode::Paragraph { children, .. }
        | rocci_rocdown::MdNode::BlockQuote { children, .. }
        | rocci_rocdown::MdNode::List { children, .. }
        | rocci_rocdown::MdNode::Item { children, .. }
        | rocci_rocdown::MdNode::TaskItem { children, .. }
        | rocci_rocdown::MdNode::Table { children, .. }
        | rocci_rocdown::MdNode::TableRow { children, .. }
        | rocci_rocdown::MdNode::TableCell { children, .. }
        | rocci_rocdown::MdNode::Emph { children, .. }
        | rocci_rocdown::MdNode::Strong { children, .. }
        | rocci_rocdown::MdNode::Strikethrough { children, .. }
        | rocci_rocdown::MdNode::FootnoteDefinition { children, .. }
        | rocci_rocdown::MdNode::Link { children, .. } => {
            for child in children {
                collect_md_node(builder, text, child, parent_id);
            }
        }
        rocci_rocdown::MdNode::ThematicBreak { .. }
        | rocci_rocdown::MdNode::Text { .. }
        | rocci_rocdown::MdNode::SoftBreak { .. }
        | rocci_rocdown::MdNode::LineBreak { .. }
        | rocci_rocdown::MdNode::Code { .. }
        | rocci_rocdown::MdNode::FootnoteReference { .. }
        | rocci_rocdown::MdNode::Image { .. }
        | rocci_rocdown::MdNode::RawHtml { .. } => {}
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

pub fn executable_roc_ranges(tree: &RegionTree) -> Vec<Span> {
    tree.regions
        .iter()
        .filter(|r| r.language == LanguageId::Roc && r.purpose == RegionPurpose::Executable)
        .map(|r| r.span)
        .collect()
}

pub fn css_ranges(tree: &RegionTree) -> Vec<Span> {
    tree.regions
        .iter()
        .filter(|r| r.language == LanguageId::Css)
        .map(|r| r.span)
        .collect()
}
