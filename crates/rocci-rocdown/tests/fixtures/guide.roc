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
                        Html.attribute("href", "/guides/rocdown-blocks/"),
                        Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                    ],
                    [
                        Html.text("Blocks"),
                    ],
                ),
                Html.text(", "),
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
                Html.text(", and the "),
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
                            Html.text(".rd-document {\n  color-scheme: light dark;\n  --rd-font-body: Inter, ui-sans-serif, system-ui, sans-serif;\n  --rd-font-heading: Inter, ui-sans-serif, system-ui, sans-serif;\n  --rd-font-code: ui-monospace, SFMono-Regular, Menlo, monospace;\n\n  --rd-color-bg: light-dark(#f3f7f5, #07110e);\n  --rd-color-surface: light-dark(#ffffff, #0b1e18);\n  --rd-color-text: light-dark(#07110e, #f3f7f5);\n  --rd-color-muted: light-dark(#4a635a, #c9d8d2);\n  --rd-color-accent: light-dark(#0d9a5c, #48eda4);\n  --rd-color-border: light-dark(#b7d4c8, #29493b);\n  --rd-color-code-bg: light-dark(#e7f2ec, #0b1e18);\n  --rd-color-code-text: light-dark(#07110e, #f8fffb);\n\n  --rd-header-1-color: var(--rd-color-text);\n  --rd-header-2-color: var(--rd-color-text);\n  --rd-header-3-color: var(--rd-color-text);\n  --rd-header-4-color: var(--rd-color-text);\n  --rd-header-5-color: var(--rd-color-text);\n  --rd-header-6-color: var(--rd-color-text);\n  --rd-header-1-font: var(--rd-font-heading);\n  --rd-header-2-font: var(--rd-font-heading);\n  --rd-header-3-font: var(--rd-font-heading);\n  --rd-header-4-font: var(--rd-font-heading);\n  --rd-header-5-font: var(--rd-font-heading);\n  --rd-header-6-font: var(--rd-font-heading);\n  --rd-paragraph-color: var(--rd-color-muted);\n  --rd-blockquote-color: var(--rd-color-muted);\n  --rd-blockquote-border: var(--rd-color-border);\n  --rd-list-color: var(--rd-color-muted);\n  --rd-link-color: var(--rd-color-accent);\n  --rd-link-hover-color: var(--rd-color-text);\n  --rd-code-color: var(--rd-color-code-text);\n  --rd-code-background: var(--rd-color-code-bg);\n  --rd-code-block-background: var(--rd-color-code-bg);\n  --rd-code-block-border: var(--rd-color-border);\n  --rd-table-border: var(--rd-color-border);\n  --rd-table-header-background: var(--rd-color-surface);\n  --rd-table-header-color: var(--rd-color-text);\n  --rd-thematic-break-color: var(--rd-color-border);\n  --rd-strong-color: var(--rd-color-text);\n  --rd-emphasis-color: var(--rd-color-muted);\n  --rd-strikethrough-color: var(--rd-color-muted);\n}\n.rd-document,\n.rd-document body {\n  scroll-behavior: smooth;\n}\n.rd-document:not([data-rd-color-scheme]) {\n  color-scheme: light dark;\n}\n.rd-document[data-rd-color-scheme=\"light\"] {\n  color-scheme: light;\n}\n.rd-document[data-rd-color-scheme=\"dark\"] {\n  color-scheme: dark;\n}\n\n.rd-document body {\n  margin: 0;\n  min-height: 100vh;\n  background: var(--rd-color-bg);\n  color: var(--rd-color-text);\n  font-family: var(--rd-font-body);\n  font-synthesis: none;\n}\n.rd-shell {\n  display: grid;\n  grid-template-columns: 16.5rem minmax(0, 1fr);\n  align-items: start;\n  min-height: 100vh;\n}\n.rd-document main {\n  box-sizing: border-box;\n  min-width: 0;\n  width: min(42rem, calc(100% - 2rem));\n  margin: 0 auto;\n  padding: 2.5rem 0 4rem;\n}\n\n.rd-toc {\n  position: sticky;\n  top: 0;\n  box-sizing: border-box;\n  min-width: 0;\n  max-height: 100vh;\n  padding: 2.15rem 1.2rem 2rem 1.5rem;\n  overflow-x: hidden;\n  overflow-y: auto;\n  user-select: none;\n}\n.rd-toc-label {\n  margin: 0 0 0.65rem;\n  color: var(--rd-color-muted);\n  font-size: 0.68rem;\n  font-weight: 700;\n  letter-spacing: 0.105em;\n  text-transform: uppercase;\n}\n.rd-toc-items {\n  display: grid;\n  gap: 0.45rem;\n  border-left: 1px solid var(--rd-color-border);\n}\n.rd-toc-link {\n  margin-left: -1px;\n  padding-left: 0.8rem;\n  border-left: 1px solid transparent;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  line-height: 1.35;\n  text-decoration: none;\n  overflow-wrap: anywhere;\n}\n.rd-toc-link:hover {\n  border-color: var(--rd-color-accent);\n  color: var(--rd-color-text);\n}\n.rd-toc-link.rd-toc-level-3 {\n  padding-left: 1.35rem;\n}\n.rd-toc:not(:has(.rd-toc-link)) {\n  display: none;\n}\n\n.rd-toc-menu {\n  display: none;\n}\n\n@media (max-width: 48rem) {\n  .rd-shell {\n    display: block;\n  }\n  .rd-toc {\n    display: none;\n  }\n  .rd-toc-menu {\n    display: block;\n    box-sizing: border-box;\n    margin: 1rem auto 0;\n    width: min(42rem, calc(100% - 2rem));\n  }\n  .rd-toc-menu summary {\n    display: flex;\n    align-items: center;\n    min-height: 2.75rem;\n    padding: 0 0.85rem;\n    border: 1px solid var(--rd-color-border);\n    border-radius: 0.5rem;\n    background: var(--rd-color-surface);\n    color: var(--rd-color-text);\n    font-size: 0.85rem;\n    font-weight: 650;\n    cursor: pointer;\n    list-style: none;\n  }\n  .rd-toc-menu summary::-webkit-details-marker {\n    display: none;\n  }\n  .rd-toc-menu .rd-toc-items {\n    margin-top: 0.65rem;\n  }\n}\n@media print {\n  .rd-toc,\n  .rd-toc-menu {\n    display: none;\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  .rd-document,\n  .rd-document body {\n    scroll-behavior: auto;\n  }\n}\n\n.rd-header-1,\n.rd-header-2,\n.rd-header-3,\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  scroll-margin-top: calc(1.25rem + var(--rd-chrome-top, 0px));\n}\n.rd-header-1:target,\n.rd-header-2:target,\n.rd-header-3:target,\n.rd-header-4:target,\n.rd-header-5:target,\n.rd-header-6:target {\n  color: var(--rd-color-accent);\n}\n.rd-header-1 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  margin: 0 0 0.75rem;\n  font-size: clamp(2rem, 5vw, 2.8rem);\n  letter-spacing: -0.03em;\n  line-height: 1.15;\n}\n.rd-header-2 {\n  color: var(--rd-header-2-color);\n  font-family: var(--rd-header-2-font);\n  margin: 2rem 0 0.6rem;\n  font-size: 1.35rem;\n  letter-spacing: -0.02em;\n}\n.rd-header-3 {\n  color: var(--rd-header-3-color);\n  font-family: var(--rd-header-3-font);\n  margin: 1.5rem 0 0.5rem;\n  font-size: 1.15rem;\n}\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  margin: 1.25rem 0 0.4rem;\n  font-size: 1rem;\n}\n.rd-header-4 {\n  color: var(--rd-header-4-color);\n  font-family: var(--rd-header-4-font);\n}\n.rd-header-5 {\n  color: var(--rd-header-5-color);\n  font-family: var(--rd-header-5-font);\n}\n.rd-header-6 {\n  color: var(--rd-header-6-color);\n  font-family: var(--rd-header-6-font);\n}\n\n.rd-paragraph,\n.rd-list-item,\n.rd-task-item {\n  color: var(--rd-paragraph-color);\n  line-height: 1.65;\n}\n.rd-paragraph {\n  margin: 0 0 1rem;\n}\n.rd-list,\n.rd-list-ordered {\n  color: var(--rd-list-color);\n}\n.rd-link {\n  color: var(--rd-link-color);\n}\n.rd-link:hover {\n  color: var(--rd-link-hover-color);\n}\n.rd-blockquote {\n  margin: 0 0 1rem;\n  padding: 0.2rem 0 0.2rem 1rem;\n  border-left: 3px solid var(--rd-blockquote-border);\n  color: var(--rd-blockquote-color);\n}\n.rd-code-block {\n  margin: 0 0 1.25rem;\n  padding: 1rem 1.1rem;\n  overflow-x: auto;\n  border: 1px solid var(--rd-code-block-border);\n  border-radius: 0.75rem;\n  background: var(--rd-code-block-background);\n}\n.rd-code {\n  font-family: var(--rd-font-code);\n  color: var(--rd-code-color);\n  font-size: 0.92em;\n}\n:not(pre) > .rd-code {\n  padding: 0.1em 0.35em;\n  border-radius: 0.3rem;\n  background: var(--rd-code-background);\n}\n.rd-table-wrap {\n  overflow-x: auto;\n  -webkit-overflow-scrolling: touch;\n  margin: 0 0 1.25rem;\n}\n.rd-table {\n  width: max-content;\n  min-width: 100%;\n  margin: 0;\n  border-collapse: collapse;\n}\n.rd-table-header,\n.rd-table-cell {\n  padding: 0.4rem 0.6rem;\n  border: 1px solid var(--rd-table-border);\n  text-align: left;\n}\n.rd-table-header {\n  background: var(--rd-table-header-background);\n  color: var(--rd-table-header-color);\n}\n.rd-thematic-break {\n  border: 0;\n  border-top: 1px solid var(--rd-thematic-break-color);\n  margin: 1.5rem 0;\n}\n.rd-image {\n  max-width: 100%;\n  height: auto;\n}\n.rd-docs-block { margin: 1.25rem 0; }\n.rd-docs-aside {\n  padding: 0.85rem 1rem;\n  border: 1px solid var(--rd-color-border);\n  border-left-width: 0.35rem;\n  border-radius: 0.55rem;\n  background: var(--rd-color-surface);\n}\n.rd-docs-note { border-left-color: var(--rd-color-accent); }\n.rd-docs-tip { border-left-color: #2f7d4a; }\n.rd-docs-caution { border-left-color: #b7791f; }\n.rd-docs-danger, .rd-docs-deprecated { border-left-color: var(--rd-color-accent); }\n.rd-docs-label {\n  margin: 0 0 0.35rem;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  font-weight: 750;\n  letter-spacing: 0.04em;\n  text-transform: uppercase;\n}\n.rd-docs-title { margin: 0 0 0.4rem; font-weight: 700; }\n.rd-docs-body > :first-child { margin-top: 0; }\n.rd-docs-body > :last-child { margin-bottom: 0; }\n.rd-docs-summary { font-weight: 700; cursor: pointer; }\n.rd-docs-card {\n  display: block;\n  padding: 0.9rem 1rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 0.6rem;\n  text-decoration: none;\n  background: var(--rd-color-surface);\n}\n.rd-docs-card-title { display: block; font-weight: 700; }\n.rd-docs-card-summary { display: block; color: var(--rd-color-muted); margin-top: 0.25rem; }\n.rd-docs-card-grid {\n  display: grid;\n  gap: 0.75rem;\n  grid-template-columns: 1fr;\n}\n.rd-docs-tree { overflow-x: auto; }\n.rd-docs-tab { margin: 1rem 0; }\n.rd-docs-tab-label { margin: 0 0 0.4rem; font-size: 1rem; }\n.rd-docs-badge-label {\n  display: inline-block;\n  padding: 0.15rem 0.55rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 999px;\n  font-size: 0.78rem;\n  font-weight: 700;\n}\n.rd-docs-steps { padding-left: 0; margin: 1.25rem 0; counter-reset: rd-step; list-style: none; }\n.rd-docs-step { margin: 0.75rem 0; padding-left: 1.6rem; counter-increment: rd-step; }\n.rd-docs-step::before {\n  content: counter(rd-step) \". \";\n  margin-left: -1.6rem;\n  font-weight: 700;\n}\n.rd-docs-verify {\n  display: inline-block;\n  margin: 0 0 0.35rem;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  font-weight: 750;\n  letter-spacing: 0.04em;\n  text-transform: uppercase;\n}\n.rd-docs-figure { margin: 1.25rem 0; }\n.rd-docs-caption, .rd-docs-credit { color: var(--rd-color-muted); }\n.rd-docs-compatibility { overflow-x: auto; }\n@media (min-width: 48rem) {\n  .rd-docs-card-grid { grid-template-columns: 1fr 1fr; }\n}\n@media (forced-colors: active) {\n  .rd-docs-aside, .rd-docs-card, .rd-docs-badge-label {\n    border: 1px solid currentColor;\n  }\n}\n@media print {\n  .rd-docs-tab { break-inside: avoid; }\n  .rd-docs-aside { border: 1px solid currentColor; background: transparent; }\n}\n.rd-strong {\n  color: var(--rd-strong-color);\n}\n.rd-emphasis {\n  color: var(--rd-emphasis-color);\n}\n.rd-strikethrough {\n  color: var(--rd-strikethrough-color);\n}\n"),
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
                        "div",
                        [
                            Html.attribute("class", "rd-shell"),
                            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                        ],
                        [
                            Html.element(
                                "nav",
                                [
                                    Html.attribute("class", "rd-toc"),
                                    Html.attribute("aria-label", "On this page"),
                                    Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                ],
                                [
                                    Html.element(
                                        "p",
                                        [
                                            Html.attribute("class", "rd-toc-label"),
                                            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                        ],
                                        [
                                            Html.text("On this page"),
                                        ],
                                    ),
                                    Html.element(
                                        "div",
                                        [
                                            Html.attribute("class", "rd-toc-items"),
                                            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                        ],
                                        [
                                            Html.element(
                                                "a",
                                                [
                                                    Html.attribute("class", "rd-toc-link"),
                                                    Html.attribute("href", "#displayed-code"),
                                                    Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                                ],
                                                [
                                                    Html.text("Displayed code"),
                                                ],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                            Html.element(
                                "details",
                                [
                                    Html.attribute("class", "rd-toc-menu"),
                                    Html.attribute("aria-label", "On this page"),
                                    Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                ],
                                [
                                    Html.element(
                                        "summary",
                                        [
                                            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                        ],
                                        [
                                            Html.text("On this page"),
                                        ],
                                    ),
                                    Html.element(
                                        "div",
                                        [
                                            Html.attribute("class", "rd-toc-items"),
                                            Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                        ],
                                        [
                                            Html.element(
                                                "a",
                                                [
                                                    Html.attribute("class", "rd-toc-link"),
                                                    Html.attribute("href", "#displayed-code"),
                                                    Html.attribute("data-rocci-css", "Guide-6f3d6b54"),
                                                ],
                                                [
                                                    Html.text("Displayed code"),
                                                ],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
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
                    Html.dangerously_include_unescaped_html("<script>(function () {\n  if (window.__rdTocScroll) {\n    return;\n  }\n  window.__rdTocScroll = true;\n  var token = 0;\n  var pending = null;\n  function yNow() {\n    return window.pageYOffset || document.documentElement.scrollTop || document.body.scrollTop || 0;\n  }\n  function ySet(y) {\n    var html = document.documentElement;\n    var body = document.body;\n    if (html) {\n      html.style.scrollBehavior = \"auto\";\n    }\n    if (body) {\n      body.style.scrollBehavior = \"auto\";\n    }\n    if (window.scrollTo) {\n      window.scrollTo(0, y);\n    }\n    if (html) {\n      html.scrollTop = y;\n    }\n    if (body) {\n      body.scrollTop = y;\n    }\n  }\n  function restorePending() {\n    if (pending) {\n      if (!pending.el.id) {\n        pending.el.id = pending.id;\n      }\n      pending = null;\n    }\n  }\n  function tocLink(node) {\n    while (node) {\n      if (node.nodeType === 1 && node.classList && node.classList.contains(\"rd-toc-link\")) {\n        return node;\n      }\n      node = node.parentNode;\n    }\n    return null;\n  }\n  function animate(to, href) {\n    var from = yNow();\n    var dist = to - from;\n    function done() {\n      ySet(to);\n      restorePending();\n      if (history.replaceState) {\n        history.replaceState(null, \"\", href);\n      }\n    }\n    if (Math.abs(dist) < 2) {\n      done();\n      return;\n    }\n    var dur = Math.min(650, 400 + Math.abs(dist) * 0.05);\n    var start = performance.now();\n    var run = ++token;\n    function frame(now) {\n      if (run !== token) {\n        return;\n      }\n      var t = (now - start) / dur;\n      if (t >= 1) {\n        done();\n        return;\n      }\n      var k = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;\n      ySet(from + dist * k);\n      requestAnimationFrame(frame);\n    }\n    requestAnimationFrame(frame);\n  }\n  document.addEventListener(\n    \"click\",\n    function (event) {\n      var link = tocLink(event.target);\n      if (!link) {\n        return;\n      }\n      var href = link.getAttribute(\"href\") || \"\";\n      if (href.charAt(0) !== \"#\") {\n        return;\n      }\n      var id = decodeURIComponent(href.slice(1));\n      var el = document.getElementById(id);\n      if (!el) {\n        return;\n      }\n      var margin = parseFloat(window.getComputedStyle(el).scrollMarginTop);\n      if (isNaN(margin)) {\n        margin = 0;\n      }\n      var nav = document.querySelector(\"rocci-preview-nav\");\n      if (nav) {\n        var chrome = nav.getBoundingClientRect().height;\n        if (chrome > margin) {\n          margin = chrome + margin;\n        }\n      }\n      var to = el.getBoundingClientRect().top + yNow() - margin;\n      event.preventDefault();\n      if (event.stopImmediatePropagation) {\n        event.stopImmediatePropagation();\n      }\n      restorePending();\n      pending = { el: el, id: id };\n      el.removeAttribute(\"id\");\n      animate(to, href);\n    },\n    true\n  );\n})();</script>"),
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
