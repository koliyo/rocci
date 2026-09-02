import Cursor

Template := [].{
    parse_body = parse_body
    kind = item_kind
}

Span : { start : U64, end : U64 }

Ident : { name : Str, span : Span }

AttrValue : [
    Static({ span : Span, value : Str }),
    Expr({ expr : Span }),
    Action({ name : Ident, args : Span }),
    Boolean,
]

Attr : { name : Ident, value : AttrValue, span : Span }

ComponentPath : { parts : List(Ident), roc_name : Str, span : Span }

TemplateItem : [
    Element({
        name : Ident,
        attrs : List(Attr),
        children : List(U64),
        self_closing : Bool,
        span : Span,
    }),
    ComponentCall({
        path : ComponentPath,
        attrs : List(Attr),
        children : List(U64),
        span : Span,
    }),
    Fragment({ children : List(U64), span : Span }),
    Text({ value : Str, span : Span }),
    Interpolation({ expr : Span, span : Span }),
    IfDirective({
        condition : Span,
        then_roots : List(U64),
        else_ifs : List({ condition : Span, roots : List(U64) }),
        else_roots : List(U64),
        span : Span,
    }),
    ForDirective({
        binder : Ident,
        collection : Span,
        body_roots : List(U64),
        span : Span,
    }),
    MatchDirective({
        scrutinee : Span,
        arms : List({ pattern : Span, value : U64, span : Span }),
        span : Span,
    }),
    LetDirective({ binder : Ident, expr : Span, span : Span }),
    Css({ body : Span, span : Span }),
]

TemplateBlock : { nodes : List(TemplateItem), roots : List(U64), span : Span }

item_kind = |item|
    match item {
        Element(_) => "Element"
        ComponentCall(_) => "ComponentCall"
        Fragment(_) => "Fragment"
        Text(_) => "TextNode"
        Interpolation(_) => "Interpolation"
        IfDirective(_) => "IfDirective"
        ForDirective(_) => "ForDirective"
        MatchDirective(_) => "MatchDirective"
        LetDirective(_) => "LetDirective"
        Css(_) => "Css"
    }

diag = |code, span, message| { code: code, span: span, message: message }

empty_block = |pos| { nodes: [], roots: [], span: { start: pos, end: pos } }

parse_body = |cur| {
    var $cur = Cursor.skip_trivia(cur)
    if Cursor.peek($cur) == Ok(123) {
        parse_template_block($cur)
    } else if Cursor.peek($cur) == Ok(60) {
        parse_html_expr_body($cur)
    } else {
        {
            cur: $cur,
            block: empty_block($cur.pos),
            diagnostics: [diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected `{` to open a template body, or a single HTML tag")],
        }
    }
}

parse_html_expr_body = |cur| {
    start = cur.pos
    match parse_tag(cur) {
        Miss(rest) => { cur: rest.cur, block: empty_block(start), diagnostics: rest.diagnostics }
        Hit(got) => {
            {
                cur: got.cur,
                block: { nodes: got.nodes, roots: [got.root], span: { start: start, end: got.cur.pos } },
                diagnostics: got.diagnostics,
            }
        }
    }
}

parse_template_block = |cur| {
    start = cur.pos
    got = Cursor.eat(cur, 123)
    if !got.eaten {
        {
            cur: cur,
            block: empty_block(start),
            diagnostics: [diag("RC1001", { start: start, end: start }, "expected `{` to open a template body")],
        }
    } else {
        parsed = parse_template_items(Cursor.skip_spaces_tabs(got.cur), Bool.True)
        var $cur = parsed.cur
        var $diagnostics = parsed.diagnostics
        close = Cursor.eat($cur, 125)
        if !close.eaten {
            $diagnostics = List.append(
                $diagnostics,
                diag("RC1002", { start: start, end: $cur.pos }, "unclosed template block; expected `}`"),
            )
        }
        $cur = close.cur
        {
            cur: $cur,
            block: { nodes: parsed.nodes, roots: parsed.roots, span: { start: start, end: $cur.pos } },
            diagnostics: $diagnostics,
        }
    }
}

parse_template_items = |cur, stop_brace| {
    var $cur = cur
    var $nodes = []
    var $roots = []
    var $diagnostics = []
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        $cur = Cursor.skip_formatting_ws($cur)
        if Cursor.is_eof($cur) {
            $loop = Bool.False
        } else if stop_brace and Cursor.peek($cur) == Ok(125) {
            $loop = Bool.False
        } else if Cursor.starts_with($cur, "</") {
            $loop = Bool.False
        } else if Cursor.peek($cur) == Ok(35) {
            $cur = Cursor.skip_comment($cur)
        } else if Cursor.starts_with($cur, "<!--") {
            comment = skip_html_comment($cur)
            $cur = comment.cur
            $diagnostics = List.concat($diagnostics, comment.diagnostics)
        } else if Cursor.peek($cur) == Ok(60) or Cursor.peek($cur) == Ok(123) or Cursor.peek($cur) == Ok(64) {
            match parse_template_item($cur, stop_brace) {
                Miss(rest) => {
                    $cur = rest.cur
                    $diagnostics = List.concat($diagnostics, rest.diagnostics)
                    $loop = Bool.False
                }
                Hit(got) => {
                    base = List.len($nodes)
                    shifted = List.map(got.nodes, |item| offset_item(item, base))
                    $nodes = List.concat($nodes, shifted)
                    $roots = List.append($roots, got.root + base)
                    $cur = got.cur
                    $diagnostics = List.concat($diagnostics, got.diagnostics)
                }
            }
        } else {
            match scan_text($cur) {
                NoneText => {
                    $loop = Bool.False
                }
                SomeText(text) => {
                    $cur = text.cur
                    if text.node.value != "" {
                        id = List.len($nodes)
                        $nodes = List.append($nodes, Text({ value: text.node.value, span: text.node.span }))
                        $roots = List.append($roots, id)
                    }
                }
            }
        }
    }
    { cur: $cur, nodes: $nodes, roots: $roots, diagnostics: $diagnostics }
}

