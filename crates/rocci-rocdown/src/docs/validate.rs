use std::collections::BTreeSet;

use crate::ast::MdNode;
use crate::catalog::{CatalogDiagnostic, ResolvedPage};
use crate::registry::{self, ChildPredicate, KindSpec};

use super::fields::{attr_nonempty, attr_some};
use super::{ArticleNode, BuildCtx, DocsNode};

pub fn validate_resolved(pages: &[ResolvedPage], diagnostics: &mut Vec<CatalogDiagnostic>) {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for page in pages {
        ids.insert(page.id.clone());
        ids.insert(page.route.clone());
        for alias in &page.aliases {
            ids.insert(alias.clone());
        }
        if let Some(stripped) = page.id.strip_prefix("docs/") {
            ids.insert(stripped.to_string());
            ids.insert(format!("/{stripped}/"));
        }
    }
    for page in pages {
        validate_link_cards(&page.article, &page.source_path, &ids, diagnostics);
    }
}

fn validate_link_cards(
    nodes: &[ArticleNode],
    path: &str,
    ids: &BTreeSet<String>,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) {
    for node in nodes {
        if let ArticleNode::Block(docs) = node {
            if docs.kind == "link-card"
                && let Some(page) = &docs.attrs.page
                && !ids.contains(page.as_str())
                && !ids.contains(page.strip_prefix("docs/").unwrap_or(page))
                && !ids.contains(&format!("docs/{page}"))
                && !ids.contains(page.strip_prefix('/').unwrap_or(page))
            {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2101",
                    path,
                    format!(
                        "line {}: `:link-card` targets unknown page `{page}`",
                        docs.line
                    ),
                ));
            }
            validate_link_cards(&docs.children, path, ids, diagnostics);
        }
    }
}

fn has_docs_child(node: &DocsNode, kind: &str) -> bool {
    node.children
        .iter()
        .any(|child| matches!(child, ArticleNode::Block(docs) if docs.kind == kind))
}

fn kind_error(ctx: &mut BuildCtx<'_>, spec: &KindSpec, line: u32, message: String) {
    ctx.diagnostics.push(CatalogDiagnostic::error(
        spec.diagnostic_code,
        ctx.source_path,
        format!("line {line}: {message}"),
    ));
}

fn validate_registry_shape(ctx: &mut BuildCtx<'_>, node: &DocsNode, parent_kind: Option<&str>) {
    let Some(spec) = registry::lookup(&node.kind) else {
        return;
    };
    let line = node.line;
    if !registry::parent_allowed(spec, parent_kind) {
        let parent = spec.parents[0];
        kind_error(
            ctx,
            spec,
            line,
            format!("`:{}` is only valid inside `:{parent}`", spec.name),
        );
    }
    let check_required = spec.parents.is_empty() || registry::parent_allowed(spec, parent_kind);
    if check_required && spec.diagnostic_code == "RD2402" {
        for field in spec.required_fields {
            if !attr_nonempty(&node.attrs, field) {
                kind_error(
                    ctx,
                    spec,
                    line,
                    format!("`:{}` requires `{field}`", spec.name),
                );
            }
        }
        for group in spec.required_one_of {
            if !group.iter().any(|field| attr_some(&node.attrs, field)) {
                let joined = group.join("` or `");
                kind_error(
                    ctx,
                    spec,
                    line,
                    format!("`:{}` requires `{joined}`", spec.name),
                );
            }
        }
    }
    validate_children(ctx, node, spec);
}

