use crate::ast::{
    CommandDecl, ContextDecl, FragmentDecl, Ident, InitDecl, LiveDecl, ViewDecl,
    ensure_handler_request_param, strip_param_defaults,
};
use crate::source_map::OriginKind;
use crate::span::Span;

use super::emitter::Emitter;
use super::{InitInfo, LiveInfo, RespondKind, RouteInfo};

pub(crate) struct RouteLowering<'a> {
    method: &'a Ident,
    path: &'a str,
    path_span: Span,
    respond: RespondKind,
    params: Option<Span>,
    body: Span,
    span: Span,
}

pub fn route_fn_name(method: &str, path: &str) -> String {
    let method = method.to_ascii_lowercase();
    let path_part = if path.is_empty() || path == "/" {
        "root".to_string()
    } else {
        path.trim_matches('/')
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect::<String>()
    };
    format!("on_{method}_{path_part}!")
}

pub(crate) fn route_header_span(src: &str, method: &Ident, path_span: Span) -> Span {
    let at = method.span.start.saturating_sub(1);
    let start = if src.as_bytes().get(at as usize) == Some(&b'@') {
        at
    } else {
        method.span.start
    };
    let end = if src.as_bytes().get(path_span.end as usize) == Some(&b')') {
        path_span.end + 1
    } else {
        path_span.end
    };
    Span { start, end }
}

impl<'a> Emitter<'a> {
    pub(crate) fn lower_context(&mut self, context: &ContextDecl) {
        self.emit_leading(&context.leading);
        let ty = context.ty.of(self.src).trim();
        if self.state_type.is_none() {
            self.state_type = Some(ty.to_string());
        }
        self.emit("State : ");
        self.emit_mapped(ty, context.ty, OriginKind::OrdinaryRoc);
        self.emit("\n");
    }

    pub(crate) fn lower_init(&mut self, init: &InitDecl) {
        self.emit_leading(&init.leading);
        self.init = Some(InitInfo { span: init.span });
        self.emit("init! = || {\n");
        self.indent += 1;
        self.emit_try_block(init.body, "rocci_state");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    pub(crate) fn lower_live(&mut self, live: &LiveDecl) {
        self.emit_leading(&live.leading);
        let method = live.method.name.to_ascii_uppercase();
        let fn_name = route_fn_name(&live.method.name, &live.path);
        self.lives.push(LiveInfo {
            method,
            path: live.path.clone(),
            fn_name: fn_name.clone(),
            span: live.span,
        });
        let params = live
            .params
            .map(|span| {
                ensure_handler_request_param(&strip_param_defaults(span.of(self.src).trim()))
            })
            .unwrap_or_else(|| "|state, _request|".to_string());
        self.emit_mapped(
            &fn_name,
            route_header_span(self.src, &live.method, live.path_span),
            OriginKind::RouteHeader,
        );
        self.emit(" = ");
        if let Some(span) = live.params {
            self.emit_mapped(&params, span, OriginKind::OrdinaryRoc);
        } else {
            self.emit(&params);
        }
        self.emit(" {\n");
        self.indent += 1;
        self.emit_try_block(live.body, "rocci_value");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    pub(crate) fn lower_view(&mut self, view: &ViewDecl) {
        self.emit_leading(&view.leading);
        self.lower_route(RouteLowering {
            method: &view.method,
            path: &view.path,
            path_span: view.path_span,
            respond: RespondKind::Document,
            params: view.params,
            body: view.body,
            span: view.span,
        });
    }

    pub(crate) fn lower_fragment_decl(&mut self, fragment: &FragmentDecl) {
        self.emit_leading(&fragment.leading);
        self.lower_route(RouteLowering {
            method: &fragment.method,
            path: &fragment.path,
            path_span: fragment.path_span,
            respond: RespondKind::Fragment,
            params: fragment.params,
            body: fragment.body,
            span: fragment.span,
        });
    }

    pub(crate) fn lower_command(&mut self, command: &CommandDecl) {
        self.emit_leading(&command.leading);
        self.lower_route(RouteLowering {
            method: &command.method,
            path: &command.path,
            path_span: command.path_span,
            respond: RespondKind::Command,
            params: command.params,
            body: command.body,
            span: command.span,
        });
    }

    pub(crate) fn lower_route(&mut self, route: RouteLowering<'_>) {
        let method_upper = route.method.name.to_ascii_uppercase();
        let fn_name = route_fn_name(&route.method.name, route.path);
        self.routes.push(RouteInfo {
            method: method_upper,
            path: route.path.to_string(),
            fn_name: fn_name.clone(),
            respond: route.respond,
            span: route.span,
        });
        let adapted = route
            .params
            .map(|param_span| {
                ensure_handler_request_param(&strip_param_defaults(param_span.of(self.src).trim()))
            })
            .unwrap_or_else(|| "|state, _request|".to_string());
        self.emit_mapped(
            &fn_name,
            route_header_span(self.src, route.method, route.path_span),
            OriginKind::RouteHeader,
        );
        self.emit(" = ");
        if let Some(param_span) = route.params {
            self.emit_mapped(&adapted, param_span, OriginKind::OrdinaryRoc);
        } else {
            self.emit(&adapted);
        }
        self.emit(" {\n");
        self.indent += 1;
        self.emit_try_block(route.body, "rocci_value");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    pub(crate) fn emit_try_block(&mut self, body: Span, result_name: &str) {
        let raw = body.of(self.src);
        let text = raw.trim();
        self.push_indent();
        if text.is_empty() {
            self.emit("Ok({})\n");
            return;
        }
        self.emit(result_name);
        self.emit(" = {\n");
        self.indent += 1;
        let mut pos = body.start as usize + (raw.len() - raw.trim_start().len());
        for line in text.lines() {
            self.push_indent();
            let emitted = line.trim_end();
            if !emitted.is_empty() {
                self.emit_mapped(
                    emitted,
                    Span::new(pos, pos + emitted.len()),
                    OriginKind::OrdinaryRoc,
                );
            }
            self.emit("\n");
            pos += line.len();
            if pos < self.src.len() && self.src.as_bytes()[pos] == b'\r' {
                pos += 1;
            }
            if pos < self.src.len() && self.src.as_bytes()[pos] == b'\n' {
                pos += 1;
            }
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
        self.push_indent();
        self.emit("Ok(");
        self.emit(result_name);
        self.emit(")\n");
    }
}
