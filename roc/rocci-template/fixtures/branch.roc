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

