import Html

Status : [Ready, Loading]

items = [{ name: "alpha" }, { name: "beta" }]

status = Ready

show_notice = True

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
    visible = List.keep_if(items, |_| True)

    Html.fragment(
        List.concat(
            List.concat(
                [
                    Html.element(
                        "h1",
                        [
                            Html.attribute("class", "rd-header-1"),
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
                            Html.attribute("class", "rd-paragraph"),
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text("Email "),
                            Html.element(
                                "a",
                                [
                                    Html.attribute("class", "rd-link"),
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
                            Html.attribute("class", "rd-paragraph"),
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.text("This is "),
                            Html.element(
                                "strong",
                                [
                                    Html.attribute("class", "rd-strong"),
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
                                    Html.attribute("class", "rd-emphasis"),
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
                            Html.attribute("class", "rd-list"),
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.element(
                                "li",
                                [
                                    Html.attribute("class", "rd-list-item"),
                                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                ],
                                [
                                    Html.element(
                                        "p",
                                        [
                                            Html.attribute("class", "rd-paragraph"),
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
                            Html.attribute("class", "rd-code-block"),
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.element(
                                "code",
                                [
                                    Html.attribute("class", "rd-code language-roc"),
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
                            Html.attribute("class", "rd-table"),
                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                        ],
                        [
                            Html.element(
                                "thead",
                                [
                                    Html.attribute("class", "rd-table-head"),
                                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                ],
                                [
                                    Html.element(
                                        "tr",
                                        [
                                            Html.attribute("class", "rd-table-row"),
                                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                        ],
                                        [
                                            Html.element(
                                                "th",
                                                [
                                                    Html.attribute("class", "rd-table-header"),
                                                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                                ],
                                                [
                                                    Html.text("col"),
                                                ],
                                            ),
                                            Html.element(
                                                "th",
                                                [
                                                    Html.attribute("class", "rd-table-header"),
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
                                    Html.attribute("class", "rd-table-body"),
                                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                ],
                                [
                                    Html.element(
                                        "tr",
                                        [
                                            Html.attribute("class", "rd-table-row"),
                                            Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                        ],
                                        [
                                            Html.element(
                                                "td",
                                                [
                                                    Html.attribute("class", "rd-table-cell"),
                                                    Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                                ],
                                                [
                                                    Html.text("a"),
                                                ],
                                            ),
                                            Html.element(
                                                "td",
                                                [
                                                    Html.attribute("class", "rd-table-cell"),
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
                    } else if List.is_empty(visible) {
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
                            "figure",
                            [
                                Html.attribute("class", "rd-docs-figure rd-docs-block"),
                                Html.attribute("data-rocci-docs", "figure"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.void_element(
                                    "img",
                                    [
                                        Html.attribute("class", "rd-image"),
                                        Html.attribute("src", "./img/yammi_banana.png"),
                                        Html.attribute("alt", "A banana"),
                                        Html.attribute("width", "50px"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                ),
                                Html.element(
                                    "figcaption",
                                    [
                                        Html.attribute("class", "rd-docs-caption"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("A banana"),
                                    ],
                                ),
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-docs-credit"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Rocci fixtures"),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "aside",
                            [
                                Html.attribute("class", "rd-docs-aside rd-docs-block rd-docs-note"),
                                Html.attribute("data-rocci-docs", "note"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-docs-label"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Note"),
                                    ],
                                ),
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-paragraph"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Don't do this."),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "aside",
                            [
                                Html.attribute("class", "rd-docs-aside rd-docs-block rd-docs-note"),
                                Html.attribute("data-rocci-docs", "note"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-docs-label"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Note"),
                                    ],
                                ),
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-docs-title"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Watch"),
                                    ],
                                ),
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-paragraph"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Nested section body."),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "details",
                            [
                                Html.attribute("class", "rd-docs-details rd-docs-block"),
                                Html.attribute("data-rocci-docs", "details"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "summary",
                                    [
                                        Html.attribute("class", "rd-docs-summary"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("More"),
                                    ],
                                ),
                                Html.element(
                                    "p",
                                    [
                                        Html.attribute("class", "rd-paragraph"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.text("Nested details body."),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "section",
                            [
                                Html.attribute("class", "rd-docs-steps rd-docs-block"),
                                Html.attribute("data-rocci-docs", "steps"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "section",
                                    [
                                        Html.attribute("class", "rd-docs-step rd-docs-block"),
                                        Html.attribute("data-rocci-docs", "step"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-docs-title"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("One"),
                                            ],
                                        ),
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-paragraph"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("First step."),
                                            ],
                                        ),
                                    ],
                                ),
                                Html.element(
                                    "section",
                                    [
                                        Html.attribute("class", "rd-docs-step rd-docs-block"),
                                        Html.attribute("data-rocci-docs", "step"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-docs-title"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("Two"),
                                            ],
                                        ),
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-paragraph"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("Second step."),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "section",
                            [
                                Html.attribute("class", "rd-docs-tabs rd-docs-block"),
                                Html.attribute("data-rocci-docs", "tabs"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "section",
                                    [
                                        Html.attribute("class", "rd-docs-tab rd-docs-block"),
                                        Html.attribute("data-rocci-docs", "tab"),
                                        Html.attribute("aria-label", "macOS"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "h3",
                                            [
                                                Html.attribute("class", "rd-docs-tab-label"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("macOS"),
                                            ],
                                        ),
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-paragraph"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("Mac panel."),
                                            ],
                                        ),
                                    ],
                                ),
                                Html.element(
                                    "section",
                                    [
                                        Html.attribute("class", "rd-docs-tab rd-docs-block"),
                                        Html.attribute("data-rocci-docs", "tab"),
                                        Html.attribute("aria-label", "Linux"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "h3",
                                            [
                                                Html.attribute("class", "rd-docs-tab-label"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("Linux"),
                                            ],
                                        ),
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-paragraph"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("Linux panel."),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        Html.element(
                            "p",
                            [
                                Html.attribute("class", "rd-paragraph"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("@if this is escaped"),
                            ],
                        ),
                        Html.element(
                            "p",
                            [
                                Html.attribute("class", "rd-paragraph"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.text("More markdown after."),
                                Html.element(
                                    "sup",
                                    [
                                        Html.attribute("class", "rd-footnote-ref"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "a",
                                            [
                                                Html.attribute("href", "#fn-note"),
                                                Html.attribute("id", "fnref-note"),
                                                Html.boolean_attribute("data-footnote-ref", True),
                                                Html.attribute("aria-label", "Footnote 1"),
                                            ],
                                            [
                                                Html.text("1"),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ),
            [
                Html.element(
                    "section",
                    [
                        Html.attribute("class", "rd-footnotes"),
                        Html.boolean_attribute("data-footnotes", True),
                        Html.attribute("aria-label", "Footnotes"),
                    ],
                    [
                        Html.element(
                            "ol",
                            [
                                Html.attribute("class", "rd-footnote-list"),
                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                            ],
                            [
                                Html.element(
                                    "li",
                                    [
                                        Html.attribute("class", "rd-footnote-definition"),
                                        Html.attribute("id", "fn-note"),
                                        Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                    ],
                                    [
                                        Html.element(
                                            "p",
                                            [
                                                Html.attribute("class", "rd-paragraph"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.text("Footnotes stay in ordinary Rocdown and include a backlink."),
                                            ],
                                        ),
                                        Html.element(
                                            "span",
                                            [
                                                Html.attribute("class", "rd-footnote-backlinks"),
                                                Html.attribute("data-rocci-css", "AllSyntax-13744130"),
                                            ],
                                            [
                                                Html.element(
                                                    "a",
                                                    [
                                                        Html.attribute("class", "rd-footnote-backref"),
                                                        Html.attribute("href", "#fnref-note"),
                                                        Html.boolean_attribute("data-footnote-backref", True),
                                                        Html.attribute("aria-label", "Back to reference note"),
                                                    ],
                                                    [
                                                        Html.text("↩"),
                                                    ],
                                                ),
                                            ],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )
    )
}

rocci_page = |{}| {
    Html.element(
        "html",
        [
            Html.attribute("lang", "en"),
            Html.attribute("class", "rd-document"),
            Html.attribute("data-rd-theme", "paper"),
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
                    Html.void_element(
                        "meta",
                        [
                            Html.attribute("name", "color-scheme"),
                            Html.attribute("content", "light dark"),
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
                            Html.text(".rd-document {\n  color-scheme: light dark;\n  --rd-font-body: ui-sans-serif, system-ui, sans-serif;\n  --rd-font-heading: ui-sans-serif, system-ui, sans-serif;\n  --rd-font-code: ui-monospace, SFMono-Regular, Menlo, monospace;\n\n  --rd-color-bg: light-dark(#fafafa, #282c34);\n  --rd-color-surface: light-dark(#ffffff, #21252b);\n  --rd-color-text: light-dark(#383a42, #abb2bf);\n  --rd-color-muted: light-dark(#696c77, #9da5b4);\n  --rd-color-accent: light-dark(#4078f2, #61afef);\n  --rd-color-border: light-dark(#e5e5e6, #3e4451);\n  --rd-color-code-bg: light-dark(#f0f0f1, #21252b);\n  --rd-color-code-text: light-dark(#383a42, #abb2bf);\n\n  --rd-header-1-color: var(--rd-color-text);\n  --rd-header-2-color: var(--rd-color-text);\n  --rd-header-3-color: var(--rd-color-text);\n  --rd-header-4-color: var(--rd-color-text);\n  --rd-header-5-color: var(--rd-color-text);\n  --rd-header-6-color: var(--rd-color-text);\n  --rd-header-1-font: var(--rd-font-heading);\n  --rd-header-2-font: var(--rd-font-heading);\n  --rd-header-3-font: var(--rd-font-heading);\n  --rd-header-4-font: var(--rd-font-heading);\n  --rd-header-5-font: var(--rd-font-heading);\n  --rd-header-6-font: var(--rd-font-heading);\n  --rd-paragraph-color: var(--rd-color-text);\n  --rd-blockquote-color: var(--rd-color-muted);\n  --rd-blockquote-border: var(--rd-color-accent);\n  --rd-list-color: var(--rd-color-text);\n  --rd-link-color: var(--rd-color-accent);\n  --rd-link-hover-color: var(--rd-color-text);\n  --rd-code-color: var(--rd-color-code-text);\n  --rd-code-background: var(--rd-color-code-bg);\n  --rd-code-block-background: var(--rd-color-code-bg);\n  --rd-code-block-border: var(--rd-color-border);\n  --rd-table-border: var(--rd-color-border);\n  --rd-table-header-background: var(--rd-color-surface);\n  --rd-table-header-color: var(--rd-color-text);\n  --rd-thematic-break-color: var(--rd-color-border);\n  --rd-strong-color: var(--rd-color-text);\n  --rd-emphasis-color: var(--rd-color-muted);\n  --rd-strikethrough-color: var(--rd-color-muted);\n}\n.rd-document,\n.rd-document body {\n  scroll-behavior: smooth;\n}\n.rd-document:not([data-rd-color-scheme]) {\n  color-scheme: light dark;\n}\n.rd-document[data-rd-color-scheme=\"light\"] {\n  color-scheme: light;\n}\n.rd-document[data-rd-color-scheme=\"dark\"] {\n  color-scheme: dark;\n}\n\n.rd-document body {\n  margin: 0;\n  min-height: 100vh;\n  background: var(--rd-color-bg);\n  color: var(--rd-color-text);\n  font-family: var(--rd-font-body);\n  font-synthesis: none;\n}\n.rd-shell {\n  display: grid;\n  grid-template-columns: 16.5rem minmax(0, 1fr);\n  align-items: start;\n  min-height: 100vh;\n}\n.rd-document main {\n  box-sizing: border-box;\n  min-width: 0;\n  width: min(42rem, calc(100% - 2rem));\n  margin: 0 auto;\n  padding: 2.5rem 0 4rem;\n}\n\n.rd-toc {\n  position: sticky;\n  top: 0;\n  box-sizing: border-box;\n  min-width: 0;\n  max-height: 100vh;\n  padding: 2.15rem 1.2rem 2rem 1.5rem;\n  overflow-x: hidden;\n  overflow-y: auto;\n  user-select: none;\n}\n.rd-toc-label {\n  margin: 0 0 0.65rem;\n  color: var(--rd-color-muted);\n  font-size: 0.68rem;\n  font-weight: 700;\n  letter-spacing: 0.105em;\n  text-transform: uppercase;\n}\n.rd-toc-items {\n  display: grid;\n  gap: 0.45rem;\n  border-left: 1px solid var(--rd-color-border);\n}\n.rd-toc-link {\n  margin-left: -1px;\n  padding-left: 0.8rem;\n  border-left: 1px solid transparent;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  line-height: 1.35;\n  text-decoration: none;\n  overflow-wrap: anywhere;\n}\n.rd-toc-link:hover {\n  border-color: var(--rd-color-accent);\n  color: var(--rd-color-text);\n}\n.rd-toc-link.rd-toc-level-3 {\n  padding-left: 1.35rem;\n}\n.rd-toc:not(:has(.rd-toc-link)) {\n  display: none;\n}\n\n@media (max-width: 48rem) {\n  .rd-shell {\n    display: block;\n  }\n  .rd-toc {\n    display: none;\n  }\n}\n@media print {\n  .rd-toc {\n    display: none;\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  .rd-document,\n  .rd-document body {\n    scroll-behavior: auto;\n  }\n}\n\n.rd-header-1,\n.rd-header-2,\n.rd-header-3,\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  scroll-margin-top: calc(1.25rem + var(--rd-chrome-top, 0px));\n}\n.rd-header-1:target,\n.rd-header-2:target,\n.rd-header-3:target,\n.rd-header-4:target,\n.rd-header-5:target,\n.rd-header-6:target {\n  color: var(--rd-color-accent);\n}\n.rd-header-1 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  margin: 0 0 0.75rem;\n  font-size: clamp(2rem, 5vw, 2.8rem);\n  letter-spacing: -0.03em;\n  line-height: 1.15;\n}\n.rd-header-2 {\n  color: var(--rd-header-2-color);\n  font-family: var(--rd-header-2-font);\n  margin: 2rem 0 0.6rem;\n  font-size: 1.35rem;\n  letter-spacing: -0.02em;\n}\n.rd-header-3 {\n  color: var(--rd-header-3-color);\n  font-family: var(--rd-header-3-font);\n  margin: 1.5rem 0 0.5rem;\n  font-size: 1.15rem;\n}\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  margin: 1.25rem 0 0.4rem;\n  font-size: 1rem;\n}\n.rd-header-4 {\n  color: var(--rd-header-4-color);\n  font-family: var(--rd-header-4-font);\n}\n.rd-header-5 {\n  color: var(--rd-header-5-color);\n  font-family: var(--rd-header-5-font);\n}\n.rd-header-6 {\n  color: var(--rd-header-6-color);\n  font-family: var(--rd-header-6-font);\n}\n\n.rd-paragraph,\n.rd-list-item,\n.rd-task-item {\n  color: var(--rd-paragraph-color);\n  line-height: 1.65;\n}\n.rd-paragraph {\n  margin: 0 0 1rem;\n}\n.rd-list,\n.rd-list-ordered {\n  color: var(--rd-list-color);\n}\n.rd-link {\n  color: var(--rd-link-color);\n}\n.rd-link:hover {\n  color: var(--rd-link-hover-color);\n}\n.rd-blockquote {\n  margin: 0 0 1rem;\n  padding: 0.2rem 0 0.2rem 1rem;\n  border-left: 3px solid var(--rd-blockquote-border);\n  color: var(--rd-blockquote-color);\n}\n.rd-code-block {\n  margin: 0 0 1.25rem;\n  padding: 1rem 1.1rem;\n  overflow-x: auto;\n  border: 1px solid var(--rd-code-block-border);\n  border-radius: 0.75rem;\n  background: var(--rd-code-block-background);\n}\n.rd-code {\n  font-family: var(--rd-font-code);\n  color: var(--rd-code-color);\n  font-size: 0.92em;\n}\n:not(pre) > .rd-code {\n  padding: 0.1em 0.35em;\n  border-radius: 0.3rem;\n  background: var(--rd-code-background);\n}\n.rd-table {\n  width: 100%;\n  margin: 0 0 1.25rem;\n  border-collapse: collapse;\n}\n.rd-table-header,\n.rd-table-cell {\n  padding: 0.4rem 0.6rem;\n  border: 1px solid var(--rd-table-border);\n  text-align: left;\n}\n.rd-table-header {\n  background: var(--rd-table-header-background);\n  color: var(--rd-table-header-color);\n}\n.rd-thematic-break {\n  border: 0;\n  border-top: 1px solid var(--rd-thematic-break-color);\n  margin: 1.5rem 0;\n}\n.rd-image {\n  max-width: 100%;\n  height: auto;\n}\n.rd-docs-block { margin: 1.25rem 0; }\n.rd-docs-aside {\n  padding: 0.85rem 1rem;\n  border: 1px solid var(--rd-color-border);\n  border-left-width: 0.35rem;\n  border-radius: 0.55rem;\n  background: var(--rd-color-surface);\n}\n.rd-docs-note { border-left-color: var(--rd-color-accent); }\n.rd-docs-tip { border-left-color: #2f7d4a; }\n.rd-docs-caution { border-left-color: #b7791f; }\n.rd-docs-danger, .rd-docs-deprecated { border-left-color: var(--rd-color-accent); }\n.rd-docs-label {\n  margin: 0 0 0.35rem;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  font-weight: 750;\n  letter-spacing: 0.04em;\n  text-transform: uppercase;\n}\n.rd-docs-title { margin: 0 0 0.4rem; font-weight: 700; }\n.rd-docs-body > :first-child { margin-top: 0; }\n.rd-docs-body > :last-child { margin-bottom: 0; }\n.rd-docs-summary { font-weight: 700; cursor: pointer; }\n.rd-docs-card {\n  display: block;\n  padding: 0.9rem 1rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 0.6rem;\n  text-decoration: none;\n  background: var(--rd-color-surface);\n}\n.rd-docs-card-title { display: block; font-weight: 700; }\n.rd-docs-card-summary { display: block; color: var(--rd-color-muted); margin-top: 0.25rem; }\n.rd-docs-card-grid {\n  display: grid;\n  gap: 0.75rem;\n  grid-template-columns: 1fr;\n}\n.rd-docs-tree { overflow-x: auto; }\n.rd-docs-tab { margin: 1rem 0; }\n.rd-docs-tab-label { margin: 0 0 0.4rem; font-size: 1rem; }\n.rd-docs-badge-label {\n  display: inline-block;\n  padding: 0.15rem 0.55rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 999px;\n  font-size: 0.78rem;\n  font-weight: 700;\n}\n.rd-docs-steps { padding-left: 0; margin: 1.25rem 0; counter-reset: rd-step; list-style: none; }\n.rd-docs-step { margin: 0.75rem 0; padding-left: 1.6rem; counter-increment: rd-step; }\n.rd-docs-step::before {\n  content: counter(rd-step) \". \";\n  margin-left: -1.6rem;\n  font-weight: 700;\n}\n.rd-docs-verify {\n  display: inline-block;\n  margin: 0 0 0.35rem;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  font-weight: 750;\n  letter-spacing: 0.04em;\n  text-transform: uppercase;\n}\n.rd-docs-figure { margin: 1.25rem 0; }\n.rd-docs-caption, .rd-docs-credit { color: var(--rd-color-muted); }\n.rd-docs-compatibility { overflow-x: auto; }\n@media (min-width: 48rem) {\n  .rd-docs-card-grid { grid-template-columns: 1fr 1fr; }\n}\n@media (forced-colors: active) {\n  .rd-docs-aside, .rd-docs-card, .rd-docs-badge-label {\n    border: 1px solid currentColor;\n  }\n}\n@media print {\n  .rd-docs-tab { break-inside: avoid; }\n  .rd-docs-aside { border: 1px solid currentColor; background: transparent; }\n}\n.rd-strong {\n  color: var(--rd-strong-color);\n}\n.rd-emphasis {\n  color: var(--rd-emphasis-color);\n}\n.rd-strikethrough {\n  color: var(--rd-strikethrough-color);\n}\n"),
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
