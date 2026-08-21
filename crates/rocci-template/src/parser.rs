use crate::ast::{
    Attr, AttrValue, CommandDecl, ComponentCall, ComponentDecl, ComponentPath, ContextDecl,
    CssDecl, Document, Element, FixtureDecl, ForDirective, Fragment, Ident, IfDirective, InitDecl,
    Interpolation, LeadingComments, LetDirective, LiveDecl, MatchArm, MatchDirective, ModuleItem,
    PatchDecl, TemplateBlock, TemplateItem, TextNode, ViewDecl,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{self, Cursor, is_ident_start};
use crate::resolve::{component_name_error, component_roc_name, is_ambiguous_pascal};
use crate::span::{SourceFile, Span};

pub struct ParseOutput {
    pub document: Document,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(source: SourceFile<'_>) -> ParseOutput {
    let mut parser = Parser {
        cur: Cursor::new(source.src),
        diagnostics: Vec::new(),
    };
    let document = parser.parse_document();
    ParseOutput {
        document,
        diagnostics: parser.diagnostics,
    }
}

pub struct ParseDeclOutput {
    pub item: ModuleItem,
    pub end: usize,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_declaration_from(src: &str, start: usize) -> Option<ParseDeclOutput> {
    let mut parser = Parser {
        cur: Cursor::at(src, start),
        diagnostics: Vec::new(),
    };
    let item = if let Some(decl) = parser.try_parse_fixture() {
        ModuleItem::Fixture(decl)
    } else if let Some(decl) = parser.try_parse_context() {
        ModuleItem::Context(decl)
    } else if let Some(decl) = parser.try_parse_init() {
        ModuleItem::Init(decl)
    } else if let Some(decl) = parser.try_parse_live() {
        ModuleItem::Live(decl)
    } else if let Some(item) = parser.try_parse_route() {
        item
    } else if let Some(decl) = parser.try_parse_component() {
        ModuleItem::Component(decl)
    } else {
        ModuleItem::Css(parser.try_parse_css()?)
    };
    Some(ParseDeclOutput {
        end: parser.cur.pos,
        item,
        diagnostics: parser.diagnostics,
    })
}

pub struct ParseTemplateOutput {
    pub item: TemplateItem,
    pub end: usize,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_template_item_from(src: &str, start: usize) -> Option<ParseTemplateOutput> {
    let mut parser = Parser {
        cur: Cursor::at(src, start),
        diagnostics: Vec::new(),
    };
    let item = parser.parse_template_item(ItemStop::BlockEnd)?;
    Some(ParseTemplateOutput {
        end: parser.cur.pos,
        item,
        diagnostics: parser.diagnostics,
    })
}

struct Parser<'a> {
    cur: Cursor<'a>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    fn src(&self) -> &'a str {
        self.cur.src
    }

    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(span, message));
    }

    fn parse_document(&mut self) -> Document {
        let mut items = Vec::new();
        let mut opaque_start = 0usize;
        let start = 0usize;

        while !self.cur.is_eof() {
            let saved = Snapshot::from(&self.cur);
            self.cur.skip_trivia();
            if self.cur.is_eof() {
                break;
            }

            if self.cur.is_top_level() {
                if let Some(fixture) = self.try_parse_fixture() {
                    if opaque_start < fixture.span.start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, fixture.span.start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(ModuleItem::Fixture(fixture));
                    continue;
                }
                if let Some(context) = self.try_parse_context() {
                    if opaque_start < context.span.start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, context.span.start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(ModuleItem::Context(context));
                    continue;
                }
                if let Some(init) = self.try_parse_init() {
                    if opaque_start < init.span.start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, init.span.start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(ModuleItem::Init(init));
                    continue;
                }
                if let Some(live) = self.try_parse_live() {
                    if opaque_start < live.span.start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, live.span.start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(ModuleItem::Live(live));
                    continue;
                }
                if let Some(item) = self.try_parse_route() {
                    if opaque_start < item.span().start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, item.span().start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(item);
                    continue;
                }
                if self.try_recover_removed_handler() {
                    opaque_start = self.cur.pos;
                    continue;
                }
                if let Some(component) = self.try_parse_component() {
                    if opaque_start < component.span.start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, component.span.start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(ModuleItem::Component(component));
                    continue;
                }
                if let Some(css) = self.try_parse_css() {
                    if opaque_start < css.span.start as usize {
                        items.push(ModuleItem::Roc {
                            span: Span::new(opaque_start, css.span.start as usize),
                        });
                    }
                    opaque_start = self.cur.pos;
                    items.push(ModuleItem::Css(css));
                    continue;
                }
            }

            saved.restore(&mut self.cur);
            self.cur.skip_roc_token();
            if self.cur.pos == saved.pos {
                self.cur.bump();
            }
        }

        if opaque_start < self.src().len() {
            items.push(ModuleItem::Roc {
                span: Span::new(opaque_start, self.src().len()),
            });
        }

        Document {
            items,
            span: Span::new(start, self.src().len()),
        }
    }

    fn try_parse_css(&mut self) -> Option<CssDecl> {
        let at = self.cur.pos;
        self.scan_at_keyword("css")?;
        let mut decl = self.parse_css_after_keyword(at);
        let (leading, span) = self.finish_leading(at, decl.span.end as usize);
        decl.leading = leading;
        decl.span = span;
        Some(decl)
    }

    fn parse_css_after_keyword(&mut self, start: usize) -> CssDecl {
        self.cur.skip_trivia();
        let body = self.scan_css_block();
        CssDecl {
            leading: None,
            body,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn scan_css_block(&mut self) -> Span {
        if !self.cur.eat('{') {
            self.error(
                Span::point(self.cur.pos),
                "expected `{` to open a `@css` block",
            );
            return Span::point(self.cur.pos);
        }
        let body_start = self.cur.pos;
        let mut depth: usize = 1;
        while !self.cur.is_eof() && depth > 0 {
            match self.cur.peek() {
                Some('"') | Some('\'') => self.skip_css_string(),
                Some('/') if self.cur.peek_at(1) == Some('*') => self.skip_css_comment(),
                Some('{') => {
                    self.cur.bump();
                    depth += 1;
                }
                Some('}') => {
                    self.cur.bump();
                    depth -= 1;
                }
                _ => {
                    self.cur.bump();
                }
            }
        }
        if depth != 0 {
            self.error(
                Span::new(body_start.saturating_sub(1), self.cur.pos),
                "unterminated `@css` block; expected `}`",
            );
            return Span::new(body_start, self.cur.pos);
        }
        Span::new(body_start, self.cur.pos - 1)
    }

    fn skip_css_string(&mut self) {
        let Some(quote) = self.cur.bump() else {
            return;
        };
        while let Some(ch) = self.cur.peek() {
            if ch == '\\' {
                self.cur.bump();
                self.cur.bump();
                continue;
            }
            self.cur.bump();
            if ch == quote {
                return;
            }
        }
    }

    fn skip_css_comment(&mut self) {
        self.cur.pos += 2;
        while !self.cur.is_eof() {
            if self.cur.eat_str("*/") {
                return;
            }
            self.cur.bump();
        }
    }

    fn try_parse_context(&mut self) -> Option<ContextDecl> {
        let at = self.cur.pos;
        self.scan_at_keyword("context")?;
        let mut decl = self.parse_context_after_keyword(at);
        let (leading, span) = self.finish_leading(at, decl.span.end as usize);
        decl.leading = leading;
        decl.span = span;
        Some(decl)
    }

    fn parse_context_after_keyword(&mut self, start: usize) -> ContextDecl {
        self.cur.skip_trivia();
        let ty = self.scan_roc_type();
        if ty.is_empty() {
            self.error(
                Span::point(self.cur.pos),
                "expected a Roc type after `@context`",
            );
        }
        ContextDecl {
            leading: None,
            ty,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn scan_roc_type(&mut self) -> Span {
        self.cur.skip_trivia();
        if self.cur.peek() == Some('{') {
            let start = self.cur.pos;
            self.cur.skip_balanced_braces();
            if self.cur.pos <= start {
                return Span::point(start);
            }
            return Span::new(start, self.cur.pos);
        }
        self.scan_roc_expr()
    }

    fn try_parse_init(&mut self) -> Option<InitDecl> {
        let at = self.cur.pos;
        self.scan_at_keyword("init")?;
        let mut decl = self.parse_init_after_keyword(at);
        let (leading, span) = self.finish_leading(at, decl.span.end as usize);
        decl.leading = leading;
        decl.span = span;
        Some(decl)
    }

    fn parse_init_after_keyword(&mut self, start: usize) -> InitDecl {
        self.cur.skip_trivia();
        let body = match self.scan_roc_block_inner() {
            Some(span) => span,
            None => {
                self.error(
                    Span::point(self.cur.pos),
                    "expected `{` to open an `@init` block",
                );
                Span::point(self.cur.pos)
            }
        };
        InitDecl {
            leading: None,
            body,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn try_parse_live(&mut self) -> Option<LiveDecl> {
        let at = self.cur.pos;
        self.scan_at_keyword("live")?;
        let mut decl = self.parse_live_after_keyword(at);
        let (leading, span) = self.finish_leading(at, decl.span.end as usize);
        decl.leading = leading;
        decl.span = span;
        Some(decl)
    }

    fn parse_live_after_keyword(&mut self, start: usize) -> LiveDecl {
        self.cur.skip_trivia();
        let mut params = None;
        if self.cur.eat('=') {
            self.cur.skip_trivia();
            params = self.scan_params();
            if params.is_none() {
                self.error(
                    Span::point(self.cur.pos),
                    "expected `|params|` after `@live =`",
                );
            }
            self.cur.skip_trivia();
        }
        let body = match self.scan_roc_block_inner() {
            Some(span) => span,
            None => {
                self.error(
                    Span::point(self.cur.pos),
                    "expected `{` to open an `@live` body",
                );
                Span::point(self.cur.pos)
            }
        };
        LiveDecl {
            leading: None,
            params,
            body,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn try_parse_route(&mut self) -> Option<ModuleItem> {
        let at = self.cur.pos;
        let saved = Snapshot::from(&self.cur);
        if !self.cur.eat('@') {
            return None;
        }
        let Some(kw) = self.cur.scan_ident() else {
            saved.restore(&mut self.cur);
            return None;
        };
        let kind = match self.cur.ident_text(kw) {
            "view" => RouteKind::View,
            "patch" => RouteKind::Patch,
            "command" => RouteKind::Command,
            _ => {
                saved.restore(&mut self.cur);
                return None;
            }
        };
        let item = self.parse_route_after_keyword(at, kind);
        Some(self.attach_route_leading(at, item))
    }

    fn parse_route_after_keyword(&mut self, start: usize, kind: RouteKind) -> ModuleItem {
        self.cur.skip_trivia();
        if self.cur.peek() == Some('[') {
            let bracket_start = self.cur.pos;
            self.cur.skip_balanced_brackets();
            self.error(
                Span::new(bracket_start, self.cur.pos),
                format!(
                    "selector brackets are not part of `@{}`; write `@{}(path)` or `@{}:delete(path)`",
                    kind.noun(),
                    kind.noun(),
                    kind.noun()
                ),
            );
            self.cur.skip_trivia();
        }

        let method = self.parse_route_method(kind);

        self.cur.skip_trivia();
        let Some(args) = self.scan_paren_inner() else {
            self.error(
                Span::point(self.cur.pos),
                format!(
                    "expected `(\"path\")` after `@{}`",
                    kind.header_without_path()
                ),
            );
            self.sync_to_next_top_level();
            return kind.to_item(ParsedRoute {
                method,
                path: String::new(),
                path_span: Span::point(self.cur.pos),
                params: None,
                body: Span::point(self.cur.pos),
                span: Span::new(start, self.cur.pos),
            });
        };
        let (path, path_span) = match string_literal(args.of(self.src()), args) {
            Some((path, path_span)) => (path, path_span),
            None => {
                self.error(
                    args,
                    format!(
                        "expected a string literal path, e.g. `@{}(\"/actions/...\")`",
                        kind.noun()
                    ),
                );
                (String::new(), args)
            }
        };

        self.cur.skip_trivia();
        if self.cur.peek() == Some('-')
            && self.src().as_bytes().get(self.cur.pos + 1) == Some(&b'>')
        {
            let arrow_start = self.cur.pos;
            self.cur.pos += 2;
            self.cur.skip_trivia();
            if let Some(span) = self.cur.scan_ident() {
                let _ = span;
            }
            self.error(
                Span::new(arrow_start, self.cur.pos),
                format!(
                    "`->` does not select a response; write `@{}(path)` or `@command(path)`",
                    kind.noun()
                ),
            );
            self.cur.skip_trivia();
        }

        if self.cur.peek().is_some_and(is_ident_start)
            && let Some(span) = self.cur.scan_ident()
        {
            let name = self.cur.ident_text(span).to_string();
            if name == "json" {
                self.error(
                    span,
                    "`json` was removed; write `@command(path)` and return Roc data",
                );
            } else {
                self.error(
                    span,
                    format!("unexpected `{name}` after `@{}` path", kind.noun()),
                );
            }
            self.cur.skip_trivia();
        }

        let mut params = None;
        if self.cur.eat('=') {
            self.cur.skip_trivia();
            params = self.scan_params();
            if params.is_none() {
                self.error(
                    Span::point(self.cur.pos),
                    format!(
                        "expected `|params|` after `@{} =`",
                        kind.header_without_path()
                    ),
                );
            }
            self.cur.skip_trivia();
        }
        let body = match self.scan_roc_block_inner() {
            Some(span) => span,
            None => {
                self.error(
                    Span::point(self.cur.pos),
                    format!("expected `{{` to open an `@{}` handler body", kind.noun()),
                );
                Span::point(self.cur.pos)
            }
        };
        kind.to_item(ParsedRoute {
            method,
            path,
            path_span,
            params,
            body,
            span: Span::new(start, self.cur.pos),
        })
    }

    fn parse_route_method(&mut self, kind: RouteKind) -> Option<Ident> {
        if kind == RouteKind::View {
            if self.cur.eat(':') {
                let method_span = self.cur.scan_ident();
                let end = method_span
                    .map(|span| span.end as usize)
                    .unwrap_or(self.cur.pos);
                self.error(
                    Span::new(self.cur.pos.saturating_sub(1), end),
                    "`@view` has no method suffix; GET is implied",
                );
            }
            return None;
        }
        if !self.cur.eat(':') {
            return None;
        }
        self.cur.skip_trivia();
        let Some(method_span) = self.cur.scan_ident() else {
            self.error(
                Span::point(self.cur.pos),
                format!(
                    "expected HTTP method after `@{}:`; write `:put`, `:patch`, or `:delete`",
                    kind.noun()
                ),
            );
            return None;
        };
        let method_name = self.cur.ident_text(method_span).to_string();
        match method_name.as_str() {
            "post" => {
                self.error(
                    method_span,
                    format!(
                        "POST is the default; write `@{}(path)` instead of `@{}:post`",
                        kind.noun(),
                        kind.noun()
                    ),
                );
                None
            }
            "get" => {
                self.error(
                    method_span,
                    format!(
                        "`@{}` cannot use GET; write `@view(path)` for documents",
                        kind.noun()
                    ),
                );
                None
            }
            "put" | "patch" | "delete" => Some(Ident {
                span: method_span,
                name: method_name,
            }),
            other => {
                self.error(
                    method_span,
                    format!("unknown HTTP method `{other}`; expected put, patch, or delete"),
                );
                None
            }
        }
    }

    fn try_recover_removed_handler(&mut self) -> bool {
        let start = self.cur.pos;
        let saved = Snapshot::from(&self.cur);
        if !self.cur.eat('@') {
            return false;
        }
        let Some(kw) = self.cur.scan_ident() else {
            saved.restore(&mut self.cur);
            return false;
        };
        match self.cur.ident_text(kw) {
            "on" => {
                self.recover_removed_on(start);
                true
            }
            "action" => {
                self.recover_removed_action(start);
                true
            }
            _ => {
                saved.restore(&mut self.cur);
                false
            }
        }
    }

    fn recover_removed_on(&mut self, start: usize) {
        self.cur.skip_trivia();
        let mut method = String::new();
        if self.cur.eat(':') {
            self.cur.skip_trivia();
            if let Some(span) = self.cur.scan_ident() {
                method = self.cur.ident_text(span).to_string();
            }
        }
        self.cur.skip_trivia();
        if self.cur.peek() == Some('[') {
            self.cur.skip_balanced_brackets();
            self.cur.skip_trivia();
        }
        let mut path = String::new();
        if let Some(args) = self.scan_paren_inner()
            && let Some((value, _)) = string_literal(args.of(self.src()), args)
        {
            path = value;
        }
        self.cur.skip_trivia();
        if self.cur.peek() == Some('-')
            && self.src().as_bytes().get(self.cur.pos + 1) == Some(&b'>')
        {
            self.cur.pos += 2;
            self.cur.skip_trivia();
            let _ = self.cur.scan_ident();
            self.cur.skip_trivia();
        }
        let mut json = false;
        if self.cur.peek().is_some_and(is_ident_start)
            && let Some(span) = self.cur.scan_ident()
        {
            json = self.cur.ident_text(span) == "json";
            self.cur.skip_trivia();
        }
        if self.cur.eat('=') {
            self.cur.skip_trivia();
            let _ = self.scan_params();
            self.cur.skip_trivia();
        }
        let _ = self.scan_roc_block_inner();
        let rewrite = removed_on_rewrite(&method, json, &path);
        self.error(
            Span::new(start, self.cur.pos.max(start + 1)),
            format!("`@on` was removed; write {rewrite}"),
        );
        if self.cur.pos == start {
            self.cur.bump();
        }
    }

    fn recover_removed_action(&mut self, start: usize) {
        self.cur.skip_trivia();
        if self.cur.peek() == Some('[') {
            self.cur.skip_balanced_brackets();
            self.cur.skip_trivia();
        }
        if self.cur.eat(':') {
            self.cur.skip_trivia();
            let _ = self.cur.scan_ident();
            self.cur.skip_trivia();
        }
        if self.cur.peek() == Some('[') {
            self.cur.skip_balanced_brackets();
            self.cur.skip_trivia();
        }
        let _ = self.scan_paren_inner();
        self.cur.skip_trivia();
        if self.cur.peek() == Some('-')
            && self.src().as_bytes().get(self.cur.pos + 1) == Some(&b'>')
        {
            self.cur.pos += 2;
            self.cur.skip_trivia();
            let _ = self.cur.scan_ident();
            self.cur.skip_trivia();
        }
        if self.cur.eat('=') {
            self.cur.skip_trivia();
            let _ = self.scan_params();
            self.cur.skip_trivia();
        }
        let _ = self.scan_roc_block_inner();
        self.error(
            Span::new(start, self.cur.pos.max(start + 1)),
            "`@action` was not adopted; write `@patch(path)` for HTML patches or `@command(path)` for command data",
        );
        if self.cur.pos == start {
            self.cur.bump();
        }
    }

    fn scan_paren_inner(&mut self) -> Option<Span> {
        if self.cur.peek() != Some('(') {
            return None;
        }
        let inner_start = self.cur.pos + 1;
        self.cur.bump();
        let mut depth = 1usize;
        while !self.cur.is_eof() && depth > 0 {
            if self.at_column_zero_at_decl() {
                break;
            }
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => self.cur.skip_comment(),
                Some('(') => {
                    self.cur.bump();
                    depth += 1;
                }
                Some(')') => {
                    self.cur.bump();
                    depth -= 1;
                }
                _ => {
                    self.cur.bump();
                }
            }
        }
        if depth != 0 {
            self.error(
                Span::new(inner_start.saturating_sub(1), self.cur.pos),
                "unterminated handler path; expected `)`",
            );
            return Some(Span::new(inner_start, self.cur.pos));
        }
        Some(lexer::trim_span(
            self.src(),
            Span::new(inner_start, self.cur.pos - 1),
        ))
    }

    fn scan_roc_block_inner(&mut self) -> Option<Span> {
        self.cur.skip_trivia();
        if self.cur.peek() != Some('{') {
            return None;
        }
        let start = self.cur.pos;
        let mut depth = 0usize;
        while !self.cur.is_eof() {
            if depth > 0 && self.at_column_zero_at_decl() {
                break;
            }
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => self.cur.skip_comment(),
                Some('{') => {
                    self.cur.bump();
                    depth += 1;
                }
                Some('}') => {
                    self.cur.bump();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    self.cur.bump();
                }
            }
        }
        if depth != 0 {
            self.error(
                Span::new(start, self.cur.pos),
                "unterminated block; expected `}`",
            );
            return Some(lexer::trim_span(
                self.src(),
                Span::new(start + 1, self.cur.pos),
            ));
        }
        if self.cur.pos <= start + 1 {
            return Some(Span::point(start + 1));
        }
        let last = self.src().as_bytes().get(self.cur.pos - 1).copied();
        if last != Some(b'}') {
            self.error(
                Span::new(start, self.cur.pos),
                "unterminated block; expected `}`",
            );
            return Some(lexer::trim_span(
                self.src(),
                Span::new(start + 1, self.cur.pos),
            ));
        }
        Some(lexer::trim_span(
            self.src(),
            Span::new(start + 1, self.cur.pos - 1),
        ))
    }

    fn try_parse_fixture(&mut self) -> Option<FixtureDecl> {
        let at = self.cur.pos;
        self.scan_at_keyword("fixture")?;
        let mut decl = self.parse_fixture_after_keyword(at);
        let (leading, span) = self.finish_leading(at, decl.span.end as usize);
        decl.leading = leading;
        decl.span = span;
        Some(decl)
    }

    fn parse_fixture_after_keyword(&mut self, start: usize) -> FixtureDecl {
        self.cur.skip_trivia();
        let target = self.parse_fixture_attrs(start);
        self.cur.skip_trivia();
        let Some(name_span) = self.cur.scan_ident() else {
            self.error(
                Span::new(start, self.cur.pos),
                "expected fixture name after `@fixture{target: ...}`",
            );
            self.sync_to_next_top_level();
            return self.empty_fixture(start, target);
        };
        let name = Ident {
            span: name_span,
            name: self.cur.ident_text(name_span).to_string(),
        };
        self.cur.skip_trivia();
        if !self.cur.eat('=') {
            self.error(name_span, "expected `=` after fixture name");
        }
        let value = self.scan_roc_expr();
        if value.is_empty() {
            self.error(
                Span::point(self.cur.pos),
                "expected a Roc expression after `@fixture` name `=`",
            );
        }
        FixtureDecl {
            leading: None,
            span: Span::new(start, self.cur.pos),
            name,
            target,
            value,
        }
    }

    fn parse_fixture_attrs(&mut self, start: usize) -> ComponentPath {
        if self.cur.peek() != Some('{') {
            self.error(
                Span::new(start, self.cur.pos),
                "expected `{target: ...}` after `@fixture`",
            );
            return empty_path(self.cur.pos);
        }
        let attrs_start = self.cur.pos;
        self.cur.bump();
        let mut target: Option<ComponentPath> = None;
        loop {
            self.cur.skip_trivia();
            if self.cur.eat('}') {
                break;
            }
            if self.cur.is_eof() {
                self.error(
                    Span::new(attrs_start, self.cur.pos),
                    "unterminated `@fixture` attributes; expected `}`",
                );
                break;
            }
            let Some(key_span) = self.cur.scan_ident() else {
                self.error(
                    Span::point(self.cur.pos),
                    "expected attribute name in `@fixture { ... }`",
                );
                self.skip_fixture_attr_rest();
                continue;
            };
            let key = self.cur.ident_text(key_span).to_string();
            self.cur.skip_trivia();
            if !self.cur.eat(':') {
                self.error(key_span, "expected `:` after `@fixture` attribute name");
            }
            self.cur.skip_trivia();
            let Some(path) = self.scan_fixture_path() else {
                self.error(
                    Span::point(self.cur.pos),
                    format!("expected component path after `{key}:`"),
                );
                self.skip_fixture_attr_rest();
                continue;
            };
            match key.as_str() {
                "target" => {
                    if target.is_some() {
                        self.error(key_span, "duplicate `target` attribute");
                    } else {
                        target = Some(path);
                    }
                }
                other => {
                    self.error(
                        key_span,
                        format!("unknown `@fixture` attribute `{other}`; expected `target`"),
                    );
                }
            }
            self.cur.skip_trivia();
            self.cur.eat(',');
        }
        target.unwrap_or_else(|| {
            self.error(
                Span::new(attrs_start, self.cur.pos),
                "expected `{target: ...}` after `@fixture`",
            );
            empty_path(attrs_start)
        })
    }

    fn skip_fixture_attr_rest(&mut self) {
        while !self.cur.is_eof() {
            match self.cur.peek() {
                Some(',' | '}') => return,
                Some('"') => self.cur.skip_string(),
                _ => {
                    self.cur.bump();
                }
            }
        }
    }

    fn scan_fixture_path(&mut self) -> Option<ComponentPath> {
        let first = self.cur.scan_ident()?;
        let mut parts = vec![Ident {
            name: self.cur.ident_text(first).to_string(),
            span: first,
        }];
        while self.cur.eat('.') {
            if let Some(next) = self.cur.scan_ident() {
                parts.push(Ident {
                    name: self.cur.ident_text(next).to_string(),
                    span: next,
                });
            } else {
                self.error(Span::point(self.cur.pos), "expected identifier after `.`");
                break;
            }
        }
        let span = Span::new(
            parts.first().unwrap().span.start as usize,
            parts.last().unwrap().span.end as usize,
        );
        let roc_name = component_roc_name(&parts);
        Some(ComponentPath {
            parts,
            roc_name,
            span,
        })
    }

    fn scan_roc_expr(&mut self) -> Span {
        self.cur.skip_trivia();
        let start = self.cur.pos;
        if self.cur.is_eof() || (self.cur.is_top_level() && self.cur.peek() == Some('@')) {
            return Span::point(start);
        }

        let start_paren = self.cur.paren;
        let start_bracket = self.cur.bracket;
        let start_brace = self.cur.brace;
        let at_start_depth = |cur: &Cursor<'_>| {
            cur.paren == start_paren && cur.bracket == start_bracket && cur.brace == start_brace
        };

        loop {
            if self.cur.is_eof() {
                break;
            }
            self.cur.skip_roc_token();
            while !self.cur.is_eof() && !at_start_depth(&self.cur) {
                self.cur.skip_roc_token();
            }
            let saved = Snapshot::from(&self.cur);
            self.cur.skip_spaces_tabs();
            match self.cur.peek() {
                Some('.') => {
                    self.cur.skip_roc_token();
                    continue;
                }
                Some('(') | Some('[') => continue,
                _ => {
                    saved.restore(&mut self.cur);
                    break;
                }
            }
        }
        lexer::trim_span(self.src(), Span::new(start, self.cur.pos))
    }

    fn empty_fixture(&mut self, start: usize, target: ComponentPath) -> FixtureDecl {
        let pos = self.cur.pos;
        FixtureDecl {
            leading: None,
            name: Ident {
                span: Span::point(pos),
                name: String::new(),
            },
            target,
            value: Span::point(pos),
            span: Span::new(start, pos),
        }
    }

    fn try_parse_component(&mut self) -> Option<ComponentDecl> {
        let at = self.cur.pos;
        if self.scan_at_component().is_some() {
            let mut decl = self.parse_component_after_keyword(at);
            let (leading, span) = self.finish_leading(at, decl.span.end as usize);
            decl.leading = leading;
            decl.span = span;
            return Some(decl);
        }

        let name_span = self.cur.scan_ident()?;
        let name = self.cur.ident_text(name_span).to_string();
        self.cur.skip_trivia();
        if !self.cur.eat('=') {
            return None;
        }
        self.cur.skip_trivia();
        if self.scan_at_component().is_none() && !self.scan_bare_component_keyword() {
            return None;
        }
        self.error(
            name_span,
            "expected `@component` at the start of the declaration; write `@component Name = |params|`",
        );
        let mut decl = self.parse_component_rest(at, name_span, name);
        let (leading, span) = self.finish_leading(at, decl.span.end as usize);
        decl.leading = leading;
        decl.span = span;
        Some(decl)
    }

    fn scan_at_component(&mut self) -> Option<Span> {
        self.scan_at_keyword("component")
    }

    fn scan_at_keyword(&mut self, keyword: &str) -> Option<Span> {
        let saved = Snapshot::from(&self.cur);
        if !self.cur.eat('@') {
            return None;
        }
        let Some(kw) = self.cur.scan_ident() else {
            saved.restore(&mut self.cur);
            return None;
        };
        if self.cur.ident_text(kw) != keyword {
            saved.restore(&mut self.cur);
            return None;
        }
        Some(Span::new(saved.pos, self.cur.pos))
    }

    fn scan_bare_component_keyword(&mut self) -> bool {
        let saved = Snapshot::from(&self.cur);
        let Some(kw) = self.cur.scan_ident() else {
            return false;
        };
        if self.cur.ident_text(kw) != "component" {
            saved.restore(&mut self.cur);
            return false;
        }
        let after_kw = Snapshot::from(&self.cur);
        self.cur.skip_trivia();
        if self.cur.peek() != Some('|') {
            saved.restore(&mut self.cur);
            return false;
        }
        after_kw.restore(&mut self.cur);
        true
    }

    fn parse_component_after_keyword(&mut self, start: usize) -> ComponentDecl {
        self.cur.skip_trivia();
        let Some(name_span) = self.cur.scan_ident() else {
            self.error(
                Span::new(start, self.cur.pos),
                "expected component name after `@component`",
            );
            self.sync_to_next_top_level();
            return ComponentDecl {
                leading: None,
                name: Ident {
                    span: Span::point(self.cur.pos),
                    name: String::new(),
                },
                params: Span::point(self.cur.pos),
                body: TemplateBlock {
                    items: Vec::new(),
                    span: Span::point(self.cur.pos),
                },
                span: Span::new(start, self.cur.pos),
            };
        };
        let name = self.cur.ident_text(name_span).to_string();
        if let Some(message) = component_name_error(&name) {
            self.error(name_span, message);
        }
        self.cur.skip_trivia();
        if !self.cur.eat('=') {
            self.error(name_span, "expected `=` after component name");
        }
        self.parse_component_rest(start, name_span, name)
    }

    fn parse_component_rest(
        &mut self,
        start: usize,
        name_span: Span,
        name: String,
    ) -> ComponentDecl {
        self.cur.skip_trivia();
        let params = match self.scan_params() {
            Some(span) => span,
            None => {
                self.error(
                    Span::new(start, self.cur.pos),
                    "expected `|params|` after `@component Name =`",
                );
                self.sync_to_next_top_level();
                return ComponentDecl {
                    leading: None,
                    name: Ident {
                        span: name_span,
                        name,
                    },
                    params: Span::point(self.cur.pos),
                    body: TemplateBlock {
                        items: Vec::new(),
                        span: Span::point(self.cur.pos),
                    },
                    span: Span::new(start, self.cur.pos),
                };
            }
        };

        self.cur.skip_trivia();
        let body = self.parse_component_body();
        ComponentDecl {
            leading: None,
            span: Span::new(start, self.cur.pos),
            name: Ident {
                span: name_span,
                name,
            },
            params,
            body,
        }
    }

    fn parse_component_body(&mut self) -> TemplateBlock {
        self.skip_body_prefix();
        if self.cur.peek() == Some('{') {
            return self.parse_template_block();
        }
        if self.cur.peek() == Some('<') {
            return self.parse_html_expr_body();
        }
        self.error(
            Span::point(self.cur.pos),
            "expected `{` to open a template body, or a single HTML tag",
        );
        TemplateBlock {
            items: Vec::new(),
            span: Span::point(self.cur.pos),
        }
    }

    fn skip_body_prefix(&mut self) {
        loop {
            self.cur.skip_trivia();
            if self.cur.starts_with("<!--") {
                let start = self.cur.pos;
                if !self.cur.skip_html_comment()
                    || !self.src()[start..self.cur.pos].ends_with("-->")
                {
                    self.error(Span::new(start, self.cur.pos), "unterminated HTML comment");
                }
                continue;
            }
            break;
        }
    }

    fn parse_html_expr_body(&mut self) -> TemplateBlock {
        let start = self.cur.pos;
        let Some(item) = self.parse_tag() else {
            return TemplateBlock {
                items: Vec::new(),
                span: Span::point(start),
            };
        };
        let end = self.cur.pos;
        let saved = Snapshot::from(&self.cur);
        self.skip_body_prefix();
        if !self.cur.is_eof() && !self.at_column_zero_def() {
            self.error(
                Span::point(self.cur.pos),
                "braceless component bodies may contain one HTML tag; wrap multiple items in `{ ... }`",
            );
            self.sync_to_next_top_level();
            return TemplateBlock {
                items: vec![item],
                span: Span::new(start, self.cur.pos),
            };
        }
        saved.restore(&mut self.cur);
        TemplateBlock {
            items: vec![item],
            span: Span::new(start, end),
        }
    }

    fn scan_params(&mut self) -> Option<Span> {
        if self.cur.peek() != Some('|') {
            return None;
        }
        let start = self.cur.pos;
        self.cur.bump();
        let mut paren = 0usize;
        let mut bracket = 0usize;
        let mut brace = 0usize;
        while !self.cur.is_eof() {
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => self.cur.skip_comment(),
                Some('(') => {
                    self.cur.bump();
                    paren += 1;
                }
                Some(')') => {
                    self.cur.bump();
                    paren = paren.saturating_sub(1);
                }
                Some('[') => {
                    self.cur.bump();
                    bracket += 1;
                }
                Some(']') => {
                    self.cur.bump();
                    bracket = bracket.saturating_sub(1);
                }
                Some('{') => {
                    self.cur.bump();
                    brace += 1;
                }
                Some('}') => {
                    self.cur.bump();
                    brace = brace.saturating_sub(1);
                }
                Some('|') if paren == 0 && bracket == 0 && brace == 0 => {
                    let after_pipe = self.cur.pos + 1;
                    let mut look = Cursor::new(self.src());
                    look.pos = after_pipe;
                    look.skip_trivia();
                    if matches!(look.peek(), Some('{') | Some('<')) || look.peek().is_none() {
                        self.cur.bump();
                        return Some(Span::new(start, self.cur.pos));
                    }
                    self.cur.bump();
                }
                Some(_) => {
                    self.cur.bump();
                }
                None => break,
            }
        }
        self.error(
            Span::new(start, self.cur.pos),
            "unterminated component parameter list",
        );
        Some(Span::new(start, self.cur.pos))
    }

    fn parse_template_block(&mut self) -> TemplateBlock {
        let start = self.cur.pos;
        if !self.cur.eat('{') {
            self.error(
                Span::point(self.cur.pos),
                "expected `{` to open a template body",
            );
            return TemplateBlock {
                items: Vec::new(),
                span: Span::point(start),
            };
        }
        let items = self.parse_template_items(ItemStop::BlockEnd);
        if !self.cur.eat('}') {
            self.error(
                Span::new(start, self.cur.pos),
                "unclosed template block; expected `}`",
            );
            self.sync_to_next_top_level();
        }
        TemplateBlock {
            items,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn parse_template_items(&mut self, stop: ItemStop) -> Vec<TemplateItem> {
        let mut items = Vec::new();
        loop {
            self.cur.skip_formatting_ws();
            if self.cur.is_eof() {
                break;
            }
            if stop == ItemStop::BlockEnd && self.cur.peek() == Some('}') {
                break;
            }
            if stop == ItemStop::CloseTag && self.cur.starts_with("</") {
                break;
            }
            if self.cur.peek() == Some('#') {
                self.cur.skip_comment();
                continue;
            }
            if self.cur.starts_with("<!--") {
                let start = self.cur.pos;
                if !self.cur.skip_html_comment()
                    || !self.src()[start..self.cur.pos].ends_with("-->")
                {
                    self.error(Span::new(start, self.cur.pos), "unterminated HTML comment");
                }
                continue;
            }
            if matches!(self.cur.peek(), Some('<' | '{' | '@')) {
                if let Some(item) = self.parse_template_item(stop) {
                    items.push(item);
                } else {
                    break;
                }
                continue;
            }
            if let Some(text) = self.scan_text() {
                if keep_text(&text.value) {
                    items.push(TemplateItem::Text(text));
                }
                continue;
            }
            break;
        }
        trim_block_ws(&mut items);
        items
    }

    fn parse_template_item(&mut self, stop: ItemStop) -> Option<TemplateItem> {
        match self.cur.peek() {
            Some('<') => self.parse_tag(),
            Some('{') => Some(TemplateItem::Interpolation(self.parse_interpolation())),
            Some('@') => self.parse_directive(stop),
            _ => None,
        }
    }

    fn parse_match_value(&mut self) -> Option<TemplateItem> {
        self.cur.skip_whitespace();
        if self.cur.peek() == Some('#') {
            self.cur.skip_comment();
            self.cur.skip_whitespace();
        }
        match self.cur.peek() {
            Some('<') | Some('{') | Some('@') => self.parse_template_item(ItemStop::MatchValue),
            Some(_) => {
                self.error(
                    Span::point(self.cur.pos),
                    "match arm must produce a tag, fragment, interpolation, or directive; wrap bare text in an element or fragment",
                );
                self.skip_until_match_sync();
                None
            }
            None => None,
        }
    }

    fn parse_tag(&mut self) -> Option<TemplateItem> {
        let start = self.cur.pos;
        if !self.cur.eat('<') {
            return None;
        }

        if self.cur.eat('>') {
            let children = self.parse_template_items(ItemStop::CloseTag);
            if !self.cur.eat_str("</>") {
                self.error(
                    Span::new(start, self.cur.pos),
                    "unclosed fragment; expected `</>`",
                );
            }
            return Some(TemplateItem::Fragment(Fragment {
                children,
                span: Span::new(start, self.cur.pos),
            }));
        }

        if self.cur.eat('/') {
            let name = self.scan_tag_path();
            self.cur.skip_spaces_tabs();
            self.cur.eat('>');
            let label = name
                .as_ref()
                .map(path_source)
                .unwrap_or_else(|| "tag".to_string());
            self.error(
                Span::new(start, self.cur.pos),
                format!("unexpected closing tag `</{label}>`"),
            );
            return None;
        }

        let Some(path) = self.scan_tag_path() else {
            self.error(Span::point(self.cur.pos), "expected tag name after `<`");
            self.sync_after_bad_tag();
            return None;
        };

        let attrs = self.parse_attrs();
        let self_closing = self.cur.eat('/');
        if !self.cur.eat('>') {
            self.error(
                Span::new(start, self.cur.pos),
                "expected `>` to end the opening tag",
            );
        }

        let is_component = path.parts.first().is_some_and(|part| {
            part.name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
        });

        if self_closing {
            return Some(if is_component {
                TemplateItem::ComponentCall(ComponentCall {
                    path,
                    attrs,
                    children: None,
                    span: Span::new(start, self.cur.pos),
                })
            } else {
                TemplateItem::Element(Element {
                    name: path.parts.into_iter().next().unwrap_or(Ident {
                        span: Span::point(start),
                        name: String::new(),
                    }),
                    attrs,
                    children: Vec::new(),
                    self_closing: true,
                    span: Span::new(start, self.cur.pos),
                })
            });
        }

        if !is_component && is_void_element(&path.parts[0].name) {
            return Some(TemplateItem::Element(Element {
                name: path.parts.into_iter().next().unwrap(),
                attrs,
                children: Vec::new(),
                self_closing: true,
                span: Span::new(start, self.cur.pos),
            }));
        }

        let children = self.parse_template_items(ItemStop::CloseTag);
        if !self.eat_closing_tag(&path) {
            self.error(
                Span::new(start, self.cur.pos),
                format!(
                    "unclosed `<{}>`; expected `</{}>`",
                    path_source(&path),
                    path_source(&path)
                ),
            );
        }

        Some(if is_component {
            TemplateItem::ComponentCall(ComponentCall {
                path,
                attrs,
                children: Some(children),
                span: Span::new(start, self.cur.pos),
            })
        } else {
            TemplateItem::Element(Element {
                name: path.parts.into_iter().next().unwrap(),
                attrs,
                children,
                self_closing: false,
                span: Span::new(start, self.cur.pos),
            })
        })
    }

    fn scan_tag_path(&mut self) -> Option<ComponentPath> {
        let first = self.cur.scan_tag_name()?;
        let mut parts = vec![Ident {
            name: self.cur.ident_text(first).to_string(),
            span: first,
        }];
        while self.cur.eat('.') {
            if let Some(next) = self.cur.scan_tag_name() {
                parts.push(Ident {
                    name: self.cur.ident_text(next).to_string(),
                    span: next,
                });
            } else {
                self.error(Span::point(self.cur.pos), "expected identifier after `.`");
                break;
            }
        }
        let span = Span::new(
            parts.first().unwrap().span.start as usize,
            parts.last().unwrap().span.end as usize,
        );
        if parts
            .last()
            .is_some_and(|part| is_ambiguous_pascal(&part.name))
        {
            self.error(
                parts.last().unwrap().span,
                format!(
                    "ambiguous component tag `<{}>`; write `<HtmlShell>` rather than `<HTMLShell>`",
                    parts.last().unwrap().name
                ),
            );
        }
        let roc_name = component_roc_name(&parts);
        Some(ComponentPath {
            parts,
            roc_name,
            span,
        })
    }

    fn parse_attrs(&mut self) -> Vec<Attr> {
        let mut attrs = Vec::new();
        loop {
            self.cur.skip_spaces_tabs();
            if self.cur.peek() == Some('\n') || self.cur.peek() == Some('\r') {
                self.cur.skip_formatting_ws();
            }
            if matches!(self.cur.peek(), Some('>' | '/') | None) {
                break;
            }
            let Some(name_span) = self.cur.scan_attr_name() else {
                break;
            };
            let name = Ident {
                name: self.cur.ident_text(name_span).to_string(),
                span: name_span,
            };
            self.cur.skip_spaces_tabs();
            let (value, end) = if self.cur.eat('=') {
                self.cur.skip_spaces_tabs();
                if self.cur.peek() == Some('{') {
                    let interp = self.parse_interpolation();
                    (AttrValue::Expr { expr: interp.expr }, interp.span.end)
                } else if self.cur.peek() == Some('"') {
                    let (span, value) = self.scan_quoted_string();
                    (AttrValue::Static { span, value }, span.end)
                } else if self.cur.peek() == Some('@') {
                    self.parse_action_attr()
                } else {
                    self.error(
                        Span::point(self.cur.pos),
                        format!(
                            "expected `\"...\"`, `{{...}}`, or a Datastar action such as `@post(\"...\")` for attribute `{}`",
                            name.name
                        ),
                    );
                    (AttrValue::Boolean, name_span.end)
                }
            } else {
                (AttrValue::Boolean, name_span.end)
            };
            attrs.push(Attr {
                span: Span::new(name_span.start as usize, end as usize),
                name,
                value,
            });
        }
        attrs
    }

    fn parse_action_attr(&mut self) -> (AttrValue, u32) {
        let start = self.cur.pos;
        self.cur.bump();
        let Some(name_span) = self.cur.scan_ident() else {
            self.error(
                Span::point(start),
                "expected Datastar action name after `@`",
            );
            return (AttrValue::Boolean, start as u32);
        };
        let name = Ident {
            name: self.cur.ident_text(name_span).to_string(),
            span: name_span,
        };
        if !is_datastar_action(&name.name) {
            self.error(
                Span::new(start, name_span.end as usize),
                format!(
                    "unknown Datastar action `@{}`; expected `@get`, `@post`, `@put`, `@patch`, or `@delete`",
                    name.name
                ),
            );
        }
        self.cur.skip_spaces_tabs();
        if self.cur.peek() != Some('(') {
            self.error(
                Span::point(self.cur.pos),
                format!("expected `(` after `@{}`", name.name),
            );
            return (
                AttrValue::Action {
                    name,
                    args: Span::point(self.cur.pos),
                },
                self.cur.pos as u32,
            );
        }
        let args = self.scan_call_args();
        let end = self.cur.pos as u32;
        let trimmed = args.of(self.src()).trim();
        if trimmed.is_empty() {
            self.error(
                Span::new(start, self.cur.pos),
                format!("expected a URI argument in `@{}(...)`", name.name),
            );
        } else if first_non_trivia_char(trimmed) == Some('\'') {
            self.error(
                args,
                format!(
                    "Datastar actions in Rocci use Roc strings: `@{}(\"/x\")`. For a literal Datastar expression, quote the whole attribute: `data-on:click=\"@{}('/x')\"`",
                    name.name, name.name
                ),
            );
        }
        (AttrValue::Action { name, args }, end)
    }

    fn scan_call_args(&mut self) -> Span {
        let paren_before = self.cur.paren;
        self.cur.skip_roc_token();
        let args_start = self.cur.pos;
        while !self.cur.is_eof() && self.cur.paren > paren_before {
            self.cur.skip_roc_token();
        }
        if self.cur.paren != paren_before {
            self.error(
                Span::new(args_start.saturating_sub(1), self.cur.pos),
                "unterminated Datastar action; expected `)`",
            );
            return lexer::trim_span(self.src(), Span::new(args_start, self.cur.pos));
        }
        let args_end = self.cur.pos.saturating_sub(')'.len_utf8());
        lexer::trim_span(self.src(), Span::new(args_start, args_end.max(args_start)))
    }

    fn scan_quoted_string(&mut self) -> (Span, String) {
        let start = self.cur.pos;
        if !self.cur.eat('"') {
            return (Span::point(start), String::new());
        }
        let mut value = String::new();
        while let Some(ch) = self.cur.peek() {
            if ch == '"' {
                self.cur.bump();
                break;
            }
            if ch == '\\' {
                self.cur.bump();
                if let Some(escaped) = self.cur.bump() {
                    value.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        other => other,
                    });
                }
                continue;
            }
            value.push(ch);
            self.cur.bump();
        }
        (Span::new(start, self.cur.pos), value)
    }

    fn eat_closing_tag(&mut self, open: &ComponentPath) -> bool {
        self.cur.skip_formatting_ws();
        if !self.cur.starts_with("</") {
            return false;
        }
        let start = self.cur.pos;
        self.cur.eat_str("</");
        if self.cur.eat('>') {
            return false;
        }
        let Some(close) = self.scan_tag_path() else {
            self.error(Span::point(self.cur.pos), "expected closing tag name");
            return false;
        };
        self.cur.skip_spaces_tabs();
        self.cur.eat('>');
        if path_source(&close) != path_source(open) {
            self.error(
                Span::new(start, self.cur.pos),
                format!(
                    "closing tag `</{}>` does not match `<{}>`",
                    path_source(&close),
                    path_source(open)
                ),
            );
        }
        true
    }

    fn parse_interpolation(&mut self) -> Interpolation {
        let start = self.cur.pos;
        self.cur.bump();
        let expr_start = self.cur.pos;
        let mut depth = 1usize;
        while !self.cur.is_eof() && depth > 0 {
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => self.cur.skip_comment(),
                Some('{') => {
                    self.cur.bump();
                    depth += 1;
                }
                Some('}') => {
                    if depth == 1 {
                        let expr =
                            lexer::trim_span(self.src(), Span::new(expr_start, self.cur.pos));
                        self.cur.bump();
                        return Interpolation {
                            expr,
                            span: Span::new(start, self.cur.pos),
                        };
                    }
                    self.cur.bump();
                    depth -= 1;
                }
                _ => {
                    self.cur.bump();
                }
            }
        }
        self.error(
            Span::new(start, self.cur.pos),
            "unterminated interpolation; expected `}`",
        );
        Interpolation {
            expr: lexer::trim_span(self.src(), Span::new(expr_start, self.cur.pos)),
            span: Span::new(start, self.cur.pos),
        }
    }

    fn parse_directive(&mut self, stop: ItemStop) -> Option<TemplateItem> {
        let start = self.cur.pos;
        if self.cur.starts_with("@@") {
            self.cur.pos += 2;
            let mut value = String::from("@");
            while let Some(ch) = self.cur.peek() {
                if matches!(ch, '<' | '{' | '@' | '}') || ch == '\n' {
                    break;
                }
                value.push(ch);
                self.cur.bump();
            }
            return Some(TemplateItem::Text(TextNode {
                span: Span::new(start, self.cur.pos),
                value,
            }));
        }
        self.cur.bump();
        let Some(name_span) = self.cur.scan_ident() else {
            self.error(Span::point(start), "expected directive name after `@`");
            return None;
        };
        let name = self.cur.ident_text(name_span);
        match name {
            "if" => Some(TemplateItem::If(self.parse_if(start))),
            "for" => Some(TemplateItem::For(self.parse_for(start))),
            "match" => Some(TemplateItem::Match(self.parse_match(start))),
            "let" => Some(TemplateItem::Let(self.parse_let(start))),
            "css" => Some(TemplateItem::Css(self.parse_css_after_keyword(start))),
            "component" | "fixture" | "context" | "init" | "live" | "view" | "patch"
            | "command" | "on" | "action" | "page" | "roc" | "render" => {
                self.error(
                    Span::new(start, name_span.end as usize),
                    format!("`@{name}` is only valid at document root, not inside a template body"),
                );
                self.skip_unknown_directive();
                None
            }
            "else" => {
                if stop == ItemStop::BlockEnd {
                    self.cur.pos = start;
                    return None;
                }
                self.error(
                    Span::new(start, name_span.end as usize),
                    "`@else` is only valid after an `@if` body",
                );
                None
            }
            other => {
                let suggestion = suggest_directive(other);
                let message = if let Some(known) = suggestion {
                    format!("unknown directive `@{other}`; did you mean `@{known}`?")
                } else {
                    format!("unknown directive `@{other}`")
                };
                self.error(Span::new(start, name_span.end as usize), message);
                self.skip_unknown_directive();
                None
            }
        }
    }

    fn parse_if(&mut self, start: usize) -> IfDirective {
        let condition = self.scan_header_expr();
        let then_body = self.parse_template_block();
        let mut else_ifs = Vec::new();
        let mut else_body = None;
        loop {
            let saved = Snapshot::from(&self.cur);
            self.cur.skip_trivia();
            if !self.cur.starts_with("@else") {
                saved.restore(&mut self.cur);
                break;
            }
            self.cur.eat_str("@else");
            self.cur.skip_spaces_tabs();
            if self.try_eat_ident("if") {
                let cond = self.scan_header_expr();
                let body = self.parse_template_block();
                else_ifs.push((cond, body));
            } else {
                self.cur.skip_trivia();
                else_body = Some(self.parse_template_block());
                break;
            }
        }
        IfDirective {
            condition,
            then_body,
            else_ifs,
            else_body,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn parse_for(&mut self, start: usize) -> ForDirective {
        self.cur.skip_whitespace();
        let Some(binder_span) = self.cur.scan_ident() else {
            self.error(Span::point(self.cur.pos), "expected binder after `@for`");
            return ForDirective {
                binder: Ident {
                    span: Span::point(self.cur.pos),
                    name: "_".into(),
                },
                collection: Span::point(self.cur.pos),
                body: self.parse_template_block(),
                span: Span::new(start, self.cur.pos),
            };
        };
        let binder_name = self.cur.ident_text(binder_span).to_string();
        if binder_name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            self.error(
                binder_span,
                "`@for` binders must be a single lowercase identifier",
            );
        }
        self.cur.skip_whitespace();
        if !self.try_eat_ident("in") {
            self.error(
                Span::point(self.cur.pos),
                "expected `in` after `@for` binder",
            );
        }
        let collection = self.scan_header_expr();
        let body = self.parse_template_block();
        ForDirective {
            binder: Ident {
                span: binder_span,
                name: binder_name,
            },
            collection,
            body,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn parse_match(&mut self, start: usize) -> MatchDirective {
        let scrutinee = self.scan_header_expr();
        self.cur.skip_trivia();
        if !self.cur.eat('{') {
            self.error(
                Span::point(self.cur.pos),
                "expected `{` to open `@match` arms",
            );
            return MatchDirective {
                scrutinee,
                arms: Vec::new(),
                span: Span::new(start, self.cur.pos),
            };
        }
        let mut arms = Vec::new();
        loop {
            self.cur.skip_formatting_ws();
            if self.cur.peek() == Some('#') {
                self.cur.skip_comment();
                continue;
            }
            if self.cur.peek() == Some('}') || self.cur.is_eof() {
                break;
            }
            let arm_start = self.cur.pos;
            let Some(pattern) = self.scan_pattern() else {
                self.error(Span::point(self.cur.pos), "expected match pattern");
                self.skip_until_match_sync();
                if self.cur.peek() == Some('}') {
                    break;
                }
                continue;
            };
            self.cur.skip_whitespace();
            if !self.cur.eat_str("=>") {
                self.error(
                    Span::point(self.cur.pos),
                    "expected `=>` after match pattern",
                );
            }
            let Some(value) = self.parse_match_value() else {
                continue;
            };
            arms.push(MatchArm {
                pattern,
                span: Span::new(arm_start, self.cur.pos),
                value: Box::new(value),
            });
        }
        if !self.cur.eat('}') {
            self.error(
                Span::new(start, self.cur.pos),
                "unclosed `@match`; expected `}`",
            );
        }
        MatchDirective {
            scrutinee,
            arms,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn parse_let(&mut self, start: usize) -> LetDirective {
        self.cur.skip_whitespace();
        let Some(binder_span) = self.cur.scan_ident() else {
            self.error(Span::point(self.cur.pos), "expected binder after `@let`");
            return LetDirective {
                binder: Ident {
                    span: Span::point(self.cur.pos),
                    name: "_".into(),
                },
                expr: Span::point(self.cur.pos),
                span: Span::new(start, self.cur.pos),
            };
        };
        self.cur.skip_whitespace();
        if !self.cur.eat('=') {
            self.error(
                Span::point(self.cur.pos),
                "expected `=` after `@let` binder",
            );
        }
        let expr = self.scan_line_expr();
        LetDirective {
            binder: Ident {
                span: binder_span,
                name: self.cur.ident_text(binder_span).to_string(),
            },
            expr,
            span: Span::new(start, self.cur.pos),
        }
    }

    fn scan_header_expr(&mut self) -> Span {
        self.cur.skip_whitespace();
        let start = self.cur.pos;
        let mut paren = 0usize;
        let mut bracket = 0usize;
        while !self.cur.is_eof() {
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => self.cur.skip_comment(),
                Some('(') => {
                    self.cur.bump();
                    paren += 1;
                }
                Some(')') => {
                    self.cur.bump();
                    paren = paren.saturating_sub(1);
                }
                Some('[') => {
                    self.cur.bump();
                    bracket += 1;
                }
                Some(']') => {
                    self.cur.bump();
                    bracket = bracket.saturating_sub(1);
                }
                Some('{') if paren == 0 && bracket == 0 => {
                    let expr = lexer::trim_span(self.src(), Span::new(start, self.cur.pos));
                    if expr.is_empty() {
                        self.error(
                            Span::point(self.cur.pos),
                            "unparenthesized record in directive header; wrap it in parentheses, e.g. `@match ({ status, items })`",
                        );
                    }
                    return expr;
                }
                Some('{') => self.cur.skip_balanced_braces(),
                Some('\n' | '\r') if paren == 0 && bracket == 0 => {
                    self.error(
                        Span::new(start, self.cur.pos),
                        "directive header must keep its body `{` on the same logical line; parenthesize expressions that need a newline",
                    );
                    return lexer::trim_span(self.src(), Span::new(start, self.cur.pos));
                }
                Some(_) => {
                    self.cur.bump();
                }
                None => break,
            }
        }
        self.error(
            Span::new(start, self.cur.pos),
            "expected `{` to open the directive body",
        );
        lexer::trim_span(self.src(), Span::new(start, self.cur.pos))
    }

    fn scan_line_expr(&mut self) -> Span {
        self.cur.skip_spaces_tabs();
        let start = self.cur.pos;
        let mut paren = 0usize;
        let mut bracket = 0usize;
        while !self.cur.is_eof() {
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => break,
                Some('(') => {
                    self.cur.bump();
                    paren += 1;
                }
                Some(')') => {
                    self.cur.bump();
                    paren = paren.saturating_sub(1);
                }
                Some('[') => {
                    self.cur.bump();
                    bracket += 1;
                }
                Some(']') => {
                    self.cur.bump();
                    bracket = bracket.saturating_sub(1);
                }
                Some('{') => self.cur.skip_balanced_braces(),
                Some('\n' | '\r') if paren == 0 && bracket == 0 => break,
                Some(_) => {
                    self.cur.bump();
                }
                None => break,
            }
        }
        lexer::trim_span(self.src(), Span::new(start, self.cur.pos))
    }

    fn scan_pattern(&mut self) -> Option<Span> {
        self.cur.skip_formatting_ws();
        let start = self.cur.pos;
        let mut paren = 0usize;
        let mut bracket = 0usize;
        let mut brace = 0usize;
        if self.cur.peek() == Some('}') {
            return None;
        }
        while !self.cur.is_eof() {
            match self.cur.peek() {
                Some('"') => self.cur.skip_string(),
                Some('#') => self.cur.skip_comment(),
                Some('(') => {
                    self.cur.bump();
                    paren += 1;
                }
                Some(')') => {
                    self.cur.bump();
                    paren = paren.saturating_sub(1);
                }
                Some('[') => {
                    self.cur.bump();
                    bracket += 1;
                }
                Some(']') => {
                    self.cur.bump();
                    bracket = bracket.saturating_sub(1);
                }
                Some('{') => {
                    self.cur.bump();
                    brace += 1;
                }
                Some('}') if paren == 0 && bracket == 0 && brace == 0 => {
                    if self.cur.pos == start {
                        return None;
                    }
                    break;
                }
                Some('}') => {
                    self.cur.bump();
                    brace = brace.saturating_sub(1);
                }
                Some('=')
                    if paren == 0 && bracket == 0 && brace == 0 && self.cur.starts_with("=>") =>
                {
                    return Some(lexer::trim_span(self.src(), Span::new(start, self.cur.pos)));
                }
                Some(_) => {
                    self.cur.bump();
                }
                None => break,
            }
        }
        if self.cur.pos == start {
            None
        } else {
            Some(lexer::trim_span(self.src(), Span::new(start, self.cur.pos)))
        }
    }

    fn scan_text(&mut self) -> Option<TextNode> {
        let start = self.cur.pos;
        let mut value = String::new();
        while let Some(ch) = self.cur.peek() {
            if matches!(ch, '<' | '{' | '}' | '@') {
                break;
            }
            value.push(ch);
            self.cur.bump();
        }
        if self.cur.pos == start {
            return None;
        }
        Some(TextNode {
            span: Span::new(start, self.cur.pos),
            value,
        })
    }

    fn try_eat_ident(&mut self, expected: &str) -> bool {
        let saved = Snapshot::from(&self.cur);
        if let Some(span) = self.cur.scan_ident()
            && self.cur.ident_text(span) == expected
        {
            return true;
        }
        saved.restore(&mut self.cur);
        false
    }

    fn skip_until_match_sync(&mut self) {
        while !self.cur.is_eof() {
            if self.cur.starts_with("=>") {
                self.cur.pos += 2;
                return;
            }
            if self.cur.peek() == Some('}') {
                return;
            }
            if self.cur.peek() == Some('"') {
                self.cur.skip_string();
                continue;
            }
            self.cur.bump();
        }
    }

    fn skip_unknown_directive(&mut self) {
        while !self.cur.is_eof() {
            if matches!(self.cur.peek(), Some('{' | '<' | '}' | '@')) {
                if self.cur.peek() == Some('{') {
                    self.cur.skip_balanced_braces();
                }
                return;
            }
            self.cur.bump();
        }
    }

    fn sync_after_bad_tag(&mut self) {
        while !self.cur.is_eof() {
            if matches!(self.cur.peek(), Some('>' | '<' | '}' | '@')) {
                self.cur.eat('>');
                return;
            }
            self.cur.bump();
        }
    }

    fn sync_to_next_top_level(&mut self) {
        while !self.cur.is_eof() {
            if self.at_column_zero_def() {
                return;
            }
            self.cur.bump();
        }
    }

    fn at_column_zero_def(&self) -> bool {
        if self.cur.pos == 0 {
            return false;
        }
        if self.src().as_bytes().get(self.cur.pos.wrapping_sub(1)) != Some(&b'\n') {
            return false;
        }
        let mut look = Cursor::new(self.src());
        look.pos = self.cur.pos;
        if look.peek().is_some_and(|ch| ch == ' ' || ch == '\t') {
            return false;
        }
        if look.eat('@') {
            let Some(kw) = look.scan_ident() else {
                return false;
            };
            return matches!(
                look.ident_text(kw),
                "component"
                    | "fixture"
                    | "css"
                    | "context"
                    | "init"
                    | "live"
                    | "view"
                    | "patch"
                    | "command"
                    | "on"
                    | "action"
            );
        }
        let Some(_) = look.scan_ident() else {
            return false;
        };
        look.skip_trivia();
        look.peek() == Some('=')
    }

    fn at_column_zero_at_decl(&self) -> bool {
        if self.cur.pos == 0 {
            return false;
        }
        if self.src().as_bytes().get(self.cur.pos.wrapping_sub(1)) != Some(&b'\n') {
            return false;
        }
        let mut look = Cursor::new(self.src());
        look.pos = self.cur.pos;
        if look.peek().is_some_and(|ch| ch == ' ' || ch == '\t') {
            return false;
        }
        if !look.eat('@') {
            return false;
        }
        let Some(kw) = look.scan_ident() else {
            return false;
        };
        matches!(
            look.ident_text(kw),
            "component"
                | "fixture"
                | "css"
                | "context"
                | "init"
                | "live"
                | "view"
                | "patch"
                | "command"
                | "on"
                | "action"
        )
    }

    fn finish_leading(&self, at: usize, end: usize) -> (Option<LeadingComments>, Span) {
        match lexer::leading_comments_before(self.src(), at) {
            Some(leading) => {
                let span = Span::new(leading.span.start as usize, end);
                (Some(leading), span)
            }
            None => (None, Span::new(at, end)),
        }
    }

    fn attach_route_leading(&self, at: usize, item: ModuleItem) -> ModuleItem {
        match item {
            ModuleItem::View(mut decl) => {
                let (leading, span) = self.finish_leading(at, decl.span.end as usize);
                decl.leading = leading;
                decl.span = span;
                ModuleItem::View(decl)
            }
            ModuleItem::Patch(mut decl) => {
                let (leading, span) = self.finish_leading(at, decl.span.end as usize);
                decl.leading = leading;
                decl.span = span;
                ModuleItem::Patch(decl)
            }
            ModuleItem::Command(mut decl) => {
                let (leading, span) = self.finish_leading(at, decl.span.end as usize);
                decl.leading = leading;
                decl.span = span;
                ModuleItem::Command(decl)
            }
            other => other,
        }
    }
}

fn empty_path(pos: usize) -> ComponentPath {
    ComponentPath {
        parts: Vec::new(),
        roc_name: String::new(),
        span: Span::point(pos),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RouteKind {
    View,
    Patch,
    Command,
}

impl RouteKind {
    fn noun(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Patch => "patch",
            Self::Command => "command",
        }
    }

    fn header_without_path(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Patch => "patch",
            Self::Command => "command",
        }
    }

    fn to_item(self, parsed: ParsedRoute) -> ModuleItem {
        match self {
            Self::View => ModuleItem::View(ViewDecl {
                leading: None,
                path: parsed.path,
                path_span: parsed.path_span,
                params: parsed.params,
                body: parsed.body,
                span: parsed.span,
            }),
            Self::Patch => ModuleItem::Patch(PatchDecl {
                leading: None,
                method: parsed.method,
                path: parsed.path,
                path_span: parsed.path_span,
                params: parsed.params,
                body: parsed.body,
                span: parsed.span,
            }),
            Self::Command => ModuleItem::Command(CommandDecl {
                leading: None,
                method: parsed.method,
                path: parsed.path,
                path_span: parsed.path_span,
                params: parsed.params,
                body: parsed.body,
                span: parsed.span,
            }),
        }
    }
}

struct ParsedRoute {
    method: Option<Ident>,
    path: String,
    path_span: Span,
    params: Option<Span>,
    body: Span,
    span: Span,
}

fn removed_on_rewrite(method: &str, json: bool, path: &str) -> String {
    let quoted = if path.is_empty() {
        "path".to_string()
    } else {
        format!("\"{path}\"")
    };
    match (method, json) {
        ("get", _) => format!("`@view({quoted})`"),
        ("post" | "", false) => format!("`@patch({quoted})`"),
        ("post" | "", true) => {
            format!("`@command({quoted})` and return Roc data (do not call `Json.to_str`)")
        }
        ("put" | "patch" | "delete", false) => format!("`@patch:{method}({quoted})`"),
        ("put" | "patch" | "delete", true) => {
            format!("`@command:{method}({quoted})` and return Roc data (do not call `Json.to_str`)")
        }
        (_, true) => {
            format!("`@command({quoted})` and return Roc data (do not call `Json.to_str`)")
        }
        _ => format!("`@view({quoted})`, `@patch({quoted})`, or `@command({quoted})`"),
    }
}

fn string_literal(text: &str, span: Span) -> Option<(String, Span)> {
    let trimmed = text.trim();
    let start_off = text.find(trimmed).unwrap_or(0);
    if !trimmed.starts_with('"') || !trimmed.ends_with('"') || trimmed.len() < 2 {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    let lit_start = span.start as usize + start_off;
    Some((out, Span::new(lit_start, lit_start + trimmed.len())))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemStop {
    BlockEnd,
    MatchValue,
    CloseTag,
}

#[derive(Clone, Copy)]
struct Snapshot {
    pos: usize,
    paren: usize,
    bracket: usize,
    brace: usize,
}

impl Snapshot {
    fn from(cur: &Cursor<'_>) -> Self {
        Self {
            pos: cur.pos,
            paren: cur.paren,
            bracket: cur.bracket,
            brace: cur.brace,
        }
    }

    fn restore(self, cur: &mut Cursor<'_>) {
        cur.pos = self.pos;
        cur.paren = self.paren;
        cur.bracket = self.bracket;
        cur.brace = self.brace;
    }
}

fn keep_text(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if value.chars().all(|ch| ch.is_whitespace()) && value.contains('\n') {
        return false;
    }
    true
}

fn trim_block_ws(items: &mut Vec<TemplateItem>) {
    while items.first().is_some_and(|item| {
        matches!(item, TemplateItem::Text(text) if text.value.chars().all(char::is_whitespace))
    }) {
        items.remove(0);
    }
    while items.last().is_some_and(|item| {
        matches!(item, TemplateItem::Text(text) if text.value.chars().all(char::is_whitespace))
    }) {
        items.pop();
    }
}

fn path_source(path: &ComponentPath) -> String {
    path.parts
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_datastar_action(name: &str) -> bool {
    matches!(name, "get" | "post" | "put" | "patch" | "delete")
}

fn first_non_trivia_char(text: &str) -> Option<char> {
    let mut rest = text.trim_start();
    loop {
        if rest.starts_with('#') {
            rest = rest.split_once('\n').map(|(_, after)| after).unwrap_or("");
            rest = rest.trim_start();
            continue;
        }
        return rest.chars().next();
    }
}

fn suggest_directive(name: &str) -> Option<&'static str> {
    const KNOWN: [&str; 13] = [
        "if", "for", "match", "let", "else", "css", "context", "init", "live", "view", "patch",
        "command", "on",
    ];
    KNOWN
        .into_iter()
        .find(|known| levenshtein(name, known) <= 2 && name != *known)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}