parse_template_item = |cur, stop_brace| {
    if Cursor.peek(cur) == Ok(60) {
        parse_tag(cur)
    } else if Cursor.peek(cur) == Ok(123) {
        interp = parse_interpolation(cur)
        Hit({
            cur: interp.cur,
            nodes: [Interpolation({ expr: interp.node.expr, span: interp.node.span })],
            root: 0.U64,
            diagnostics: interp.diagnostics,
        })
    } else if Cursor.peek(cur) == Ok(64) {
        parse_directive(cur, stop_brace)
    } else {
        Miss({ cur: cur, diagnostics: [] })
    }
}

offset_item = |item, delta|
    match item {
        Element({ name, attrs, children, self_closing, span }) =>
            Element({
                name: name,
                attrs: attrs,
                children: List.map(children, |i| i + delta),
                self_closing: self_closing,
                span: span,
            })
        ComponentCall({ path, attrs, children, span }) =>
            ComponentCall({
                path: path,
                attrs: attrs,
                children: List.map(children, |i| i + delta),
                span: span,
            })
        Fragment({ children, span }) =>
            Fragment({ children: List.map(children, |i| i + delta), span: span })
        Text(n) => Text(n)
        Interpolation(n) => Interpolation(n)
        IfDirective({ condition, then_roots, else_ifs, else_roots, span }) =>
            IfDirective({
                condition: condition,
                then_roots: List.map(then_roots, |i| i + delta),
                else_ifs: List.map(
                    else_ifs,
                    |arm| { condition: arm.condition, roots: List.map(arm.roots, |i| i + delta) },
                ),
                else_roots: List.map(else_roots, |i| i + delta),
                span: span,
            })
        ForDirective({ binder, collection, body_roots, span }) =>
            ForDirective({
                binder: binder,
                collection: collection,
                body_roots: List.map(body_roots, |i| i + delta),
                span: span,
            })
        MatchDirective({ scrutinee, arms, span }) =>
            MatchDirective({
                scrutinee: scrutinee,
                arms: List.map(
                    arms,
                    |arm| { pattern: arm.pattern, value: arm.value + delta, span: arm.span },
                ),
                span: span,
            })
        LetDirective(n) => LetDirective(n)
        Css(n) => Css(n)
    }

append_block = |nodes, block| {
    delta = List.len(nodes)
    shifted = List.map(block.nodes, |item| offset_item(item, delta))
    {
        nodes: List.concat(nodes, shifted),
        roots: List.map(block.roots, |i| i + delta),
    }
}

skip_html_comment = |cur| {
    start = cur.pos
    var $cur = Cursor.eat_str(cur, "<!--")
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        if Cursor.starts_with($cur, "-->") {
            $cur = Cursor.eat_str($cur, "-->")
            $loop = Bool.False
        } else {
            before = $cur.pos
            $cur = Cursor.bump($cur)
            if $cur.pos <= before {
                $cur = Cursor.bump($cur)
            }
        }
    }
    diagnostics =
        if $loop {
            [diag("RC1002", { start: start, end: $cur.pos }, "unterminated HTML comment")]
        } else {
            []
        }
    { cur: $cur, diagnostics: diagnostics }
}

parse_interpolation = |cur| {
    scan = Cursor.scan_interpolation(cur.text, cur.pos)
    next = Cursor.at(cur.text, scan.span.end)
    moved = { ..next, paren: cur.paren, bracket: cur.bracket, brace: cur.brace }
    diagnostics =
        if scan.terminated {
            []
        } else {
            [diag("RC1002", scan.span, "unterminated interpolation; expected `}`")]
        }
    { cur: moved, node: { expr: scan.expr, span: scan.span }, diagnostics: diagnostics }
}

scan_text = |cur| {
    start = cur.pos
    var $cur = cur
    var $value = ""
    while !Cursor.is_eof($cur) {
        if Cursor.peek($cur) == Ok(60) or Cursor.peek($cur) == Ok(123) or Cursor.peek($cur) == Ok(125) or Cursor.peek($cur) == Ok(64) {
            break
        }
        ch = Cursor.slice($cur.text, { start: $cur.pos, end: $cur.pos + 1 })
        $value = Str.concat($value, ch)
        $cur = Cursor.bump($cur)
    }
    if $cur.pos == start {
        NoneText
    } else {
        SomeText({ cur: $cur, node: { value: $value, span: { start: start, end: $cur.pos } } })
    }
}

