Query := [].{
    parse = |query|
        List.fold(
            Str.split_on(query, "&"),
            [],
            |acc, pair|
                match split_pair(pair) {
                    Ok(arg) => List.append(acc, arg)
                    Err(_) => acc
                },
        )

    find = |args, name|
        List.fold(
            args,
            Err(NotFound),
            |acc, pair|
                match acc {
                    Ok(found) => Ok(found)
                    Err(err) =>
                        if pair.name == name {
                            Ok(pair.value)
                        } else {
                            Err(err)
                        }
                },
        )

    arg_str = |args, name| {
        match find(args, name) {
            Ok(value) =>
                if value == "" {
                    Err(NotFound)
                } else {
                    Ok(value)
                }
            Err(err) => Err(err)
        }
    }

    arg_i64 = |args, name| {
        match arg_str(args, name) {
            Ok(text) => I64.from_str(text)
            Err(_) => Err(NotFound)
        }
    }

    arg_u64 = |args, name| {
        match arg_str(args, name) {
            Ok(text) => U64.from_str(text)
            Err(_) => Err(NotFound)
        }
    }

    arg_f64 = |args, name| {
        match arg_str(args, name) {
            Ok(text) => F64.from_str(text)
            Err(_) => Err(NotFound)
        }
    }

    arg_dec = |args, name| {
        match arg_str(args, name) {
            Ok(text) => Dec.from_str(text)
            Err(_) => Err(NotFound)
        }
    }

    arg_bool = |args, name| {
        match find(args, name) {
            Ok(value) => Ok(value == "true" or value == "on" or value == "1")
            Err(err) => Err(err)
        }
    }

    encode = |text|
        List.fold(
            Str.to_utf8(text),
            "",
            |acc, byte|
                if is_unreserved(byte) {
                    "${acc}${Str.from_utf8_lossy([byte])}"
                } else if byte == 32 {
                    "${acc}+"
                } else if byte == 38 {
                    "${acc}%26"
                } else if byte == 61 {
                    "${acc}%3D"
                } else if byte == 37 {
                    "${acc}%25"
                } else if byte == 34 {
                    "${acc}%22"
                } else if byte == 35 {
                    "${acc}%23"
                } else {
                    "${acc}${Str.from_utf8_lossy([byte])}"
                },
        )
}

split_pair = |pair|
    if pair == "" {
        Err(Empty)
    } else {
        parts = Str.split_on(pair, "=")
        match (List.get(parts, 0), List.get(parts, 1)) {
            (Ok(name), Ok(value)) => {
                rest = List.drop_first(parts, 2)
                joined = List.fold(rest, value, |acc, part| "${acc}=${part}")
                Ok({
                    name: percent_decode(name),
                    value: percent_decode(joined),
                })
            }
            (Ok(name), Err(_)) => Ok({ name: percent_decode(name), value: "" })
            _ => Err(Empty)
        }
    }

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

is_unreserved = |byte|
    (byte >= 48 and byte <= 57)
    or (byte >= 65 and byte <= 90)
    or (byte >= 97 and byte <= 122)
    or byte == 45
    or byte == 46
    or byte == 95
    or byte == 126
