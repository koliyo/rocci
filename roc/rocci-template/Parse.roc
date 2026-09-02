import Cursor

Span : { start : U64, end : U64 }

Ident : { name : Str, span : Span }

TemplateBlock : { span : Span }

ComponentDecl : {
    name : Ident,
    params : Span,
    body : TemplateBlock,
    span : Span,
}

FixtureDecl : {
    name : Ident,
    target : Ident,
    value : Span,
    span : Span,
}

TestDecl : {
    name : Ident,
    value : Span,
    span : Span,
}

CssDecl : {
    body : Span,
    span : Span,
}

ModuleItem : [
    RocRegion({ span : Span }),
    Component(ComponentDecl),
    Fixture(FixtureDecl),
    Test(TestDecl),
    Css(CssDecl),
]

Parse := [].{
    parse = do_parse
}

sat_sub = |n| {
    if n == 0 {
        0
    } else {
        n - 1
    }
}

is_expr_stop = |byte_try|
    match byte_try {
        Err(_) => Bool.True
        Ok(10) => Bool.True
        Ok(13) => Bool.True
        Ok(64) => Bool.True
        Ok(125) => Bool.True
        Ok(41) => Bool.True
        Ok(93) => Bool.True
        _ => Bool.False
    }

do_parse = |src| {
    var $cur = Cursor.new(src)
    var $items = []
    var $diagnostics = []
    var $opaque_start = 0.U64
    start = 0
    while !Cursor.is_eof($cur) {
        saved = $cur
        $cur = Cursor.skip_trivia($cur)
        if Cursor.is_eof($cur) {
            break
        }
        if Cursor.is_top_level($cur) {
            match try_parse_decl($cur) {
                Hit({ item, cur, diagnostics }) => {
                    item_start = item_span(item).start
                    if $opaque_start < item_start {
                        $items = List.append(
                            $items,
                            RocRegion({ span: { start: $opaque_start, end: item_start } }),
                        )
                    }
                    $items = List.append($items, item)
                    $diagnostics = List.concat($diagnostics, diagnostics)
                    $opaque_start = cur.pos
                    $cur = cur
                }
                Skip({ cur, diagnostics }) => {
                    $diagnostics = List.concat($diagnostics, diagnostics)
                    $opaque_start = cur.pos
                    $cur = cur
                }
                Miss => {
                    $cur = saved
                    $cur = Cursor.skip_roc_token($cur)
                    if $cur.pos == saved.pos {
                        $cur = Cursor.bump($cur)
                    }
                }
            }
        } else {
            $cur = saved
            $cur = Cursor.skip_roc_token($cur)
            if $cur.pos == saved.pos {
                $cur = Cursor.bump($cur)
            }
        }
    }
    src_len = Str.to_utf8(src).len()
    if $opaque_start < src_len {
        $items = List.append(
            $items,
            RocRegion({ span: { start: $opaque_start, end: src_len } }),
        )
    }
    {
        document: { items: $items, span: { start: start, end: src_len } },
        diagnostics: $diagnostics,
    }
}

item_span = |item|
    match item {
        RocRegion({ span }) => span
        Component(decl) => decl.span
        Fixture(decl) => decl.span
        Test(decl) => decl.span
        Css(decl) => decl.span
    }

try_parse_decl = |cur| {
    match try_parse_fixture(cur) {
        Hit(got) => Hit({ item: Fixture(got.decl), cur: got.cur, diagnostics: got.diagnostics })
        Skip(got) => Skip(got)
        Miss =>
            match try_parse_test(cur) {
                Hit(got) => Hit({ item: Test(got.decl), cur: got.cur, diagnostics: got.diagnostics })
                Skip(got) => Skip(got)
                Miss =>
                    match try_parse_component(cur) {
                        Hit(got) => Hit({ item: Component(got.decl), cur: got.cur, diagnostics: got.diagnostics })
                        Skip(got) => Skip(got)
                        Miss =>
                            match try_parse_css(cur) {
                                Hit(got) => Hit({ item: Css(got.decl), cur: got.cur, diagnostics: got.diagnostics })
                                Skip(got) => Skip(got)
                                Miss => try_skip_unknown_at(cur)
                            }
                    }
            }
    }
}