parse_tag = |cur| {
    start = cur.pos
    got = Cursor.eat(cur, 60)
    if !got.eaten {
        Miss({ cur: cur, diagnostics: [] })
    } else {
        var $cur = got.cur
        if Cursor.peek($cur) == Ok(62) {
            $cur = Cursor.bump($cur)
            kids = parse_template_items($cur, Bool.False)
            $cur = kids.cur
            var $frag_diag = kids.diagnostics
            if Cursor.starts_with($cur, "</>") {
                $cur = Cursor.eat_str($cur, "</>")
            } else {
                $frag_diag = List.append(
                    $frag_diag,
                    diag("RC1002", { start: start, end: $cur.pos }, "unclosed fragment; expected `</>`"),
                )
            }
            Hit({
                cur: $cur,
                nodes: List.append(kids.nodes, Fragment({ children: kids.roots, span: { start: start, end: $cur.pos } })),
                root: List.len(kids.nodes),
                diagnostics: $frag_diag,
            })
        } else if Cursor.peek($cur) == Ok(47) {
            $cur = Cursor.bump($cur)
            $cur =
                match scan_tag_path($cur) {
                    Miss(rest) => rest.cur
                    Hit(path) => path.cur
                }
            $cur = Cursor.skip_spaces_tabs($cur)
            gt = Cursor.eat($cur, 62)
            Miss({
                cur: gt.cur,
                diagnostics: [diag("RC1005", { start: start, end: gt.cur.pos }, "unexpected closing tag")],
            })
        } else {
            match scan_tag_path($cur) {
                Miss(rest) => Miss({
                    cur: rest.cur,
                    diagnostics: List.append(
                        rest.diagnostics,
                        diag("RC1005", { start: $cur.pos, end: $cur.pos }, "expected tag name after `<`"),
                    ),
                })
                Hit(path_got) => {
                    $cur = path_got.cur
                    attrs_got = parse_attrs($cur)
                    $cur = attrs_got.cur
                    slash = Cursor.eat($cur, 47)
                    self_closing = slash.eaten
                    $cur = slash.cur
                    gt = Cursor.eat($cur, 62)
                    var $diagnostics = List.concat(path_got.diagnostics, attrs_got.diagnostics)
                    if !gt.eaten {
                        $diagnostics = List.append(
                            $diagnostics,
                            diag("RC1001", { start: start, end: $cur.pos }, "expected `>` to end the opening tag"),
                        )
                    }
                    $cur = gt.cur
                    is_component = path_is_component(path_got.path)
                    first_name =
                        match List.get(path_got.path.parts, 0) {
                            Ok(part) => part
                            Err(_) => { name: "", span: { start: start, end: start } }
                        }
                    if self_closing or (!is_component and is_void(first_name.name)) {
                        item =
                            if is_component {
                                ComponentCall({
                                    path: path_got.path,
                                    attrs: attrs_got.attrs,
                                    children: [],
                                    span: { start: start, end: $cur.pos },
                                })
                            } else {
                                Element({
                                    name: first_name,
                                    attrs: attrs_got.attrs,
                                    children: [],
                                    self_closing: Bool.True,
                                    span: { start: start, end: $cur.pos },
                                })
                            }
                        Hit({ cur: $cur, nodes: [item], root: 0.U64, diagnostics: $diagnostics })
                    } else {
                        kids = parse_template_items($cur, Bool.False)
                        $cur = kids.cur
                        $diagnostics = List.concat($diagnostics, kids.diagnostics)
                        $cur = Cursor.skip_formatting_ws($cur)
                        if Cursor.starts_with($cur, "</") {
                            close = eat_closing_tag($cur)
                            $cur = close.cur
                            $diagnostics = List.concat($diagnostics, close.diagnostics)
                        } else {
                            $diagnostics = List.append(
                                $diagnostics,
                                diag(
                                    "RC1002",
                                    { start: start, end: $cur.pos },
                                    "unclosed tag; expected a closing tag",
                                ),
                            )
                        }
                        item =
                            if is_component {
                                ComponentCall({
                                    path: path_got.path,
                                    attrs: attrs_got.attrs,
                                    children: kids.roots,
                                    span: { start: start, end: $cur.pos },
                                })
                            } else {
                                Element({
                                    name: first_name,
                                    attrs: attrs_got.attrs,
                                    children: kids.roots,
                                    self_closing: Bool.False,
                                    span: { start: start, end: $cur.pos },
                                })
                            }
                        Hit({
                            cur: $cur,
                            nodes: List.append(kids.nodes, item),
                            root: List.len(kids.nodes),
                            diagnostics: $diagnostics,
                        })
                    }
                }
            }
        }
    }
}

eat_closing_tag = |cur| {
    var $cur = Cursor.eat_str(cur, "</")
    match Cursor.scan_tag_name($cur) {
        Ok(close) => {
            $cur = close.cur
            $cur = Cursor.skip_spaces_tabs($cur)
            gt = Cursor.eat($cur, 62)
            { cur: gt.cur, diagnostics: [] }
        }
        Err(rest) => {
            if Cursor.peek(rest) == Ok(62) {
                { cur: Cursor.bump(rest), diagnostics: [] }
            } else {
                { cur: rest, diagnostics: [] }
            }
        }
    }
}

