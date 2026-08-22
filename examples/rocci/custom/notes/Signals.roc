import pf.Server

Signals := [].{
    from_request! : Server.Request => Try(Str, [RequestBodyErr(Server.Body.Err)])
    from_request! = |request| {
        from_query = datastar_query(request.target())
        if from_query != "" {
            Ok(from_query)
        } else {
            bytes = request.body().with_limit(64 * 1024).read_all!()?
            Ok(Str.from_utf8_lossy(bytes))
        }
    }

    str : Str, Str -> Str
    str = |json, key| {
        needle = "\"${key}\":"
        parts = Str.split_on(json, needle)
        match List.get(parts, 1) {
            Ok(after) => json_string_value(skip_ws(after))
            Err(_) => ""
        }
    }

    js_str : Str -> Str
    js_str = |text| "'${escape_js(text)}'"
}

datastar_query = |target|
    match target {
        Resource({ raw_query: Present(query), .. }) =>
            List.fold(
                Str.split_on(query, "&"),
                "",
                |acc, pair|
                    if acc != "" {
                        acc
                    } else {
                        parts = Str.split_on(pair, "=")
                        match (List.get(parts, 0), List.get(parts, 1)) {
                            (Ok("datastar"), Ok(value)) => percent_decode(value)
                            _ => acc
                        }
                    },
            )
        _ => ""
    }

skip_ws = |text| skip_ws_bytes(Str.to_utf8(text))

skip_ws_bytes = |bytes|
    match List.get(bytes, 0) {
        Ok(byte) if byte == 32 or byte == 9 or byte == 10 or byte == 13 =>
            skip_ws_bytes(List.drop_first(bytes, 1))
        _ => Str.from_utf8_lossy(bytes)
    }

json_string_value = |text|
    if Str.starts_with(text, "\"") {
        read_json_string(Str.to_utf8(Str.drop_prefix(text, "\"")), 0, [], False)
    } else {
        ""
    }

read_json_string = |bytes, index, acc, escape|
    match List.get(bytes, index) {
        Err(_) => Str.from_utf8_lossy(acc)
        Ok(34) if !escape => Str.from_utf8_lossy(acc)
        Ok(92) if !escape => read_json_string(bytes, index + 1, acc, True)
        Ok(byte) if escape =>
            read_json_string(
                bytes,
                index + 1,
                List.append(
                    acc,
                    match byte {
                        34 => 34
                        92 => 92
                        47 => 47
                        110 => 10
                        116 => 9
                        114 => 13
                        _ => byte
                    },
                ),
                False,
            )
        Ok(byte) => read_json_string(bytes, index + 1, List.append(acc, byte), False)
    }

escape_js = |text|
    List.fold(
        Str.to_utf8(text),
        "",
        |acc, byte|
            if byte == 39 or byte == 92 {
                "${acc}\\${Str.from_utf8_lossy([byte])}"
            } else if byte == 10 {
                "${acc}\\n"
            } else if byte == 13 {
                "${acc}\\r"
            } else {
                "${acc}${Str.from_utf8_lossy([byte])}"
            },
    )

percent_decode = |input| Str.from_utf8_lossy(percent_decode_bytes(Str.to_utf8(input), 0, []))

percent_decode_bytes = |bytes, index, acc|
    match List.get(bytes, index) {
        Err(_) => acc
        Ok(43) => percent_decode_bytes(bytes, index + 1, List.append(acc, 32))
        Ok(37) =>
            match (List.get(bytes, index + 1), List.get(bytes, index + 2)) {
                (Ok(hi), Ok(lo)) =>
                    percent_decode_bytes(
                        bytes,
                        index + 3,
                        List.append(acc, hex_nibble(hi) * 16 + hex_nibble(lo)),
                    )
                _ => percent_decode_bytes(bytes, index + 1, List.append(acc, 37))
            }
        Ok(byte) => percent_decode_bytes(bytes, index + 1, List.append(acc, byte))
    }

hex_nibble = |byte|
    if byte >= 48 and byte <= 57 {
        byte - 48
    } else if byte >= 65 and byte <= 70 {
        byte - 55
    } else if byte >= 97 and byte <= 102 {
        byte - 87
    } else {
        0
    }