scan_at_keyword = |cur, keyword| {
    saved = cur
    got = Cursor.eat(cur, 64)
    if !got.eaten {
        Err(saved)
    } else {
        match Cursor.scan_ident(got.cur) {
            Ok(ident) => {
                if Cursor.ident_text(ident.cur, ident.span) == keyword {
                    Ok(ident.cur)
                } else {
                    Err(saved)
                }
            }
            Err(_) => Err(saved)
        }
    }
}

try_parse_component = |cur| {
    at = cur.pos
    match scan_at_keyword(cur, "component") {
        Err(_) => Miss
        Ok(after) => {
            var $cur = Cursor.skip_trivia(after)
            var $diagnostics = []
            match Cursor.scan_ident($cur) {
                Err(_) => {
                    $diagnostics = List.append(
                        $diagnostics,
                        {
                            code: "RC1001",
                            span: { start: at, end: $cur.pos },
                            message: "expected component name after `@component`",
                        },
                    )
                    $cur = sync_to_next_top_level($cur)
                    Hit({
                        decl: {
                            name: { name: "", span: { start: $cur.pos, end: $cur.pos } },
                            params: { start: $cur.pos, end: $cur.pos },
                            body: { span: { start: $cur.pos, end: $cur.pos } },
                            span: { start: at, end: $cur.pos },
                        },
                        cur: $cur,
                        diagnostics: $diagnostics,
                    })
                }
                Ok(ident) => {
                    $cur = ident.cur
                    name = {
                        name: Cursor.ident_text($cur, ident.span),
                        span: ident.span,
                    }
                    $cur = Cursor.skip_trivia($cur)
                    eq = Cursor.eat($cur, 61)
                    if !eq.eaten {
                        $diagnostics = List.append(
                            $diagnostics,
                            {
                                code: "RC1001",
                                span: ident.span,
                                message: "expected `=` after component name",
                            },
                        )
                    }
                    $cur = Cursor.skip_trivia(eq.cur)
                    match scan_params($cur) {
                        Err(rest) => {
                            $diagnostics = List.append(
                                $diagnostics,
                                {
                                    code: "RC1001",
                                    span: { start: at, end: rest.pos },
                                    message: "expected `|params|` after `@component Name =`",
                                },
                            )
                            rest2 = sync_to_next_top_level(rest)
                            Hit({
                                decl: {
                                    name: name,
                                    params: { start: rest2.pos, end: rest2.pos },
                                    body: { span: { start: rest2.pos, end: rest2.pos } },
                                    span: { start: at, end: rest2.pos },
                                },
                                cur: rest2,
                                diagnostics: $diagnostics,
                            })
                        }
                        Ok(params) => {
                            $cur = Cursor.skip_trivia(params.cur)
                            body = parse_component_body($cur)
                            Hit({
                                decl: {
                                    name: name,
                                    params: params.span,
                                    body: body.block,
                                    span: { start: at, end: body.cur.pos },
                                },
                                cur: body.cur,
                                diagnostics: List.concat($diagnostics, body.diagnostics),
                            })
                        }
                    }
                }
            }
        }
    }
}

scan_params = |cur| {
    if Cursor.peek(cur) != Ok(124) {
        Err(cur)
    } else {
        start = cur.pos
        var $cur = Cursor.bump(cur)
        var $paren = 0.U64
        var $bracket = 0.U64
        var $brace = 0.U64
        var $done = Bool.False
        while !Cursor.is_eof($cur) and !$done {
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
                Ok(124) if $paren == 0 and $bracket == 0 and $brace == 0 => {
                    $cur = Cursor.bump($cur)
                    $done = Bool.True
                }
                _ => {
                    $cur = Cursor.bump($cur)
                }
            }
            if !$done and $cur.pos <= before {
                $cur = Cursor.bump($cur)
            }
        }
        Ok({ cur: $cur, span: { start: start, end: $cur.pos } })
    }
}