scan_tag_path = |cur| {
    match Cursor.scan_tag_name(cur) {
        Err(rest) => Miss({ cur: rest, diagnostics: [] })
        Ok(first) => {
            first_ident = { name: Cursor.ident_text(first.cur, first.span), span: first.span }
            var $cur = first.cur
            var $parts = [first_ident]
            var $diagnostics = []
            var $loop = Bool.True
            while $loop {
                dot = Cursor.eat($cur, 46)
                if !dot.eaten {
                    $loop = Bool.False
                } else {
                    match Cursor.scan_tag_name(dot.cur) {
                        Ok(next) => {
                            $cur = next.cur
                            ident = { name: Cursor.ident_text($cur, next.span), span: next.span }
                            $parts = List.append($parts, ident)
                        }
                        Err(rest) => {
                            $cur = rest
                            $diagnostics = List.append(
                                $diagnostics,
                                diag("RC1001", { start: rest.pos, end: rest.pos }, "expected identifier after `.`"),
                            )
                            $loop = Bool.False
                        }
                    }
                }
            }
            last =
                match List.get($parts, List.len($parts) - 1) {
                    Ok(part) => part
                    Err(_) => first_ident
                }
            path = {
                parts: $parts,
                roc_name: path_roc_name($parts),
                span: { start: first_ident.span.start, end: last.span.end },
            }
            Hit({ cur: $cur, path: path, diagnostics: $diagnostics })
        }
    }
}

path_is_component = |path|
    match List.get(path.parts, 0) {
        Ok(part) => is_pascal(part.name)
        Err(_) => Bool.False
    }

path_roc_name = |parts| {
    match List.get(parts, List.len(parts) - 1) {
        Ok(part) => pascal_to_camel(part.name)
        Err(_) => ""
    }
}

parse_attrs = |cur| {
    var $cur = cur
    var $attrs = []
    var $diagnostics = []
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        $cur = Cursor.skip_spaces_tabs($cur)
        if Cursor.peek($cur) == Ok(10) or Cursor.peek($cur) == Ok(13) {
            $cur = Cursor.skip_formatting_ws($cur)
        }
        if Cursor.peek($cur) == Ok(62) or Cursor.peek($cur) == Ok(47) or Cursor.is_eof($cur) {
            $loop = Bool.False
        } else {
            match Cursor.scan_attr_name($cur) {
                Err(_) => {
                    $loop = Bool.False
                }
                Ok(name_got) => {
                    name = { name: Cursor.ident_text(name_got.cur, name_got.span), span: name_got.span }
                    $cur = Cursor.skip_spaces_tabs(name_got.cur)
                    eq = Cursor.eat($cur, 61)
                    if !eq.eaten {
                        $attrs = List.append(
                            $attrs,
                            { name: name, value: Boolean, span: name.span },
                        )
                    } else {
                        $cur = Cursor.skip_spaces_tabs(eq.cur)
                        if Cursor.peek($cur) == Ok(123) {
                            interp = parse_interpolation($cur)
                            $cur = interp.cur
                            $diagnostics = List.concat($diagnostics, interp.diagnostics)
                            $attrs = List.append(
                                $attrs,
                                {
                                    name: name,
                                    value: Expr({ expr: interp.node.expr }),
                                    span: { start: name.span.start, end: interp.node.span.end },
                                },
                            )
                        } else if Cursor.peek($cur) == Ok(34) {
                            quoted = scan_quoted($cur)
                            $cur = quoted.cur
                            $attrs = List.append(
                                $attrs,
                                {
                                    name: name,
                                    value: Static({ span: quoted.span, value: quoted.value }),
                                    span: { start: name.span.start, end: quoted.span.end },
                                },
                            )
                        } else if Cursor.peek($cur) == Ok(64) {
                            action = parse_action_attr($cur)
                            $cur = action.cur
                            $diagnostics = List.concat($diagnostics, action.diagnostics)
                            $attrs = List.append(
                                $attrs,
                                {
                                    name: name,
                                    value: action.value,
                                    span: { start: name.span.start, end: action.end },
                                },
                            )
                        } else {
                            $diagnostics = List.append(
                                $diagnostics,
                                diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected attribute value"),
                            )
                            $attrs = List.append(
                                $attrs,
                                { name: name, value: Boolean, span: name.span },
                            )
                            $loop = Bool.False
                        }
                    }
                }
            }
        }
    }
    { cur: $cur, attrs: $attrs, diagnostics: $diagnostics }
}

scan_quoted = |cur| {
    start = cur.pos
    closed = Cursor.skip_string(cur)
    value =
        if closed.pos > start + 1 {
            Cursor.slice(cur.text, { start: start + 1, end: sat_sub(closed.pos) })
        } else {
            ""
        }
    { cur: closed, span: { start: start, end: closed.pos }, value: value }
}

sat_sub = |n| {
    if n == 0 {
        0.U64
    } else {
        n - 1
    }
}

