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
                    Html.text("@scope ([data-rocci-css~=\"Guide-fd48e9e1\"]) {\n* { box-sizing: border-box; }\n    body {\n        background:\n            radial-gradient(circle at 78% 8%, rgba(55, 242, 159, 0.13), transparent 31rem),\n            linear-gradient(145deg, var(--rd-color-bg) 0%, var(--rd-color-bg) 55%, var(--rd-color-surface) 100%);\n    }\n}\n@scope ([data-rocci-css~=\"featureCount-d0c4058e\"]) {\n.feature-count {\n            display: inline-flex;\n            margin: 0 0 1.5rem;\n            padding: 0.35rem 0.75rem;\n            border: 1px solid color-mix(in srgb, var(--rd-color-accent) 28%, transparent);\n            border-radius: 999px;\n            background: color-mix(in srgb, var(--rd-color-accent) 10%, transparent);\n            color: var(--rd-color-accent);\n            font-size: 0.8rem;\n            font-weight: 700;\n            letter-spacing: 0.04em;\n        }\n}"),
                ],
            ),
            Html.element(
                "p",
                [
                    Html.attribute("class", "feature-count"),
                    Html.attribute("data-rocci-css", "Guide-fd48e9e1 featureCount-d0c4058e"),
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
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.text("Rocdown"),
            ],
        ),
        Html.element(
            "p",
            [
                Html.attribute("class", "rd-paragraph"),
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.text("Rocdown is a content-first format. Published "),
                Html.text(published),
                Html.text(". Email us at"),
                Html.text("\n"),
                Html.element(
                    "a",
                    [
                        Html.attribute("class", "rd-link"),
                        Html.attribute("href", "mailto:docs@example.com"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                    ],
                    [
                        Html.text("docs@example.com"),
                    ],
                ),
                Html.text(" or mention @roclang normally."),
            ],
        ),
        featureCount(
            { count: feature_count },
        ),
        Html.element(
            "p",
            [
                Html.attribute("class", "rd-paragraph"),
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.text("See also "),
                Html.element(
                    "a",
                    [
                        Html.attribute("class", "rd-link"),
                        Html.attribute("href", "/guides/rocdown-blocks"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                        Html.attribute("href", "/guides/rocdown-interactive"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                        Html.attribute("href", "/guides/rocdown-interactive"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.text("Displayed code"),
            ],
        ),
        Html.element(
            "p",
            [
                Html.attribute("class", "rd-paragraph"),
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.text("This fence is documentation and is never evaluated:"),
            ],
        ),
        Html.element(
            "pre",
            [
                Html.attribute("class", "rd-code-block"),
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.element(
                    "code",
                    [
                        Html.attribute("class", "rd-code language-roc"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
            ],
            [
                Html.text("This page is "),
                Html.element(
                    "strong",
                    [
                        Html.attribute("class", "rd-strong"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                    ],
                    [
                        Html.text("hydrate"),
                    ],
                ),
                Html.text(": "),
                Html.element(
                    "code",
                    [
                        Html.attribute("class", "rd-code"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                    ],
                    [
                        Html.text("@{published}"),
                    ],
                ),
                Html.text(" is a Markdown "),
                Html.element(
                    "code",
                    [
                        Html.attribute("class", "rd-code"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                    ],
                    [
                        Html.text("Str"),
                    ],
                ),
                Html.text(" hole, and"),
                Html.text("\n"),
                Html.element(
                    "code",
                    [
                        Html.attribute("class", "rd-code"),
                        Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                    ],
                    [
                        Html.text("<FeatureCount />"),
                    ],
                ),
                Html.text(" is a document-root island."),
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
            Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                            Html.text(".rd-document {\n  color-scheme: light dark;\n  --rd-font-body: Inter, ui-sans-serif, system-ui, sans-serif;\n  --rd-font-heading: Inter, ui-sans-serif, system-ui, sans-serif;\n  --rd-font-code: ui-monospace, SFMono-Regular, Menlo, monospace;\n\n  --rd-color-bg: light-dark(#f3f7f5, #07110e);\n  --rd-color-surface: light-dark(#ffffff, #0b1e18);\n  --rd-color-text: light-dark(#07110e, #f3f7f5);\n  --rd-color-muted: light-dark(#4a635a, #c9d8d2);\n  --rd-color-accent: light-dark(#0d9a5c, #48eda4);\n  --rd-color-border: light-dark(#b7d4c8, #29493b);\n  --rd-color-code-bg: light-dark(#e7f2ec, #0b1e18);\n  --rd-color-code-text: light-dark(#07110e, #f8fffb);\n\n  --rd-header-1-color: var(--rd-color-text);\n  --rd-header-2-color: var(--rd-color-text);\n  --rd-header-3-color: var(--rd-color-text);\n  --rd-header-4-color: var(--rd-color-text);\n  --rd-header-5-color: var(--rd-color-text);\n  --rd-header-6-color: var(--rd-color-text);\n  --rd-header-1-font: var(--rd-font-heading);\n  --rd-header-2-font: var(--rd-font-heading);\n  --rd-header-3-font: var(--rd-font-heading);\n  --rd-header-4-font: var(--rd-font-heading);\n  --rd-header-5-font: var(--rd-font-heading);\n  --rd-header-6-font: var(--rd-font-heading);\n  --rd-paragraph-color: var(--rd-color-muted);\n  --rd-blockquote-color: var(--rd-color-muted);\n  --rd-blockquote-border: var(--rd-color-border);\n  --rd-list-color: var(--rd-color-muted);\n  --rd-link-color: var(--rd-color-accent);\n  --rd-link-hover-color: var(--rd-color-text);\n  --rd-code-color: var(--rd-color-code-text);\n  --rd-code-background: var(--rd-color-code-bg);\n  --rd-code-block-background: var(--rd-color-code-bg);\n  --rd-code-block-border: var(--rd-color-border);\n  --rd-table-border: var(--rd-color-border);\n  --rd-table-header-background: var(--rd-color-surface);\n  --rd-table-header-color: var(--rd-color-text);\n  --rd-thematic-break-color: var(--rd-color-border);\n  --rd-strong-color: var(--rd-color-text);\n  --rd-emphasis-color: var(--rd-color-muted);\n  --rd-strikethrough-color: var(--rd-color-muted);\n}\n.rd-document,\n.rd-document body {\n  scroll-behavior: smooth;\n}\n.rd-document:not([data-rd-color-scheme]) {\n  color-scheme: light dark;\n}\n.rd-document[data-rd-color-scheme=\"light\"] {\n  color-scheme: light;\n}\n.rd-document[data-rd-color-scheme=\"dark\"] {\n  color-scheme: dark;\n}\n\n.rd-document body {\n  margin: 0;\n  min-height: 100vh;\n  background: var(--rd-color-bg);\n  color: var(--rd-color-text);\n  font-family: var(--rd-font-body);\n  font-synthesis: none;\n}\n.rd-shell {\n  position: relative;\n  display: grid;\n  grid-template-columns: var(--rocci-nav-width, 16.5rem) minmax(0, 1fr);\n  align-items: stretch;\n  min-height: 100vh;\n}\n.rd-shell:has(> .rd-toc),\n.rd-shell:has(> #okf-toc) {\n  grid-template-columns: var(--rocci-nav-width, 16.5rem) minmax(0, 1fr) var(--rocci-outline-width, 13.5rem);\n}\n.rocci-col-resizer {\n  position: absolute;\n  top: 0;\n  bottom: 0;\n  width: 11px;\n  max-width: 11px;\n  z-index: 5;\n  cursor: col-resize;\n  touch-action: none;\n  background: transparent;\n  justify-self: start;\n  box-sizing: border-box;\n}\n.rocci-col-resizer[data-rocci-resize=\"nav\"] {\n  grid-column: 1 / 2;\n  grid-row: 1 / -1;\n  left: auto;\n  right: -5px;\n}\n.rocci-col-resizer[data-rocci-resize=\"outline\"] {\n  grid-column: -2 / -1;\n  grid-row: 1 / -1;\n  left: -5px;\n  right: auto;\n}\n.rocci-col-resizer::after {\n  content: \"\";\n  position: absolute;\n  top: 0;\n  bottom: 0;\n  left: 5px;\n  width: 1px;\n  background: transparent;\n}\n.rocci-col-resizer:hover::after,\n.rocci-col-resizer:focus-visible::after,\n.rocci-col-resizer.is-active::after {\n  background: var(--rd-color-accent);\n}\nbody.is-col-resizing {\n  user-select: none;\n}\n.rd-document main {\n  box-sizing: border-box;\n  min-width: 0;\n  width: min(42rem, calc(100% - 2rem));\n  margin: 0 auto;\n  padding: 2.5rem 0 4rem;\n}\n\n.rd-toc {\n  position: sticky;\n  top: 0;\n  box-sizing: border-box;\n  min-width: 0;\n  max-height: 100vh;\n  padding: 2.15rem 1.2rem 2rem 1.5rem;\n  overflow-x: hidden;\n  overflow-y: auto;\n  user-select: none;\n}\n.rd-toc-label {\n  margin: 0 0 0.65rem;\n  color: var(--rd-color-muted);\n  font-size: 0.68rem;\n  font-weight: 700;\n  letter-spacing: 0.105em;\n  text-transform: uppercase;\n}\n.rd-toc-items {\n  display: grid;\n  gap: 0.45rem;\n  border-left: 1px solid var(--rd-color-border);\n}\n.rd-toc-link {\n  margin-left: -1px;\n  padding-left: 0.8rem;\n  border-left: 1px solid transparent;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  line-height: 1.35;\n  text-decoration: none;\n  overflow-wrap: anywhere;\n}\n.rd-toc-link:hover,\n.rd-toc-link.is-current,\n.rd-toc-link[aria-current=\"location\"] {\n  border-color: var(--rd-color-accent);\n  color: var(--rd-color-text);\n}\n.rd-toc-link.rd-toc-level-3 {\n  padding-left: 1.35rem;\n}\n.rd-toc:not(:has(.rd-toc-link)) {\n  display: none;\n}\n\n.rd-toc-menu {\n  display: none;\n}\n\n@media (max-width: 48rem) {\n  .rd-shell {\n    display: block;\n  }\n  .rd-toc {\n    display: none;\n  }\n  .rocci-col-resizer {\n    display: none;\n  }\n  .rd-toc-menu {\n    display: block;\n    box-sizing: border-box;\n    margin: 1rem auto 0;\n    width: min(42rem, calc(100% - 2rem));\n  }\n  .rd-toc-menu summary {\n    display: flex;\n    align-items: center;\n    min-height: 2.75rem;\n    padding: 0 0.85rem;\n    border: 1px solid var(--rd-color-border);\n    border-radius: 0.5rem;\n    background: var(--rd-color-surface);\n    color: var(--rd-color-text);\n    font-size: 0.85rem;\n    font-weight: 650;\n    cursor: pointer;\n    list-style: none;\n  }\n  .rd-toc-menu summary::-webkit-details-marker {\n    display: none;\n  }\n  .rd-toc-menu .rd-toc-items {\n    margin-top: 0.65rem;\n  }\n}\n@media print {\n  .rd-toc,\n  .rd-toc-menu {\n    display: none;\n  }\n}\n@media (prefers-reduced-motion: reduce) {\n  .rd-document,\n  .rd-document body {\n    scroll-behavior: auto;\n  }\n}\n\n.rd-header-1,\n.rd-header-2,\n.rd-header-3,\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  scroll-margin-top: calc(1.25rem + var(--rd-chrome-top, 0px));\n}\n.rd-header-1:target,\n.rd-header-2:target,\n.rd-header-3:target,\n.rd-header-4:target,\n.rd-header-5:target,\n.rd-header-6:target {\n  color: var(--rd-color-accent);\n}\n.rd-header-1 {\n  color: var(--rd-header-1-color);\n  font-family: var(--rd-header-1-font);\n  margin: 0 0 0.75rem;\n  font-size: clamp(2rem, 5vw, 2.8rem);\n  letter-spacing: -0.03em;\n  line-height: 1.15;\n}\n.rd-header-2 {\n  color: var(--rd-header-2-color);\n  font-family: var(--rd-header-2-font);\n  margin: 2rem 0 0.6rem;\n  font-size: 1.35rem;\n  letter-spacing: -0.02em;\n}\n.rd-header-3 {\n  color: var(--rd-header-3-color);\n  font-family: var(--rd-header-3-font);\n  margin: 1.5rem 0 0.5rem;\n  font-size: 1.15rem;\n}\n.rd-header-4,\n.rd-header-5,\n.rd-header-6 {\n  margin: 1.25rem 0 0.4rem;\n  font-size: 1rem;\n}\n.rd-header-4 {\n  color: var(--rd-header-4-color);\n  font-family: var(--rd-header-4-font);\n}\n.rd-header-5 {\n  color: var(--rd-header-5-color);\n  font-family: var(--rd-header-5-font);\n}\n.rd-header-6 {\n  color: var(--rd-header-6-color);\n  font-family: var(--rd-header-6-font);\n}\n\n.rd-paragraph,\n.rd-list-item,\n.rd-task-item {\n  color: var(--rd-paragraph-color);\n  line-height: 1.65;\n}\n.rd-paragraph {\n  margin: 0 0 1rem;\n}\n.rd-list,\n.rd-list-ordered {\n  color: var(--rd-list-color);\n}\n.rd-link {\n  color: var(--rd-link-color);\n}\n.rd-link:hover {\n  color: var(--rd-link-hover-color);\n}\n.rd-blockquote {\n  margin: 0 0 1rem;\n  padding: 0.2rem 0 0.2rem 1rem;\n  border-left: 3px solid var(--rd-blockquote-border);\n  color: var(--rd-blockquote-color);\n}\n.rd-code-block {\n  margin: 0 0 1.25rem;\n  padding: 1rem 1.1rem;\n  overflow-x: auto;\n  border: 1px solid var(--rd-code-block-border);\n  border-radius: 0.75rem;\n  background: var(--rd-code-block-background);\n}\n.rd-code {\n  font-family: var(--rd-font-code);\n  color: var(--rd-code-color);\n  font-size: 0.92em;\n}\n:not(pre) > .rd-code {\n  padding: 0.1em 0.35em;\n  border-radius: 0.3rem;\n  background: var(--rd-code-background);\n}\n.rd-table-wrap {\n  overflow-x: auto;\n  -webkit-overflow-scrolling: touch;\n  margin: 0 0 1.25rem;\n}\n.rd-table {\n  width: max-content;\n  min-width: 100%;\n  margin: 0;\n  border-collapse: collapse;\n}\n.rd-table-header,\n.rd-table-cell {\n  padding: 0.4rem 0.6rem;\n  border: 1px solid var(--rd-table-border);\n  text-align: left;\n}\n.rd-table-header {\n  background: var(--rd-table-header-background);\n  color: var(--rd-table-header-color);\n}\n.rd-thematic-break {\n  border: 0;\n  border-top: 1px solid var(--rd-thematic-break-color);\n  margin: 1.5rem 0;\n}\n.rd-image {\n  max-width: 100%;\n  height: auto;\n}\n.rd-docs-block { margin: 1.25rem 0; }\n.rd-docs-aside {\n  padding: 0.9rem 1rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 0.6rem;\n  background: var(--rd-color-surface);\n}\n.rd-docs-note {\n  border-color: var(--rd-color-accent);\n  background: color-mix(in srgb, var(--rd-color-accent) 34%, var(--rd-color-surface));\n}\n.rd-docs-note .rd-docs-label { color: var(--rd-color-accent); }\n.rd-docs-tip {\n  border-color: #98c379;\n  background: color-mix(in srgb, #98c379 34%, var(--rd-color-surface));\n}\n.rd-docs-tip .rd-docs-label { color: #98c379; }\n.rd-docs-caution,\n.rd-docs-warning {\n  border-color: #e5c07b;\n  background: color-mix(in srgb, #e5c07b 34%, var(--rd-color-surface));\n}\n.rd-docs-caution .rd-docs-label,\n.rd-docs-warning .rd-docs-label { color: #e5c07b; }\n.rd-docs-danger {\n  border-color: #e06c75;\n  background: color-mix(in srgb, #e06c75 34%, var(--rd-color-surface));\n}\n.rd-docs-danger .rd-docs-label { color: #e06c75; }\n.rd-docs-deprecated {\n  border-color: #c678dd;\n  background: color-mix(in srgb, #c678dd 34%, var(--rd-color-surface));\n}\n.rd-docs-deprecated .rd-docs-label { color: #c678dd; }\n.rd-docs-label {\n  margin: 0 0 0.35rem;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  font-weight: 750;\n  letter-spacing: 0.04em;\n  text-transform: uppercase;\n}\n.rd-docs-title { margin: 0 0 0.4rem; font-weight: 700; }\n.rd-docs-body > :first-child { margin-top: 0; }\n.rd-docs-body > :last-child { margin-bottom: 0; }\n.rd-docs-summary { font-weight: 700; cursor: pointer; }\n.rd-docs-card {\n  display: block;\n  padding: 0.9rem 1rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 0.6rem;\n  text-decoration: none;\n  background: var(--rd-color-surface);\n}\n.rd-docs-card-title { display: block; font-weight: 700; }\n.rd-docs-card-summary { display: block; color: var(--rd-color-muted); margin-top: 0.25rem; }\n.rd-docs-card-grid {\n  display: grid;\n  gap: 0.75rem;\n  grid-template-columns: 1fr;\n}\n.rd-docs-tree { overflow-x: auto; }\n.rd-docs-tab { margin: 1rem 0; }\n.rd-docs-tab-label { margin: 0 0 0.4rem; font-size: 1rem; }\n.rd-docs-badge-label {\n  display: inline-block;\n  padding: 0.15rem 0.55rem;\n  border: 1px solid var(--rd-color-border);\n  border-radius: 999px;\n  font-size: 0.78rem;\n  font-weight: 700;\n}\n.rd-docs-steps { padding-left: 0; margin: 1.25rem 0; counter-reset: rd-step; list-style: none; }\n.rd-docs-step {\n  display: grid;\n  grid-template-columns: 1.6rem minmax(0, 1fr);\n  margin: 0.75rem 0;\n  counter-increment: rd-step;\n}\n.rd-docs-step::before {\n  content: counter(rd-step) \".\";\n  grid-column: 1;\n  grid-row: 1;\n  font-weight: 700;\n}\n.rd-docs-step > .rd-docs-verify,\n.rd-docs-step > .rd-docs-title {\n  grid-column: 2;\n  margin: 0 0 0.15rem;\n}\n.rd-docs-step:not(:has(.rd-docs-verify)) > .rd-docs-title {\n  grid-row: 1;\n}\n.rd-docs-step > .rd-docs-body {\n  grid-column: 2;\n}\n.rd-docs-verify {\n  display: inline-block;\n  margin: 0 0 0.35rem;\n  color: var(--rd-color-muted);\n  font-size: 0.78rem;\n  font-weight: 750;\n  letter-spacing: 0.04em;\n  text-transform: uppercase;\n}\n.rd-docs-figure { margin: 1.25rem 0; }\n.rd-docs-caption, .rd-docs-credit { color: var(--rd-color-muted); }\n.rd-docs-compatibility { overflow-x: auto; }\n@media (min-width: 48rem) {\n  .rd-docs-card-grid { grid-template-columns: 1fr 1fr; }\n}\n@media (forced-colors: active) {\n  .rd-docs-aside, .rd-docs-card, .rd-docs-badge-label {\n    border: 1px solid currentColor;\n  }\n}\n@media print {\n  .rd-docs-tab { break-inside: avoid; }\n  .rd-docs-aside { border: 1px solid currentColor; background: transparent; }\n}\n.rd-strong {\n  color: var(--rd-strong-color);\n}\n.rd-emphasis {\n  color: var(--rd-emphasis-color);\n}\n.rd-strikethrough {\n  color: var(--rd-strikethrough-color);\n}\n"),
                        ],
                    ),
                    Html.element(
                        "style",
                        [],
                        [
                            Html.text("@scope ([data-rocci-css~=\"Guide-fd48e9e1\"]) {\n* { box-sizing: border-box; }\n    body {\n        background:\n            radial-gradient(circle at 78% 8%, rgba(55, 242, 159, 0.13), transparent 31rem),\n            linear-gradient(145deg, var(--rd-color-bg) 0%, var(--rd-color-bg) 55%, var(--rd-color-surface) 100%);\n    }\n}"),
                        ],
                    ),
                ],
            ),
            Html.element(
                "body",
                [
                    Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                ],
                [
                    Html.element(
                        "div",
                        [
                            Html.attribute("class", "rd-shell"),
                            Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                        ],
                        [
                            Html.element(
                                "nav",
                                [
                                    Html.attribute("class", "rd-toc"),
                                    Html.attribute("aria-label", "On this page"),
                                    Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                ],
                                [
                                    Html.element(
                                        "p",
                                        [
                                            Html.attribute("class", "rd-toc-label"),
                                            Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                        ],
                                        [
                                            Html.text("On this page"),
                                        ],
                                    ),
                                    Html.element(
                                        "div",
                                        [
                                            Html.attribute("class", "rd-toc-items"),
                                            Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                        ],
                                        [
                                            Html.element(
                                                "a",
                                                [
                                                    Html.attribute("class", "rd-toc-link"),
                                                    Html.attribute("href", "#displayed-code"),
                                                    Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                                    Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                ],
                                [
                                    Html.element(
                                        "summary",
                                        [
                                            Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                        ],
                                        [
                                            Html.text("On this page"),
                                        ],
                                    ),
                                    Html.element(
                                        "div",
                                        [
                                            Html.attribute("class", "rd-toc-items"),
                                            Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                        ],
                                        [
                                            Html.element(
                                                "a",
                                                [
                                                    Html.attribute("class", "rd-toc-link"),
                                                    Html.attribute("href", "#displayed-code"),
                                                    Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
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
                                    Html.attribute("data-rocci-css", "Guide-fd48e9e1"),
                                ],
                                [
                                    rocci_content({}),
                                ],
                            ),
                        ],
                    ),
                    Html.dangerously_include_unescaped_html("<script>(function () {\n  if (window.__rocciToc) {\n    return;\n  }\n  var token = 0;\n  var pending = null;\n  var spyFrame = 0;\n\n  function tocLink(node) {\n    while (node) {\n      if (\n        node.nodeType === 1 &&\n        node.classList &&\n        (node.classList.contains(\"rd-toc-link\") || node.classList.contains(\"outline-link\"))\n      ) {\n        return node;\n      }\n      node = node.parentNode;\n    }\n    return null;\n  }\n\n  function isScrollableY(node) {\n    if (!node || node === document.body || node === document.documentElement) {\n      return false;\n    }\n    var style = window.getComputedStyle(node);\n    var overflowY = style.overflowY;\n    return (\n      (overflowY === \"auto\" || overflowY === \"scroll\" || overflowY === \"overlay\") &&\n      node.scrollHeight > node.clientHeight + 1\n    );\n  }\n\n  function scrollerFor(el) {\n    var node = el.parentElement;\n    while (node && node !== document.body && node !== document.documentElement) {\n      if (isScrollableY(node)) {\n        return node;\n      }\n      node = node.parentElement;\n    }\n    return null;\n  }\n\n  function yNow(scroller) {\n    if (scroller) {\n      return scroller.scrollTop;\n    }\n    return window.pageYOffset || document.documentElement.scrollTop || document.body.scrollTop || 0;\n  }\n\n  function ySet(scroller, y) {\n    if (scroller) {\n      scroller.scrollTop = y;\n      return;\n    }\n    var html = document.documentElement;\n    var body = document.body;\n    if (html) {\n      html.style.scrollBehavior = \"auto\";\n    }\n    if (body) {\n      body.style.scrollBehavior = \"auto\";\n    }\n    if (window.scrollTo) {\n      window.scrollTo(0, y);\n    }\n    if (html) {\n      html.scrollTop = y;\n    }\n    if (body) {\n      body.scrollTop = y;\n    }\n  }\n\n  function chromeOffset() {\n    var nav = document.querySelector(\"rocci-preview-nav\");\n    if (!nav) {\n      return 0;\n    }\n    var height = nav.getBoundingClientRect().height;\n    return height > 0 ? height : 0;\n  }\n\n  function targetY(el, scroller) {\n    var margin = parseFloat(window.getComputedStyle(el).scrollMarginTop);\n    if (isNaN(margin)) {\n      margin = 0;\n    }\n    var chrome = chromeOffset();\n    if (chrome + 8 > margin) {\n      margin = chrome + 8;\n    }\n    if (scroller) {\n      return (\n        scroller.scrollTop +\n        el.getBoundingClientRect().top -\n        scroller.getBoundingClientRect().top -\n        margin\n      );\n    }\n    return el.getBoundingClientRect().top + yNow(null) - margin;\n  }\n\n  function restorePending() {\n    if (pending) {\n      if (!pending.el.id) {\n        pending.el.id = pending.id;\n      }\n      pending = null;\n    }\n  }\n\n  function animate(scroller, to, href) {\n    var from = yNow(scroller);\n    var dist = to - from;\n    function done() {\n      ySet(scroller, to);\n      restorePending();\n      if (history.replaceState) {\n        history.replaceState(null, \"\", href);\n      }\n      syncSpy();\n    }\n    if (Math.abs(dist) < 2) {\n      done();\n      return;\n    }\n    var dur = Math.min(650, 400 + Math.abs(dist) * 0.05);\n    var start = performance.now();\n    var run = ++token;\n    function frame(now) {\n      if (run !== token) {\n        return;\n      }\n      var t = (now - start) / dur;\n      if (t >= 1) {\n        done();\n        return;\n      }\n      var k = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;\n      ySet(scroller, from + dist * k);\n      requestAnimationFrame(frame);\n    }\n    requestAnimationFrame(frame);\n  }\n\n  function tocLinks() {\n    return document.querySelectorAll(\".rd-toc-link[href^='#'], .outline-link[href^='#']\");\n  }\n\n  function headingId(href) {\n    if (!href || href.charAt(0) !== \"#\") {\n      return \"\";\n    }\n    try {\n      return decodeURIComponent(href.slice(1));\n    } catch (err) {\n      return href.slice(1);\n    }\n  }\n\n  function syncSpy() {\n    var links = tocLinks();\n    if (!links.length) {\n      return;\n    }\n    var mark = chromeOffset() + 48;\n    var currentId = \"\";\n    var firstId = \"\";\n    for (var i = 0; i < links.length; i++) {\n      var id = headingId(links[i].getAttribute(\"href\") || \"\");\n      if (!id) {\n        continue;\n      }\n      var el = document.getElementById(id);\n      if (!el) {\n        continue;\n      }\n      if (!firstId) {\n        firstId = id;\n      }\n      if (el.getBoundingClientRect().top <= mark) {\n        currentId = id;\n      }\n    }\n    if (!currentId) {\n      currentId = firstId;\n    }\n    for (var j = 0; j < links.length; j++) {\n      var on = headingId(links[j].getAttribute(\"href\") || \"\") === currentId;\n      links[j].classList.toggle(\"is-current\", on);\n      if (on) {\n        links[j].setAttribute(\"aria-current\", \"location\");\n      } else if (links[j].getAttribute(\"aria-current\") === \"location\") {\n        links[j].removeAttribute(\"aria-current\");\n      }\n    }\n  }\n\n  function requestSpy() {\n    if (spyFrame) {\n      return;\n    }\n    spyFrame = requestAnimationFrame(function () {\n      spyFrame = 0;\n      syncSpy();\n    });\n  }\n\n  function enhance() {\n    syncSpy();\n  }\n\n  document.addEventListener(\n    \"click\",\n    function (event) {\n      var link = tocLink(event.target);\n      if (!link) {\n        return;\n      }\n      var href = link.getAttribute(\"href\") || \"\";\n      if (href.charAt(0) !== \"#\") {\n        return;\n      }\n      var id = headingId(href);\n      var el = document.getElementById(id);\n      if (!el) {\n        return;\n      }\n      var scroller = scrollerFor(el);\n      var to = Math.max(0, targetY(el, scroller));\n      event.preventDefault();\n      if (event.stopImmediatePropagation) {\n        event.stopImmediatePropagation();\n      }\n      restorePending();\n      pending = { el: el, id: id };\n      el.removeAttribute(\"id\");\n      animate(scroller, to, href);\n    },\n    true\n  );\n  document.addEventListener(\"scroll\", requestSpy, true);\n  window.addEventListener(\"resize\", requestSpy);\n  if (document.readyState === \"loading\") {\n    document.addEventListener(\"DOMContentLoaded\", enhance);\n  } else {\n    enhance();\n  }\n  window.__rdTocScroll = true;\n  window.__rocciToc = { enhance: enhance };\n})();</script>"),
                ],
            ),
        ],
    )
}

on_get_guides_rocdown! = |_state, _request| {
    rocci_value = {
        rocci_page({})
    }
    Ok(rocci_value)
}
