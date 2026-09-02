

card = |{}| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"css-e7b6899e\"]) {\nbody { margin: 0; }\n}\n@scope ([data-rocci-css~=\"card-98509670\"]) {\n.card { color: red; }\n}"),
                ],
            ),
            Html.element(
                "div",
                [
                    Html.attribute("class", "card"),
                    Html.attribute("data-rocci-css", "css-e7b6899e card-98509670"),
                ],
                [
                    Html.text("x"),
                ],
            ),
        ],
    )
}

