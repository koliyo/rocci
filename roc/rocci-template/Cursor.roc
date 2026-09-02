Cursor := [].{
    new = cursor_new
    at = cursor_at
    is_eof = cursor_is_eof
    peek = cursor_peek
    bump = cursor_bump
    eat = cursor_eat
    eat_str = cursor_eat_str
    starts_with = cursor_starts_with
    skip_comment = cursor_skip_comment
    skip_string = cursor_skip_string
    skip_spaces_tabs = cursor_skip_spaces_tabs
    skip_whitespace = cursor_skip_whitespace
    skip_trivia = cursor_skip_trivia
    skip_roc_token = cursor_skip_roc_token
    skip_balanced_braces = cursor_skip_balanced_braces
    skip_balanced_parens = cursor_skip_balanced_parens
    scan_ident = cursor_scan_ident
    scan_interpolation = cursor_scan_interpolation
    is_top_level = cursor_is_top_level
    ident_text = cursor_ident_text
    slice = cursor_slice
    trim_span = do_trim_span
}

Cur : {
    text : Str,
    src : List(U8),
    pos : U64,
    paren : U64,
    bracket : U64,
    brace : U64,
}

Span : { start : U64, end : U64 }

InterpolationScan : {
    expr : Span,
    span : Span,
    terminated : Bool,
}

cursor_new = |text| {
    {
        text: text,
        src: Str.to_utf8(text),
        pos: 0.U64,
        paren: 0.U64,
        bracket: 0.U64,
        brace: 0.U64,
    }
}

cursor_at = |text, pos| {
    {
        text: text,
        src: Str.to_utf8(text),
        pos: pos,
        paren: 0.U64,
        bracket: 0.U64,
        brace: 0.U64,
    }
}

cursor_is_eof = |cur| cur.pos >= cur.src.len()

cursor_peek = |cur|
    match List.get(cur.src, cur.pos) {
        Ok(b) => Ok(b)
        Err(_) => Err({})
    }

cursor_bump = |cur| {
    if cursor_is_eof(cur) {
        cur
    } else {
        { ..cur, pos: cur.pos + 1 }
    }
}

cursor_eat : Cur, U8 -> { eaten : Bool, cur : Cur }
cursor_eat = |cur, byte| {
    match cursor_peek(cur) {
        Ok(b) if b == byte => { eaten: Bool.True, cur: cursor_bump(cur) }
        _ => { eaten: Bool.False, cur: cur }
    }
}

cursor_starts_with = |cur, s| {
    needle = Str.to_utf8(s)
    nlen = needle.len()
    if cur.pos + nlen > cur.src.len() {
        Bool.False
    } else {
        List.sublist(cur.src, { start: cur.pos, len: nlen }) == needle
    }
}

cursor_eat_str = |cur, s| {
    if cursor_starts_with(cur, s) {
        { ..cur, pos: cur.pos + Str.to_utf8(s).len() }
    } else {
        cur
    }
}

cursor_is_top_level = |cur| cur.paren == 0 and cur.bracket == 0 and cur.brace == 0

cursor_ident_text = |cur, span| cursor_slice(cur.text, span)

is_ident_start = |b| (b >= 65 and b <= 90) or (b >= 97 and b <= 122) or b == 95

is_ident_continue = |b| is_ident_start(b) or (b >= 48 and b <= 57)

cursor_skip_spaces_tabs = |cur| {
    var $cur = cur
    while !cursor_is_eof($cur) {
        match cursor_peek($cur) {
            Ok(32) => {
                $cur = cursor_bump($cur)
            }
            Ok(9) => {
                $cur = cursor_bump($cur)
            }
            _ => break
        }
    }
    $cur
}

cursor_skip_whitespace = |cur| {
    var $cur = cur
    while !cursor_is_eof($cur) {
        match cursor_peek($cur) {
            Ok(b) if is_ws(b) => {
                $cur = cursor_bump($cur)
            }
            _ => break
        }
    }
    $cur
}

cursor_skip_trivia = |cur| {
    var $cur = cur
    var $again = Bool.True
    while $again {
        $cur = cursor_skip_whitespace($cur)
        if cursor_peek($cur) == Ok(35) {
            $cur = cursor_skip_comment($cur)
        } else {
            $again = Bool.False
        }
    }
    $cur
}

