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
    scan_interpolation = cursor_scan_interpolation
    slice = cursor_slice
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
        pos: 0,
        paren: 0,
        bracket: 0,
        brace: 0,
    }
}

cursor_at = |text, pos| {
    {
        text: text,
        src: Str.to_utf8(text),
        pos: pos,
        paren: 0,
        bracket: 0,
        brace: 0,
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
    var $depth = 1
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

trim_span = |text, start, end| {
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
        var $depth = 1
        var $done = Bool.False
        var $result = {
            expr: trim_span(text, expr_start, $cur.pos),
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
                        expr = trim_span(text, expr_start, $cur.pos)
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
                expr: trim_span(text, expr_start, $cur.pos),
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