pub(crate) fn validate_model(ctx: &mut BuildCtx<'_>, node: &DocsNode, parent_kind: Option<&str>) {
    let path = ctx.source_path;
    let line = node.line;
    if !node.attrs.unknown.is_empty() {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2402",
            path,
            format!(
                "line {line}: unknown `:{}` field `{}`",
                node.kind,
                node.attrs.unknown.join(", ")
            ),
        ));
    }
    if let Some(spec) = registry::lookup(&node.kind) {
        let known: Vec<&str> = spec
            .required_fields
            .iter()
            .copied()
            .chain(spec.optional_fields.iter().copied())
            .chain(spec.paint_fields().iter().map(|field| field.attr))
            .collect();
        let extras: Vec<&String> = node
            .attrs
            .extra
            .keys()
            .filter(|key| !known.contains(&key.as_str()))
            .collect();
        if !extras.is_empty() {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2402",
                path,
                format!(
                    "line {line}: unknown `:{}` field `{}`",
                    node.kind,
                    extras
                        .iter()
                        .map(|key| key.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }
    validate_registry_shape(ctx, node, parent_kind);
    match node.kind.as_str() {
        "badge" => {
            if let Some(tone) = &node.attrs.tone
                && !registry::BADGE_TONE_VALUES.contains(&tone.as_str())
            {
                malformed(ctx, line, &format!("invalid badge tone `{tone}`"));
            }
        }
        "compatibility" => {
            if !contains_table(&node.children) {
                malformed(ctx, line, "`:compatibility` body must be a table");
            }
        }
        "file-tree" => {
            if !node.children.iter().any(|child| {
                matches!(
                    child,
                    ArticleNode::Markdown(MdNode::List { ordered: false, .. })
                )
            }) {
                malformed(ctx, line, "`:file-tree` body must be an unordered list");
            }
        }
        "tabs" => validate_tabs(ctx, node),
        "example" => validate_example(ctx, node),
        _ => {}
    }
}

fn validate_children(ctx: &mut BuildCtx<'_>, node: &DocsNode, spec: &KindSpec) {
    let line = node.line;
    for kind in spec.forbids {
        if has_docs_child(node, kind) {
            malformed(
                ctx,
                line,
                &format!("`:{}` cannot contain `:{kind}`", spec.name),
            );
        }
    }
    match spec.child_predicate {
        ChildPredicate::StepsXorList => validate_steps_xor_list(ctx, node),
        ChildPredicate::FigureOneImage => {
            if count_images(&node.children) != 1 {
                malformed(ctx, line, "`:figure` body must contain exactly one image");
            }
        }
        ChildPredicate::None => validate_accepted_children(ctx, node, spec),
    }
    for child_kind in spec.requires {
        if !has_docs_child(node, child_kind) {
            kind_error(
                ctx,
                spec,
                line,
                format!("`:{}` requires `:{child_kind}` children", spec.name),
            );
        }
    }
}

fn validate_accepted_children(ctx: &mut BuildCtx<'_>, node: &DocsNode, spec: &KindSpec) {
    for child in &node.children {
        match child {
            ArticleNode::Block(docs) => {
                if !spec.accepts_block_child(&docs.kind) {
                    malformed(
                        ctx,
                        docs.line,
                        &format!("`:{}` cannot contain `:{}`", spec.name, docs.kind),
                    );
                }
            }
            ArticleNode::Markdown(md) => {
                if spec.rejects_markdown() && !md.is_whitespace_only_paragraph() {
                    malformed(
                        ctx,
                        node.line,
                        &format!("`:{}` cannot contain Markdown", spec.name),
                    );
                }
            }
            ArticleNode::Image(_) => {
                if !spec.accepts_block_child("img") {
                    malformed(
                        ctx,
                        node.line,
                        &format!("`:{}` cannot contain `:img`", spec.name),
                    );
                }
            }
            ArticleNode::Island => {
                if spec.rejects_markdown() || !spec.accepts.is_empty() {
                    malformed(
                        ctx,
                        node.line,
                        &format!("`:{}` cannot contain an island", spec.name),
                    );
                }
            }
        }
    }
}

fn validate_steps_xor_list(ctx: &mut BuildCtx<'_>, node: &DocsNode) {
    let line = node.line;
    let has_step = has_docs_child(node, "step");
    let has_list = node.children.iter().any(|child| {
        matches!(
            child,
            ArticleNode::Markdown(MdNode::List { ordered: true, .. })
        )
    });
    let extra = node.children.iter().any(|child| match child {
        ArticleNode::Block(docs) => docs.kind != "step",
        ArticleNode::Markdown(MdNode::List { ordered: true, .. }) => false,
        ArticleNode::Markdown(md) => !md.is_whitespace_only_paragraph(),
        ArticleNode::Image(_) | ArticleNode::Island => true,
    });
    if has_step && has_list {
        malformed(ctx, line, "`:steps` cannot mix a list with `:step`");
    } else if extra {
        malformed(
            ctx,
            line,
            "`:steps` body must be an ordered list or `:step` children",
        );
    } else if !has_step && !has_list {
        malformed(ctx, line, "`:steps` requires steps");
    }
}

fn validate_tabs(ctx: &mut BuildCtx<'_>, node: &DocsNode) {
    let Some(spec) = registry::lookup("tabs") else {
        return;
    };
    let tab_spec = registry::lookup("tab");
    let path = ctx.source_path;
    let line = node.line;
    for field in spec.required_fields {
        if *field == "kind" {
            continue;
        }
        if !attr_nonempty(&node.attrs, field) {
            kind_error(ctx, spec, line, format!("`:tabs` requires `{field}`"));
        }
    }
    let Some(kind) = node.attrs.tab_kind.as_deref() else {
        kind_error(ctx, spec, line, "`:tabs` requires `kind`".to_string());
        return;
    };
    if !registry::TAB_KIND_VALUES.contains(&kind) {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2405",
            path,
            format!("line {line}: `:tabs` kind must be language, platform, or tool"),
        ));
    }
    let tabs: Vec<_> = node
        .children
        .iter()
        .filter_map(|child| match child {
            ArticleNode::Block(docs) if docs.kind == "tab" => Some(docs),
            _ => None,
        })
        .collect();
    let mut seen = BTreeSet::new();
    for tab in tabs {
        if let Some(tab_spec) = tab_spec {
            for field in tab_spec.required_fields {
                if !attr_nonempty(&tab.attrs, field) {
                    kind_error(
                        ctx,
                        tab_spec,
                        tab.line,
                        format!("`:tab` requires `{field}`"),
                    );
                    continue;
                }
                if *field == "id" {
                    let id = tab.attrs.id.as_deref().unwrap_or("");
                    if !seen.insert(id) {
                        ctx.diagnostics.push(CatalogDiagnostic::error(
                            "RD2405",
                            path,
                            format!("line {}: duplicate tab id `{id}`", tab.line),
                        ));
                    }
                }
            }
        }
        if tab.children.is_empty() {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2405",
                path,
                format!("line {}: `:tab` cannot be empty", tab.line),
            ));
        }
    }
}

