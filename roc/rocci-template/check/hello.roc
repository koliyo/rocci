Html := [].{
    element = |name, _attrs, _children| name
    text = |s| s
    empty = ""
    fragment = |_nodes| ""
    attribute = |name, _value| name
    void_element = |name, _attrs| name
}

hello = |{ name }| {
    Html.element(
        "p",
        [],
        [
            Html.text("Hello, "),
            Html.text(name),
            Html.text("!"),
        ],
    )
}
