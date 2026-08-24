import Html
import OkfTheme

skip_ws = |bytes, i|
    match List.get(bytes, i) {
        Ok(byte) if byte == 32 or byte == 9 or byte == 10 or byte == 13 => skip_ws(bytes, i + 1)
        _ => i
    }

take_len = |list, n, acc|
    if n == 0 {
        acc
    } else {
        match List.get(list, 0) {
            Err(_) => acc
            Ok(item) => take_len(List.drop_first(list, 1), n - 1, List.append(acc, item))
        }
    }

slice_str = |bytes, start, end| {
    len = if end <= start {
        0
    } else {
        end - start
    }
    Str.from_utf8_lossy(take_len(List.drop_first(bytes, start), len, []))
}

matches_at = |hay, needle, i, k|
    match List.get(needle, k) {
        Err(_) => True
        Ok(nb) =>
            match List.get(hay, i + k) {
                Ok(hb) if hb == nb => matches_at(hay, needle, i, k + 1)
                _ => False
            }
    }

find_sub = |hay, needle, i| {
    nlen = List.len(needle)
    hlen = List.len(hay)
    if nlen == 0 {
        Ok(i)
    } else if i + nlen > hlen {
        Err("not found")
    } else if matches_at(hay, needle, i, 0) {
        Ok(i + nlen)
    } else {
        find_sub(hay, needle, i + 1)
    }
}

scan_span = |bytes, i, open, close| {
    walk = |j, depth, in_str, escape|
        match List.get(bytes, j) {
            Err(_) => j
            Ok(92) if in_str and !escape => walk(j + 1, depth, True, True)
            Ok(34) if !escape => walk(j + 1, depth, !in_str, False)
            Ok(_) if in_str => walk(j + 1, depth, True, False)
            Ok(b) if b == open => walk(j + 1, depth + 1, False, False)
            Ok(b) if b == close =>
                if depth == 1 {
                    j + 1
                } else if depth == 0 {
                    j + 1
                } else {
                    walk(j + 1, depth - 1, False, False)
                }
            Ok(_) => walk(j + 1, depth, False, False)
        }
    walk(i, 0, False, False)
}