parse_component_body = |cur| {
    var $cur = Cursor.skip_trivia(cur)
    var $diagnostics = []
    match Cursor.peek($cur) {
        Ok(123) => {
            start = $cur.pos
            $cur = Cursor.skip_balanced_braces($cur)
            { cur: $cur, block: { span: { start: start, end: $cur.pos } }, diagnostics: $diagnostics }
        }
        Ok(60) => {
            start = $cur.pos
            $cur = skip_html_span($cur)
            { cur: $cur, block: { span: { start: start, end: $cur.pos } }, diagnostics: $diagnostics }
        }
        _ => {
            $diagnostics = List.append(
                $diagnostics,
                {
                    code: "RC1001",
                    span: { start: $cur.pos, end: $cur.pos },
                    message: "expected `{` to open a template body, or a single HTML tag",
                },
            )
            { cur: $cur, block: { span: { start: $cur.pos, end: $cur.pos } }, diagnostics: $diagnostics }
        }
    }
}

skip_html_span = |cur| {
    var $cur = cur
    if Cursor.peek($cur) != Ok(60) {
        $cur
    } else {
        $cur = Cursor.bump($cur)
        while !Cursor.is_eof($cur) and Cursor.peek($cur) != Ok(62) {
            before = $cur.pos
            if Cursor.peek($cur) == Ok(34) {
                $cur = Cursor.skip_string($cur)
            } else {
                $cur = Cursor.bump($cur)
            }
            if $cur.pos <= before {
                $cur = Cursor.bump($cur)
            }
        }
        if Cursor.peek($cur) == Ok(62) {
            Cursor.bump($cur)
        } else {
            $cur
        }
    }
}

try_parse_css = |cur| {
    at = cur.pos
    match scan_at_keyword(cur, "css") {
        Err(_) => Miss
        Ok(after) => {
            var $cur = Cursor.skip_trivia(after)
            var $diagnostics = []
            body = scan_css_block($cur)
            Hit({
                decl: { body: body.span, span: { start: at, end: body.cur.pos } },
                cur: body.cur,
                diagnostics: List.concat($diagnostics, body.diagnostics),
            })
        }
    }
}

scan_css_block = |cur| {
    got = Cursor.eat(cur, 123)
    if !got.eaten {
        {
            cur: cur,
            span: { start: cur.pos, end: cur.pos },
            diagnostics: [
                {
                    code: "RC1001",
                    span: { start: cur.pos, end: cur.pos },
                    message: "expected `{` to open a `@css` block",
                },
            ],
        }
    } else {
        body_start = got.cur.pos
        var $cur = got.cur
        var $depth = 1.U64
        while !Cursor.is_eof($cur) and $depth > 0 {
            before = $cur.pos
            match Cursor.peek($cur) {
                Ok(34) => {
                    $cur = skip_css_string($cur)
                }
                Ok(39) => {
                    $cur = skip_css_string($cur)
                }
                Ok(47) if Cursor.starts_with($cur, "/*") => {
                    $cur = skip_css_comment($cur)
                }
                Ok(123) => {
                    $cur = Cursor.bump($cur)
                    $depth = $depth + 1
                }
                Ok(125) => {
                    $cur = Cursor.bump($cur)
                    $depth = $depth - 1
                }
                _ => {
                    $cur = Cursor.bump($cur)
                }
            }
            if $cur.pos <= before {
                $cur = Cursor.bump($cur)
            }
        }
        end =
            if $depth == 0 and $cur.pos > 0 {
                $cur.pos - 1
            } else {
                $cur.pos
            }
        diagnostics =
            if $depth != 0 {
                [
                    {
                        code: "RC1002",
                        span: { start: body_start - 1, end: $cur.pos },
                        message: "unterminated `@css` block; expected `}`",
                    },
                ]
            } else {
                []
            }
        { cur: $cur, span: { start: body_start, end: end }, diagnostics: diagnostics }
    }
}

