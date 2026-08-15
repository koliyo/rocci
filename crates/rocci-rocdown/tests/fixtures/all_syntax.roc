import Html

Status : [Ready, Loading]

items = [{ name: "alpha" }, { name: "beta" }]

status = Ready

show_notice = Bool.true

hello = |{ name }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-13744130\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            Html.element(
                "p",
                [
                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                ],
                [
                    Html.text("Hello, "),
                    Html.text(name),
                ],
            ),
        ],
    )
}
helloSample = { name: "Roc" }

rocci_meta = {
        title: "All syntax",
        description: "Rocdown kitchen sink",
    }

rocci_content = |{}| {
    visible = List.keepIf(items, |_| Bool.true)

    Html.fragment(
        List.concat(
            [
                Html.element(
                    "h1",
                    [
                        Html.attribute("id", "all-syntax"),
                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                    ],
                    [
                        Html.text("All syntax"),
                    ],
                ),
                Html.element(
                    "p",
                    [
                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                    ],
                    [
                        Html.text("Email "),
                        Html.element(
                            "a",
                            [
                                Html.attribute("href", "mailto:docs@example.com"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("docs@example.com"),
                            ],
                        ),
                        Html.text(" or mention @roclang."),
                    ],
                ),
                Html.element(
                    "p",
                    [
                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                    ],
                    [
                        Html.text("This is "),
                        Html.element(
                            "strong",
                            [
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("bold"),
                            ],
                        ),
                        Html.text(" and "),
                        Html.element(
                            "em",
                            [
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("italic"),
                            ],
                        ),
                        Html.text("."),
                    ],
                ),
                Html.element(
                    "ul",
                    [
                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                    ],
                    [
                        Html.element(
                            "li",
                            [
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("list item"),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
                Html.element(
                    "pre",
                    [
                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                    ],
                    [
                        Html.element(
                            "code",
                            [
                                Html.attribute("class", "language-roc"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("answer = 42\n"),
                            ],
                        ),
                    ],
                ),
                Html.element(
                    "table",
                    [
                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                    ],
                    [
                        Html.element(
                            "thead",
                            [
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "tr",
                                    [
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "th",
                                            [
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("col"),
                                            ],
                                        ),
                                        Html.element(
                                            "th",
                                            [
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("val"),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "tbody",
                            [
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "tr",
                                    [
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "td",
                                            [
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("a"),
                                            ],
                                        ),
                                        Html.element(
                                            "td",
                                            [
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("b"),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
                hello({ name: "render" }),
                if show_notice {
                    Html.element(
                        "p",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text("Notice"),
                        ],
                    )
                } else if List.isEmpty(visible) {
                    Html.element(
                        "p",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text("Empty"),
                        ],
                    )
                } else {
                    Html.element(
                        "ul",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        List.map(visible, |item| {
                            Html.element(
                                "li",
                                [
                                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                ],
                                [
                                    Html.text(item.name),
                                ],
                            )
                        }),
                    )
                },
            ],
            List.concat(
                List.map(visible, |item| {
                    Html.element(
                        "li",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text(item.name),
                        ],
                    )
                }),
                [
                    match status {
                        Loading => Html.element(
                            "p",
                            [
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("Loading"),
                            ],
                        )
                        Ready => hello(
                            { name: "ready" },
                        )
                    },
                    Html.element(
                        "p",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text("@if this is escaped"),
                        ],
                    ),
                    Html.element(
                        "p",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text("More markdown after."),
                        ],
                    ),
                ],
            ),
        )
    )
}

rocci_page = |{}| {
    Html.element(
        "html",
        [
            Html.attribute("lang", "en"),
            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
        ],
        [
            Html.element(
                "head",
                [],
                [
                    Html.void_element(
                        "meta",
                        [
                            Html.attribute("charset", "utf-8"),
                        ],
                    ),
                    Html.void_element(
                        "meta",
                        [
                            Html.attribute("name", "viewport"),
                            Html.attribute("content", "width=device-width, initial-scale=1"),
                        ],
                    ),
                    Html.element(
                        "title",
                        [],
                        [
                            Html.text("All syntax"),
                        ],
                    ),
                    Html.element(
                        "style",
                        [],
                        [
                            Html.text("@scope ([data-rocci-css~=\"AllSyntax-13744130\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                        ],
                    ),
                ],
            ),
            Html.element(
                "body",
                [
                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                ],
                [
                    Html.element(
                        "main",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            rocci_content({}),
                        ],
                    ),
                ],
            ),
        ],
    )
}

on_get_all_syntax! = |_state| {
    rocci_value = {
        rocci_page({})
    }
    Ok(rocci_value)
}