parse_action_attr = |cur| {
    start = cur.pos
    var $cur = Cursor.bump(cur)
    match Cursor.scan_ident($cur) {
        Err(_) => {
            {
                cur: $cur,
                value: Boolean,
                end: start,
                diagnostics: [diag("RC1001", { start: start, end: start }, "expected Datastar action name after `@`")],
            }
        }
        Ok(got) => {
            name = { name: Cursor.ident_text(got.cur, got.span), span: got.span }
            $cur = Cursor.skip_spaces_tabs(got.cur)
            var $diagnostics = []
            if !is_datastar(name.name) {
                $diagnostics = List.append(
                    $diagnostics,
                    diag("RC1001", { start: start, end: name.span.end }, "unknown Datastar action"),
                )
            }
            paren_before = $cur.paren
            $cur = Cursor.skip_roc_token($cur)
            args_start = $cur.pos
            while !Cursor.is_eof($cur) and $cur.paren > paren_before {
                before = $cur.pos
                $cur = Cursor.skip_roc_token($cur)
                if $cur.pos <= before {
                    $cur = Cursor.bump($cur)
                }
            }
            args_end = sat_sub($cur.pos)
            args = Cursor.trim_span($cur.text, args_start, args_end)
            {
                cur: $cur,
                value: Action({ name: name, args: args }),
                end: $cur.pos,
                diagnostics: $diagnostics,
            }
        }
    }
}

is_datastar = |name|
    name == "get" or name == "post" or name == "put" or name == "patch" or name == "delete"

parse_directive = |cur, _stop_brace| {
    start = cur.pos
    if Cursor.starts_with(cur, "@@") {
        var $cur = Cursor.eat_str(cur, "@@")
        var $value = "@"
        while !Cursor.is_eof($cur) {
            if Cursor.peek($cur) == Ok(60) or Cursor.peek($cur) == Ok(123) or Cursor.peek($cur) == Ok(64) or Cursor.peek($cur) == Ok(125) or Cursor.peek($cur) == Ok(10) {
                break
            }
            ch = Cursor.slice($cur.text, { start: $cur.pos, end: $cur.pos + 1 })
            $value = Str.concat($value, ch)
            $cur = Cursor.bump($cur)
        }
        Hit({
            cur: $cur,
            nodes: [Text({ value: $value, span: { start: start, end: $cur.pos } })],
            root: 0.U64,
            diagnostics: [],
        })
    } else {
        var $cur = Cursor.bump(cur)
        match Cursor.scan_ident($cur) {
            Err(_) => Miss({
                cur: $cur,
                diagnostics: [diag("RC1001", { start: start, end: start }, "expected directive name after `@`")],
            })
            Ok(ident) => {
                name = Cursor.ident_text(ident.cur, ident.span)
                $cur = ident.cur
                if name == "if" {
                    parse_if($cur, start)
                } else if name == "for" {
                    parse_for($cur, start)
                } else if name == "match" {
                    parse_match($cur, start)
                } else if name == "let" {
                    parse_let($cur, start)
                } else if name == "css" {
                    parse_body_css($cur, start)
                } else if name == "else" {
                    Miss({ cur: cur, diagnostics: [] })
                } else {
                    Miss({
                        cur: $cur,
                        diagnostics: [diag("RC1001", { start: start, end: ident.span.end }, "unknown directive")],
                    })
                }
            }
        }
    }
}

parse_if = |cur, start| {
    cond = scan_header_expr(cur)
    then_block = parse_template_block(cond.cur)
    var $cur = then_block.cur
    var $diagnostics = List.concat(cond.diagnostics, then_block.diagnostics)
    var $nodes = then_block.block.nodes
    then_roots = then_block.block.roots
    var $else_ifs = []
    var $else_roots = []
    var $loop = Bool.True
    while $loop {
        skipped = Cursor.skip_trivia($cur)
        if !Cursor.starts_with(skipped, "@else") {
            $loop = Bool.False
        } else {
            $cur = Cursor.eat_str(skipped, "@else")
            $cur = Cursor.skip_spaces_tabs($cur)
            if try_ident($cur, "if") {
                $cur = eat_ident($cur)
                elif_cond = scan_header_expr($cur)
                elif_block = parse_template_block(elif_cond.cur)
                absorbed = append_block($nodes, elif_block.block)
                $nodes = absorbed.nodes
                $else_ifs = List.append(
                    $else_ifs,
                    { condition: elif_cond.span, roots: absorbed.roots },
                )
                $cur = elif_block.cur
                $diagnostics = List.concat($diagnostics, List.concat(elif_cond.diagnostics, elif_block.diagnostics))
            } else {
                $cur = Cursor.skip_trivia($cur)
                else_block = parse_template_block($cur)
                absorbed = append_block($nodes, else_block.block)
                $nodes = absorbed.nodes
                $else_roots = absorbed.roots
                $cur = else_block.cur
                $diagnostics = List.concat($diagnostics, else_block.diagnostics)
                $loop = Bool.False
            }
        }
    }
    item = IfDirective({
        condition: cond.span,
        then_roots: then_roots,
        else_ifs: $else_ifs,
        else_roots: $else_roots,
        span: { start: start, end: $cur.pos },
    })
    Hit({
        cur: $cur,
        nodes: List.append($nodes, item),
        root: List.len($nodes),
        diagnostics: $diagnostics,
    })
}

