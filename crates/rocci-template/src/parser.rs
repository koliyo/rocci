use crate::ast::{
    Attr, AttrValue, ComponentCall, ComponentDecl, ComponentPath, Document, Element, ForDirective,
    Fragment, Ident, IfDirective, Interpolation, LetDirective, MatchArm, MatchDirective,
    ModuleItem, TemplateBlock, TemplateItem, TextNode,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{self, Cursor};
use crate::resolve::{component_roc_name, is_ambiguous_pascal};
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

            if self.cur.is_top_level()
                && let Some(component) = self.try_parse_component()
            {
                if opaque_start < component.name.span.start as usize {
                    items.push(ModuleItem::Roc {
                        span: Span::new(opaque_start, component.name.span.start as usize),
                    });
                }
                opaque_start = self.cur.pos;
                items.push(ModuleItem::Component(component));
                continue;
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

    fn try_parse_component(&mut self) -> Option<ComponentDecl> {
        let name_span = self.cur.scan_ident()?;
        let name = self.cur.ident_text(name_span).to_string();
        self.cur.skip_trivia();
        if !self.cur.eat('=') {
            return None;
        }
        self.cur.skip_trivia();
        let kw = self.scan_component_keyword()?;

        self.cur.skip_trivia();
        let params = match self.scan_params() {
            Some(span) => span,
            None => {
                self.error(kw, "expected `|params|` after `@component`");
                self.sync_to_next_top_level();
                return Some(ComponentDecl {
                    name: Ident {
                        span: name_span,
                        name,
                    },
                    params: Span::point(self.cur.pos),
                    body: TemplateBlock {
                        items: Vec::new(),
                        span: Span::point(self.cur.pos),
                    },
                    span: Span::new(name_span.start as usize, self.cur.pos),
                });
            }
        };

        self.cur.skip_trivia();
        let body = self.parse_template_block();
        Some(ComponentDecl {
            span: Span::new(name_span.start as usize, self.cur.pos),
            name: Ident {
                span: name_span,
                name,
            },
            params,
            body,
        })
    }

    fn scan_component_keyword(&mut self) -> Option<Span> {
        let saved = Snapshot::from(&self.cur);
        let start = self.cur.pos;
        if self.cur.eat('@') {
            let Some(kw) = self.cur.scan_ident() else {
                saved.restore(&mut self.cur);
                return None;
            };
            if self.cur.ident_text(kw) != "component" {
                saved.restore(&mut self.cur);
                return None;
            }
            return Some(Span::new(start, self.cur.pos));
        }

        let Some(kw) = self.cur.scan_ident() else {
            saved.restore(&mut self.cur);
            return None;
        };
        if self.cur.ident_text(kw) != "component" {
            saved.restore(&mut self.cur);
            return None;
        }
        let after_kw = Snapshot::from(&self.cur);
        self.cur.skip_trivia();
        if self.cur.peek() != Some('|') {
            saved.restore(&mut self.cur);
            return None;
        }
        let end = after_kw.pos;
        after_kw.restore(&mut self.cur);
        self.error(
            Span::new(start, end),
            "expected `@component`; write `name = @component |params|`",
        );
        Some(Span::new(start, end))
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
                    if look.peek() == Some('{') {
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
                } else {
                    self.error(
                        Span::point(self.cur.pos),
                        format!(
                            "expected `\"...\"` or `{{...}}` for attribute `{}`",
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
        let Some(_) = look.scan_ident() else {
            return false;
        };
        look.skip_trivia();
        look.peek() == Some('=')
    }
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

fn suggest_directive(name: &str) -> Option<&'static str> {
    const KNOWN: [&str; 5] = ["if", "for", "match", "let", "else"];
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
