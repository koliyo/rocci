use std::collections::{HashMap, HashSet};

use crate::ast::{
    Document, FixtureDecl, ModuleItem, RouteDecl, TemplateItem, default_field_type,
    handler_param_arity, parse_component_params,
};
use crate::diagnostic::Diagnostic;
use crate::lower::route_fn_name;
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
    let mut routes: Vec<(String, String, Span)> = Vec::new();
    let mut generated_names: HashMap<String, (String, String, Span)> = HashMap::new();
    let mut has_record_handler = false;

    for item in &document.items {
        match item {
            ModuleItem::Component(component) => {
                let mut saw_render = false;
                validate_items(&component.body.items, diagnostics, &mut saw_render, true);
                validate_defaulted_props(src, component.params, diagnostics);
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
            ModuleItem::Route(route) => {
                let parts = route_parts(route);
                validate_route_handler(
                    &mut RouteValidation {
                        src,
                        diagnostics,
                        has_record_handler: &mut has_record_handler,
                        routes: &mut routes,
                        generated_names: &mut generated_names,
                    },
                    parts,
                );
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
                ModuleItem::Route(route)
                    if handler_has_record_params(src, route_parts(route).params) =>
                {
                    Some(route.span())
                }
                _ => None,
            })
            .unwrap_or(document.span);
        diagnostics.push(Diagnostic::error(
            span,
            "route handlers that destructure a record require `@context`",
        ));
    }
}

fn validate_defaulted_props(src: &str, params: Span, diagnostics: &mut Vec<Diagnostic>) {
    let parsed = parse_component_params(src, params);
    if !parsed.first_param_is_record {
        return;
    }
    let prop_count = parsed.param_names.len() - parsed.body_params.len();
    let prop_names = &parsed.param_names[..prop_count];
    for (name, default) in &parsed.param_defaults {
        if !prop_names.iter().any(|n| n == name) {
            continue;
        }
        let authored_ty = parsed
            .param_types
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, ty)| ty.as_str());
        if default_field_type(authored_ty, default).is_none() {
            diagnostics.push(Diagnostic::error(
                params,
                format!(
                  "defaulted field `{name}` needs a type (`{name} : Type ?? {default}`); string, Bool, and integer defaults are inferred"
                ),
            ));
        }
    }
}

struct RouteParts<'a> {
    role: &'static str,
    method: &'a str,
    method_span: Span,
    path: &'a str,
    path_span: Span,
    params: Option<Span>,
    span: Span,
}

fn route_parts(route: &RouteDecl) -> RouteParts<'_> {
    match route {
        RouteDecl::View(decl) => RouteParts {
            role: "view",
            method: &decl.method.name,
            method_span: decl.method.span,
            path: &decl.path,
            path_span: decl.path_span,
            params: decl.params,
            span: decl.span,
        },
        RouteDecl::Fragment(decl) => RouteParts {
            role: "fragment",
            method: &decl.method.name,
            method_span: decl.method.span,
            path: &decl.path,
            path_span: decl.path_span,
            params: decl.params,
            span: decl.span,
        },
        RouteDecl::Command(decl) => RouteParts {
            role: "command",
            method: &decl.method.name,
            method_span: decl.method.span,
            path: &decl.path,
            path_span: decl.path_span,
            params: decl.params,
            span: decl.span,
        },
        RouteDecl::Live(decl) => RouteParts {
            role: "live",
            method: &decl.method.name,
            method_span: decl.method.span,
            path: &decl.path,
            path_span: decl.path_span,
            params: decl.params,
            span: decl.span,
        },
    }
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
    routes: &'d mut Vec<(String, String, Span)>,
    generated_names: &'d mut HashMap<String, (String, String, Span)>,
}

fn validate_route_handler<'a, 'd>(validation: &mut RouteValidation<'a, 'd>, route: RouteParts<'_>) {
    if let Some(params) = route.params
        && handler_param_arity(params.of(validation.src)) > 2
    {
        validation.diagnostics.push(Diagnostic::error(
            params,
            format!(
                "`@{}:{}` handlers take at most two parameters: state and request",
                route.method, route.role
            ),
        ));
    }
    if handler_has_record_params(validation.src, route.params) {
        *validation.has_record_handler = true;
    }
    let method_known = matches!(route.method, "get" | "post" | "put" | "patch" | "delete");
    if !method_known {
        validation.diagnostics.push(Diagnostic::error(
            route.method_span,
            format!(
                "unknown HTTP method `{}`; expected get, post, put, patch, or delete",
                route.method
            ),
        ));
    } else if !legal_pair(route.method, route.role) {
        let guidance = if route.method == "get" {
            "GET accepts view, fragment, or live"
        } else {
            "mutation methods accept fragment or command"
        };
        validation.diagnostics.push(Diagnostic::error(
            route.method_span,
            format!(
                "illegal handler pair `@{}:{}`; {guidance}",
                route.method, route.role
            ),
        ));
    }
    if route.path.is_empty() {
        validation.diagnostics.push(Diagnostic::error(
            route.path_span,
            format!(
                "`@{}:{}` requires a non-empty literal path",
                route.method, route.role
            ),
        ));
        return;
    }
    if validation
        .routes
        .iter()
        .any(|(existing_method, existing_path, _)| {
            existing_method == route.method && existing_path == route.path
        })
    {
        let header = route_header(route.method, route.role, route.path);
        validation.diagnostics.push(Diagnostic::error(
            route.span,
            format!("duplicate `{header}` handler"),
        ));
    } else {
        let fn_name = route_fn_name(route.method, route.path);
        if let Some((first_method, first_path, _)) = validation.generated_names.get(&fn_name) {
            validation.diagnostics.push(Diagnostic::error(
                route.span,
                format!(
                    "route `{} {}` and `{} {}` both generate Roc handler `{fn_name}`; choose paths with distinct normalized names",
                    first_method.to_ascii_uppercase(),
                    first_path,
                    route.method.to_ascii_uppercase(),
                    route.path,
                ),
            ));
        } else {
            validation.generated_names.insert(
                fn_name,
                (route.method.to_string(), route.path.to_string(), route.span),
            );
        }
        validation
            .routes
            .push((route.method.to_string(), route.path.to_string(), route.span));
    }
}

fn legal_pair(method: &str, role: &str) -> bool {
    match method {
        "get" => matches!(role, "view" | "fragment" | "live"),
        "post" | "put" | "patch" | "delete" => matches!(role, "fragment" | "command"),
        _ => false,
    }
}

fn route_header(method: &str, role: &str, path: &str) -> String {
    format!("@{method}:{role}(\"{path}\")")
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
