use std::collections::HashSet;

use crate::ast::{
    Document, FixtureDecl, ModuleItem, OnDecl, TemplateItem, handler_param_arity,
    parse_component_params,
};
use crate::diagnostic::Diagnostic;
use crate::resolve::{fixture_target_name_error, pascal_to_camel};

pub fn validate_template_items(items: &[TemplateItem], diagnostics: &mut Vec<Diagnostic>) {
    let mut saw_render = false;
    validate_items(items, diagnostics, &mut saw_render, false);
}

pub fn validate(src: &str, document: &Document, diagnostics: &mut Vec<Diagnostic>) {
    let component_names: HashSet<String> = document
        .items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Component(component) => Some(pascal_to_camel(&component.name.name)),
            _ => None,
        })
        .collect();

    let mut context_span = None;
    let mut init_span = None;
    let mut routes: Vec<(&str, &str, crate::span::Span)> = Vec::new();
    let mut has_record_handler = false;

    for item in &document.items {
        match item {
            ModuleItem::Component(component) => {
                let mut saw_render = false;
                validate_items(&component.body.items, diagnostics, &mut saw_render, true);
            }
            ModuleItem::Fixture(fixture) => {
                validate_fixture(fixture, &component_names, diagnostics)
            }
            ModuleItem::Context(context) => {
                if context_span.is_some() {
                    diagnostics.push(Diagnostic::error(
                        context.span,
                        "duplicate `@context`; a module may declare app state once",
                    ));
                } else {
                    context_span = Some(context.span);
                }
            }
            ModuleItem::Init(init) => {
                if init_span.is_some() {
                    diagnostics.push(Diagnostic::error(
                        init.span,
                        "duplicate `@init`; a module may initialize app state once",
                    ));
                } else {
                    init_span = Some(init.span);
                }
            }
            ModuleItem::On(on) => {
                if let Some(params) = on.params
                    && handler_param_arity(params.of(src)) > 2
                {
                    diagnostics.push(Diagnostic::error(
                        params,
                        "`@on` handlers take at most two parameters: state and request",
                    ));
                }
                if handler_has_record_params(src, on) {
                    has_record_handler = true;
                }
                if on.path.is_empty() || on.method.name.is_empty() {
                    continue;
                }
                if let Some((_, _, _)) = routes
                    .iter()
                    .find(|(method, path, _)| *method == on.method.name && *path == on.path)
                {
                    diagnostics.push(Diagnostic::error(
                        on.span,
                        format!(
                            "duplicate `@on:{}(\"{}\")` handler",
                            on.method.name, on.path
                        ),
                    ));
                } else {
                    routes.push((on.method.name.as_str(), on.path.as_str(), on.span));
                }
            }
            ModuleItem::Roc { .. } | ModuleItem::Css(_) => {}
        }
    }

    if let (Some(init), None) = (init_span, context_span) {
        diagnostics.push(Diagnostic::error(
            init,
            "`@init` requires `@context` to declare the app state type",
        ));
    }
    if let (Some(ctx), None) = (context_span, init_span) {
        diagnostics.push(Diagnostic::error(
            ctx,
            "`@context` requires `@init` to produce the app state value",
        ));
    }
    if has_record_handler && context_span.is_none() {
        let span = document
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::On(on) if handler_has_record_params(src, on) => Some(on.span),
                _ => None,
            })
            .unwrap_or(document.span);
        diagnostics.push(Diagnostic::error(
            span,
            "`@on` handlers that destructure a record require `@context`",
        ));
    }
}

fn handler_has_record_params(src: &str, on: &OnDecl) -> bool {
    let Some(params) = on.params else {
        return false;
    };
    parse_component_params(src, params).first_param_is_record
}

fn validate_fixture(
    fixture: &FixtureDecl,
    component_names: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(last) = fixture.target.parts.last() else {
        return;
    };
    if let Some(message) = fixture_target_name_error(&last.name) {
        diagnostics.push(Diagnostic::error(last.span, message));
        return;
    }
    if fixture.target.parts.len() != 1 {
        return;
    }
    let name = fixture.target.roc_name.as_str();
    if name.is_empty() {
        return;
    }
    if !component_names.contains(name) {
        let source = fixture.target.source_name();
        diagnostics.push(Diagnostic::error(
            fixture.target.span,
            format!("unknown fixture target `{source}`; no `@component {source}` in this module"),
        ));
    }
}

fn validate_items(
    items: &[TemplateItem],
    diagnostics: &mut Vec<Diagnostic>,
    saw_render: &mut bool,
    allow_css: bool,
) {
    for item in items {
        match item {
            TemplateItem::Let(let_dir) if *saw_render => {
                diagnostics.push(Diagnostic::error(
                    let_dir.span,
                    "`@let` must appear before render-producing items in this block",
                ));
            }
            TemplateItem::Let(_) => {}
            TemplateItem::Css(css) if !allow_css => {
                diagnostics.push(Diagnostic::error(
                    css.span,
                    "`@css` is only valid at the start of a component body",
                ));
            }
            TemplateItem::Css(css) if *saw_render => {
                diagnostics.push(Diagnostic::error(
                    css.span,
                    "`@css` must appear before render-producing items in this block",
                ));
            }
            TemplateItem::Css(_) => {}
            other => {
                *saw_render = true;
                match other {
                    TemplateItem::Element(el) => {
                        let mut nested = false;
                        validate_items(&el.children, diagnostics, &mut nested, false);
                    }
                    TemplateItem::ComponentCall(call) => {
                        if let Some(children) = &call.children {
                            let mut nested = false;
                            validate_items(children, diagnostics, &mut nested, false);
                        }
                    }
                    TemplateItem::Fragment(frag) => {
                        let mut nested = false;
                        validate_items(&frag.children, diagnostics, &mut nested, false);
                    }
                    TemplateItem::If(dir) => {
                        let mut nested = false;
                        validate_items(&dir.then_body.items, diagnostics, &mut nested, false);
                        for (_, body) in &dir.else_ifs {
                            let mut nested = false;
                            validate_items(&body.items, diagnostics, &mut nested, false);
                        }
                        if let Some(body) = &dir.else_body {
                            let mut nested = false;
                            validate_items(&body.items, diagnostics, &mut nested, false);
                        }
                    }
                    TemplateItem::For(dir) => {
                        let mut nested = false;
                        validate_items(&dir.body.items, diagnostics, &mut nested, false);
                    }
                    TemplateItem::Match(dir) => {
                        for arm in &dir.arms {
                            let mut nested = false;
                            validate_items(
                                std::slice::from_ref(&*arm.value),
                                diagnostics,
                                &mut nested,
                                false,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
