replace = |haystack, needle, replacement|
    Str.join_with(Str.split_on(haystack, needle), replacement)

escape = |value|
    replace(
        replace(
            replace(
                replace(replace(value, "&", "&amp;"), "<", "&lt;"),
                ">",
                "&gt;",
            ),
            "\"",
            "&quot;",
        ),
        "'",
        "&#39;",
    )

join = |parts| Str.join_with(parts, "")

Html := [].{
    attribute = |name, value| " ${name}=\"${escape(value)}\""

    boolean_attribute = |name, enabled|
        if enabled {
            " ${name}"
        } else {
            ""
        }

    dangerously_include_unescaped_html = |html| html

    element = |name, attrs, children|
        "<${name}${join(attrs)}>${join(children)}</${name}>"

    empty = ""

    fragment = |nodes| join(nodes)

    render = |html| html

    render_document = |html| "<!DOCTYPE html>\n${html}"

    render_fragment = |html| html

    render_without_doc_type = |html| html

    text = |value| escape(value)

    void_element = |name, attrs| "<${name}${join(attrs)} />"
}