cursor_scan_ident = |cur| {
    start = cur.pos
    match cursor_peek(cur) {
        Ok(b) if is_ident_start(b) => {
            var $cur = cursor_bump(cur)
            while !cursor_is_eof($cur) {
                match cursor_peek($cur) {
                    Ok(c) if is_ident_continue(c) => {
                        $cur = cursor_bump($cur)
                    }
                    _ => break
                }
            }
            Ok({ cur: $cur, span: { start: start, end: $cur.pos } })
        }
        _ => Err(cur)
    }
}

cursor_skip_number = |cur| {
    var $cur = cur
    if cursor_starts_with($cur, "0x") or cursor_starts_with($cur, "0X") {
        $cur = { ..$cur, pos: $cur.pos + 2 }
        while !cursor_is_eof($cur) {
            match cursor_peek($cur) {
                Ok(b) if (b >= 48 and b <= 57) or (b >= 65 and b <= 70) or (b >= 97 and b <= 102) => {
                    $cur = cursor_bump($cur)
                }
                _ => break
            }
        }
        $cur
    } else {
        while !cursor_is_eof($cur) {
            match cursor_peek($cur) {
                Ok(b) if b >= 48 and b <= 57 => {
                    $cur = cursor_bump($cur)
                }
                _ => break
            }
        }
        $cur
    }
}

sat_sub = |n| {
    if n == 0 {
        0
    } else {
        n - 1
    }
}

cursor_skip_roc_token = |cur| {
    var $cur = cursor_skip_trivia(cur)
    match cursor_peek($cur) {
        Err(_) => $cur
        Ok(34) => cursor_skip_string($cur)
        Ok(b) if is_ident_start(b) => {
            match cursor_scan_ident($cur) {
                Ok(got) => got.cur
                Err(rest) => rest
            }
        }
        Ok(b) if b >= 48 and b <= 57 => cursor_skip_number($cur)
        Ok(40) => { ..cursor_bump($cur), paren: $cur.paren + 1 }
        Ok(41) => { ..cursor_bump($cur), paren: sat_sub($cur.paren) }
        Ok(91) => { ..cursor_bump($cur), bracket: $cur.bracket + 1 }
        Ok(93) => { ..cursor_bump($cur), bracket: sat_sub($cur.bracket) }
        Ok(123) => { ..cursor_bump($cur), brace: $cur.brace + 1 }
        Ok(125) => { ..cursor_bump($cur), brace: sat_sub($cur.brace) }
        _ => cursor_bump($cur)
    }
}

cursor_skip_balanced_braces = |cur| {
    if cursor_peek(cur) != Ok(123) {
        cur
    } else {
        var $cur = cur
        var $depth = 0
        while !cursor_is_eof($cur) {
            before = $cur.pos
            match cursor_peek($cur) {
                Ok(34) => {
                    $cur = cursor_skip_string($cur)
                }
                Ok(35) => {
                    $cur = cursor_skip_comment($cur)
                }
                Ok(123) => {
                    $cur = cursor_bump($cur)
                    $depth = $depth + 1
                }
                Ok(125) => {
                    $cur = cursor_bump($cur)
                    $depth = $depth - 1
                    if $depth == 0 {
                        break
                    }
                }
                _ => {
                    $cur = cursor_bump($cur)
                }
            }
            if $cur.pos <= before {
                $cur = cursor_bump($cur)
            }
        }
        $cur
    }
}

cursor_skip_balanced_parens = |cur| {
    if cursor_peek(cur) != Ok(40) {
        cur
    } else {
        var $cur = cur
        var $depth = 0
        while !cursor_is_eof($cur) {
            before = $cur.pos
            match cursor_peek($cur) {
                Ok(34) => {
                    $cur = cursor_skip_string($cur)
                }
                Ok(35) => {
                    $cur = cursor_skip_comment($cur)
                }
                Ok(40) => {
                    $cur = cursor_bump($cur)
                    $depth = $depth + 1
                }
                Ok(41) => {
                    $cur = cursor_bump($cur)
                    $depth = $depth - 1
                    if $depth == 0 {
                        break
                    }
                }
                _ => {
                    $cur = cursor_bump($cur)
                }
            }
            if $cur.pos <= before {
                $cur = cursor_bump($cur)
            }
        }
        $cur
    }
}

