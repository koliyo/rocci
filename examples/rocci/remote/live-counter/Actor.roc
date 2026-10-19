import Html
import CounterCore
import Origin
import LiveCounterUi
import CounterDocument

Actor := [].{
    # Private, length-bounded byte ABI. HTTP JSON is decoded in the host.
    # Strings in the state header are hex UTF-8; HTML follows a blank line.
    call : Str -> Str
    call = |input|
        match run(input) {
            Ok(output) => output
            Err(_) => "1\ninvalid\n\n"
        }
}

run = |input| {
    lines = Str.split_on(input, "\n")
    version = at(lines, 0)?
    if version != "1" { return Err(Invalid) }
    mode = at(lines, 1)?
    count = integer(at(lines, 2)?)?
    revision = integer(at(lines, 3)?)?
    now = integer(at(lines, 4)?)?
    tag = at(lines, 5)?
    origin = unhex(at(lines, 6)?)?
    tz = unhex(at(lines, 7)?)?
    clicks = List.fold(
        List.drop_first(lines, 8),
        Ok([]),
        |acc, line| {
            rows = acc?
            parts = Str.split_on(line, "|")
            if parts.len() != 4 { return Err(Invalid) }
            seq = integer(at(parts, 0)?)?
            ts = integer(at(parts, 1)?)?
            chosen = match at(parts, 2)? {
                "0" => False
                "1" => True
                _ => return Err(Invalid)
            }
            zone = unhex(at(parts, 3)?)?
            Ok(List.append(rows, { seq, ts, tz: zone, chosen }))
        },
    )?
    if count < 0 or count > CounterCore.max_integer or revision < 0 or revision > CounterCore.max_integer or clicks.len() > 50 { return Err(Invalid) }
    state = { count, revision, clicks }
    next = match tag {
        "read" => state
        "increment" => match CounterCore.decide(state, Increment({ origin, tz }), now) {
            Ok(value) => value
            Err(_) => return Ok("1\nrejected\noverflow\n\n")
        }
        "reset" => match CounterCore.decide(state, Reset, now) {
            Ok(value) => value
            Err(_) => return Ok("1\nrejected\noverflow\n\n")
        }
        _ => return Err(Invalid)
    }
    view = CounterCore.view(next, now)
    html = match mode {
        "live" => Html.render(LiveCounterUi.liveSlice(view))
        "document" => Html.render_document(CounterDocument.counterPage({ count: view.count, clicks: view.clicks, remote: True }))
        _ => return Err(Invalid)
    }
    rows = List.map(next.clicks, |click| {
        chosen = if click.chosen { "1" } else { "0" }
        "${click.seq.to_str()}|${click.ts.to_str()}|${chosen}|${hex(click.tz)}"
    })
    header = Str.join_with(List.concat(["1", "ok", next.count.to_str(), next.revision.to_str()], rows), "\n")
    Ok("${header}\n\n${html}")
}

at = |items, index| {
    value = List.get(items, index) ? |_err| Invalid
    Ok(value)
}
integer = |text| {
    value = I64.from_str(text) ? |_err| Invalid
    Ok(value)
}

hex = |text| {
    if text == "" { return "-" }
    digits = Str.to_utf8("0123456789abcdef")
    bytes = List.fold(Str.to_utf8(text), [], |out, byte| {
        hi = match List.get(digits, (byte.div_trunc_by(16)).to_u64()) { Ok(v) => v, Err(_) => 48.U8 }
        lo = match List.get(digits, (byte % 16).to_u64()) { Ok(v) => v, Err(_) => 48.U8 }
        List.append(List.append(out, hi), lo)
    })
    Str.from_utf8_lossy(bytes)
}

unhex = |text| {
    if text == "-" { return Ok("") }
    parsed = List.fold(Str.to_utf8(text), Ok({ high: None, bytes: [] }), |acc, byte| {
        state = acc?
        nibble = match byte {
            b if b >= 48 and b <= 57 => b - 48
            b if b >= 97 and b <= 102 => b - 87
            _ => return Err(Invalid)
        }
        match state.high {
            None => Ok({ ..state, high: Some(nibble) })
            Some(high) => Ok({ high: None, bytes: List.append(state.bytes, high * 16 + nibble) })
        }
    })?
    match parsed.high {
        Some(_) => Err(Invalid)
        None => {
            value = Str.from_utf8(parsed.bytes) ? |_err| Invalid
            Ok(value)
        }
    }
}
