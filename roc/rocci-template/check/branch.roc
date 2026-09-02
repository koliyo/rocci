Html := [].{
    element = |name, _attrs, _children| name
    text = |s| s
    empty = ""
    fragment = |_nodes| ""
    attribute = |name, _value| name
    void_element = |name, _attrs| name
}

branch = |{ ready }| {
    if ready {
        Html.element(
            "p",
            [],
            [
                Html.text("ok"),
            ],
        )
    } else {
        Html.element(
            "p",
            [],
            [
                Html.text("no"),
            ],
        )
    }
}
