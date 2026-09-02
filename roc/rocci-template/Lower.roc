import Cursor
import Parse

Lower := [].{
    lower = do_compile
    compile = do_compile
    file_scope_id = scope_id_for
}

do_compile = |src, file_name| {
    out = Parse.parse(src)
    var $roc = ""
    imported_ds = str_contains(src, "import Datastar")
    needs_ds =
        str_contains(src, "@get(")
        or str_contains(src, "@post(")
        or str_contains(src, "@put(")
        or str_contains(src, "@patch(")
        or str_contains(src, "@delete(")
    if needs_ds and !imported_ds {
        $roc = "import Datastar\n\n"
    }
    items = out.document.items
    var $i = 0.U64
    while $i < List.len(items) {
        match List.get(items, $i) {
            Ok(item) => {
                $roc = Str.concat($roc, lower_module_item(src, file_name, item))
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    if !str_ends_with($roc, "\n") and $roc != "" {
        $roc = Str.concat($roc, "\n")
    }
    $roc
}

lower_module_item = |src, file_name, item|
    match item {
        RocRegion({ span }) => Cursor.slice(src, span)
        Component(decl) => lower_component(src, file_name, decl)
        Fixture(decl) => {
            name = pascal_to_camel(decl.name.name)
            value = Str.trim(Cursor.slice(src, decl.value))
            Str.concat(Str.concat(name, " = "), Str.concat(value, "\n"))
        }
        Test(_) => ""
        Css(_) => ""
        _ => ""
    }

lower_component = |src, _file_name, decl| {
    roc_name = pascal_to_camel(decl.name.name)
    pattern = param_pattern(Cursor.slice(src, decl.params))
    body_src = Cursor.slice(src, decl.body.span)
    _ = body_src
    inner = "    Html.empty"
    Str.concat(
        Str.concat(roc_name, " = "),
        Str.concat(pattern, Str.concat(" {\n", Str.concat(inner, "\n}\n"))),
    )
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
        names = list_idents(inner_t)
        if List.len(names) == 0 {
            if inner_t == "_" or inner_t == "" {
                t
            } else {
                t
            }
        } else {
            Str.concat("|", Str.concat(join_comma(names), "|"))
        }
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

is_ident_start = |b| (b >= 65 and b <= 90) or (b >= 97 and b <= 122) or b == 95

is_ident_continue = |b| is_ident_start(b) or (b >= 48 and b <= 57)

sat_sub = |n| {
    if n == 0 {
        0.U64
    } else {
        n - 1
    }
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

scope_id_for = |file_name| {
    key = file_basename(file_name)
    stem = file_stem(key)
    hash = fnv1a32(Str.to_utf8(key))
    Str.concat(stem, Str.concat("-", hex8(hash)))
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
    idx = n
    Cursor.slice(digits, { start: idx, end: idx + 1 })
}

expect scope_id_for("hello.rocci") == "hello-e162554d"
