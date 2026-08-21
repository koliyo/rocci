use std::collections::HashSet;

use crate::ast::{
    Document, FixtureDecl, LiveDecl, ModuleItem, TemplateItem, handler_param_arity,
    parse_component_params,
};
use crate::diagnostic::Diagnostic;
use crate::resolve::{fixture_target_name_error, pascal_to_camel};
use crate::span::Span;

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
    let mut live_span = None;
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
            ModuleItem::Live(live) => {
                if live_span.is_some() {
                    diagnostics.push(Diagnostic::error(
                        live.span,
                        "duplicate `@live`; a module may declare one live render",
                    ));
                } else {
                    live_span = Some(live.span);
                }
                if let Some(params) = live.params
                    && handler_param_arity(params.of(src)) > 2
                {
                    diagnostics.push(Diagnostic::error(
                        params,
                        "`@live` takes at most two parameters: state and request",
                    ));
                }
                if live_has_record_params(src, live) {
                    has_record_handler = true;
                }
            }
            ModuleItem::View(view) => {
                validate_route_handler(
                    &mut RouteValidation {
                        src,
                        diagnostics,
                        has_record_handler: &mut has_record_handler,
                        routes: &mut routes,
                    },
                    "view",
                    "get",
                    &view.path,
                    view.params,
                    view.span,
                );
            }
            ModuleItem::Patch(patch) => {
                let method = patch
                    .method
                    .as_ref()
                    .map(|ident| ident.name.as_str())
                    .unwrap_or("post");
                validate_route_handler(
                    &mut RouteValidation {
                        src,
                        diagnostics,
                        has_record_handler: &mut has_record_handler,
                        routes: &mut routes,
                    },
                    "patch",
                    method,
                    &patch.path,
                    patch.params,
                    patch.span,
                );
            }
            ModuleItem::Command(command) => {
                let method = command
                    .method
                    .as_ref()
                    .map(|ident| ident.name.as_str())
                    .unwrap_or("post");
                validate_route_handler(
                    &mut RouteValidation {
                        src,
                        diagnostics,
                        has_record_handler: &mut has_record_handler,
                        routes: &mut routes,
                    },
                    "command",
                    method,
                    &command.path,
                    command.params,
                    command.span,
                );
            }
            ModuleItem::Roc { .. } | ModuleItem::Css(_) => {}
        }
    }

    if live_span.is_some()
        && let Some((_, _, span)) = routes
            .iter()
            .find(|(method, path, _)| *method == "get" && *path == "/sse")
    {
        diagnostics.push(Diagnostic::error(
            *span,
            "`@view(\"/sse\")` conflicts with generated `@live` stream",
        ));
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
                ModuleItem::View(view) if handler_has_record_params(src, view.params) => {
                    Some(view.span)
                }
                ModuleItem::Patch(patch) if handler_has_record_params(src, patch.params) => {
                    Some(patch.span)
                }
                ModuleItem::Command(command) if handler_has_record_params(src, command.params) => {
                    Some(command.span)
                }
                ModuleItem::Live(live) if live_has_record_params(src, live) => Some(live.span),
                _ => None,
            })
            .unwrap_or(document.span);
        diagnostics.push(Diagnostic::error(
            span,
            "`@view`, `@patch`, `@command`, and `@live` handlers that destructure a record require `@context`",
        ));
    }
}

fn live_has_record_params(src: &str, live: &LiveDecl) -> bool {
    let Some(params) = live.params else {
        return false;
    };
    parse_component_params(src, params).first_param_is_record
}

fn handler_has_record_params(src: &str, params: Option<Span>) -> bool {
    let Some(params) = params else {
        return false;
    };
    parse_component_params(src, params).first_param_is_record
}

struct RouteValidation<'a, 'd> {
    src: &'a str,
    diagnostics: &'d mut Vec<Diagnostic>,
    has_record_handler: &'d mut bool,
    routes: &'d mut Vec<(&'a str, &'a str, Span)>,
}

fn validate_route_handler<'a, 'd>(
    validation: &mut RouteValidation<'a, 'd>,
    noun: &str,
    method: &'a str,
    path: &'a str,
    params: Option<Span>,
    span: Span,
) {
    if let Some(params) = params
        && handler_param_arity(params.of(validation.src)) > 2
    {
        validation.diagnostics.push(Diagnostic::error(
            params,
            format!("`@{noun}` handlers take at most two parameters: state and request"),
        ));
    }
    if handler_has_record_params(validation.src, params) {
        *validation.has_record_handler = true;
    }
    if path.is_empty() {
        return;
    }
    if let Some((_, _, _)) = validation
        .routes
        .iter()
        .find(|(existing_method, existing_path, _)| {
            *existing_method == method && *existing_path == path
        })
    {
        let header = route_header(noun, method, path);
        validation.diagnostics.push(Diagnostic::error(
            span,
            format!("duplicate `{header}` handler"),
        ));
    } else {
        validation.routes.push((method, path, span));
    }
}

fn route_header(noun: &str, method: &str, path: &str) -> String {
    match (noun, method) {
        ("view", _) => format!("@view(\"{path}\")"),
        (_, "post") => format!("@{noun}(\"{path}\")"),
        _ => format!("@{noun}:{method}(\"{path}\")"),
    }
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