parse_for = |cur, start| {
    var $cur = Cursor.skip_whitespace(cur)
    var $diagnostics = []
    match Cursor.scan_ident($cur) {
        Err(_) => {
            $diagnostics = List.append(
                $diagnostics,
                diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected binder after `@for`"),
            )
            binder = { name: "_", span: { start: $cur.pos, end: $cur.pos } }
            body = parse_template_block($cur)
            item = ForDirective({
                binder: binder,
                collection: { start: $cur.pos, end: $cur.pos },
                body_roots: body.block.roots,
                span: { start: start, end: body.cur.pos },
            })
            Hit({
                cur: body.cur,
                nodes: List.append(body.block.nodes, item),
                root: List.len(body.block.nodes),
                diagnostics: List.concat($diagnostics, body.diagnostics),
            })
        }
        Ok(got) => {
            binder = { name: Cursor.ident_text(got.cur, got.span), span: got.span }
            $cur = Cursor.skip_whitespace(got.cur)
            if !try_ident($cur, "in") {
                $diagnostics = List.append(
                    $diagnostics,
                    diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected `in` after `@for` binder"),
                )
            } else {
                $cur = eat_ident($cur)
            }
            coll = scan_header_expr($cur)
            body = parse_template_block(coll.cur)
            item = ForDirective({
                binder: binder,
                collection: coll.span,
                body_roots: body.block.roots,
                span: { start: start, end: body.cur.pos },
            })
            Hit({
                cur: body.cur,
                nodes: List.append(body.block.nodes, item),
                root: List.len(body.block.nodes),
                diagnostics: List.concat($diagnostics, List.concat(coll.diagnostics, body.diagnostics)),
            })
        }
    }
}

parse_match = |cur, start| {
    scrut = scan_header_expr(cur)
    var $cur = Cursor.skip_trivia(scrut.cur)
    var $diagnostics = scrut.diagnostics
    open = Cursor.eat($cur, 123)
    if !open.eaten {
        Hit({
            cur: $cur,
            nodes: [
                MatchDirective({
                    scrutinee: scrut.span,
                    arms: [],
                    span: { start: start, end: $cur.pos },
                }),
            ],
            root: 0.U64,
            diagnostics: List.append(
                $diagnostics,
                diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected `{` to open `@match` arms"),
            ),
        })
    } else {
        $cur = open.cur
        var $nodes = []
        var $arms = []
        var $loop = Bool.True
        while $loop and !Cursor.is_eof($cur) {
            $cur = Cursor.skip_formatting_ws($cur)
            if Cursor.peek($cur) == Ok(35) {
                $cur = Cursor.skip_comment($cur)
            } else if Cursor.peek($cur) == Ok(125) or Cursor.is_eof($cur) {
                $loop = Bool.False
            } else {
                arm_start = $cur.pos
                match scan_pattern($cur) {
                    Miss(rest) => {
                        $cur = rest.cur
                        $diagnostics = List.concat($diagnostics, rest.diagnostics)
                        $loop = Bool.False
                    }
                    Hit(pat) => {
                        $cur = Cursor.skip_whitespace(pat.cur)
                        if !Cursor.starts_with($cur, "=>") {
                            $diagnostics = List.append(
                                $diagnostics,
                                diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected `=>` after match pattern"),
                            )
                        } else {
                            $cur = Cursor.eat_str($cur, "=>")
                        }
                        match parse_match_value($cur) {
                            Miss(rest) => {
                                $cur = rest.cur
                                $diagnostics = List.concat($diagnostics, rest.diagnostics)
                            }
                            Hit(val) => {
                                absorbed = append_block($nodes, { nodes: val.nodes, roots: [val.root], span: { start: 0, end: 0 } })
                                $nodes = absorbed.nodes
                                value_id =
                                    match List.get(absorbed.roots, 0) {
                                        Ok(id) => id
                                        Err(_) => 0.U64
                                    }
                                $arms = List.append(
                                    $arms,
                                    { pattern: pat.span, value: value_id, span: { start: arm_start, end: val.cur.pos } },
                                )
                                $cur = val.cur
                                $diagnostics = List.concat($diagnostics, val.diagnostics)
                            }
                        }
                    }
                }
            }
        }
        close = Cursor.eat($cur, 125)
        if !close.eaten {
            $diagnostics = List.append(
                $diagnostics,
                diag("RC1002", { start: start, end: $cur.pos }, "unclosed `@match`; expected `}`"),
            )
        }
        $cur = close.cur
        item = MatchDirective({
            scrutinee: scrut.span,
            arms: $arms,
            span: { start: start, end: $cur.pos },
        })
        Hit({
            cur: $cur,
            nodes: List.append($nodes, item),
            root: List.len($nodes),
            diagnostics: $diagnostics,
        })
    }
}

parse_match_value = |cur| {
    var $cur = Cursor.skip_whitespace(cur)
    if Cursor.peek($cur) == Ok(35) {
        $cur = Cursor.skip_comment($cur)
        $cur = Cursor.skip_whitespace($cur)
    }
    if Cursor.peek($cur) == Ok(60) or Cursor.peek($cur) == Ok(123) or Cursor.peek($cur) == Ok(64) {
        parse_template_item($cur, Bool.False)
    } else {
        Miss({
            cur: $cur,
            diagnostics: [diag("RC1001", { start: $cur.pos, end: $cur.pos }, "match arm must produce a tag, fragment, interpolation, or directive")],
        })
    }
}

