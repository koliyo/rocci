# Host smoke for generated Hello. `Html.render` is not in bound here;
# it needs the web platform. `roc check` proves the component is ordinary Roc.

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

sample = hello({ name: "Ada" })