fn validate_example(ctx: &mut BuildCtx<'_>, node: &DocsNode) {
    let has_path = !node.attrs.path.as_deref().unwrap_or("").is_empty();
    let has_code = contains_code(&node.children);
    if has_path && has_code && !only_caption(&node.children) {
        malformed(
            ctx,
            node.line,
            "`:example` cannot combine `path` with a code body unless the body is a caption",
        );
    }
    if !node.attrs.test.is_empty() {
        if argv_unsafe(&node.attrs.test) {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2602",
                ctx.source_path,
                format!(
                    "line {}: example `test` must be a simple argument list without shell metacharacters",
                    node.line
                ),
            ));
        }
        if node.attrs.language.as_deref().unwrap_or("").is_empty() && !has_path {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2602",
                ctx.source_path,
                format!(
                    "line {}: example with `test` requires `language` or `path`",
                    node.line
                ),
            ));
        }
        if node.attrs.expect.is_none() {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2602",
                ctx.source_path,
                format!("line {}: example with `test` requires `expect`", node.line),
            ));
        }
    } else {
        ctx.diagnostics.push(CatalogDiagnostic::warning(
            "RD2601",
            ctx.source_path,
            format!("line {}: untested `:example`", node.line),
        ));
    }
}

fn argv_unsafe(argv: &[String]) -> bool {
    argv.iter().any(|part| {
        part.chars()
            .any(|ch| matches!(ch, '|' | '&' | ';' | '$' | '`' | '\n' | '>' | '<'))
    })
}

fn malformed(ctx: &mut BuildCtx<'_>, line: u32, message: &str) {
    ctx.diagnostics.push(CatalogDiagnostic::error(
        "RD2402",
        ctx.source_path,
        format!("line {line}: {message}"),
    ));
}

fn count_images(nodes: &[ArticleNode]) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            ArticleNode::Markdown(MdNode::Paragraph { children, .. }) => children
                .iter()
                .filter(|child| matches!(child, MdNode::Image { .. }))
                .count(),
            ArticleNode::Markdown(MdNode::Image { .. }) | ArticleNode::Image(_) => 1,
            ArticleNode::Block(docs) => count_images(&docs.children),
            _ => 0,
        })
        .sum()
}

fn contains_table(nodes: &[ArticleNode]) -> bool {
    nodes.iter().any(|node| match node {
        ArticleNode::Markdown(MdNode::Table { .. }) => true,
        ArticleNode::Block(docs) => contains_table(&docs.children),
        _ => false,
    })
}

fn contains_code(nodes: &[ArticleNode]) -> bool {
    nodes.iter().any(|node| match node {
        ArticleNode::Markdown(MdNode::CodeBlock { .. }) => true,
        ArticleNode::Block(docs) => contains_code(&docs.children),
        _ => false,
    })
}

fn only_caption(nodes: &[ArticleNode]) -> bool {
    nodes.iter().all(|node| match node {
        ArticleNode::Markdown(MdNode::Paragraph { .. }) => true,
        ArticleNode::Markdown(MdNode::CodeBlock { .. }) => false,
        ArticleNode::Block(_) => false,
        _ => true,
    })
}