parse_let = |cur, start| {
    var $cur = Cursor.skip_whitespace(cur)
    match Cursor.scan_ident($cur) {
        Err(_) => Hit({
            cur: $cur,
            nodes: [
                LetDirective({
                    binder: { name: "_", span: { start: $cur.pos, end: $cur.pos } },
                    expr: { start: $cur.pos, end: $cur.pos },
                    span: { start: start, end: $cur.pos },
                }),
            ],
            root: 0.U64,
            diagnostics: [diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected binder after `@let`")],
        })
        Ok(got) => {
            binder = { name: Cursor.ident_text(got.cur, got.span), span: got.span }
            $cur = Cursor.skip_whitespace(got.cur)
            var $diagnostics = []
            eq = Cursor.eat($cur, 61)
            if !eq.eaten {
                $diagnostics = List.append(
                    $diagnostics,
                    diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected `=` after `@let` binder"),
                )
            }
            expr = scan_line_expr(eq.cur)
            Hit({
                cur: expr.cur,
                nodes: [
                    LetDirective({
                        binder: binder,
                        expr: expr.span,
                        span: { start: start, end: expr.cur.pos },
                    }),
                ],
                root: 0.U64,
                diagnostics: List.concat($diagnostics, expr.diagnostics),
            })
        }
    }
}

parse_body_css = |cur, start| {
    var $cur = Cursor.skip_trivia(cur)
    open = Cursor.eat($cur, 123)
    if !open.eaten {
        Hit({
            cur: $cur,
            nodes: [Css({ body: { start: $cur.pos, end: $cur.pos }, span: { start: start, end: $cur.pos } })],
            root: 0.U64,
            diagnostics: [diag("RC1001", { start: $cur.pos, end: $cur.pos }, "expected `{` to open a `@css` block")],
        })
    } else {
        body_start = open.cur.pos
        $cur = Cursor.skip_balanced_braces({ ..open.cur, pos: sat_sub(open.cur.pos) })
        body_end =
            if $cur.pos > body_start {
                sat_sub($cur.pos)
            } else {
                $cur.pos
            }
        Hit({
            cur: $cur,
            nodes: [
                Css({
                    body: { start: body_start, end: body_end },
                    span: { start: start, end: $cur.pos },
                }),
            ],
            root: 0.U64,
            diagnostics: [],
        })
    }
}

scan_header_expr = |cur| {
    var $cur = Cursor.skip_whitespace(cur)
    start = $cur.pos
    var $paren = 0.U64
    var $bracket = 0.U64
    var $diagnostics = []
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        before = $cur.pos
        match Cursor.peek($cur) {
            Ok(34) => {
                $cur = Cursor.skip_string($cur)
            }
            Ok(35) => {
                $cur = Cursor.skip_comment($cur)
            }
            Ok(40) => {
                $cur = Cursor.bump($cur)
                $paren = $paren + 1
            }
            Ok(41) => {
                $cur = Cursor.bump($cur)
                $paren = sat_sub($paren)
            }
            Ok(91) => {
                $cur = Cursor.bump($cur)
                $bracket = $bracket + 1
            }
            Ok(93) => {
                $cur = Cursor.bump($cur)
                $bracket = sat_sub($bracket)
            }
            Ok(123) if $paren == 0 and $bracket == 0 => {
                $loop = Bool.False
            }
            Ok(123) => {
                $cur = Cursor.skip_balanced_braces($cur)
            }
            Ok(10) if $paren == 0 and $bracket == 0 => {
                $diagnostics = List.append(
                    $diagnostics,
                    diag("RC1001", { start: start, end: $cur.pos }, "directive header must keep its body `{` on the same logical line"),
                )
                $loop = Bool.False
            }
            Ok(13) if $paren == 0 and $bracket == 0 => {
                $loop = Bool.False
            }
            _ => {
                $cur = Cursor.bump($cur)
            }
        }
        if $loop and $cur.pos <= before {
            $cur = Cursor.bump($cur)
        }
    }
    span = Cursor.trim_span($cur.text, start, $cur.pos)
    { cur: $cur, span: span, diagnostics: $diagnostics }
}

scan_line_expr = |cur| {
    var $cur = Cursor.skip_spaces_tabs(cur)
    start = $cur.pos
    var $paren = 0.U64
    var $bracket = 0.U64
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        before = $cur.pos
        match Cursor.peek($cur) {
            Ok(34) => {
                $cur = Cursor.skip_string($cur)
            }
            Ok(35) => {
                $loop = Bool.False
            }
            Ok(40) => {
                $cur = Cursor.bump($cur)
                $paren = $paren + 1
            }
            Ok(41) => {
                $cur = Cursor.bump($cur)
                $paren = sat_sub($paren)
            }
            Ok(91) => {
                $cur = Cursor.bump($cur)
                $bracket = $bracket + 1
            }
            Ok(93) => {
                $cur = Cursor.bump($cur)
                $bracket = sat_sub($bracket)
            }
            Ok(10) if $paren == 0 and $bracket == 0 => {
                $loop = Bool.False
            }
            Ok(13) if $paren == 0 and $bracket == 0 => {
                $loop = Bool.False
            }
            _ => {
                $cur = Cursor.bump($cur)
            }
        }
        if $loop and $cur.pos <= before {
            $cur = Cursor.bump($cur)
        }
    }
    { cur: $cur, span: Cursor.trim_span($cur.text, start, $cur.pos), diagnostics: [] }
}