is_ws = |b| b == 32 or b == 9 or b == 10 or b == 13

cursor_skip_comment = |cur| {
    got = cursor_eat(cur, 35)
    if !got.eaten {
        cur
    } else {
        var $cur = got.cur
        while !cursor_is_eof($cur) {
            match cursor_peek($cur) {
                Ok(10) => break
                _ => {
                    $cur = cursor_bump($cur)
                }
            }
        }
        $cur
    }
}

cursor_skip_balanced_braces_inner = |cur| {
    var $cur = cur
    var $depth = 1.U64
    while !cursor_is_eof($cur) and $depth > 0 {
        before = $cur.pos
        match cursor_peek($cur) {
            Ok(34) => {
                $cur = cursor_skip_string($cur)
            }
            Ok(35) => {
                $cur = cursor_skip_comment($cur)
            }
            Ok(123) => {
                $cur = cursor_bump($cur)
                $depth = $depth + 1
            }
            Ok(125) => {
                $cur = cursor_bump($cur)
                $depth = $depth - 1
            }
            _ => {
                $cur = cursor_bump($cur)
            }
        }
        if $cur.pos <= before {
            $cur = cursor_bump($cur)
        }
    }
    $cur
}

dollar_brace = "\${"

cursor_skip_string = |cur| {
    var $cur = cur
    if cursor_starts_with($cur, "\"\"\"") {
        $cur = cursor_eat_str($cur, "\"\"\"")
        while !cursor_is_eof($cur) {
            before = $cur.pos
            if cursor_starts_with($cur, "\"\"\"") {
                return cursor_eat_str($cur, "\"\"\"")
            } else if cursor_starts_with($cur, dollar_brace) {
                $cur = cursor_eat_str($cur, dollar_brace)
                $cur = cursor_skip_balanced_braces_inner($cur)
            } else if cursor_peek($cur) == Ok(92) {
                $cur = cursor_bump($cur)
                $cur = cursor_bump($cur)
            } else {
                $cur = cursor_bump($cur)
            }
            if $cur.pos <= before {
                $cur = cursor_bump($cur)
            }
        }
        $cur
    } else {
        got = cursor_eat($cur, 34)
        if !got.eaten {
            cur
        } else {
            $cur = got.cur
            while !cursor_is_eof($cur) {
                before = $cur.pos
                if cursor_peek($cur) == Ok(92) {
                    $cur = cursor_bump($cur)
                    $cur = cursor_bump($cur)
                } else if cursor_peek($cur) == Ok(34) {
                    return cursor_bump($cur)
                } else if cursor_starts_with($cur, dollar_brace) {
                    $cur = cursor_eat_str($cur, dollar_brace)
                    $cur = cursor_skip_balanced_braces_inner($cur)
                } else {
                    $cur = cursor_bump($cur)
                }
                if $cur.pos <= before {
                    $cur = cursor_bump($cur)
                }
            }
            $cur
        }
    }
}

cursor_slice = |text, span| {
    if span.end <= span.start {
        ""
    } else {
        bytes = Str.to_utf8(text)
        available = bytes.len()
        start = span.start
        if start >= available {
            ""
        } else {
            want = span.end - start
            take =
                if start + want > available {
                    available - start
                } else {
                    want
                }
            Str.from_utf8_lossy(List.sublist(bytes, { start: start, len: take }))
        }
    }
}

do_trim_span = |text, start, end| {
    bytes = Str.to_utf8(text)
    var $s = start
    var $e = end
    if $e > bytes.len() {
        $e = bytes.len()
    }
    while $s < $e {
        match List.get(bytes, $s) {
            Ok(b) if is_ws(b) => {
                $s = $s + 1
            }
            _ => break
        }
    }
    while $e > $s {
        match List.get(bytes, $e - 1) {
            Ok(b) if is_ws(b) => {
                $e = $e - 1
            }
            _ => break
        }
    }
    { start: $s, end: $e }
}

