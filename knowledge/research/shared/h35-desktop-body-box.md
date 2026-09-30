---
type: Research Report
title: Desktop host body box and file-scoped :scope
description: The h35-desktop split is sound. The full-viewport site bar came from bare :scope height inside file @scope; the original pin comparison is historical.
tags: [domain/rocci, domain/desktop, domain/okf, concern/architecture, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-09-23T12:55:04Z }
stale_after: 2026-12-23
authority: exploratory
owners: [human:nils]
sources:
  - id: desktop-readme
    resource: ../../../crates/rocci-desktop/README.md
    title: rocci-desktop facade contract
    author: process:git
    last_modified: 2026-09-11
  - id: site-shell
    resource: ../../../site/theme/SiteShell.rocci
    title: Rocci site shell
    author: process:git
    last_modified: 2026-09-23
  - id: rocdown-theme
    resource: ../../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown theme shell
    author: process:git
    last_modified: 2026-09-23
  - id: emitter
    resource: ../../../crates/rocci-template/src/lower/emitter.rs
    title: File CSS @scope wrapper
    author: process:git
    last_modified: 2026-09-11
  - id: rocci-pin
    resource: ../../../Cargo.toml
    title: Rocci h35-desktop pin
    author: process:git
    last_modified: 2026-09-11
  - id: okmate-desktop
    resource: ../../../../okmate/src/desktop.rs
    title: Okmate preview host call
    author: process:git
    last_modified: 2026-09-11
  - id: okmate-pin
    resource: ../../../../okmate/Cargo.toml
    title: Okmate h35-desktop pin
    author: process:git
    last_modified: 2026-09-11
  - id: inspector-plan
    resource: ../../plans/rocci/preview-inspector-repair.md
    title: Inspector repair plan, CSS shell fix
    author: process:cursor
    last_modified: 2026-08-19
  - id: host-chrome
    resource: ../rocci/desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
---

# Desktop host body box and file-scoped :scope

The h35-desktop split is the right structure. The September 2026 site
failure was a file-scoped `:scope` height rule. It is not a reason to
fold the host back into Rocci or Okmate.

## What the host owns

`h35-desktop` owns the window, the toolbar row, and the grid that
reserves the remaining body box. Product shells fill `body`. They do
not size themselves with `100vh` or `100dvh`, and they do not pad for
an overlay toolbar. Rocci reaches that host through `rocci-desktop`,
and its page scripts read `--h35-chrome-*` directly. Okmate calls
`h35_desktop::preview` itself, with goto off and tab shortcuts
on.[^desktop-readme][^okmate-desktop]

Host chrome and product pages stay separate. Richer compiler UI stays
a preview-origin panel, not markup inside the host overlay.[^host-chrome]

## Why the page broke

File `@css` is wrapped as `@scope ([data-rocci-css~="id"])`, and
lowering stamps that attribute on every element from the file. A bare
`:scope` rule therefore matches every stamped element. When specificity
ties, scoping proximity prefers the element's own scope over an
ancestor scope.[^emitter]

`SiteShell` had used `:scope, :scope body { height: 100% }` so the
document would fill the host body box. That declaration also matched
the header and the skip link. The header grew to the viewport and
pushed the article below the fold. The skip link is absolutely
positioned with the accent background, so it painted the full-height
blue bar over the wordmark. The same mistake was already called out
for the inspector: do not style `html` and `body` only through
`@scope`.[^site-shell][^inspector-plan]

The bar reproduced in Chromium on the site origin with no host script
injected. The host grid was not required to see it.

## Pins

At initial drafting, both products included the body-box commit
`0bee21a` but diverged afterward: Rocci used `57fbbec` and Okmate
used `9e38d99`. The subsequent `c465c34` merge combined those lines,
and both product pins moved there. The split audit then recommended
exact **tested compatible** revisions, with each product advancing
independently after a contract check. The current local implementation
pins the published h35 host commit `85676e8`. Host CI passed on that
commit on macOS and Linux; local product changes have no hosted CI result.
[^rocci-pin][^okmate-pin][^okmate-desktop]

## Local repair

In this working tree, not as a release, `SiteShell` limits document
height to `:scope:is(html), body`. The Rocdown theme keeps custom
properties on `:scope` and limits height the same way. A 1400×900
preview of `/` and `/docs/` showed a 68px header, an off-screen skip
link, and a scrolling docs sidebar. That check included the host grid
CSS injected beside the page.[^site-shell][^rocdown-theme]

## What not to change

Do not collapse windowing back into either product. Do not change the
host grid to paper over theme selectors. A later lowering change that
makes only the component root a scope root is a different project; the
authoring rule is enough to keep document boxes correct.[^emitter][^inspector-plan]

Follow-up work is [one host pin and an authoring rule](/plans/shared/h35-desktop-body-box.md).

[^desktop-readme]: Facade contract: fill `body`; toolbar is a layout row.
[^okmate-desktop]: `preview` with `goto: false` and `tab_shortcuts: true`.
[^host-chrome]: Overlay chrome versus preview-origin panels.
[^emitter]: `@scope ([data-rocci-css~=id])` around file CSS.
[^site-shell]: Document height is `:scope:is(html), body` in this working tree.
[^inspector-plan]: Do not style `html, body` only through `@scope`.
[^rocdown-theme]: Theme height is `:scope:is(html)` plus `body`.
[^rocci-pin]: The current Rocci manifest pins the tested local h35 commit; `57fbbec766f8272737c2c8265d15db3ba63033c2` was the original research snapshot.
[^okmate-pin]: The current Okmate manifest pins the tested local h35 commit; `9e38d9915a848b0f4f92a28a58265e8ac13a9743` was the original research snapshot.