skip_css_string = |cur| {
    match Cursor.peek(cur) {
        Ok(quote) => {
            var $cur = Cursor.bump(cur)
            while !Cursor.is_eof($cur) {
                match Cursor.peek($cur) {
                    Ok(92) => {
                        $cur = Cursor.bump($cur)
                        $cur = Cursor.bump($cur)
                    }
                    Ok(b) if b == quote => {
                        return Cursor.bump($cur)
                    }
                    _ => {
                        $cur = Cursor.bump($cur)
                    }
                }
            }
            $cur
        }
        _ => cur
    }
}

skip_css_comment = |cur| {
    var $cur = { ..cur, pos: cur.pos + 2 }
    while !Cursor.is_eof($cur) {
        if Cursor.starts_with($cur, "*/") {
            return Cursor.eat_str($cur, "*/")
        }
        $cur = Cursor.bump($cur)
    }
    $cur
}

try_parse_fixture = |cur| {
    at = cur.pos
    match scan_at_keyword(cur, "fixture") {
        Err(_) => Miss
        Ok(after) => {
            var $cur = Cursor.skip_trivia(after)
            var $diagnostics = []
            target = parse_fixture_target($cur)
            $cur = target.cur
            $diagnostics = List.concat($diagnostics, target.diagnostics)
            $cur = Cursor.skip_trivia($cur)
            match Cursor.scan_ident($cur) {
                Err(_) => {
                    $diagnostics = List.append(
                        $diagnostics,
                        {
                            code: "RC1001",
                            span: { start: at, end: $cur.pos },
                            message: "expected fixture name after `@fixture`",
                        },
                    )
                    $cur = sync_to_next_top_level($cur)
                    Hit({
                        decl: {
                            name: { name: "", span: { start: $cur.pos, end: $cur.pos } },
                            target: target.ident,
                            value: { start: $cur.pos, end: $cur.pos },
                            span: { start: at, end: $cur.pos },
                        },
                        cur: $cur,
                        diagnostics: $diagnostics,
                    })
                }
                Ok(ident) => {
                    $cur = ident.cur
                    name = {
                        name: Cursor.ident_text($cur, ident.span),
                        span: ident.span,
                    }
                    $cur = Cursor.skip_trivia($cur)
                    eq = Cursor.eat($cur, 61)
                    $cur = eq.cur
                    value = scan_roc_expr($cur)
                    Hit({
                        decl: {
                            name: name,
                            target: target.ident,
                            value: value.span,
                            span: { start: at, end: value.cur.pos },
                        },
                        cur: value.cur,
                        diagnostics: $diagnostics,
                    })
                }
            }
        }
    }
}

parse_fixture_target = |cur| {
    empty = { name: "", span: { start: cur.pos, end: cur.pos } }
    if Cursor.peek(cur) != Ok(123) {
        {
            cur: cur,
            ident: empty,
            diagnostics: [
                {
                    code: "RC1001",
                    span: { start: cur.pos, end: cur.pos },
                    message: "expected `{target: ...}` after `@fixture`",
                },
            ],
        }
    } else {
        var $cur = Cursor.bump(cur)
        var $ident = empty
        var $diagnostics = []
        var $done = Bool.False
        while !Cursor.is_eof($cur) and !$done {
            $cur = Cursor.skip_trivia($cur)
            close = Cursor.eat($cur, 125)
            if close.eaten {
                $cur = close.cur
                $done = Bool.True
            } else {
                match Cursor.scan_ident($cur) {
                    Err(_) => {
                        $done = Bool.True
                    }
                    Ok(key) => {
                        $cur = Cursor.skip_trivia(key.cur)
                        colon = Cursor.eat($cur, 58)
                        $cur = Cursor.skip_trivia(colon.cur)
                        match Cursor.scan_ident($cur) {
                            Ok(val) => {
                                $ident = {
                                    name: Cursor.ident_text(val.cur, val.span),
                                    span: val.span,
                                }
                                $cur = val.cur
                            }
                            Err(rest) => {
                                $cur = rest
                            }
                        }
                    }
                }
            }
        }
        { cur: $cur, ident: $ident, diagnostics: $diagnostics }
    }
}