after_key = |bytes, key| {
    needle = Str.to_utf8("\"${key}\":")
    match find_sub(bytes, needle, 0) {
        Ok(colon_end) => Ok(skip_ws(bytes, colon_end))
        Err(e) => Err(e)
    }
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

string_end = |bytes, index, escape|
    match List.get(bytes, index) {
        Err(_) => index
        Ok(34) if !escape => index + 1
        Ok(92) if !escape => string_end(bytes, index + 1, True)
        Ok(_) => string_end(bytes, index + 1, False)
    }

json_str = |obj, key| {
    bytes = Str.to_utf8(obj)
    match after_key(bytes, key) {
        Ok(i) =>
            match List.get(bytes, i) {
                Ok(34) => read_json_string(bytes, i + 1, [], False)
                _ => ""
            }
        Err(_) => ""
    }
}

json_bool = |obj, key| {
    bytes = Str.to_utf8(obj)
    match after_key(bytes, key) {
        Ok(i) =>
            match List.get(bytes, i) {
                Ok(116) => True
                _ => False
            }
        Err(_) => False
    }
}

json_object = |obj, key| {
    bytes = Str.to_utf8(obj)
    match after_key(bytes, key) {
        Ok(i) =>
            match List.get(bytes, i) {
                Ok(123) => slice_str(bytes, i, scan_span(bytes, i, 123, 125))
                _ => "{}"
            }
        Err(_) => "{}"
    }
}

collect_objects = |bytes, i, acc| {
    j = skip_ws(bytes, i)
    match List.get(bytes, j) {
        Ok(93) => acc
        Ok(44) => collect_objects(bytes, j + 1, acc)
        Ok(123) => {
            end = scan_span(bytes, j, 123, 125)
            collect_objects(bytes, end, List.append(acc, slice_str(bytes, j, end)))
        }
        _ => acc
    }
}

json_objects = |obj, key| {
    bytes = Str.to_utf8(obj)
    match after_key(bytes, key) {
        Ok(i) =>
            match List.get(bytes, i) {
                Ok(91) => collect_objects(bytes, i + 1, [])
                _ => []
            }
        Err(_) => []
    }
}

collect_strings = |bytes, i, acc| {
    j = skip_ws(bytes, i)
    match List.get(bytes, j) {
        Ok(93) => acc
        Ok(44) => collect_strings(bytes, j + 1, acc)
        Ok(34) => {
            value = read_json_string(bytes, j + 1, [], False)
            collect_strings(bytes, string_end(bytes, j + 1, False), List.append(acc, value))
        }
        _ => acc
    }
}

json_strings = |obj, key| {
    bytes = Str.to_utf8(obj)
    match after_key(bytes, key) {
        Ok(i) =>
            match List.get(bytes, i) {
                Ok(91) => collect_strings(bytes, i + 1, [])
                _ => []
            }
        Err(_) => []
    }
}

empty_sources = List.drop_first([{ id: "", resource: "", href: "", author: "", is_drifted: False }], 1)
empty_other = List.drop_first([{ key: "", val: "" }], 1)
empty_tags = List.drop_first([""], 1)
empty_outline = List.drop_first([{ id: "", title: "", level: "" }], 1)
empty_objects = List.drop_first([""], 1)

empty_meta = {
    concept_type: "",
    status: "",
    authority: "",
    trust_slug: "",
    trust_label: "",
    stale: False,
    stale_after: "",
    is_action_required: False,
    action_detail: "",
    description: "",
    has_provenance: False,
    owners: "",
    verifier: "",
    generated: "",
    has_sources: False,
    source_count: "0",
    drift_summary: "",
    sources: empty_sources,
    has_other_meta: False,
    other_meta_count: "0",
    other_meta: empty_other,
    has_tags: False,
    tags: empty_tags,
}

dummy_page = {
    output_path: "",
    article_path: "",
    article_html: "",
    nav_html: "",
    title: "",
    view: {
        has_outline: False,
        outline: empty_outline,
        has_meta: False,
        meta: empty_meta,
    },
}

empty_pages = List.drop_first([dummy_page], 1)

parse_source = |obj| {
    {
        id: json_str(obj, "id"),
        resource: json_str(obj, "resource"),
        href: json_str(obj, "href"),
        author: json_str(obj, "author"),
        is_drifted: json_bool(obj, "is_drifted"),
    }
}

parse_other = |obj| {
    {
        key: json_str(obj, "key"),
        val: json_str(obj, "val"),
    }
}

parse_outline = |obj| {
    {
        id: json_str(obj, "id"),
        title: json_str(obj, "title"),
        level: json_str(obj, "level"),
    }
}

parse_meta = |obj| {
    {
        concept_type: json_str(obj, "concept_type"),
        status: json_str(obj, "status"),
        authority: json_str(obj, "authority"),
        trust_slug: json_str(obj, "trust_slug"),
        trust_label: json_str(obj, "trust_label"),
        stale: json_bool(obj, "stale"),
        stale_after: json_str(obj, "stale_after"),
        is_action_required: json_bool(obj, "is_action_required"),
        action_detail: json_str(obj, "action_detail"),
        description: json_str(obj, "description"),
        has_provenance: json_bool(obj, "has_provenance"),
        owners: json_str(obj, "owners"),
        verifier: json_str(obj, "verifier"),
        generated: json_str(obj, "generated"),
        has_sources: json_bool(obj, "has_sources"),
        source_count: json_str(obj, "source_count"),
        drift_summary: json_str(obj, "drift_summary"),
        sources: List.map(json_objects(obj, "sources"), parse_source),
        has_other_meta: json_bool(obj, "has_other_meta"),
        other_meta_count: json_str(obj, "other_meta_count"),
        other_meta: List.map(json_objects(obj, "other_meta"), parse_other),
        has_tags: json_bool(obj, "has_tags"),
        tags: json_strings(obj, "tags"),
    }
}

parse_page = |obj| {
    meta_obj = json_object(obj, "meta")
    {
        output_path: json_str(obj, "output_path"),
        article_path: json_str(obj, "article_path"),
        article_html: "",
        nav_html: json_str(obj, "nav_html"),
        title: json_str(obj, "title"),
        view: {
            has_outline: json_bool(obj, "has_outline"),
            outline: List.map(json_objects(obj, "outline"), parse_outline),
            has_meta: json_bool(obj, "has_meta"),
            meta: if meta_obj == "{}" {
                empty_meta
            } else {
                parse_meta(meta_obj)
            },
        },
    }
}

parse_pages = |json| {
    bytes = Str.to_utf8(json)
    match after_key(bytes, "pages") {
        Ok(i) =>
            match List.get(bytes, i) {
                Ok(91) => List.map(collect_objects(bytes, i + 1, empty_objects), parse_page)
                _ => empty_pages
            }
        Err(_) => empty_pages
    }
}

with_article = |record, html| {
    {
        output_path: record.output_path,
        article_path: record.article_path,
        article_html: html,
        nav_html: record.nav_html,
        title: record.title,
        view: record.view,
    }
}

render_page = |item| {
    nav_node = Html.dangerously_include_unescaped_html(item.nav_html)
    article_node = Html.dangerously_include_unescaped_html(item.article_html)
    page_html = Html.render(
        OkfTheme.knowledgeShell(
            item.view,
            nav_node,
            article_node,
        ),
    )
    "<!doctype html><html lang=\"en\" class=\"rd-document\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"color-scheme\" content=\"dark\"><title>${item.title}</title><link rel=\"stylesheet\" href=\"/__rocci_okf/app.css\"><script src=\"/__rocci_okf/session.js\" defer></script><script src=\"/__rocci_okf/goto.js\" defer></script><script src=\"/__rocci_okf/reload.js\" defer></script></head><body>${page_html}</body></html>\n"
}

OkfBuild := [].{
    parse_pages = parse_pages
    with_article = with_article
    render_page = render_page
    render_all = |pages| List.map(pages, render_page)
}