scan_pattern = |cur| {
    var $cur = Cursor.skip_formatting_ws(cur)
    start = $cur.pos
    if Cursor.peek($cur) == Ok(125) {
        Miss({ cur: $cur, diagnostics: [diag("RC1001", { start: start, end: start }, "expected match pattern")] })
    } else {
        var $paren = 0.U64
        var $bracket = 0.U64
        var $brace = 0.U64
        var $loop = Bool.True
        while $loop and !Cursor.is_eof($cur) {
            if $paren == 0 and $bracket == 0 and $brace == 0 and Cursor.starts_with($cur, "=>") {
                $loop = Bool.False
            } else if Cursor.peek($cur) == Ok(125) and $paren == 0 and $bracket == 0 and $brace == 0 {
                $loop = Bool.False
            } else {
                before = $cur.pos
                match Cursor.peek($cur) {
                    Ok(34) => {
                        $cur = Cursor.skip_string($cur)
                    }
                    Ok(35) => {
                        $cur = Cursor.skip_comment($cur)
                    }
                    Ok(40) => {
                        $cur = Cursor.bump($cur)
                        $paren = $paren + 1
                    }
                    Ok(41) => {
                        $cur = Cursor.bump($cur)
                        $paren = sat_sub($paren)
                    }
                    Ok(91) => {
                        $cur = Cursor.bump($cur)
                        $bracket = $bracket + 1
                    }
                    Ok(93) => {
                        $cur = Cursor.bump($cur)
                        $bracket = sat_sub($bracket)
                    }
                    Ok(123) => {
                        $cur = Cursor.bump($cur)
                        $brace = $brace + 1
                    }
                    Ok(125) => {
                        $cur = Cursor.bump($cur)
                        $brace = sat_sub($brace)
                    }
                    _ => {
                        $cur = Cursor.bump($cur)
                    }
                }
                if $cur.pos <= before {
                    $cur = Cursor.bump($cur)
                }
            }
        }
        if $cur.pos == start {
            Miss({ cur: $cur, diagnostics: [diag("RC1001", { start: start, end: start }, "expected match pattern")] })
        } else {
            Hit({ cur: $cur, span: Cursor.trim_span($cur.text, start, $cur.pos), diagnostics: [] })
        }
    }
}

try_ident = |cur, want|
    match Cursor.scan_ident(cur) {
        Ok(got) => Cursor.ident_text(got.cur, got.span) == want
        Err(_) => Bool.False
    }

eat_ident = |cur|
    match Cursor.scan_ident(cur) {
        Ok(got) => got.cur
        Err(rest) => rest
    }

is_pascal = |name| {
    bytes = Str.to_utf8(name)
    match List.get(bytes, 0) {
        Ok(b) if b >= 65 and b <= 90 => Bool.True
        _ => Bool.False
    }
}

pascal_to_camel = |name| {
    bytes = Str.to_utf8(name)
    match List.get(bytes, 0) {
        Ok(b) if b >= 65 and b <= 90 => {
            rest = List.drop_first(bytes, 1)
            Str.from_utf8_lossy(List.prepend(rest, b + 32))
        }
        _ => name
    }
}

is_void = |name|
    name == "area"
    or name == "base"
    or name == "br"
    or name == "col"
    or name == "embed"
    or name == "hr"
    or name == "img"
    or name == "input"
    or name == "link"
    or name == "meta"
    or name == "param"
    or name == "source"
    or name == "track"
    or name == "wbr"

root_kind = |block, id|
    match List.get(block.nodes, id) {
        Ok(item) => item_kind(item)
        Err(_) => ""
    }

nodes_have_kind = |nodes, kind|
    List.fold(
        nodes,
        Bool.False,
        |acc, item| acc or item_kind(item) == kind,
    )

expect {
    out = parse_body(Cursor.new("<p/>"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element"
        Err(_) => Bool.False
    }
}

expect {
    out = parse_body(Cursor.new("<Hello />"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "ComponentCall"
        Err(_) => Bool.False
    }
}

expect {
    out = parse_body(Cursor.new("<br>"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element"
        Err(_) => Bool.False
    }
}

expect {
    out = parse_body(Cursor.new("{ <p>Hello, {name}!</p> }"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element" and nodes_have_kind(out.block.nodes, "Interpolation")
        Err(_) => Bool.False
    }
}

expect {
    out = parse_body(Cursor.new("<p>{name"))
    List.len(out.diagnostics) > 0
}

expect {
    out = parse_body(Cursor.new("<Badge>ok</Badge>"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "ComponentCall"
        Err(_) => Bool.False
    }
}

expect {
    out = parse_body(Cursor.new("<div>x"))
    List.len(out.diagnostics) > 0
}

expect {
    src = "{ @if ready { <p>ok</p> } @else { <p>no</p> } }"
    out = parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "IfDirective"
        Err(_) => Bool.False
    }
}

expect {
    src = "{ @for item in items { <li>{item}</li> } }"
    out = parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "ForDirective"
        Err(_) => Bool.False
    }
}

expect {
    src = "{ @match x { Ok(v) => <p>{v}</p> Err(_) => <p>no</p> } }"
    out = parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "MatchDirective"
        Err(_) => Bool.False
    }
}

expect {
    src = "{ @let n = 1\n<p>{n}</p> }"
    out = parse_body(Cursor.new(src))
    nodes_have_kind(out.block.nodes, "LetDirective")
}

expect {
    src = "{\n    <p>Hello, {name}!</p>\n}"
    out = parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element" and nodes_have_kind(out.block.nodes, "Interpolation")
        Err(_) => Bool.False
    }
}