try_parse_test = |cur| {
    at = cur.pos
    match scan_at_keyword(cur, "test") {
        Err(_) => Miss
        Ok(after) => {
            var $cur = Cursor.skip_trivia(after)
            if Cursor.peek($cur) == Ok(123) {
                $cur = Cursor.skip_balanced_braces($cur)
                $cur = Cursor.skip_trivia($cur)
            }
            match Cursor.scan_ident($cur) {
                Err(_) => {
                    rest = sync_to_next_top_level($cur)
                    Hit({
                        decl: {
                            name: { name: "", span: { start: rest.pos, end: rest.pos } },
                            value: { start: rest.pos, end: rest.pos },
                            span: { start: at, end: rest.pos },
                        },
                        cur: rest,
                        diagnostics: [
                            {
                                code: "RC1001",
                                span: { start: at, end: $cur.pos },
                                message: "expected test name after `@test`",
                            },
                        ],
                    })
                }
                Ok(ident) => {
                    $cur = ident.cur
                    name = {
                        name: Cursor.ident_text($cur, ident.span),
                        span: ident.span,
                    }
                    $cur = Cursor.skip_trivia($cur)
                    eq = Cursor.eat($cur, 61)
                    value = scan_roc_expr(eq.cur)
                    Hit({
                        decl: {
                            name: name,
                            value: value.span,
                            span: { start: at, end: value.cur.pos },
                        },
                        cur: value.cur,
                        diagnostics: [],
                    })
                }
            }
        }
    }
}

try_skip_unknown_at = |cur| {
    if Cursor.peek(cur) != Ok(64) {
        Miss
    } else {
        at = cur.pos
        got = Cursor.eat(cur, 64)
        match Cursor.scan_ident(got.cur) {
            Err(_) => Miss
            Ok(ident) => {
                kw = Cursor.ident_text(ident.cur, ident.span)
                if kw == "context" or kw == "init" or kw == "get" or kw == "post" or kw == "put" or kw == "patch" or kw == "delete" or kw == "live" or kw == "view" or kw == "command" or kw == "on" or kw == "action" {
                    var $cur = ident.cur
                    if Cursor.peek($cur) == Ok(58) {
                        $cur = Cursor.bump($cur)
                        match Cursor.scan_ident($cur) {
                            Ok(role) => {
                                $cur = role.cur
                            }
                            Err(rest) => {
                                $cur = rest
                            }
                        }
                    }
                    $cur = Cursor.skip_trivia($cur)
                    if Cursor.peek($cur) == Ok(40) {
                        $cur = Cursor.skip_balanced_parens($cur)
                    }
                    $cur = Cursor.skip_trivia($cur)
                    if Cursor.peek($cur) == Ok(123) {
                        $cur = Cursor.skip_balanced_braces($cur)
                    } else {
                        value = scan_roc_expr($cur)
                        $cur = value.cur
                    }
                    Skip({
                        cur: $cur,
                        diagnostics: [
                            {
                                code: "RC1003",
                                span: { start: at, end: ident.span.end },
                                message: "skipped `@${kw}` in the template-subset POC",
                            },
                        ],
                    })
                } else {
                    Miss
                }
            }
        }
    }
}

scan_roc_expr = |cur| {
    var $cur = Cursor.skip_trivia(cur)
    start = $cur.pos
    if Cursor.is_eof($cur) or (Cursor.is_top_level($cur) and Cursor.peek($cur) == Ok(64)) {
        { cur: $cur, span: { start: start, end: start } }
    } else {
        start_paren = $cur.paren
        start_bracket = $cur.bracket
        start_brace = $cur.brace
        at_start = |c| c.paren == start_paren and c.bracket == start_bracket and c.brace == start_brace
        var $loop = Bool.True
        while $loop {
            if Cursor.is_eof($cur) {
                $loop = Bool.False
            } else {
                if $cur.pos > start and at_start($cur) {
                    saved = $cur
                    $cur = Cursor.skip_spaces_tabs($cur)
                    if is_expr_stop(Cursor.peek($cur)) {
                        $cur = saved
                        $loop = Bool.False
                    } else {
                        $cur = saved
                    }
                }
                if $loop {
                    $cur = Cursor.skip_roc_token($cur)
                    while !Cursor.is_eof($cur) and !at_start($cur) {
                        $cur = Cursor.skip_roc_token($cur)
                    }
                }
            }
        }
        { cur: $cur, span: Cursor.trim_span($cur.text, start, $cur.pos) }
    }
}

