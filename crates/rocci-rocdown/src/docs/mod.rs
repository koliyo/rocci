use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rocci_template::SourceFile;
use serde::Serialize;

use crate::ast::{Document, MdNode};
use crate::catalog::CatalogDiagnostic;
use crate::img::StaticImage;

mod examples;
mod fields;
mod includes;
mod render;
mod tree;
mod validate;

pub use examples::{ExampleTestOptions, run_examples};
pub use fields::{DocsField, docs_fields_from_params, field_bool, field_string, field_strings};
pub use includes::{extract_lines, extract_region, include_path_error, resolve_include_path};
#[allow(unused_imports)]
pub use render::collect_article_interpolation_gates;
pub(crate) use render::rewrite_urls;
pub use render::{
    markdown_fragment, markdown_fragment_gated, plan_segments, plan_segments_with_islands,
    render_article, render_article_gated, search_text,
};
use tree::nodes_from_items;
pub use tree::{
    collect_headings, collect_images, collect_kinds, collect_links, fill_link_cards,
    rewrite_resolved_links,
};
pub use validate::validate_resolved;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageDocs {
    pub article: Vec<ArticleNode>,
    pub examples: Vec<ExampleRecord>,
    pub includes: Vec<IncludeOrigin>,
    pub snippet_paths: Vec<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleNode {
    Markdown(MdNode),
    Block(DocsNode),
    Image(StaticImage),
    Island,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsNode {
    pub kind: String,
    pub attrs: DocsAttrs,
    pub children: Vec<ArticleNode>,
    pub origin: Option<IncludeOrigin>,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocsAttrs {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub label: Option<String>,
    pub term: Option<String>,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub credit: Option<String>,
    pub tone: Option<String>,
    pub page: Option<String>,
    pub href: Option<String>,
    pub group: Option<String>,
    pub tab_kind: Option<String>,
    pub id: Option<String>,
    pub path: Option<String>,
    pub region: Option<String>,
    pub language: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub test: Vec<String>,
    pub expect: Option<String>,
    pub open: bool,
    pub verify: bool,
    pub allow_network: bool,
    pub unknown: Vec<String>,
    pub extra: BTreeMap<String, String>,
    pub extra_bool: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct IncludeOrigin {
    pub source_path: String,
    pub region: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ExampleRecord {
    pub id: String,
    pub language: String,
    pub path: Option<String>,
    pub region: Option<String>,
    pub test: Vec<String>,
    pub expect: Option<String>,
    pub allow_network: bool,
    pub origin: IncludeOrigin,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedNode {
    Html { path: String },
    Widget(PlannedWidget),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedWidget {
    pub kind: String,
    pub component: String,
    pub props: Vec<PlannedProp>,
    pub children: Vec<PlannedNode>,
    pub paint_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedProp {
    Str { name: String, value: String },
    Bool { name: String, value: bool },
}

impl PlannedNode {
    pub fn widget_kind(&self) -> Option<&str> {
        match self {
            Self::Widget(widget) => Some(widget.kind.as_str()),
            Self::Html { .. } => None,
        }
    }
}

impl PlannedWidget {
    pub fn str_prop(&self, name: &str) -> Option<&str> {
        self.props.iter().find_map(|prop| match prop {
            PlannedProp::Str {
                name: prop_name,
                value,
            } if prop_name == name => Some(value.as_str()),
            _ => None,
        })
    }

    pub fn bool_prop(&self, name: &str) -> Option<bool> {
        self.props.iter().find_map(|prop| match prop {
            PlannedProp::Bool {
                name: prop_name,
                value,
            } if prop_name == name => Some(*value),
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IncludeOptions<'a> {
    pub root: &'a Path,
    pub snippet_roots: &'a [PathBuf],
}

pub(crate) struct BuildCtx<'a> {
    pub(crate) source: SourceFile<'a>,
    pub(crate) source_path: &'a str,
    pub(crate) includes: IncludeOptions<'a>,
    pub(crate) stack: Vec<PathBuf>,
    pub(crate) diagnostics: &'a mut Vec<CatalogDiagnostic>,
    pub(crate) examples: Vec<ExampleRecord>,
    pub(crate) origins: Vec<IncludeOrigin>,
    pub(crate) snippet_paths: BTreeSet<String>,
}

pub fn load_page_docs(
    source: SourceFile<'_>,
    document: &Document,
    source_path: &str,
    includes: IncludeOptions<'_>,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> PageDocs {
    let mut ctx = BuildCtx {
        source,
        source_path,
        includes,
        stack: Vec::new(),
        diagnostics,
        examples: Vec::new(),
        origins: Vec::new(),
        snippet_paths: BTreeSet::new(),
    };
    let article = nodes_from_items(&mut ctx, &document.items, None);
    PageDocs {
        article,
        examples: ctx.examples,
        includes: ctx.origins,
        snippet_paths: ctx.snippet_paths.into_iter().collect(),
    }
}

pub(crate) fn line_number(src: &str, offset: usize) -> u32 {
    src.get(..offset.min(src.len()))
        .unwrap_or("")
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1
}

#[cfg(test)]
mod tests;