cursor_scan_interpolation = |text, open_brace| {
    var $cur = cursor_at(text, open_brace)
    start = $cur.pos
    got = cursor_eat($cur, 123)
    if !got.eaten {
        {
            expr: { start: start, end: start },
            span: { start: start, end: start },
            terminated: Bool.False,
        }
    } else {
        $cur = got.cur
        expr_start = $cur.pos
        var $depth = 1.U64
        var $done = Bool.False
        var $result = {
            expr: do_trim_span(text, expr_start, $cur.pos),
            span: { start: start, end: $cur.pos },
            terminated: Bool.False,
        }
        while !cursor_is_eof($cur) and $depth > 0 and !$done {
            before = $cur.pos
            match cursor_peek($cur) {
                Ok(34) => {
                    $cur = cursor_skip_string($cur)
                }
                Ok(35) => {
                    $cur = cursor_skip_comment($cur)
                }
                Ok(123) => {
                    $cur = cursor_bump($cur)
                    $depth = $depth + 1
                }
                Ok(125) => {
                    if $depth == 1 {
                        expr = do_trim_span(text, expr_start, $cur.pos)
                        $cur = cursor_bump($cur)
                        $result = {
                            expr: expr,
                            span: { start: start, end: $cur.pos },
                            terminated: Bool.True,
                        }
                        $done = Bool.True
                    } else {
                        $cur = cursor_bump($cur)
                        $depth = $depth - 1
                    }
                }
                _ => {
                    $cur = cursor_bump($cur)
                }
            }
            if !$done and $cur.pos <= before {
                $cur = cursor_bump($cur)
            }
        }
        if $done {
            $result
        } else {
            {
                expr: do_trim_span(text, expr_start, $cur.pos),
                span: { start: start, end: $cur.pos },
                terminated: Bool.False,
            }
        }
    }
}

expect {
    cur = Cursor.new("ab")
    bumped = Cursor.bump(cur)
    bumped.pos == 1
}

expect {
    src = "\"hello \${name} world\" and more"
    closed = "\"hello \${name} world\""
    cur = Cursor.skip_string(Cursor.new(src))
    cur.pos == Str.to_utf8(closed).len()
}

expect {
    unclosed = "\"unclosed string with \${nested"
    cur = Cursor.skip_string(Cursor.new(unclosed))
    Cursor.is_eof(cur)
}

expect {
    src = "# comment\nnext"
    cur = Cursor.skip_comment(Cursor.new(src))
    cur.pos == 9
}

expect {
    src = "{date}"
    scan = Cursor.scan_interpolation(src, 0)
    scan.terminated
    and scan.span.start == 0
    and scan.span.end == 6
    and Cursor.slice(src, scan.expr) == "date"
}

expect {
    nested = "{a + {b}}"
    scan = Cursor.scan_interpolation(nested, 0)
    scan.terminated and Cursor.slice(nested, scan.expr) == "a + {b}"
}

expect {
    src = "{ \"hello \${name}!\" }"
    scan = Cursor.scan_interpolation(src, 0)
    scan.terminated and Cursor.slice(src, scan.expr).trim() == "\"hello \${name}!\""
}

expect {
    src = "{date"
    scan = Cursor.scan_interpolation(src, 0)
    !scan.terminated
    and scan.span.end == Str.to_utf8(src).len()
    and Cursor.slice(src, scan.expr) == "date"
}

expect {
    got = Cursor.eat(Cursor.new("@component"), 64)
    ident = Cursor.scan_ident(got.cur)
    match ident {
        Ok(got_ident) => Cursor.ident_text(got_ident.cur, got_ident.span) == "component"
        Err(_) => Bool.False
    }
}

expect {
    src = "module X exposing [x]\n"
    var $cur = Cursor.new(src)
    var $n = 0.U64
    while !Cursor.is_eof($cur) and $n < 200 {
        saved = $cur.pos
        $cur = Cursor.skip_roc_token($cur)
        if $cur.pos == saved {
            $cur = Cursor.bump($cur)
        }
        $n = $n + 1
    }
    Cursor.is_eof($cur)
}
