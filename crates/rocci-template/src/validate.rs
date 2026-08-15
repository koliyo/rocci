use std::collections::HashSet;

use crate::ast::{Document, FixtureDecl, ModuleItem, TemplateItem};
use crate::diagnostic::Diagnostic;

pub fn validate(document: &Document, diagnostics: &mut Vec<Diagnostic>) {
    let component_names: HashSet<&str> = document
        .items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Component(component) => Some(component.name.name.as_str()),
            _ => None,
        })
        .collect();

    for item in &document.items {
        match item {
            ModuleItem::Component(component) => {
                let mut saw_render = false;
                validate_items(&component.body.items, diagnostics, &mut saw_render, true);
            }
            ModuleItem::Fixture(fixture) => {
                validate_fixture(fixture, &component_names, diagnostics)
            }
            ModuleItem::Roc { .. } | ModuleItem::Css(_) => {}
        }
    }
}

fn validate_fixture(
    fixture: &FixtureDecl,
    component_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if fixture.target.parts.len() != 1 {
        return;
    }
    let name = fixture.target.roc_name.as_str();
    if name.is_empty() {
        return;
    }
    if !component_names.contains(name) {
        diagnostics.push(Diagnostic::error(
            fixture.target.span,
            format!("unknown fixture target `{name}`; no `@component {name}` in this module"),
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
