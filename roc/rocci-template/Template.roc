import Cursor

Template := [].{
    compile = do_compile_src
    lower = do_compile_src
    parse_body = do_parse_body
    kind = item_kind
    ping = ping_str
    lower_body = do_lower_body
    component_source = emit_component
    fixture_source = emit_named_fixture
    file_css = collect_file_css
    to_camel = pascal_to_camel
}

ping_str = |s| s

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
    BodyCss({ body : Span, span : Span }),
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
        BodyCss(_) => "Css"
    }

diag = |code, span, message| { code: code, span: span, message: message }

empty_block = |pos| { nodes: [], roots: [], span: { start: pos, end: pos } }

do_parse_body = |cur| {
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
        NoForest(rest) => { cur: rest.cur, block: empty_block(start), diagnostics: rest.diagnostics }
        Forest(got) => {
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
                NoForest(rest) => {
                    $cur = rest.cur
                    $diagnostics = List.concat($diagnostics, rest.diagnostics)
                    $loop = Bool.False
                }
                Forest(got) => {
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
        Forest({
            cur: interp.cur,
            nodes: [Interpolation({ expr: interp.node.expr, span: interp.node.span })],
            root: 0.U64,
            diagnostics: interp.diagnostics,
        })
    } else if Cursor.peek(cur) == Ok(64) {
        parse_directive(cur, stop_brace)
    } else {
        NoForest({ cur: cur, diagnostics: [] })
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
        BodyCss(n) => BodyCss(n)
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
        NoForest({ cur: cur, diagnostics: [] })
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
            Forest({
                cur: $cur,
                nodes: List.append(kids.nodes, Fragment({ children: kids.roots, span: { start: start, end: $cur.pos } })),
                root: List.len(kids.nodes),
                diagnostics: $frag_diag,
            })
        } else if Cursor.peek($cur) == Ok(47) {
            $cur = Cursor.bump($cur)
            $cur =
                match scan_tag_path($cur) {
                    NoForest(rest) => rest.cur
                    Forest(path) => path.cur
                }
            $cur = Cursor.skip_spaces_tabs($cur)
            gt = Cursor.eat($cur, 62)
            NoForest({
                cur: gt.cur,
                diagnostics: [diag("RC1005", { start: start, end: gt.cur.pos }, "unexpected closing tag")],
            })
        } else {
            match scan_tag_path($cur) {
                NoForest(rest) => NoForest({
                    cur: rest.cur,
                    diagnostics: List.append(
                        rest.diagnostics,
                        diag("RC1005", { start: $cur.pos, end: $cur.pos }, "expected tag name after `<`"),
                    ),
                })
                Forest(path_got) => {
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
                        Forest({ cur: $cur, nodes: [item], root: 0.U64, diagnostics: $diagnostics })
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
                        Forest({
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
        Err(rest) => NoForest({ cur: rest, diagnostics: [] })
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
            Forest({ cur: $cur, path: path, diagnostics: $diagnostics })
        }
    }
}

path_is_component = |path|
    match List.get(path.parts, 0) {
        Ok(part) => is_pascal(part.name)
        Err(_) => Bool.False
    }

path_roc_name = |parts| {
    last = sat_sub(List.len(parts))
    List.fold(
        parts,
        { i: 0.U64, s: "" },
        |acc, part| {
            name =
                if acc.i == last {
                    pascal_to_camel(part.name)
                } else {
                    part.name
                }
            s =
                if acc.i == 0 {
                    name
                } else {
                    Str.concat(acc.s, Str.concat(".", name))
                }
            { i: acc.i + 1, s: s }
        },
    ).s
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
        Forest({
            cur: $cur,
            nodes: [Text({ value: $value, span: { start: start, end: $cur.pos } })],
            root: 0.U64,
            diagnostics: [],
        })
    } else {
        var $cur = Cursor.bump(cur)
        match Cursor.scan_ident($cur) {
            Err(_) => NoForest({
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
                    NoForest({ cur: cur, diagnostics: [] })
                } else {
                    NoForest({
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
    Forest({
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
            Forest({
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
            Forest({
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
        Forest({
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
                    NoForest(rest) => {
                        $cur = rest.cur
                        $diagnostics = List.concat($diagnostics, rest.diagnostics)
                        $loop = Bool.False
                    }
                    Forest(pat) => {
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
                            NoForest(rest) => {
                                $cur = rest.cur
                                $diagnostics = List.concat($diagnostics, rest.diagnostics)
                            }
                            Forest(val) => {
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
        Forest({
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
        NoForest({
            cur: $cur,
            diagnostics: [diag("RC1001", { start: $cur.pos, end: $cur.pos }, "match arm must produce a tag, fragment, interpolation, or directive")],
        })
    }
}

parse_let = |cur, start| {
    var $cur = Cursor.skip_whitespace(cur)
    match Cursor.scan_ident($cur) {
        Err(_) => Forest({
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
            Forest({
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
        Forest({
            cur: $cur,
            nodes: [BodyCss({ body: { start: $cur.pos, end: $cur.pos }, span: { start: start, end: $cur.pos } })],
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
        Forest({
            cur: $cur,
            nodes: [
                BodyCss({
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
        NoForest({ cur: $cur, diagnostics: [diag("RC1001", { start: start, end: start }, "expected match pattern")] })
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
            NoForest({ cur: $cur, diagnostics: [diag("RC1001", { start: start, end: start }, "expected match pattern")] })
        } else {
            Forest({ cur: $cur, span: Cursor.trim_span($cur.text, start, $cur.pos), diagnostics: [] })
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

do_lower_body = |src, indent| {
    do_lower_html(src, indent, "", "")
}

do_lower_html = |src, indent, stamp, inject_css| {
    parsed = do_parse_body(Cursor.new(src))
    emit_html_value(parsed.block, src, stamp, indent, inject_css)
}

html_root_ids = |block| {
    List.fold(
        block.roots,
        [],
        |acc, id| {
            kind = root_kind(block, id)
            if kind == "Css" or kind == "LetDirective" {
                acc
            } else {
                List.append(acc, id)
            }
        },
    )
}

collect_body_css = |block, src| {
    List.fold(
        block.nodes,
        "",
        |acc, item| {
            kind = item_kind(item)
            if kind == "Css" {
                inner = body_css_text(src, item)
                if acc == "" {
                    inner
                } else if inner == "" {
                    acc
                } else {
                    Str.concat(acc, Str.concat("\n", inner))
                }
            } else {
                acc
            }
        },
    )
}

body_css_text = |src, item| {
    match item {
        BodyCss({ body, span: _ }) => Str.trim(css_inner(src, body))
        _ => ""
    }
}

css_inner = |src, span| {
    bytes = Str.to_utf8(Cursor.slice(src, span))
    if bytes.len() >= 2 {
        Cursor.slice(src, { start: span.start + 1, end: sat_sub(span.end) })
    } else {
        ""
    }
}

emit_html_value = |block, src, stamp, indent, inject_css| {
    ids = html_root_ids(block)
    if inject_css != "" {
        var $out = "Html.fragment(\n"
        $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "[\n"))
        $out = Str.concat(
            $out,
            Str.concat(emit_spaces(indent + 2), Str.concat(emit_style_element(inject_css, indent + 2), ",\n")),
        )
        inner =
            if List.len(ids) == 0 {
                "Html.empty"
            } else if List.len(ids) == 1 {
                match List.get(ids, 0) {
                    Ok(id) => emit_node(block, src, stamp, id, indent + 2)
                    Err(_) => "Html.empty"
                }
            } else {
                Str.concat("Html.fragment(\n", Str.concat(emit_node_array(block, src, stamp, ids, indent + 3), Str.concat(",\n", Str.concat(emit_spaces(indent + 2), ")"))))
            }
        $out = Str.concat($out, Str.concat(emit_spaces(indent + 2), Str.concat(inner, ",\n")))
        $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "],\n"))
        Str.concat($out, Str.concat(emit_spaces(indent), ")"))
    } else {
        emit_roots_ids(block, src, stamp, ids, indent)
    }
}

emit_style_element = |css, indent| {
    var $out = "Html.element(\n"
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "\"style\",\n"))
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "[],\n"))
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "[\n"))
    $out = Str.concat(
        $out,
        Str.concat(emit_spaces(indent + 2), Str.concat("Html.text(", Str.concat(emit_quote(css), "),\n"))),
    )
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "],\n"))
    Str.concat($out, Str.concat(emit_spaces(indent), ")"))
}

emit_roots = |block, src, stamp, indent| emit_roots_ids(block, src, stamp, block.roots, indent)

emit_roots_ids = |block, src, stamp, ids, indent| {
    if List.len(ids) == 0 {
        Str.concat(emit_spaces(indent), "Html.empty")
    } else if List.len(ids) == 1 {
        match List.get(ids, 0) {
            Ok(id) => Str.concat(emit_spaces(indent), emit_node(block, src, stamp, id, indent))
            Err(_) => Str.concat(emit_spaces(indent), "Html.empty")
        }
    } else {
        emit_node_array(block, src, stamp, ids, indent)
    }
}

emit_node_array = |block, src, stamp, ids, indent| {
    var $out = "[\n"
    var $i = 0.U64
    while $i < List.len(ids) {
        match List.get(ids, $i) {
            Ok(id) => {
                $out = Str.concat(
                    $out,
                    Str.concat(emit_spaces(indent + 1), Str.concat(emit_node(block, src, stamp, id, indent + 1), ",\n")),
                )
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.concat($out, Str.concat(emit_spaces(indent), "]"))
}

emit_node = |block, src, stamp, id, indent|
    match List.get(block.nodes, id) {
        Err(_) => "Html.empty"
        Ok(item) =>
            match item {
                Element({ name, attrs, children, self_closing, span: _ }) =>
                    emit_element(block, src, stamp, name.name, attrs, children, self_closing, indent)
                ComponentCall({ path, attrs, children, span: _ }) =>
                    emit_call(block, src, stamp, path, attrs, children, indent)
                Fragment({ children, span: _ }) =>
                    Str.concat("Html.fragment(\n", Str.concat(emit_node_array(block, src, stamp, children, indent + 1), Str.concat(",\n", Str.concat(emit_spaces(indent), ")"))))
                Text({ value, span: _ }) => Str.concat("Html.text(", Str.concat(emit_quote(value), ")"))
                Interpolation({ expr, span: _ }) => {
                    e = Str.trim(Cursor.slice(src, expr))
                    Str.concat("Html.text(", Str.concat(e, ")"))
                }
                IfDirective({ condition, then_roots, else_ifs, else_roots, span: _ }) =>
                    emit_if(block, src, stamp, condition, then_roots, else_ifs, else_roots, indent)
                ForDirective({ binder, collection, body_roots, span: _ }) =>
                    emit_for(block, src, stamp, binder, collection, body_roots, indent)
                MatchDirective({ scrutinee, arms, span: _ }) =>
                    emit_match(block, src, stamp, scrutinee, arms, indent)
                LetDirective(_) => "Html.empty"
                BodyCss(_) => "Html.empty"
            }
    }

emit_element = |block, src, stamp, name, attrs, children, self_closing, indent| {
    void_el = self_closing and is_void(name)
    head = if void_el { "Html.void_element(\n" } else { "Html.element(\n" }
    var $out = head
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), Str.concat(emit_quote(name), ",\n")))
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), emit_attrs(src, stamp, attrs, indent + 1)))
    if !void_el {
        $out = Str.concat($out, ",\n")
        $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), emit_node_array(block, src, stamp, children, indent + 1)))
    }
    $out = Str.concat($out, ",\n")
    $out = Str.concat($out, Str.concat(emit_spaces(indent), ")"))
    $out
}

emit_attrs = |src, stamp, attrs, indent| {
    if List.len(attrs) == 0 and stamp == "" {
        "[]"
    } else {
        var $out = "[\n"
        var $i = 0.U64
        while $i < List.len(attrs) {
            match List.get(attrs, $i) {
                Ok(attr) => {
                    $out = Str.concat(
                        $out,
                        Str.concat(emit_spaces(indent + 1), Str.concat(emit_attr(src, attr), ",\n")),
                    )
                }
                Err(_) => {}
            }
            $i = $i + 1
        }
        if stamp != "" {
            $out = Str.concat(
                $out,
                Str.concat(
                    emit_spaces(indent + 1),
                    Str.concat("Html.attribute(\"data-rocci-css\", ", Str.concat(emit_quote(stamp), "),\n")),
                ),
            )
        }
        Str.concat($out, Str.concat(emit_spaces(indent), "]"))
    }
}

emit_attr = |src, attr|
    match attr.value {
        Static({ value, span: _ }) =>
            Str.concat("Html.attribute(", Str.concat(emit_quote(attr.name.name), Str.concat(", ", Str.concat(emit_quote(value), ")"))))
        Expr({ expr }) =>
            Str.concat("Html.attribute(", Str.concat(emit_quote(attr.name.name), Str.concat(", ", Str.concat(Str.trim(Cursor.slice(src, expr)), ")"))))
        Action({ name, args }) =>
            Str.concat("Html.attribute(", Str.concat(emit_quote(attr.name.name), Str.concat(", ", Str.concat("Datastar.", Str.concat(name.name, Str.concat("(", Str.concat(Str.trim(Cursor.slice(src, args)), "))")))))))
        Boolean =>
            Str.concat("Html.boolean_attribute(", Str.concat(emit_quote(attr.name.name), ", True)"))
    }

emit_call = |block, src, stamp, path, attrs, children, indent| {
    var $out = Str.concat(path.roc_name, "(\n")
    $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), emit_props(src, attrs)))
    if List.len(children) > 0 {
        $out = Str.concat($out, ",\n")
        child =
            if List.len(children) == 1 {
                match List.get(children, 0) {
                    Ok(id) => emit_node(block, src, stamp, id, indent + 1)
                    Err(_) => "Html.empty"
                }
            } else {
                Str.concat("Html.fragment(\n", Str.concat(emit_node_array(block, src, stamp, children, indent + 2), Str.concat(",\n", Str.concat(emit_spaces(indent + 1), ")"))))
            }
        $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), child))
    }
    $out = Str.concat($out, ",\n")
    $out = Str.concat($out, Str.concat(emit_spaces(indent), ")"))
    $out
}

emit_props = |src, attrs| {
    if List.len(attrs) == 0 {
        "{}"
    } else {
        var $out = "{ "
        var $i = 0.U64
        while $i < List.len(attrs) {
            match List.get(attrs, $i) {
                Ok(attr) => {
                    if $i > 0 {
                        $out = Str.concat($out, ", ")
                    }
                    $out = Str.concat($out, Str.concat(attr.name.name, ": "))
                    $out = Str.concat($out, emit_prop_value(src, attr.value))
                }
                Err(_) => {}
            }
            $i = $i + 1
        }
        Str.concat($out, " }")
    }
}

emit_prop_value = |src, value|
    match value {
        Static({ value: v, span: _ }) => emit_quote(v)
        Expr({ expr }) => Str.trim(Cursor.slice(src, expr))
        Action({ name, args }) =>
            Str.concat("Datastar.", Str.concat(name.name, Str.concat("(", Str.concat(Str.trim(Cursor.slice(src, args)), ")"))))
        Boolean => "Bool.true"
    }

emit_if = |block, src, stamp, condition, then_roots, else_ifs, else_roots, indent| {
    cond = Str.trim(Cursor.slice(src, condition))
    var $out = Str.concat("if ", Str.concat(cond, " {\n"))
    $out = Str.concat($out, Str.concat(emit_roots_ids(block, src, stamp, then_roots, indent + 1), "\n"))
    $out = Str.concat($out, emit_spaces(indent))
    var $i = 0.U64
    while $i < List.len(else_ifs) {
        match List.get(else_ifs, $i) {
            Ok(arm) => {
                c = Str.trim(Cursor.slice(src, arm.condition))
                $out = Str.concat($out, Str.concat("} else if ", Str.concat(c, " {\n")))
                $out = Str.concat($out, Str.concat(emit_roots_ids(block, src, stamp, arm.roots, indent + 1), "\n"))
                $out = Str.concat($out, emit_spaces(indent))
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $out = Str.concat($out, "} else {\n")
    if List.len(else_roots) == 0 {
        $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), "Html.empty\n"))
    } else {
        $out = Str.concat($out, Str.concat(emit_roots_ids(block, src, stamp, else_roots, indent + 1), "\n"))
    }
    Str.concat($out, Str.concat(emit_spaces(indent), "}"))
}

emit_for = |block, src, stamp, binder, collection, body_roots, indent| {
    coll = Str.trim(Cursor.slice(src, collection))
    var $out = Str.concat("List.map(", Str.concat(coll, ", |"))
    $out = Str.concat($out, Str.concat(binder.name, "| {\n"))
    $out = Str.concat($out, Str.concat(emit_roots_ids(block, src, stamp, body_roots, indent + 1), "\n"))
    Str.concat($out, Str.concat(emit_spaces(indent), "})"))
}

emit_match = |block, src, stamp, scrutinee, arms, indent| {
    s = Str.trim(Cursor.slice(src, scrutinee))
    var $out = Str.concat("match ", Str.concat(s, " {\n"))
    var $i = 0.U64
    while $i < List.len(arms) {
        match List.get(arms, $i) {
            Ok(arm) => {
                pat = Str.trim(Cursor.slice(src, arm.pattern))
                $out = Str.concat($out, Str.concat(emit_spaces(indent + 1), Str.concat(pat, " => ")))
                $out = Str.concat($out, Str.concat(emit_node(block, src, stamp, arm.value, indent + 1), "\n"))
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.concat($out, Str.concat(emit_spaces(indent), "}"))
}

emit_spaces = |n| {
    var $s = ""
    var $i = 0.U64
    while $i < n {
        $s = Str.concat($s, "    ")
        $i = $i + 1
    }
    $s
}

emit_quote = |value| {
    bytes = Str.to_utf8(value)
    var $out = "\""
    var $i = 0.U64
    while $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(92) => {
                $out = Str.concat($out, "\\\\")
            }
            Ok(34) => {
                $out = Str.concat($out, "\\\"")
            }
            Ok(10) => {
                $out = Str.concat($out, "\\n")
            }
            Ok(13) => {
                $out = Str.concat($out, "\\r")
            }
            Ok(9) => {
                $out = Str.concat($out, "\\t")
            }
            Ok(_) => {
                ch = Cursor.slice(value, { start: $i, end: $i + 1 })
                $out = Str.concat($out, ch)
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.concat($out, "\"")
}

do_compile_src = |src, file_name| {
    bytes = Str.to_utf8(src)
    file_css = collect_file_css(src)
    imported_ds = str_contains(src, "import Datastar")
    needs_ds =
        str_contains(src, "@get(")
        or str_contains(src, "@post(")
        or str_contains(src, "@put(")
        or str_contains(src, "@patch(")
        or str_contains(src, "@delete(")
    var $out =
        if needs_ds and !imported_ds {
            "import Datastar\n\n"
        } else {
            ""
        }
    var $i = 0.U64
    var $opaque = 0.U64
    while $i < bytes.len() {
        $i = skip_ws_bytes(bytes, $i)
        if $i >= bytes.len() {
            break
        }
        if at_bytes(bytes, $i, "@component") {
            start_i = $i
            if $opaque < $i {
                $out = Str.concat($out, Cursor.slice(src, { start: $opaque, end: $i }))
            }
            taken = take_component_bytes(src, bytes, $i)
            $out = Str.concat(
                $out,
                emit_component(file_name, file_css, taken.name, taken.params, taken.body),
            )
            $opaque = taken.end
            $i = taken.end
            if $i <= start_i {
                $i = start_i + 1
            }
        } else if at_bytes(bytes, $i, "@fixture") {
            if $opaque < $i {
                $out = Str.concat($out, Cursor.slice(src, { start: $opaque, end: $i }))
            }
            taken = take_fixture_bytes(src, bytes, $i)
            $out = Str.concat($out, emit_named_fixture(taken.name, taken.value))
            $opaque = taken.end
            $i = taken.end
        } else if at_bytes(bytes, $i, "@css") {
            $i = skip_at_block_bytes(bytes, $i)
            $opaque = $i
        } else if at_bytes(bytes, $i, "@test") {
            $i = skip_test_bytes(bytes, $i)
            $opaque = $i
        } else if at_bytes(bytes, $i, "@") {
            $i = skip_at_block_bytes(bytes, $i)
            $opaque = $i
        } else {
            $i = $i + 1
        }
    }
    src_len = bytes.len()
    if $opaque < src_len {
        $out = Str.concat($out, Cursor.slice(src, { start: $opaque, end: src_len }))
    }
    $out
}

emit_component = |file_name, file_css, name, params, body| {
    roc_name = pascal_to_camel(name)
    pattern = param_pattern(params)
    parsed = do_parse_body(Cursor.new(body))
    comp_css = collect_body_css(parsed.block, body)
    file_id = if file_css == "" { "" } else { file_scope_id(file_name) }
    comp_id = if comp_css == "" { "" } else { component_scope_id(file_name, roc_name) }
    stamp = join_stamp(file_id, comp_id)
    inject = join_scoped_css(file_css, file_id, comp_css, comp_id)
    inner = emit_html_value(parsed.block, body, stamp, 1, inject)
    Str.concat(roc_name, Str.concat(" = ", Str.concat(pattern, Str.concat(" {\n", Str.concat(inner, "\n}\n")))))
}

emit_named_fixture = |name, value| {
    roc_name = pascal_to_camel(name)
    Str.concat(roc_name, Str.concat(" = ", Str.concat(value, "\n")))
}

join_stamp = |file_id, comp_id| {
    if file_id == "" {
        comp_id
    } else if comp_id == "" {
        file_id
    } else {
        Str.concat(file_id, Str.concat(" ", comp_id))
    }
}

join_scoped_css = |file_css, file_id, comp_css, comp_id| {
    file_part = if file_css == "" or file_id == "" { "" } else { scope_css(file_css, file_id) }
    comp_part = if comp_css == "" or comp_id == "" { "" } else { scope_css(comp_css, comp_id) }
    if file_part == "" {
        comp_part
    } else if comp_part == "" {
        file_part
    } else {
        Str.concat(file_part, Str.concat("\n", comp_part))
    }
}

scope_css = |css, id| {
    trimmed = Str.trim(css)
    Str.concat("@scope ([data-rocci-css~=\"", Str.concat(id, Str.concat("\"]) {\n", Str.concat(trimmed, "\n}"))))
}

collect_file_css = |src| {
    bytes = Str.to_utf8(src)
    var $i = 0.U64
    var $css = ""
    var $depth = 0.U64
    while $i < bytes.len() {
        if $depth == 0 and at_bytes(bytes, $i, "@css") {
            $i = skip_ws_bytes(bytes, $i + 4)
            if at_bytes(bytes, $i, "{") {
                end = skip_braces_bytes(bytes, $i)
                inner = Str.trim(Cursor.slice(src, { start: $i + 1, end: sat_sub(end) }))
                if $css == "" {
                    $css = inner
                } else {
                    $css = Str.concat($css, Str.concat("\n", inner))
                }
                $i = end
            } else {
                $i = $i + 1
            }
        } else {
            match List.get(bytes, $i) {
                Ok(34) => {
                    $i = $i + 1
                    while $i < bytes.len() and List.get(bytes, $i) != Ok(34) {
                        if List.get(bytes, $i) == Ok(92) {
                            $i = $i + 1
                        }
                        $i = $i + 1
                    }
                    $i = $i + 1
                }
                Ok(123) => {
                    $depth = $depth + 1
                    $i = $i + 1
                }
                Ok(125) => {
                    $depth = sat_sub($depth)
                    $i = $i + 1
                }
                _ => {
                    $i = $i + 1
                }
            }
        }
    }
    $css
}

at_bytes = |bytes, i, needle| {
    n = Str.to_utf8(needle)
    nlen = n.len()
    if i + nlen > bytes.len() {
        Bool.False
    } else {
        List.sublist(bytes, { start: i, len: nlen }) == n
    }
}

skip_ws_bytes = |bytes, i| {
    var $i = i
    var $loop = Bool.True
    while $loop and $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(32) => {
                $i = $i + 1
            }
            Ok(9) => {
                $i = $i + 1
            }
            Ok(10) => {
                $i = $i + 1
            }
            Ok(13) => {
                $i = $i + 1
            }
            _ => {
                $loop = Bool.False
            }
        }
    }
    $i
}

skip_braces_bytes = |bytes, i| {
    var $i = i
    var $depth = 0.U64
    var $loop = Bool.True
    while $loop and $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(34) => {
                $i = $i + 1
                while $i < bytes.len() and List.get(bytes, $i) != Ok(34) {
                    if List.get(bytes, $i) == Ok(92) {
                        $i = $i + 1
                    }
                    $i = $i + 1
                }
                $i = $i + 1
            }
            Ok(123) => {
                $depth = $depth + 1
                $i = $i + 1
            }
            Ok(125) => {
                $depth = sat_sub($depth)
                $i = $i + 1
                if $depth == 0 {
                    $loop = Bool.False
                }
            }
            _ => {
                $i = $i + 1
            }
        }
    }
    $i
}

take_component_bytes = |src, bytes, start| {
    var $i = start + 10
    $i = skip_ws_bytes(bytes, $i)
    name = scan_ident_bytes(src, bytes, $i)
    $i = skip_ws_bytes(bytes, name.end)
    if at_bytes(bytes, $i, "=") {
        $i = skip_ws_bytes(bytes, $i + 1)
    }
    params = take_pipes_bytes(src, bytes, $i)
    $i = skip_ws_bytes(bytes, params.end)
    body_start = $i
    if at_bytes(bytes, $i, "{") {
        $i = skip_braces_bytes(bytes, $i)
    }
    {
        name: name.text,
        params: params.text,
        body: Cursor.slice(src, { start: body_start, end: $i }),
        end: $i,
    }
}

take_fixture_bytes = |src, bytes, start| {
    var $i = start + 8
    $i = skip_ws_bytes(bytes, $i)
    if at_bytes(bytes, $i, "{") {
        $i = skip_braces_bytes(bytes, $i)
    }
    $i = skip_ws_bytes(bytes, $i)
    name = scan_ident_bytes(src, bytes, $i)
    $i = skip_ws_bytes(bytes, name.end)
    if at_bytes(bytes, $i, "=") {
        $i = $i + 1
    }
    $i = skip_ws_bytes(bytes, $i)
    val_start = $i
    $i = skip_expr_bytes(bytes, $i)
    {
        name: name.text,
        value: Str.trim(Cursor.slice(src, { start: val_start, end: $i })),
        end: $i,
    }
}

skip_test_bytes = |bytes, start| {
    var $i = start + 5
    $i = skip_ws_bytes(bytes, $i)
    if at_bytes(bytes, $i, "{") {
        $i = skip_braces_bytes(bytes, $i)
    }
    $i = skip_ws_bytes(bytes, $i)
    name = scan_ident_bytes_only(bytes, $i)
    $i = skip_ws_bytes(bytes, name)
    if at_bytes(bytes, $i, "=") {
        $i = $i + 1
    }
    skip_expr_bytes(bytes, skip_ws_bytes(bytes, $i))
}

skip_at_block_bytes = |bytes, start| {
    var $i = start + 1
    $i = scan_ident_bytes_only(bytes, $i)
    if at_bytes(bytes, $i, ":") {
        $i = scan_ident_bytes_only(bytes, $i + 1)
    }
    $i = skip_ws_bytes(bytes, $i)
    if at_bytes(bytes, $i, "(") {
        $i = skip_parens_bytes(bytes, $i)
    }
    $i = skip_ws_bytes(bytes, $i)
    if at_bytes(bytes, $i, "{") {
        skip_braces_bytes(bytes, $i)
    } else {
        skip_expr_bytes(bytes, $i)
    }
}

take_pipes_bytes = |src, bytes, start| {
    if !at_bytes(bytes, start, "|") {
        { text: "||", end: start }
    } else {
        var $i = start + 1
        while $i < bytes.len() and List.get(bytes, $i) != Ok(124) {
            $i = $i + 1
        }
        if at_bytes(bytes, $i, "|") {
            $i = $i + 1
        }
        { text: Cursor.slice(src, { start: start, end: $i }), end: $i }
    }
}

scan_ident_bytes = |src, bytes, start| {
    end = scan_ident_bytes_only(bytes, start)
    { text: Cursor.slice(src, { start: start, end: end }), end: end }
}

scan_ident_bytes_only = |bytes, start| {
    var $i = start
    if $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(b) if (b >= 65 and b <= 90) or (b >= 97 and b <= 122) or b == 95 => {
                $i = $i + 1
                while $i < bytes.len() {
                    match List.get(bytes, $i) {
                        Ok(c) if (c >= 65 and c <= 90) or (c >= 97 and c <= 122) or (c >= 48 and c <= 57) or c == 95 => {
                            $i = $i + 1
                        }
                        _ => break
                    }
                }
            }
            _ => {}
        }
    }
    $i
}

skip_parens_bytes = |bytes, i| {
    var $i = i
    var $depth = 0.U64
    var $loop = Bool.True
    while $loop and $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(40) => {
                $depth = $depth + 1
                $i = $i + 1
            }
            Ok(41) => {
                $depth = sat_sub($depth)
                $i = $i + 1
                if $depth == 0 {
                    $loop = Bool.False
                }
            }
            Ok(34) => {
                $i = $i + 1
                while $i < bytes.len() and List.get(bytes, $i) != Ok(34) {
                    $i = $i + 1
                }
                $i = $i + 1
            }
            _ => {
                $i = $i + 1
            }
        }
    }
    $i
}

skip_expr_bytes = |bytes, start| {
    var $i = start
    var $paren = 0.U64
    var $bracket = 0.U64
    var $brace = 0.U64
    var $loop = Bool.True
    while $loop and $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(10) if $paren == 0 and $bracket == 0 and $brace == 0 and $i > start => {
                $loop = Bool.False
            }
            Ok(64) if $paren == 0 and $bracket == 0 and $brace == 0 and $i > start => {
                $loop = Bool.False
            }
            Ok(40) => {
                $paren = $paren + 1
                $i = $i + 1
            }
            Ok(41) => {
                $paren = sat_sub($paren)
                $i = $i + 1
            }
            Ok(91) => {
                $bracket = $bracket + 1
                $i = $i + 1
            }
            Ok(93) => {
                $bracket = sat_sub($bracket)
                $i = $i + 1
            }
            Ok(123) => {
                $brace = $brace + 1
                $i = $i + 1
            }
            Ok(125) => {
                $brace = sat_sub($brace)
                $i = $i + 1
            }
            Ok(34) => {
                $i = $i + 1
                while $i < bytes.len() and List.get(bytes, $i) != Ok(34) {
                    if List.get(bytes, $i) == Ok(92) {
                        $i = $i + 1
                    }
                    $i = $i + 1
                }
                $i = $i + 1
            }
            _ => {
                $i = $i + 1
            }
        }
    }
    $i
}

file_scope_id = |file_name| {
    key = file_basename(file_name)
    stem = file_stem(key)
    hash = fnv1a32(Str.to_utf8(key))
    Str.concat(stem, Str.concat("-", hex8(hash)))
}

component_scope_id = |file_name, component| {
    key = file_basename(file_name)
    bytes = List.concat(Str.to_utf8(key), List.prepend(Str.to_utf8(component), 0))
    hash = fnv1a32(bytes)
    Str.concat(sanitize_ident(component), Str.concat("-", hex8(hash)))
}

file_basename = |path| {
    bytes = Str.to_utf8(path)
    var $i = bytes.len()
    var $start = 0.U64
    while $i > 0 {
        $i = $i - 1
        match List.get(bytes, $i) {
            Ok(47) => {
                $start = $i + 1
                break
            }
            Ok(92) => {
                $start = $i + 1
                break
            }
            _ => {}
        }
    }
    Cursor.slice(path, { start: $start, end: bytes.len() })
}

file_stem = |name| {
    bytes = Str.to_utf8(name)
    var $dot = bytes.len()
    var $i = bytes.len()
    var $found = Bool.False
    while $i > 0 and !$found {
        $i = $i - 1
        if List.get(bytes, $i) == Ok(46) {
            $dot = $i
            $found = Bool.True
        }
    }
    raw = if $found { Cursor.slice(name, { start: 0, end: $dot }) } else { name }
    sanitize_ident(raw)
}

sanitize_ident = |name| {
    bytes = Str.to_utf8(name)
    var $out = ""
    var $i = 0.U64
    while $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(b) if (b >= 48 and b <= 57) or (b >= 65 and b <= 90) or (b >= 97 and b <= 122) or b == 45 or b == 95 => {
                $out = Str.concat($out, Cursor.slice(name, { start: $i, end: $i + 1 }))
            }
            Ok(_) => {
                if $out != "" and !str_ends_with($out, "-") {
                    $out = Str.concat($out, "-")
                }
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    if $out == "" {
        "file"
    } else {
        $out
    }
}

fnv1a32 = |bytes| {
    var $hash = 2166136261.U64
    var $i = 0.U64
    while $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(b) => {
                $hash = U64.bitwise_xor($hash, U8.to_u64(b))
                $hash = $hash * 16777619
                $hash = U64.bitwise_and($hash, 4294967295)
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $hash
}

hex8 = |n| {
    var $out = ""
    var $v = n
    var $k = 0.U64
    while $k < 8 {
        nib = U64.bitwise_and($v, 15)
        $out = Str.concat(hex_digit(nib), $out)
        $v = $v / 16
        $k = $k + 1
    }
    $out
}

hex_digit = |n| {
    digits = "0123456789abcdef"
    Cursor.slice(digits, { start: n, end: n + 1 })
}

str_contains = |text, needle| {
    bytes = Str.to_utf8(text)
    n = Str.to_utf8(needle)
    nlen = n.len()
    if nlen == 0 {
        Bool.True
    } else {
        var $i = 0.U64
        var $found = Bool.False
        while $i + nlen <= bytes.len() and !$found {
            if List.sublist(bytes, { start: $i, len: nlen }) == n {
                $found = Bool.True
            } else {
                $i = $i + 1
            }
        }
        $found
    }
}

str_ends_with = |s, needle| {
    sb = Str.to_utf8(s)
    nb = Str.to_utf8(needle)
    nlen = nb.len()
    slen = sb.len()
    if nlen > slen {
        Bool.False
    } else {
        List.sublist(sb, { start: slen - nlen, len: nlen }) == nb
    }
}

extract_component_name = |src| {
    var $cur = Cursor.new(src)
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        if Cursor.starts_with($cur, "@component") {
            $cur = Cursor.eat_str($cur, "@component")
            $cur = Cursor.skip_trivia($cur)
            match Cursor.scan_ident($cur) {
                Ok(got) => {
                    return Cursor.ident_text(got.cur, got.span)
                }
                Err(_) => {
                    $loop = Bool.False
                }
            }
        } else {
            before = $cur.pos
            $cur = Cursor.bump($cur)
            if $cur.pos <= before {
                $loop = Bool.False
            }
        }
    }
    "component"
}

extract_param_pattern = |src| {
    var $cur = Cursor.new(src)
    var $loop = Bool.True
    while $loop and !Cursor.is_eof($cur) {
        if Cursor.peek($cur) == Ok(124) {
            start = $cur.pos
            $cur = Cursor.bump($cur)
            while !Cursor.is_eof($cur) and Cursor.peek($cur) != Ok(124) {
                $cur = Cursor.bump($cur)
            }
            if Cursor.peek($cur) == Ok(124) {
                $cur = Cursor.bump($cur)
            }
            raw = Cursor.slice(src, { start: start, end: $cur.pos })
            return param_pattern(raw)
        }
        $cur = Cursor.bump($cur)
    }
    "||"
}

param_pattern = |raw| {
    t = Str.trim(raw)
    inner = strip_pipes(t)
    inner_t = Str.trim(inner)
    if str_starts_with(inner_t, "{") {
        names = record_field_names(inner_t)
        if List.len(names) == 0 {
            "|{}|"
        } else {
            Str.concat("|", Str.concat(Str.concat("{ ", Str.concat(join_comma(names), " }")), "|"))
        }
    } else {
        t
    }
}

strip_pipes = |s| {
    bytes = Str.to_utf8(s)
    start =
        if List.get(bytes, 0) == Ok(124) {
            1.U64
        } else {
            0.U64
        }
    end = bytes.len()
    last = if end > 0 { end - 1 } else { 0.U64 }
    stop =
        if end > start and List.get(bytes, last) == Ok(124) {
            last
        } else {
            end
        }
    if stop > start {
        Cursor.slice(s, { start: start, end: stop })
    } else {
        ""
    }
}

record_field_names = |record| {
    bytes = Str.to_utf8(record)
    inner_start = 1.U64
    inner_end = sat_sub(bytes.len())
    inner = Cursor.slice(record, { start: inner_start, end: inner_end })
    list_idents(inner)
}

list_idents = |s| {
    bytes = Str.to_utf8(s)
    var $i = 0.U64
    var $names = []
    var $depth = 0.U64
    while $i < bytes.len() {
        match List.get(bytes, $i) {
            Ok(34) => {
                $i = $i + 1
                while $i < bytes.len() and List.get(bytes, $i) != Ok(34) {
                    if List.get(bytes, $i) == Ok(92) {
                        $i = $i + 1
                    }
                    $i = $i + 1
                }
                $i = $i + 1
            }
            Ok(123) => {
                $depth = $depth + 1
                $i = $i + 1
            }
            Ok(125) => {
                $depth = sat_sub($depth)
                $i = $i + 1
            }
            Ok(b) if $depth == 0 and is_ident_start(b) => {
                start = $i
                $i = $i + 1
                while $i < bytes.len() {
                    match List.get(bytes, $i) {
                        Ok(c) if is_ident_continue(c) => {
                            $i = $i + 1
                        }
                        _ => break
                    }
                }
                name = Cursor.slice(s, { start: start, end: $i })
                if name != "in" {
                    $names = List.append($names, name)
                }
                while $i < bytes.len() and List.get(bytes, $i) != Ok(44) {
                    match List.get(bytes, $i) {
                        Ok(34) => {
                            $i = $i + 1
                            while $i < bytes.len() and List.get(bytes, $i) != Ok(34) {
                                $i = $i + 1
                            }
                        }
                        _ => {
                            $i = $i + 1
                        }
                    }
                }
            }
            _ => {
                $i = $i + 1
            }
        }
    }
    $names
}

join_comma = |names| {
    List.fold(
        names,
        { i: 0.U64, s: "" },
        |acc, name|
            if acc.i == 0 {
                { i: 1.U64, s: name }
            } else {
                { i: acc.i + 1, s: Str.concat(acc.s, Str.concat(", ", name)) }
            },
    ).s
}

is_ident_start = |b| (b >= 65 and b <= 90) or (b >= 97 and b <= 122) or b == 95

is_ident_continue = |b| is_ident_start(b) or (b >= 48 and b <= 57)

extract_template_body = |src| {
    var $cur = Cursor.new(src)
    var $seen_eq = Bool.False
    while !Cursor.is_eof($cur) {
        if !$seen_eq and Cursor.peek($cur) == Ok(61) {
            $seen_eq = Bool.True
            $cur = Cursor.bump($cur)
            $cur = Cursor.skip_trivia($cur)
            if Cursor.peek($cur) == Ok(124) {
                $cur = Cursor.bump($cur)
                while !Cursor.is_eof($cur) and Cursor.peek($cur) != Ok(124) {
                    $cur = Cursor.bump($cur)
                }
                if Cursor.peek($cur) == Ok(124) {
                    $cur = Cursor.bump($cur)
                }
            }
        } else if $seen_eq and Cursor.peek($cur) == Ok(123) {
            start = $cur.pos
            $cur = Cursor.skip_balanced_braces($cur)
            return Cursor.slice(src, { start: start, end: $cur.pos })
        } else {
            $cur = Cursor.bump($cur)
        }
    }
    "{}"
}

str_starts_with = |s, needle| {
    sb = Str.to_utf8(s)
    nb = Str.to_utf8(needle)
    nlen = nb.len()
    if nlen > sb.len() {
        Bool.False
    } else {
        List.sublist(sb, { start: 0, len: nlen }) == nb
    }
}

expect {
    out = do_parse_body(Cursor.new("<p/>"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element"
        Err(_) => Bool.False
    }
}

expect {
    out = do_parse_body(Cursor.new("<Hello />"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "ComponentCall"
        Err(_) => Bool.False
    }
}

expect {
    out = do_parse_body(Cursor.new("<br>"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element"
        Err(_) => Bool.False
    }
}

expect {
    out = do_parse_body(Cursor.new("{ <p>Hello, {name}!</p> }"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element" and nodes_have_kind(out.block.nodes, "Interpolation")
        Err(_) => Bool.False
    }
}

expect {
    out = do_parse_body(Cursor.new("<p>{name"))
    List.len(out.diagnostics) > 0
}

expect {
    out = do_parse_body(Cursor.new("<Badge>ok</Badge>"))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "ComponentCall"
        Err(_) => Bool.False
    }
}

expect {
    out = do_parse_body(Cursor.new("<div>x"))
    List.len(out.diagnostics) > 0
}

expect {
    src = "{ @if ready { <p>ok</p> } @else { <p>no</p> } }"
    out = do_parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "IfDirective"
        Err(_) => Bool.False
    }
}

expect {
    src = "{ @for item in items { <li>{item}</li> } }"
    out = do_parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "ForDirective"
        Err(_) => Bool.False
    }
}

expect {
    src = "{ @match x { Ok(v) => <p>{v}</p> Err(_) => <p>no</p> } }"
    out = do_parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "MatchDirective"
        Err(_) => Bool.False
    }
}

expect {
    src = "{ @let n = 1\n<p>{n}</p> }"
    out = do_parse_body(Cursor.new(src))
    nodes_have_kind(out.block.nodes, "LetDirective")
}

expect {
    src = "{\n    <p>Hello, {name}!</p>\n}"
    out = do_parse_body(Cursor.new(src))
    match List.get(out.block.roots, 0) {
        Ok(id) => root_kind(out.block, id) == "Element" and nodes_have_kind(out.block.nodes, "Interpolation")
        Err(_) => Bool.False
    }
}

hello_emitted = "hello = |{ name }| {\n    Html.element(\n        \"p\",\n        [],\n        [\n            Html.text(\"Hello, \"),\n            Html.text(name),\n            Html.text(\"!\"),\n        ],\n    )\n}\n\n"

expect {
    inner = do_lower_body("{\n    <p>Hello, {name}!</p>\n}", 1)
    got = Str.concat("hello = |{ name }| {\n", Str.concat(inner, "\n}\n\n"))
    got == hello_emitted
}

expect {
    src = "@component Hello = |{ name : Str }| {\n    <p>Hello, {name}!</p>\n}\n"
    do_compile_src(src, "hello.rocci") == hello_emitted
}

branch_emitted = "branch = |{ ready }| {\n    if ready {\n        Html.element(\n            \"p\",\n            [],\n            [\n                Html.text(\"ok\"),\n            ],\n        )\n    } else {\n        Html.element(\n            \"p\",\n            [],\n            [\n                Html.text(\"no\"),\n            ],\n        )\n    }\n}\n\n"

expect {
    src = "@component Branch = |{ ready : Bool }| {\n    @if ready {\n        <p>ok</p>\n    } @else {\n        <p>no</p>\n    }\n}\n"
    do_compile_src(src, "branch.rocci") == branch_emitted
}

expect {
    src = "@css {\n    body { margin: 0; }\n}\n\n@component Card = |{}| {\n    @css {\n        .card { color: red; }\n    }\n    <div class=\"card\">x</div>\n}\n"
    got = do_compile_src(src, "css.rocci")
    str_contains(got, "css-e7b6899e")
    and str_contains(got, "card-98509670")
    and str_contains(got, "Html.fragment")
    and str_contains(got, "data-rocci-css")
}

expect {
    src = "@component Hello = |{ name : Str }| {\n    <p>{name}</p>\n}\n\n@fixture{target: Hello}\nhelloSample = { name: \"Ada\" }\n\n@test helloNamePresent = Bool.true\n"
    got = do_compile_src(src, "markers.rocci")
    str_contains(got, "helloSample = { name: \"Ada\" }") and !str_contains(got, "@test") and !str_contains(got, "@fixture")
}
