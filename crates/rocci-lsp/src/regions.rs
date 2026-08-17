use lsp_types::{Position, Range};
use rocci_template::{
    AttrValue, Document as RocciDocument, ModuleItem, PositionEncoding, SourceFile, Span,
    TemplateItem,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Roc,
    Css,
    Markdown,
    RocciTemplate,
    Html,
    Other(String),
}

impl Language {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "roc" => Self::Roc,
            "css" => Self::Css,
            "markdown" | "md" | "rocdown" => Self::Markdown,
            "rocci" => Self::RocciTemplate,
            "html" | "htm" => Self::Html,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Roc => "roc",
            Self::Css => "css",
            Self::Markdown => "markdown",
            Self::RocciTemplate => "rocci",
            Self::Html => "html",
            Self::Other(s) => s.as_str(),
        }
    }
}

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
    pub language: Language,
    pub context: RegionContext,
    pub purpose: RegionPurpose,
    pub span: Span,
    pub parent: Option<usize>,
    pub priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedRegion {
    pub id: usize,
    pub language: String,
    pub context: String,
    pub purpose: String,
    pub span: RegionSpan,
    pub range: Range,
    pub parent: Option<usize>,
    pub priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionTree {
    pub regions: Vec<Region>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegionValidationError {
    OutOfBounds {
        id: usize,
        span: Span,
        len: usize,
    },
    InvertedSpan {
        id: usize,
        span: Span,
    },
    ChildNotContained {
        child_id: usize,
        child_span: Span,
        parent_id: usize,
        parent_span: Span,
    },
    InvalidParentId {
        id: usize,
        parent_id: usize,
    },
    ParentCycle {
        id: usize,
    },
    ExecutableContainsDisplayFence {
        exec_id: usize,
        fence_id: usize,
    },
}

impl RegionTree {
    pub fn new(regions: Vec<Region>) -> Self {
        Self { regions }
    }

    pub fn find_at(&self, offset: usize) -> Option<&Region> {
        let mut best: Option<&Region> = None;
        for region in &self.regions {
            let start = region.span.start as usize;
            let end = region.span.end as usize;
            if start <= offset && offset < end {
                if let Some(prev) = best {
                    let prev_len = prev.span.len();
                    let cur_len = region.span.len();
                    if cur_len < prev_len
                        || (cur_len == prev_len && region.priority > prev.priority)
                        || (cur_len == prev_len
                            && region.priority == prev.priority
                            && region.id > prev.id)
                    {
                        best = Some(region);
                    }
                } else {
                    best = Some(region);
                }
            }
        }
        best
    }

    pub fn overlapping(&self, span: Span) -> Vec<&Region> {
        self.regions
            .iter()
            .filter(|region| region.span.start < span.end && span.start < region.span.end)
            .collect()
    }

    pub fn validate(&self, src_len: usize) -> Result<(), RegionValidationError> {
        let num_regions = self.regions.len();
        for (i, region) in self.regions.iter().enumerate() {
            if region.id != i {
                return Err(RegionValidationError::InvalidParentId {
                    id: region.id,
                    parent_id: i,
                });
            }
            let start = region.span.start as usize;
            let end = region.span.end as usize;
            if start > end {
                return Err(RegionValidationError::InvertedSpan {
                    id: region.id,
                    span: region.span,
                });
            }
            if end > src_len {
                return Err(RegionValidationError::OutOfBounds {
                    id: region.id,
                    span: region.span,
                    len: src_len,
                });
            }
            if let Some(parent_id) = region.parent {
                if parent_id >= num_regions {
                    return Err(RegionValidationError::InvalidParentId {
                        id: region.id,
                        parent_id,
                    });
                }
                let parent = &self.regions[parent_id];
                if region.span.start < parent.span.start || region.span.end > parent.span.end {
                    return Err(RegionValidationError::ChildNotContained {
                        child_id: region.id,
                        child_span: region.span,
                        parent_id,
                        parent_span: parent.span,
                    });
                }
                // Check cycle
                let mut curr = parent_id;
                let mut visited = 0;
                while let Some(next_parent) = self.regions[curr].parent {
                    if next_parent == region.id {
                        return Err(RegionValidationError::ParentCycle { id: region.id });
                    }
                    curr = next_parent;
                    visited += 1;
                    if visited > num_regions {
                        return Err(RegionValidationError::ParentCycle { id: region.id });
                    }
                }
            }
        }

        // Enforce invariant: executable regions never include display-only fences
        for region in &self.regions {
            if region.purpose == RegionPurpose::Executable {
                for fence in &self.regions {
                    if fence.purpose == RegionPurpose::DisplayOnly
                        && fence.context == RegionContext::Fence
                        && region.span.start <= fence.span.start
                        && fence.span.end <= region.span.end
                        && !fence.span.is_empty()
                    {
                        return Err(RegionValidationError::ExecutableContainsDisplayFence {
                            exec_id: region.id,
                            fence_id: fence.id,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

struct RegionBuilder {
    regions: Vec<Region>,
}

impl RegionBuilder {
    fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    fn add(
        &mut self,
        language: Language,
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

pub fn extract_rocci_regions(_name: &str, text: &str, doc: &RocciDocument) -> RegionTree {
    let mut builder = RegionBuilder::new();
    let root = builder.add(
        Language::RocciTemplate,
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
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    c.span,
                    Some(root),
                    10,
                );
                if !c.params.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    f.span,
                    Some(root),
                    10,
                );
                if !f.value.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    css.span,
                    Some(root),
                    10,
                );
                if !css.body.is_empty() {
                    builder.add(
                        Language::Css,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    ctx.span,
                    Some(root),
                    10,
                );
                if !ctx.ty.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    init.span,
                    Some(root),
                    10,
                );
                if !init.body.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
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
                        Language::Roc,
                        RegionContext::Params,
                        RegionPurpose::Executable,
                        params,
                        Some(on_id),
                        20,
                    );
                }
                if !on.body.is_empty() {
                    builder.add(
                        Language::Roc,
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

pub fn extract_rocdown_regions(
    _name: &str,
    text: &str,
    doc: &rocci_rocdown::Document,
    _headings: &[rocci_rocdown::HeadingInfo],
) -> RegionTree {
    let mut builder = RegionBuilder::new();
    let root = builder.add(
        Language::Markdown,
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
                collect_md_node(builder, md_node, parent_id);
            }
            rocci_rocdown::Item::Page(page) => {
                let page_id = builder.add(
                    Language::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    page.span,
                    Some(parent_id),
                    10,
                );
                if !page.body.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    roc.span,
                    Some(parent_id),
                    10,
                );
                if !roc.body.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::Markdown,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    render.span,
                    Some(parent_id),
                    10,
                );
                if !render.expr.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    c.span,
                    Some(parent_id),
                    10,
                );
                if !c.params.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    f.span,
                    Some(parent_id),
                    10,
                );
                if !f.value.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    css.span,
                    Some(parent_id),
                    10,
                );
                if !css.body.is_empty() {
                    builder.add(
                        Language::Css,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    ctx.span,
                    Some(parent_id),
                    10,
                );
                if !ctx.ty.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    init.span,
                    Some(parent_id),
                    10,
                );
                if !init.body.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
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
                        Language::Roc,
                        RegionContext::Params,
                        RegionPurpose::Executable,
                        params,
                        Some(on_id),
                        20,
                    );
                }
                if !on.body.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::Markdown,
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
                    Language::Markdown,
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

fn collect_md_node(builder: &mut RegionBuilder, node: &rocci_rocdown::MdNode, parent_id: usize) {
    match node {
        rocci_rocdown::MdNode::CodeBlock { info, span, .. } => {
            if !span.is_empty() {
                let lang = Language::from_str(info);
                builder.add(
                    lang,
                    RegionContext::Fence,
                    RegionPurpose::DisplayOnly,
                    *span,
                    Some(parent_id),
                    20,
                );
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
                collect_md_node(builder, child, parent_id);
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

fn collect_template_items(builder: &mut RegionBuilder, items: &[TemplateItem], parent_id: usize) {
    for item in items {
        match item {
            TemplateItem::Element(el) => {
                let el_id = builder.add(
                    Language::RocciTemplate,
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
                                    Language::Roc,
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
                                    Language::Roc,
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
                    Language::RocciTemplate,
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
                                    Language::Roc,
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
                                    Language::Roc,
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
                    Language::RocciTemplate,
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
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.condition.is_empty() {
                    builder.add(
                        Language::Roc,
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
                            Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.collection.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.scrutinee.is_empty() {
                    builder.add(
                        Language::Roc,
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
                            Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    dir.span,
                    Some(parent_id),
                    10,
                );
                if !dir.expr.is_empty() {
                    builder.add(
                        Language::Roc,
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
                    Language::RocciTemplate,
                    RegionContext::Body,
                    RegionPurpose::HostStructure,
                    css.span,
                    Some(parent_id),
                    10,
                );
                if !css.body.is_empty() {
                    builder.add(
                        Language::Css,
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

pub fn inspect_regions(
    source: SourceFile<'_>,
    tree: &RegionTree,
    encoding: PositionEncoding,
) -> Vec<InspectedRegion> {
    tree.regions
        .iter()
        .map(|r| {
            let ((start_line, start_col), (end_line, end_col)) = source.range(r.span, encoding);
            InspectedRegion {
                id: r.id,
                language: r.language.as_str().to_string(),
                context: r.context.as_str().to_string(),
                purpose: r.purpose.as_str().to_string(),
                span: r.span.into(),
                range: Range {
                    start: Position::new(start_line, start_col),
                    end: Position::new(end_line, end_col),
                },
                parent: r.parent,
                priority: r.priority,
            }
        })
        .collect()
}

pub fn executable_roc_ranges(
    source: SourceFile<'_>,
    tree: &RegionTree,
    encoding: PositionEncoding,
) -> Vec<Range> {
    tree.regions
        .iter()
        .filter(|r| r.language == Language::Roc && r.purpose == RegionPurpose::Executable)
        .map(|r| {
            let ((start_line, start_col), (end_line, end_col)) = source.range(r.span, encoding);
            Range {
                start: Position::new(start_line, start_col),
                end: Position::new(end_line, end_col),
            }
        })
        .collect()
}

pub fn css_ranges(
    source: SourceFile<'_>,
    tree: &RegionTree,
    encoding: PositionEncoding,
) -> Vec<Range> {
    tree.regions
        .iter()
        .filter(|r| r.language == Language::Css)
        .map(|r| {
            let ((start_line, start_col), (end_line, end_col)) = source.range(r.span, encoding);
            Range {
                start: Position::new(start_line, start_col),
                end: Position::new(end_line, end_col),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_well_formed_tree() {
        let tree = RegionTree::new(vec![
            Region {
                id: 0,
                language: Language::RocciTemplate,
                context: RegionContext::Document,
                purpose: RegionPurpose::HostStructure,
                span: Span::new(0, 100),
                parent: None,
                priority: 0,
            },
            Region {
                id: 1,
                language: Language::RocciTemplate,
                context: RegionContext::Body,
                purpose: RegionPurpose::HostStructure,
                span: Span::new(10, 50),
                parent: Some(0),
                priority: 10,
            },
            Region {
                id: 2,
                language: Language::Roc,
                context: RegionContext::Expression,
                purpose: RegionPurpose::Executable,
                span: Span::new(20, 30),
                parent: Some(1),
                priority: 20,
            },
        ]);
        assert_eq!(tree.validate(100), Ok(()));
    }

    #[test]
    fn validate_out_of_bounds() {
        let tree = RegionTree::new(vec![Region {
            id: 0,
            language: Language::RocciTemplate,
            context: RegionContext::Document,
            purpose: RegionPurpose::HostStructure,
            span: Span::new(0, 150),
            parent: None,
            priority: 0,
        }]);
        assert!(matches!(
            tree.validate(100),
            Err(RegionValidationError::OutOfBounds { .. })
        ));
    }

    #[test]
    fn validate_inverted_span() {
        let tree = RegionTree::new(vec![Region {
            id: 0,
            language: Language::RocciTemplate,
            context: RegionContext::Document,
            purpose: RegionPurpose::HostStructure,
            span: Span::new(50, 20),
            parent: None,
            priority: 0,
        }]);
        assert!(matches!(
            tree.validate(100),
            Err(RegionValidationError::InvertedSpan { .. })
        ));
    }

    #[test]
    fn validate_child_not_contained() {
        let tree = RegionTree::new(vec![
            Region {
                id: 0,
                language: Language::RocciTemplate,
                context: RegionContext::Document,
                purpose: RegionPurpose::HostStructure,
                span: Span::new(10, 50),
                parent: None,
                priority: 0,
            },
            Region {
                id: 1,
                language: Language::Roc,
                context: RegionContext::Expression,
                purpose: RegionPurpose::Executable,
                span: Span::new(5, 30),
                parent: Some(0),
                priority: 20,
            },
        ]);
        assert!(matches!(
            tree.validate(100),
            Err(RegionValidationError::ChildNotContained { .. })
        ));
    }

    #[test]
    fn validate_executable_contains_fence() {
        let tree = RegionTree::new(vec![
            Region {
                id: 0,
                language: Language::Markdown,
                context: RegionContext::Document,
                purpose: RegionPurpose::HostStructure,
                span: Span::new(0, 100),
                parent: None,
                priority: 0,
            },
            Region {
                id: 1,
                language: Language::Roc,
                context: RegionContext::Module,
                purpose: RegionPurpose::Executable,
                span: Span::new(10, 80),
                parent: Some(0),
                priority: 10,
            },
            Region {
                id: 2,
                language: Language::Roc,
                context: RegionContext::Fence,
                purpose: RegionPurpose::DisplayOnly,
                span: Span::new(20, 50),
                parent: Some(1),
                priority: 20,
            },
        ]);
        assert!(matches!(
            tree.validate(100),
            Err(RegionValidationError::ExecutableContainsDisplayFence { .. })
        ));
    }

    #[test]
    fn find_at_returns_deepest_matching_region() {
        let tree = RegionTree::new(vec![
            Region {
                id: 0,
                language: Language::RocciTemplate,
                context: RegionContext::Document,
                purpose: RegionPurpose::HostStructure,
                span: Span::new(0, 100),
                parent: None,
                priority: 0,
            },
            Region {
                id: 1,
                language: Language::RocciTemplate,
                context: RegionContext::Body,
                purpose: RegionPurpose::HostStructure,
                span: Span::new(10, 60),
                parent: Some(0),
                priority: 10,
            },
            Region {
                id: 2,
                language: Language::Roc,
                context: RegionContext::Expression,
                purpose: RegionPurpose::Executable,
                span: Span::new(20, 30),
                parent: Some(1),
                priority: 20,
            },
        ]);
        assert_eq!(tree.find_at(5).map(|r| r.id), Some(0));
        assert_eq!(tree.find_at(15).map(|r| r.id), Some(1));
        assert_eq!(tree.find_at(25).map(|r| r.id), Some(2));
        assert_eq!(tree.find_at(40).map(|r| r.id), Some(1));
        assert_eq!(tree.find_at(80).map(|r| r.id), Some(0));
        assert_eq!(tree.find_at(150), None);
    }
}
