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
                    Html.text("@scope ([data-rocci-css~=\"Guide-6f3d6b54\"]) {\n* { box-sizing: border-box; }\n    body {\n        background:\n            radial-gradient(circle at 78% 8%, rgba(55, 242, 159, 0.13), transparent 31rem),\n            linear-gradient(145deg, var(--rd-color-bg) 0%, var(--rd-color-bg) 55%, var(--rd-color-surface) 100%);\n    }\n}\n@scope ([data-rocci-css~=\"featureCount-b5481b01\"]) {\n.feature-count {\n            display: inline-flex;\n            margin: 0 0 1.5rem;\n            padding: 0.35rem 0.75rem;\n            border: 1px solid color-mix(in srgb, var(--rd-color-accent) 28%, transparent);\n            border-radius: 999px;\n            background: color-mix(in srgb, var(--rd-color-accent) 10%, transparent);\n            color: var(--rd-color-accent);\n            font-size: 0.8rem;\n            font-weight: 700;\n            letter-spacing: 0.04em;\n        }\n}"),
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
                Html.attribute("class", "rd-header-1"),
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
                Html.attribute("class", "rd-paragraph"),
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("Rocdown is a content-first format. Email us at "),
                Html.element(
                    "a",
                    [
                        Html.attribute("class", "rd-link"),
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
        featureCount(
            { count: feature_count },
        ),
        Html.element(
            "p",
            [
                Html.attribute("class", "rd-paragraph"),
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("See also "),
                Html.element(
                    "a",
                    [
                        Html.attribute("class", "rd-link"),
                        Html.attribute("href", "/guides/rocdown-interactive/"),
                        Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                    ],
                    [
                        Html.text("Interactive"),
                    ],
                ),
                Html.text(" and the "),
                Html.element(
                    "a",
                    [
                        Html.attribute("class", "rd-link"),
                        Html.attribute("href", "/guides/rocdown-interactive/"),
                        Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                    ],
                    [
                        Html.text("interactive guide"),
                    ],
                ),
                Html.text("."),
            ],
        ),
        Html.element(
            "h2",
            [
                Html.attribute("class", "rd-header-2"),
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
                Html.attribute("class", "rd-paragraph"),
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.text("This fence is documentation and is never evaluated:"),
            ],
        ),
        Html.element(
            "pre",
            [
                Html.attribute("class", "rd-code-block"),
                Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
            ],
            [
                Html.element(
                    "code",
                    [
                        Html.attribute("class", "rd-code language-roc"),
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
                Html.attribute("class", "rd-paragraph"),
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
            Html.attribute("class", "rd-document"),
            Html.attribute("data-rd-theme", "rocci"),
            Html.attribute("data-rd-color-scheme", "light"),
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
                    Html.void_element(
                        "meta",
                        [
                            Html.attribute("name", "color-scheme"),
                            Html.attribute("content", "light"),
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
                            Html.text(".rd-document {\n  color-scheme: light dark;\n  --rd-font-body: Inter, ui-sans-serif, system-ui, sans-serif;\n  --rd-font-heading: Inter, ui-sans-serif, system-ui, sans-serif;\n  --rd-font-code: ui-monospace, SFMono-Regular, Menlo, monospace;\n\n  --rd-color-bg: light-dark(#f3f7f5, #07110e);\n  --rd-color-surface: light-dark(#ffffff, #0b1e18);\n  --rd-color-text: light-dark(#07110e, #f3f7f5);\n  --rd-color-muted: light-dark(#4a635a, #c9d8d2);\n  --rd-color-accent: light-dark(#0d9a5c, #48eda4);\n  --rd-color-border: light-dark(#b7d4c8, #29493b);\n  --rd-color-code-bg: light-dark(#e7f2ec, #0b1e18);\n  --rd-color-code-text: light-dark(#07110e, #f8fffb);\n\n  --rd-header-1-color: var(--rd-color-text);\n  --rd-header-2-color: var(--rd-color-text);\n  --rd-header-3-color: var(--rd-color-text);\n  --rd-header-4-color: var(--rd-color-text);\n  --rd-header-5-color: var(--rd-color-text);\n  --rd-header-6-color: var(--rd-color-text);\n  --rd-header-1-font: var(--rd-font-heading);\n  --rd-header-2-font: var(--rd-font-heading);\n  --rd-header-3-font: var(--rd-font-heading);\n  --rd-header-4-font: var(--rd-font-heading);\n  --rd-header-5-font: var(--rd-font-heading);\n  --rd-header-6-font: var(--rd-font-heading);\n  --rd-paragraph-color: var(--rd-color-muted);\n  --rd-blockquote-color: var(--rd-color-muted);\n  --rd-blockquote-border: var(--rd-color-border);\n  --rd-list-color: var(--rd-color-muted);\n  --rd-link-color: var(--rd-color-accent);\n  --rd-link-hover-color: var(--rd-color-text);\n  --rd-code-color: var(--rd-color-code-text);\n  --rd-code-background: var(--rd-color-code-bg);\n  --rd-code-block-background: var(--rd-color-code-bg);\n  --rd-code-block-border: var(--rd-color-border);\n  --rd-table-border: var(--rd-color-border);\n  --rd-table-header-background: var(--rd-color-surface);\n  --rd-table-header-color: var(--rd-color-text);\n  --rd-thematic-break-color: var(--rd-color-border);\n  --rd-strong-color: var(--rd-color-text);\n  --rd-emphasis-color: var(--rd-color-muted);\n  --rd-strikethrough-color: var(--rd-color-muted);\n}\n.rd-document:not([data-rd-color-scheme]) {\n  color-scheme: light dark;\n}\n.rd-document[data-rd-color-scheme=\"light\"] {\n  color-scheme: light;\n}\n.rd-document[data-rd-color-scheme=\"dark\"] {\n  color-scheme: dark;\n}\n\n.rd-document body {\n  margin: 0;\n  min-height: 100vh;\n  background: var(--rd-color-bg);\n  color: var(--rd-color-text);\n  font-family: var(--rd-font-body);\n  font-synthesis: none;\n}\n.rd-document main {\n  width: min(42rem, calc(100% - 2rem));\n  margin: 0 auto;\n  padding: 2.5rem 0 4rem;\n}\n\n.rd-header-1,\n.rd-header-2,\n.rd-header-3,\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n}\n.rd-header-1 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  margin: 0 0 0.75rem;\n  font-size: clamp(2rem, 5vw, 2.8rem);\n  letter-spacing: -0.03em;\n  line-height: 1.15;\n}\n.rd-header-2 {\n  color: var(--rd-header-2-color);\n  font-family: var(--rd-header-2-font);\n  margin: 2rem 0 0.6rem;\n  font-size: 1.35rem;\n  letter-spacing: -0.02em;\n}\n.rd-header-3 {\n  color: var(--rd-header-3-color);\n  font-family: var(--rd-header-3-font);\n  margin: 1.5rem 0 0.5rem;\n  font-size: 1.15rem;\n}\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  margin: 1.25rem 0 0.4rem;\n  font-size: 1rem;\n}\n.rd-header-4 {\n  color: var(--rd-header-4-color);\n  font-family: var(--rd-header-4-font);\n}\n.rd-header-5 {\n  color: var(--rd-header-5-color);\n  font-family: var(--rd-header-5-font);\n}\n.rd-header-6 {\n  color: var(--rd-header-6-color);\n  font-family: var(--rd-header-6-font);\n}\n\n.rd-paragraph,\n.rd-list-item,\n.rd-task-item {\n  color: var(--rd-paragraph-color);\n  line-height: 1.65;\n}\n.rd-paragraph {\n  margin: 0 0 1rem;\n}\n.rd-list,\n.rd-list-ordered {\n  color: var(--rd-list-color);\n}\n.rd-link {\n  color: var(--rd-link-color);\n}\n.rd-link:hover {\n  color: var(--rd-link-hover-color);\n}\n.rd-blockquote {\n  margin: 0 0 1rem;\n  padding: 0.2rem 0 0.2rem 1rem;\n  border-left: 3px solid var(--rd-blockquote-border);\n  color: var(--rd-blockquote-color);\n}\n.rd-code-block {\n  margin: 0 0 1.25rem;\n  padding: 1rem 1.1rem;\n  overflow-x: auto;\n  border: 1px solid var(--rd-code-block-border);\n  border-radius: 0.75rem;\n  background: var(--rd-code-block-background);\n}\n.rd-code {\n  font-family: var(--rd-font-code);\n  color: var(--rd-code-color);\n  font-size: 0.92em;\n}\n:not(pre) > .rd-code {\n  padding: 0.1em 0.35em;\n  border-radius: 0.3rem;\n  background: var(--rd-code-background);\n}\n.rd-table {\n  width: 100%;\n  margin: 0 0 1.25rem;\n  border-collapse: collapse;\n}\n.rd-table-header,\n.rd-table-cell {\n  padding: 0.4rem 0.6rem;\n  border: 1px solid var(--rd-table-border);\n  text-align: left;\n}\n.rd-table-header {\n  background: var(--rd-table-header-background);\n  color: var(--rd-table-header-color);\n}\n.rd-thematic-break {\n  border: 0;\n  border-top: 1px solid var(--rd-thematic-break-color);\n  margin: 1.5rem 0;\n}\n.rd-image {\n  max-width: 100%;\n  height: auto;\n}\n.rd-strong {\n  color: var(--rd-strong-color);\n}\n.rd-emphasis {\n  color: var(--rd-emphasis-color);\n}\n.rd-strikethrough {\n  color: var(--rd-strikethrough-color);\n}\n"),
                        ],
                    ),
                    Html.element(
                        "style",
                        [],
                        [
                            Html.text("@scope ([data-rocci-css~=\"Guide-6f3d6b54\"]) {\n* { box-sizing: border-box; }\n    body {\n        background:\n            radial-gradient(circle at 78% 8%, rgba(55, 242, 159, 0.13), transparent 31rem),\n            linear-gradient(145deg, var(--rd-color-bg) 0%, var(--rd-color-bg) 55%, var(--rd-color-surface) 100%);\n    }\n}"),
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