sync_to_next_top_level = |cur| {
    var $cur = cur
    while !Cursor.is_eof($cur) {
        if at_column_zero_def($cur) {
            break
        }
        $cur = Cursor.bump($cur)
    }
    $cur
}

at_column_zero_def = |cur| {
    if cur.pos == 0 {
        Bool.False
    } else {
        match List.get(cur.src, cur.pos - 1) {
            Ok(10) => {
                look = cur
                match Cursor.peek(look) {
                    Ok(32) => Bool.False
                    Ok(9) => Bool.False
                    _ => {
                        at = Cursor.eat(look, 64)
                        if at.eaten {
                            match Cursor.scan_ident(at.cur) {
                                Ok(ident) => {
                                    kw = Cursor.ident_text(ident.cur, ident.span)
                                    kw == "component"
                                    or kw == "fixture"
                                    or kw == "test"
                                    or kw == "css"
                                    or kw == "context"
                                    or kw == "init"
                                    or kw == "get"
                                    or kw == "post"
                                    or kw == "put"
                                    or kw == "delete"
                                    or kw == "live"
                                    or kw == "view"
                                    or kw == "patch"
                                    or kw == "command"
                                    or kw == "on"
                                    or kw == "action"
                                }
                                Err(_) => Bool.False
                            }
                        } else {
                            match Cursor.scan_ident(look) {
                                Ok(ident) => {
                                    after = Cursor.skip_trivia(ident.cur)
                                    Cursor.peek(after) == Ok(61)
                                }
                                Err(_) => Bool.False
                            }
                        }
                    }
                }
            }
            _ => Bool.False
        }
    }
}

item_kind = |item|
    match item {
        RocRegion(_) => "RocRegion"
        Component(_) => "ComponentDecl"
        Fixture(_) => "FixtureDecl"
        Test(_) => "TestDecl"
        Css(_) => "CssDecl"
    }

component_name = |item|
    match item {
        Component(decl) => decl.name.name
        _ => ""
    }

component_params = |item, src|
    match item {
        Component(decl) => Cursor.slice(src, decl.params)
        _ => ""
    }

list_has = |items, needle| {
    List.fold(
        items,
        Bool.False,
        |acc, item| acc or item == needle,
    )
}

list_any = |items, pred| {
    List.fold(
        items,
        Bool.False,
        |acc, item| acc or pred(item),
    )
}

str_contains = |text, needle| {
    bytes = Str.to_utf8(text)
    n = Str.to_utf8(needle)
    nlen = n.len()
    if nlen == 0 {
        Bool.True
    } else {
        var $i = 0
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

expect {
    src = "module X exposing [x]\n\n@component Name = |{ }| <p/>\n"
    out = Parse.parse(src)
    kinds = List.map(out.document.items, item_kind)
    names = List.map(out.document.items, component_name)
    params = List.map(out.document.items, |item| component_params(item, src))
    list_has(kinds, "ComponentDecl")
    and list_has(names, "Name")
    and list_any(params, |p| p == "|{ }|")
}

expect {
    src = "@component A = |_| <p/>\n\nhelper = |x| x + 1\n\n@component B = |_| <span/>\n"
    out = Parse.parse(src)
    kinds = List.map(out.document.items, item_kind)
    helper_kept = list_any(
        out.document.items,
        |item|
            match item {
                RocRegion({ span }) => str_contains(Cursor.slice(src, span), "helper")
                _ => Bool.False
            },
    )
    list_has(kinds, "ComponentDecl") and helper_kept
}

expect {
    src = "@get:view(\"/\") {\n    x\n}\n\n@component Hello = |{ }| <p/>\n"
    out = Parse.parse(src)
    kinds = List.map(out.document.items, item_kind)
    list_has(kinds, "ComponentDecl")
    and list_any(out.diagnostics, |d| str_contains(d.message, "skipped"))
}
