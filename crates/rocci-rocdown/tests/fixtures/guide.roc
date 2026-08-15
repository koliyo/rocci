import Html

published = "2026-08-15"

feature_count = 3.I64

featureCount = |{ count }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"Guide-6f3d6b54\"]) {\n* { box-sizing: border-box; }\n    body {\n        color-scheme: dark;\n        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif;\n        color: #f3f7f5;\n        font-synthesis: none;\n        margin: 0;\n        min-height: 100vh;\n        background:\n            radial-gradient(circle at 78% 8%, rgba(55, 242, 159, 0.13), transparent 31rem),\n            linear-gradient(145deg, #0a1713 0%, #07110e 55%, #071512 100%);\n    }\n    main {\n        width: min(40rem, calc(100% - 2rem));\n        margin: 0 auto;\n        padding: 2.5rem 0 4rem;\n    }\n    h1 {\n        margin: 0 0 0.75rem;\n        font-size: clamp(2.4rem, 6vw, 3.4rem);\n        letter-spacing: -0.04em;\n        line-height: 1.1;\n    }\n    h2 {\n        margin: 2rem 0 0.6rem;\n        font-size: 1.35rem;\n        letter-spacing: -0.02em;\n    }\n    p {\n        margin: 0 0 1rem;\n        color: #c9d8d2;\n        font-size: 1.05rem;\n        line-height: 1.65;\n    }\n    a {\n        color: #48eda4;\n        text-decoration-color: rgba(72, 237, 164, 0.4);\n        text-underline-offset: 0.18em;\n    }\n    a:hover { color: #79f4bd; }\n    pre {\n        margin: 0 0 1.25rem;\n        padding: 1rem 1.1rem;\n        overflow-x: auto;\n        border: 1px solid rgba(109, 236, 177, 0.2);\n        border-radius: 12px;\n        background: rgba(11, 30, 24, 0.72);\n    }\n    code {\n        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;\n        font-size: 0.92rem;\n        color: #f8fffb;\n    }\n}\n@scope ([data-rocci-css~=\"featureCount-b5481b01\"]) {\n.feature-count {\n            display: inline-flex;\n            margin: 0 0 1.5rem;\n            padding: 0.35rem 0.75rem;\n            border: 1px solid rgba(109, 236, 177, 0.28);\n            border-radius: 999px;\n            background: rgba(72, 237, 164, 0.1);\n            color: #48eda4;\n            font-size: 0.8rem;\n            font-weight: 700;\n            letter-spacing: 0.04em;\n        }\n}"),
                ],
            ),
            Html.element(
                "p",
                [
                    Html.attribute("class", "feature-count"),
                    Html.attribute("data-rocci-css", "Guide-6f3d6b54 featureCount-b5481b01"),
                ],
                [
                    Html.text(count.to_str()),
                    Html.text(" core ideas"),
                ],
            ),
        ],
    )
}

rocci_meta = {
        title: "Rocdown",
        description: "Markdown content with explicit Roc and Rocci islands",
    }

rocci_content = |{}| {
    Html.fragment([
        Html.element(
            "h1",
            [
                Html.attribute("id", "rocdown"),
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("Rocdown"),
            ],
        ),
        Html.element(
            "p",
            [
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("Rocdown is a content-first format. Email us at "),
                Html.element(
                    "a",
                    [
                        Html.attribute("href", "mailto:docs@example.com"),
                        Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                    ],
                    [
                        Html.text("docs@example.com"),
                    ],
                ),
                Html.text(" or mention"),
                Html.text("\n"),
                Html.text("@roclang normally."),
            ],
        ),
        featureCount({ count: feature_count }),
        Html.element(
            "h2",
            [
                Html.attribute("id", "displayed-code"),
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("Displayed code"),
            ],
        ),
        Html.element(
            "p",
            [
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("This fence is documentation and is never evaluated:"),
            ],
        ),
        Html.element(
            "pre",
            [
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.element(
                    "code",
                    [
                        Html.attribute("class", "language-roc"),
                        Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                    ],
                    [
                        Html.text("answer = 42\n"),
                    ],
                ),
            ],
        ),
        Html.element(
            "p",
            [
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("The page remains static unless it references an island or defines server"),
                Html.text("\n"),
                Html.text("routes."),
            ],
        ),
    ])
}

rocci_page = |{}| {
    Html.element(
        "html",
        [
            Html.attribute("lang", "en"),
            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
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
                            Html.text("Rocdown"),
                        ],
                    ),
                    Html.element(
                        "style",
                        [],
                        [
                            Html.text("@scope ([data-rocci-css~=\"Guide-6f3d6b54\"]) {\n* { box-sizing: border-box; }\n    body {\n        color-scheme: dark;\n        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif;\n        color: #f3f7f5;\n        font-synthesis: none;\n        margin: 0;\n        min-height: 100vh;\n        background:\n            radial-gradient(circle at 78% 8%, rgba(55, 242, 159, 0.13), transparent 31rem),\n            linear-gradient(145deg, #0a1713 0%, #07110e 55%, #071512 100%);\n    }\n    main {\n        width: min(40rem, calc(100% - 2rem));\n        margin: 0 auto;\n        padding: 2.5rem 0 4rem;\n    }\n    h1 {\n        margin: 0 0 0.75rem;\n        font-size: clamp(2.4rem, 6vw, 3.4rem);\n        letter-spacing: -0.04em;\n        line-height: 1.1;\n    }\n    h2 {\n        margin: 2rem 0 0.6rem;\n        font-size: 1.35rem;\n        letter-spacing: -0.02em;\n    }\n    p {\n        margin: 0 0 1rem;\n        color: #c9d8d2;\n        font-size: 1.05rem;\n        line-height: 1.65;\n    }\n    a {\n        color: #48eda4;\n        text-decoration-color: rgba(72, 237, 164, 0.4);\n        text-underline-offset: 0.18em;\n    }\n    a:hover { color: #79f4bd; }\n    pre {\n        margin: 0 0 1.25rem;\n        padding: 1rem 1.1rem;\n        overflow-x: auto;\n        border: 1px solid rgba(109, 236, 177, 0.2);\n        border-radius: 12px;\n        background: rgba(11, 30, 24, 0.72);\n    }\n    code {\n        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;\n        font-size: 0.92rem;\n        color: #f8fffb;\n    }\n}"),
                        ],
                    ),
                ],
            ),
            Html.element(
                "body",
                [
                    Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                ],
                [
                    Html.element(
                        "main",
                        [
                            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
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

on_get_guides_rocdown! = |_state| {
    rocci_value = {
        rocci_page({})
    }
    Ok(rocci_value)
}
