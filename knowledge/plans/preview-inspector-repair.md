---
type: Implementation Plan
title: Investigate and repair the preview inspector
description: "Phased investigation plus repair of the shipped Dev inspector. Tabs, dock, source views, and a runtime console already exist, but source modes do not scroll, overlay dock chrome covers the tab strip, several OKF routes have no snapshot, and source is unhighlighted. Phase 1 finishes the product/window matrix in the real preview window; later phases fix shell CSS, dock chrome, theme inset, snapshot coverage, and syntax highlighting."
tags: [domain/rocci, domain/desktop, domain/runtime, domain/rocdown, domain/rocci-okf, concern/ui, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T21:40:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: old-plan
    resource: preview-inspector.md
    title: Extended preview-window inspector implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: source-plan
    resource: inspector-source-views.md
    title: Preview inspector source views
    author: process:cursor
    last_modified: 2026-08-19
  - id: research
    resource: ../research/preview-inspector.md
    title: Extended preview-window inspector research
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-nav-js
    resource: ../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview chrome host script, Dev iframe, dock, and postMessage
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-rs
    resource: ../../crates/rocci-cli/src/inspector.rs
    title: Inspector panel HTML, tabs, source pane, sibling InspectorServer
    author: process:git
    last_modified: 2026-08-19
  - id: inspect-rs
    resource: ../../crates/rocci-cli/src/inspect.rs
    title: InspectSnapshot, views, capabilities, and JSON
    author: process:git
    last_modified: 2026-08-19
  - id: metrics-panel
    resource: ../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: Inspector template whose CSS is extracted into the panel
    author: process:git
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../crates/rocci-template/src/lower.rs
    title: File CSS wrapped in @scope ([data-rocci-css~=id])
    author: process:git
    last_modified: 2026-08-18
  - id: highlight-lib
    resource: ../../crates/rocci-highlight/src/lib.rs
    title: rocci-highlight public highlight_source API
    author: process:git
    last_modified: 2026-08-17
  - id: highlight-composite
    resource: ../../crates/rocci-highlight/src/composite.rs
    title: highlight() returns no spans for Rocdown and Markdown
    author: process:git
    last_modified: 2026-08-17
  - id: highlight-token
    resource: ../../crates/rocci-highlight/src/token.rs
    title: HighlightKind css_class tok-* contract
    author: process:git
    last_modified: 2026-08-17
  - id: article-highlight
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Existing render_highlighted_code for fenced blocks
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-highlight
    resource: ../../crates/rocci-rocdown/src/highlight.rs
    title: highlight_rocdown owned by rocci-rocdown
    author: process:git
    last_modified: 2026-08-19
  - id: okf-inspect
    resource: ../../crates/rocci-okf/src/inspect.rs
    title: OKF inspect snapshot fills concepts and bundle index.md only
    author: process:git
    last_modified: 2026-08-19
  - id: okf-pages
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF pages.json includes /review/ and collection indexes
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-inspect
    resource: ../../crates/rocci-rocdown/src/inspect_snapshot.rs
    title: Rocdown site inspect snapshot from loaded pages
    author: process:git
    last_modified: 2026-08-19
  - id: standalone-rs
    resource: ../../crates/rocci-rocdown/src/standalone.rs
    title: Standalone Rocdown inspect pages with html None
    author: process:git
    last_modified: 2026-08-19
  - id: driver-rs
    resource: ../../crates/rocci-cli/src/driver.rs
    title: capture_html_from_origin skipped for --no-window
    author: process:git
    last_modified: 2026-08-19
  - id: theme-rocdown
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown sidebar height uses 100vh minus --header-height
    author: process:git
    last_modified: 2026-08-19
  - id: okf-chrome-css
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF .okf-chrome sticky max-height uses --rocci-chrome-top only
    author: process:git
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop overlay dock contract
    author: process:git
    last_modified: 2026-08-19
  - id: cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: rocci-cli Dev inspector contract
    author: process:git
    last_modified: 2026-08-19
  - id: cli-cargo
    resource: ../../crates/rocci-cli/Cargo.toml
    title: rocci-cli already depends on rocci-highlight
    author: process:git
    last_modified: 2026-08-19
  - id: browser-adapter
    resource: ../../crates/rocci-browser/src/adapter.rs
    title: Browser host inspector_url_for /__rocci/dev
    author: process:git
    last_modified: 2026-08-19
  - id: preview-rs
    resource: ../../crates/rocci-desktop/src/preview.rs
    title: Preview Navigate forwards inspector_url to overlay
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: rocci run sibling InspectorServer
    author: process:git
    last_modified: 2026-08-19
  - id: logs-rs
    resource: ../../crates/rocci-cli/src/logs.rs
    title: Runtime LogHub tee
    author: process:git
    last_modified: 2026-08-19
  - id: chrome-rs
    resource: ../../crates/rocci-desktop/src/chrome.rs
    title: Overlay asset embedding and setInspectorUrl
    author: process:git
    last_modified: 2026-08-19
  - id: playground-css
    resource: ../../playground/src/styles.css
    title: tok-* syntax colors already used by the playground
    author: process:git
    last_modified: 2026-08-18
---

# Investigate and repair the preview inspector

## Goal and scope

Treat the shipped Dev inspector as **present but not usable**, then **investigate
the remaining matrix in the real preview window** and **fix** what that
investigation plus the 2026-08-19 `--no-window` session already showed.

This plan **does not restart** inspect JSON, tabs, dock persistence keys, or
the runtime log hub. Those already exist. It **does** require a first
investigation phase so WKWebView overlay behavior is not inferred only from
Chrome against the inspector document. Syntax highlighting in Source is in
scope; it was optional polish on the earlier plans and is now a required
phase.[^old-plan][^source-plan][^cli-readme][^desktop-readme][^research]

Exploratory; Phase 1 (investigation) is the first work. Current findings below
are starting evidence, not a substitute for that phase.

## Relationship to earlier plans

The [source-views plan](inspector-source-views.md) is the inspect-JSON
contract (`source` / `ast` / `roc` / `html` plus capabilities). The
[extended inspector plan](preview-inspector.md) specified dock, tabs, Source
DX, and Console. That plan still says no phase started; the tree has already
landed those shells.[^old-plan][^source-plan][^inspector-rs][^preview-nav-js]

This record **owns remaining investigation and repair**, including highlighting
(old source-views Phase 5 / extended-inspector Phase 7). Do not implement
those leftover optional phases from the earlier records.

## Established baseline (shipped, 2026-08-19)

Verified in code and by curling `rocdown run docs --no-window` (`127.0.0.1:18765`)
and `rocci-okf run knowledge --no-window` (`127.0.0.1:18766`):

| Surface | Shipped behavior |
| --- | --- |
| Dev control | Overlay iframe to `PreviewOptions.inspector_url` |
| Panel | `GET /__rocci/dev?tab=&route=&view=` (aliases `/__rocdown/`, `/__rocci_okf/`) |
| Tabs | Performance / Source / Console in the iframe document |
| Source dropdown | GET form, values `source`, `ast`, `roc`, `html` |
| Inspect JSON | `GET /__rocci/inspect?route=` |
| Overlay sync | Tuple compare `(origin, path, tab, route)`; iframe `postMessage` `{type:"rocci-inspector",tab,view}` |
| Dock | Overlay classes `dock-right` / `dock-bottom`; CSS vars `--rocci-chrome-right` / `--rocci-chrome-bottom`; sessionStorage |
| Console | `LogHub`, `GET /__rocci/logs`, SSE `/logs/events`, `POST /logs/clear` |
| `rocci run` | Sibling `InspectorServer` on another loopback port |
| Highlighting | None in the inspector pane (`<pre><code>` escaped text only) |

`MetricsPanel.rocci` compiles as a fixture and supplies **extracted CSS
only**. Rust in `inspector.rs` emits the HTML. Overlay assets still must not
embed compiler output. `rocci-desktop` still has no language-crate
dependency.[^metrics-panel][^inspector-rs][^desktop-readme][^chrome-rs]

## Current findings (2026-08-19 investigation)

Code review plus live `--no-window` HTTP and a Chrome session against the
inspector document. Phase 1 (2026-08-19) repeated the CSS and dock rows
inside Tao/Wry (WKWebView 605.1.15) and a same-engine WKWebView probe of
the inspector document. Historical Chrome numbers below are kept; wry
confirmations follow each finding.

### F1. Source pane cannot scroll (root cause)

Chrome DevTools on
`/__rocci/dev?tab=source&route=/guides/rocci-browser/&view=roc`:

| Element | Computed | Expected from template CSS |
| --- | --- | --- |
| `.inspector-panel` (`<section>`, also `@scope` root) | `display: block`, `overflow: visible`, height **12250px** | `display: flex; flex-direction: column; height: 100%; overflow: hidden` |
| `html` / `body` | `height: 1081px; overflow: hidden` (unscoped `DOCUMENT_CSS`) | clip the overflowing panel |
| `.code-pane` | height **12106px** (content), `overflow: auto` but `clientHeight == scrollHeight` | bounded pane that scrolls |

File CSS is wrapped as `@scope ([data-rocci-css~="MetricsPanel-…"]) { … }`.
Scope-relative `.inspector-panel { … }` does **not** match the scope root
itself (it is implicitly `:scope .inspector-panel`). Descendant rules such as
`.inspector-tabs` and `.inspector-body { display: flex }` do apply. Unscoped
`html, body { overflow: hidden }` then **clips** the tall panel, so the View
dropdown appears to "do nothing" past the first screen of Roc/HTML. There is
no inner scrollbar.[^lower-rs][^metrics-panel][^inspector-rs]

This is the primary reason source modes look broken on long artifacts.

**Phase 1 wry: confirmed.** WebKit does **not** apply the scoped
`.inspector-panel` rule to the scope root. On
`/__rocci/dev?tab=source&route=/guides/rocci-browser/&view=roc` in WKWebView:
panel `display: block`, `overflow: visible`, height **11972px**; `html`/`body`
`height: 700px; overflow: hidden` (probe viewport); `.code-pane` `clientH`
11818 ≈ `scrollH` 11835 (no useful vertical scrollbar) while `clientW` 855 <
`scrollW` 1852 (horizontal scroll only). The same pattern held for docs
generated HTML, OKF generated HTML, `rocci run` Counter Roc/HTML, and
standalone `Guide.rocdown` Roc/HTML. Performance and Console descendant
bodies *do* get `display: flex; overflow: auto` (scoped descendant match).
Do not rewrite F1 before Phase 2.

### F2. Overlay dock chrome sits on the iframe

`preview-nav.js` appends absolutely positioned **R** / **B** buttons
(`top: 0; left: 8px` when docked right; `top: 8px; right: 8px` when docked
bottom) as siblings of the iframe, which is `width/height: 100%` of the dock.
Those buttons overlay the inspector **tab strip**. Phase 1 must click-test in
wry; the CSS already predicts Performance / Source (right dock) or Console
(bottom dock) are hard to hit.[^preview-nav-js]

`--no-window` has **no overlay**, so dock could not be screenshot-tested
against a live preview window in this session.

**Phase 1 wry: confirmed, with a bottom-dock nuance.** In a 1280×860 preview
of `docs/` and `knowledge/`, opening Dev and measuring overlay vs iframe
geometry: R/B sit in the iframe's top 44px tab band (`docksOverlapTabBand`).
Right dock: both **R** and **B** overlap the **Performance** tab (page-space
rects). Source and Console stay clear. Bottom dock: R/B sit at the **right**
of the tab band and did **not** cover the Console *label* at this width
(Console ends ~x=228, buttons start ~x=1201). They still intercept clicks in
the empty right of the tab strip. Overlay tabs are **not** required; Phase 3
should pad/reserve space in the overlay so in-iframe tabs stay. Dock toggle
did not need `frame.src !== next` (existing tuple compare).

### F3. Page inset vs theme `100vh`

Overlay sets `html { padding-right/bottom: var(--rocci-chrome-*) }`. Rocdown
theme sidebars use `height: calc(100vh - var(--header-height))` and ignore
`--rocci-chrome-bottom` / `--rocci-chrome-right`. OKF `.okf-chrome` already
uses `--rocci-chrome-top` for sticky `max-height`, but not the right/bottom
vars. Bottom dock in particular will leave sticky columns running **under**
the inspector.[^theme-rocdown][^okf-chrome-css][^preview-nav-js]

**Phase 1 wry: confirmed.** Right dock set `--rocci-chrome-right: 28rem`
(padding-right 448px). Bottom dock set `--rocci-chrome-bottom: 36vh`
(padding-bottom ~310px). Rocdown `.sidebar` height stayed **792px** with
bottom dock (viewport 860): taller than the padded content box (~503px), so
the sticky column runs under the inspector. OKF `.okf-chrome` `max-height`
stayed **812px** (`100vh` minus `--rocci-chrome-top` only). Home nav content
was short (~247px) so it did not visually overflow on `/`; the max-height
bug still applies on taller chrome.

### F4. Source views by product (HTTP)

`docs/` Rocdown site, all four views available on `/`,
`/guides/rocci-browser/`, and `/reference/cli/` (trailing-slash aliases
resolve). GET form switching `view=ast` on an OKF concept navigated to
`view=ast` and showed the capability reason.[^rocdown-inspect][^inspect-rs]

OKF concepts (`/`, `/plans/preview-inspector/`,
`/architecture/system-overview/`):

| View | Result |
| --- | --- |
| `source` | Markdown of the record (no `tok-*` spans) |
| `html` | Served page HTML present |
| `ast` | Unavailable: "OKF records are not Rocci or Rocdown syntax trees." |
| `roc` | Unavailable: "OKF preview does not expose a Rocci/Rocdown compiled module." |

Empty pane plus reason is **correct** for OKF AST/Roc. It still reads as a
broken mode unless Source chrome states that clearly (and highlighting does
not apply).[^okf-inspect][^inspect-rs]

**Phase 1: confirmed. Do not "fix" OKF AST/Roc unavailability.** Panel
`view=ast` / `view=roc` on a concept shows the existing reason, empty
`.code-pane`, and no `<pre>`. Rocdown site and `rocci run` Counter expose all
four views. Standalone `Guide.rocdown` inspect route is `/guides/rocdown/`
(not `/`). Dropdown `view=` is preserved on the GET form. Overlay stores
`view` in `sessionStorage` and omits it from later tuple compares.

### F5. Missing OKF inspect routes

`pages.json` lists `/review/` and collection indexes such as `/plans/`
(`plans/index.md`). `from_bundle` only pushes **concepts** (`/{id}/`) and
bundle-root `index.md` as `/`. Live inspect:

- `GET /__rocci/inspect?route=/plans/` → 404 `{error:"route not found"}`
- Panel Source: "No inspect snapshot for this route."
- Same for `/review/`

Navigating Home → Plans index or Governance & Review with Dev open therefore
looks like Source is broken.[^okf-inspect][^okf-pages]

**Phase 1: confirmed and expanded.** Live `pages.json` has 80 routes; inspect
returns 200 for 70 concept pages plus `/`. The **10** chrome routes below
404 with `{error:"route not found"}` and the panel reason "No inspect
snapshot for this route.":

`/architecture/`, `/audits/`, `/case-studies/`, `/decisions/`, `/design/`,
`/plans/`, `/reference/`, `/research/`, `/review/`, `/status/`.

Phase 5 must fill every collection `*/index.md` route and `/review/`, not
only `/plans/` and `/review/`.

### F6. No syntax highlighting

Source/Roc/HTML are escaped text in `<pre><code>`. Panel HTML contains no
`tok-keyword` (or other `tok-*`) classes. `rocci-cli` already depends on
`rocci-highlight`. `highlight()` emits spans for Roc, HTML, CSS, and Rocci;
**Rocdown and Markdown return `Vec::new()`**. Full Rocdown highlighting lives
in `rocci-rocdown` (`highlight_rocdown`); `rocci-rocdown` already depends on
`rocci-cli`, so the inspector **must not** take a reverse
`rocci-cli → rocci-rocdown` edge. Article fenced-code rendering already walks
spans into `<span class="tok-…">`.[^cli-cargo][^highlight-lib][^highlight-composite][^rocdown-highlight][^article-highlight][^highlight-token][^inspector-rs]

**Phase 1: confirmed.** Panel HTML for Rocci/Roc/HTML/Rocdown/Markdown
fixtures contained no `tok-*`. Largest inspected bodies were tens of KB
(`docs` `/reference/cli/` Roc 73957 bytes, OKF concept HTML 39278), not
multi-megabyte; Phase 6 need not add a truncate marker unless later
artifacts grow.

### F7. Other code defects to re-test

- `capture_html_from_origin` runs only when a preview **window** is opened.
  `--no-window` `rocci run` / standalone Rocdown keep HTML unavailable unless
  the snapshot was filled from disk.[^driver-rs][^standalone-rs]
- Standalone inspect pages pass `html: None` at compile; they rely on that
  capture. Not live-tested here (`datastar.js` copy failed in a sandbox
  `--no-window` run).[^standalone-rs]
- OKF generated-HTML inspect body can include preview-internal strings such
  as `reload.js` when the stored page HTML was live-reload injected or the
  theme embeds that URL. Confirm whether inspect should snapshot **disk**
  HTML before inject.
- `receivedInspectorMessage` in overlay JS is assigned and never read.
- Console on both products returned the serving tee line (`source: runtime`).
  Rebuild/watch lines exist at `logs::tee` sites but were not exercised with
  a file edit in this session.[^logs-rs]
- `rocci-browser` computes `http://127.0.0.1:{port}/__rocci/dev` and
  `Navigate` forwards `inspector_url`. Live picker switch was not
  click-tested; unit tests already assert the Navigate payload.
  [^browser-adapter][^preview-rs][^serve-rs]

**Phase 1 wry / windowed CLIs:**

- `capture_html_from_origin` **does** fill HTML when a window is opened:
  standalone `Guide.rocdown` `/guides/rocdown/` HTML 16591 (capability true);
  Counter `/` HTML 5992. `--no-window` `rocci run` / standalone **do not
  spawn** `InspectorServer` at all (`/__rocci/dev` 404 on the product
  origin). Phase 5 should pick capture-after-listen for `--no-window` if
  those hosts need inspect without a window.
- OKF inspect HTML on the live knowledge server **did** contain `reload.js`;
  Rocdown `docs/` inspect HTML did not. Keep Phase 0 gate 5: snapshot
  disk/emitted HTML, not live-reload-injected bytes.
- `receivedInspectorMessage` remains assigned and unread.
- Console: same-origin docs/OKF `/__rocci/logs` returned the serving tee
  (`source: runtime`). Sibling `InspectorServer` (Counter, standalone)
  returned `[]` — it owns a **new** `LogHub`, not the runtime tee. Rebuild
  lines were not exercised with a file edit.
- Overlay `setInspectorUrl` / Navigate forwarding remain as coded. Live
  rocci-browser product switch was not click-tested.

### F8. What already works (do not "fix" these)

- Inspect JSON keys and Rocdown site artifact fill for ordinary docs routes.
- Tab query `?tab=source` omits the profiling table; `?tab=performance`
  omits `<pre><code>`.
- Overlay tuple compare (no `frame.src !== next`).
- OKF AST/Roc **unavailability reasons** (keep; improve labeling in Source
  chrome if Phase 1 says users cannot tell "unsupported" from "failed").
- Native Web Inspector remains a separate menu.

## Investigation contract (Phase 1)

Phase 1 is **work**, not a paper freeze. Copy the findings table into this
record (or a short research addendum) with a wry/Chrome column. Do not start
highlighting or snapshot-fill changes until F1 is confirmed or replaced with
a better root cause from WKWebView.

### Matrix (required)

For each host, open Dev, then walk tabs, the four Source views, dock right,
dock bottom, resize, reload the preview window, and in-window navigation.

| Host | Target | Overlay / inspector | Source scroll | Dock R/B vs tabs | Page inset | Console | Views |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `rocdown run` window | `docs/` (`/guides/rocci-browser/`) | pass — overlay iframe same origin `/__rocci/dev` | **fail** F1: panel `display:block`, long Roc/HTML clipped by `html` overflow; horizontal pane scroll only | **fail** F2: right dock R/B cover Performance; bottom R/B in tab-band right (Console label clear at 1280px) | **fail** F3: `--rocci-chrome-*` padding set; `.sidebar` still `100vh - header` (792px under bottom dock) | pass — serving tee `source: runtime`; rebuild not file-edit tested | pass — source/ast/roc/html 200; `view` on GET form; no `tok-*` |
| `rocdown run` window | standalone `examples/rocdown/Guide.rocdown` | pass — sibling `InspectorServer` (not on product origin) | **fail** F1 same WebKit numbers on `/guides/rocdown/` Roc/HTML | same overlay as `docs/` (shared `preview-nav.js`) | n/a standalone document (no site sidebar) | **fail** sibling logs `[]` (separate `LogHub`) | pass — inspect route `/guides/rocdown/` all four views; HTML captured with window; `/` 404 |
| `rocci-okf run` window | `knowledge/` `/`, a concept, `/plans/`, `/review/` | pass — overlay same origin | **fail** F1 on concept HTML/source; AST/Roc empty+reason (not a scroll bug) | **fail** F2 same as docs | **fail** F3: `.okf-chrome` max-height 812px ignores chrome-bottom | pass — serving tee; rebuild not file-edit tested | pass on concepts; **fail** `/plans/` `/review/` and 8 other indexes (F5); no `tok-*` |
| `rocci run` window | `examples/counter` | pass — sibling origin; product origin `/__rocci/dev` 404 | **fail** F1 on Roc/HTML | same overlay | Counter app, not docs chrome | **fail** sibling logs `[]` | pass — `/` all four views; HTML captured with window |
| `rocci run` `--no-window` | `examples/counter` | **fail** — no `InspectorServer` | blocked | blocked (no overlay) | n/a | blocked | blocked |
| `rocci-browser` | switch OKF → Rocdown | **blocked** live picker; **pass** unit `navigate_event_forwards_inspector_url`; overlay `setInspectorUrl` present | same as product origin once iframe URL updates | same overlay | same as product theme | same as product origin | same as product origin |

Legend: pass = observed good; fail = defect for a later phase; blocked = could not exercise that cell.

Reload / in-window navigation: overlay tuple compare omits `view`; `sessionStorage` keeps `tab` / `view` / dock. Not re-tested with a manual window reload. Resize: dock clamp still in overlay JS; not pixel-hunted.

### Methods used (Phase 1)

- `--no-window` `curl` of `/__rocci/dev` and `/__rocci/inspect` on `docs/` (`127.0.0.1:18765`) and `knowledge/` (`127.0.0.1:18766`).
- Windowed `rocci run` / standalone `rocdown run` sibling inspectors (`InspectorServer` on loopback).
- Tao/Wry preview (`rocci-desktop::preview`) with Dev opened, dock right then bottom, geometry of R/B vs tab strip, `--rocci-chrome-*`, sidebar/`okf-chrome` heights.
- WKWebView (WebKit 605.1.15) computed styles of `.inspector-panel` / `.code-pane` on long Roc/HTML. Scoped `.inspector-panel` did **not** match the scope root; F1 stands.

## Inspector repair contract

Unchanged from the extended plan unless Phase 1 records a gate change:

- Dock right (default) and bottom only; no left / undock.
- Tabs stay in the iframe. Source views stay a dropdown, not extra tabs.
- Overlay never assigns `iframe.src` for a `view`-only change.
- Unavailable views: reason in Source chrome, empty `.code-pane`, no fake AST.
- Generated HTML is the emitted document, not the live DOM.
- Console stays **runtime** tee only. No Rocci app `log` API.
- Highlighting uses `rocci-highlight` `tok-*` spans (same classes as
  playground / Rocdown fenced code), not CodeMirror and not overlay JS.
- `rocci-cli` does not depend on `rocci-okf` or `rocci-rocdown`.

### Highlighting mapping

| Source view | Language | How |
| --- | --- | --- |
| `source` + `language=rocci` | Rocci | `rocci_highlight::highlight` in the panel |
| `source` + `language=rocdown` | Rocdown | Pre-render inner HTML when filling `InspectPage` in `rocci-rocdown` (uses `highlight_rocdown`), **or** a new optional field the panel prefers |
| `source` + `language=markdown` | Markdown / OKF | Highlight at fill time in `rocci-okf` once Markdown/frontmatter spans exist, **or** add Markdown spans in `rocci-highlight` without a `rocci-cli → rocci-rocdown` edge |
| `roc` | Roc | Panel render via `rocci-highlight` |
| `html` | HTML | Panel render via `rocci-highlight` |
| `ast` | plaintext (optional cheap sexp later) | Escaped `<pre>` is acceptable for v1 |

Cap or truncate very large Roc/HTML in the pane with an explicit marker if
later artifacts are multi-megabyte. Phase 1 bodies were tens of KB; skip the
marker in Phase 6 unless that changes. Reuse the playground `tok-*` colors
(light-dark) in inspector CSS.[^playground-css][^highlight-token][^article-highlight]

### CSS shell fix (F1)

Do **not** try to style `html, body` only through `@scope`. Keep unscoped
document CSS. Make the panel root a flex column that **is** the scope root
using `:scope` (or an inner wrapper that scoped `.inspector-panel` can
match). `.code-pane` must end up with `min-height: 0` and a height **less
than** the viewport so `overflow: auto` creates a scrollbar.

Tests: panel HTML contains the unscoped or `:scope` height/flex rules;
Chrome/`--no-window` computed `display` of the panel root is `flex` and
`.code-pane.clientHeight < .code-pane.scrollHeight` on the Rocdown generated
Roc fixture.

## Ownership

| Change | Owner |
| --- | --- |
| Investigation notes in this record | this plan |
| `:scope` / unscoped panel layout, token CSS in the panel | `rocci-cli` `inspector` + `MetricsPanel.rocci` |
| Overlay dock buttons, iframe inset, splitter vs tabs | `rocci-desktop` `preview-nav.js` spacer CSS |
| Rocdown / OKF theme `100vh` vs `--rocci-chrome-right/bottom` | `rocci-rocdown` theme and `rocci-okf` shell CSS |
| OKF inspect pages for indexes and `/review/` | `rocci-okf` `inspect` |
| Standalone HTML fill | `rocci-rocdown` / `rocci-cli` driver capture policy |
| `render_spans` HTML helper | `rocci-highlight` (extract from article) |
| Rocdown source highlighting into the snapshot | `rocci-rocdown` |
| Markdown/OKF source highlighting | `rocci-highlight` and/or `rocci-okf` |
| READMEs | `rocci-cli`, `rocci-desktop`; public docs only if Dev is described today |

## Phased implementation

### Phase 0 — freeze gates from current findings

Answer:

1. Tab strip stays in the iframe (yes; Phase 1 F2 is overlay padding, not
   overlay-owned tabs).
2. OKF AST/Roc stay unavailable (yes).
3. Highlighting is required, not optional polish.
4. Console v1 remains runtime-only. Wire sibling `InspectorServer` to the
   same runtime `LogHub` as a leftover (Phase 7), not a new log API.
5. Inspect HTML should be disk/emitted HTML, not live-reload-injected bytes
   (Phase 1: OKF inspect currently can include `reload.js`; docs site did
   not).

**Exit:** This section plus the findings table above are the working
baseline. Phase 1 may amend F1–F7 with wry evidence.

### Phase 1 — finish the investigation matrix

Execute the [Investigation contract](#investigation-contract-phase-1). Prefer
`--no-window` curls for JSON; **must** open the preview window for dock and
WKWebView CSS.

Update this record's findings (keep historical F1–F7; mark confirmed /
overturned). Do not "fix" F4 OKF AST/Roc unavailability.

**Exit:** Matrix filled. F1 confirmed in WKWebView (not rewritten). F2
click-tested in wry (right dock covers Performance; keep tabs in iframe).
Standalone Rocdown and `rocci run` sibling inspector included. `cargo test -p
rocci-cli` / `rocci-desktop` unchanged except this record.

### Phase 2 — Source DX: scrolling shell

Fix F1 only.

- Unscoped or `:scope { display: flex; height: 100%; min-height: 0;
  overflow: hidden; }` on the panel root.
- Keep `.code-pane { flex: 1 1 auto; min-height: 0; overflow: auto }` and
  `pre { overflow: visible; white-space: pre }`.
- Tests on rendered HTML + a `--no-window` curl of a long `view=roc` page:
  computed pane scrolls; tab strip stays in the iframe viewport.

**Exit:** Rocdown generated Roc and OKF generated HTML scroll inside
`.code-pane` in Chrome and in wry (Phase 1 host). Performance and Console
still scroll their own bodies. `cargo test -p rocci-cli`.

### Phase 3 — Overlay dock chrome

Fix F2 (and Phase 1 wry notes).

- Reserve space for R/B (padding on the iframe or a real inspector chrome
  strip in overlay). Buttons must not cover tabs.
- Dock toggle must not reload `iframe.src`.
- Find-in-page already uses `--rocci-chrome-right`; verify bottom dock.
- Desktop tests: spacer CSS still has both dock classes and no
  `frame.src !== next`.

**Exit:** In wry, tabs remain clickable on right and bottom. Switching dock
insets the page (subject to Phase 4 theme bugs). `cargo test -p rocci-desktop`.

### Phase 4 — Theme inset for docked Dev

Fix F3.

- Rocdown theme: subtract `--rocci-chrome-bottom` (and right, if columns use
  `100vw` / `100vh`) from sticky sidebar / outline heights, or rely on html
  padding **and** stop using raw `100vh` where it ignores that padding.
- OKF `.okf-chrome` / `.rd-shell`: same for `--rocci-chrome-right` and
  `--rocci-chrome-bottom`.
- Do not move compiler panels into overlay HTML.

**Exit:** Opening Dev on `docs/` and `knowledge/` **shrinks** article +
nav, including bottom dock. `rocci-desktop` still has no theme dependency.

### Phase 5 — Snapshot coverage

Fix F5 and F7 capture gaps Phase 1 still cares about.

- OKF: inspect pages for collection `*/index.md` routes and `/review/`
  (source from the index file or the review template; HTML from the built
  path). Cover the ten 404 routes listed under F5. Do not invent an AST.
- Standalone Rocdown: fill HTML from the served document or capture even for
  `--no-window` after listen (pick one in this phase; prefer capture once
  the origin is up, including `--no-window`).
- Tests: `rocci-okf` inspect JSON for `/plans/` and `/review/`; standalone
  fixture HTML capability true after serve.

**Exit:** Navigating those OKF chrome routes with Dev open shows Original
source (and HTML if built), not "No inspect snapshot".

### Phase 6 — Syntax highlighting

Required.

- Extract a `rocci_highlight` HTML renderer (span walk + escape) from
  `article.rs` so Rocdown fenced code and the inspector share one helper.
- Panel: highlighted inner HTML for views in the mapping table; token CSS
  in `MetricsPanel` `@css` (and unscoped if `@scope` would miss `pre code`
  colors — Phase 2's `:scope` should make descendant `tok-*` work).
- Rocdown original source: fill from `highlight_rocdown` at snapshot time
  (no new `rocci-cli → rocci-rocdown` edge).
- OKF / Markdown: add enough `rocci-highlight` spans (frontmatter keys,
  headings, fenced code) **or** highlight in `rocci-okf` with a small
  Markdown path; empty `highlight(Markdown)` today is not acceptable for
  Source.
- Tests: panel HTML for a Rocci/Roc/HTML fixture contains `tok-`; Rocdown
  source view contains `tok-` for `@page` or a heading; OKF source view
  contains at least heading or frontmatter token classes. Unavailable views
  stay empty without a giant highlighted blank.

**Exit:** Source / Roc / HTML are highlighted in the scrolling pane. AST may
stay plain. `cargo test -p rocci-highlight -p rocci-cli -p rocci-rocdown -p
rocci-okf` at the owning boundary.

### Phase 7 — Docs and leftover DX

- `rocci-cli` / `rocci-desktop` READMEs: scrolling Source, dock chrome, OKF
  unavailable AST/Roc, highlighting languages.
- Public docs only if the preview-window page describes Dev today.
- Optional if Phase 1 asked: copy current view in Source chrome; truncate
  marker for huge HTML.

**Exit:** README sentences match shipped routes, dock sides, and highlight
languages.

## Acceptance criteria

- Phase 1 matrix exists in this record (or a linked research revision) and
  covers Rocdown, OKF, `rocci run`, both docks, and all four Source views.
- Dev docks right or bottom **without covering tabs**. The inspected page is
  inset, including sticky theme columns, after Phase 4.
- Source dropdown switches original source, AST, Roc, and HTML. Long bodies
  **scroll inside `.code-pane`**. Unavailable OKF AST/Roc show the existing
  reasons, not an empty success.
- OKF `/plans/` and `/review/` (and other `pages.json` inspectable routes
  Phase 1 lists) have snapshots.
- Original source, generated Roc, and generated HTML are `tok-*`
  highlighted per the mapping table.
- Overlay still does not embed compiler output. Tests do not require a
  preview window except Phase 1's human/wry matrix.

## Decision gates

Human approval before treating these as normative:

1. Confirm F1 in WKWebView before merging a Chrome-only CSS hypothesis
   (Phase 1).
2. Overlay-owned tab strip versus padding the iframe so in-iframe tabs stay
   (recommended: keep tabs in iframe, fix F2).
3. Markdown highlighting: extend `rocci-highlight` vs OKF-only fill-time
   HTML.
4. AST highlighting: skip (recommended) vs sexp colors.
5. `--no-window` HTML capture for `rocci run` / standalone.

[^old-plan]: Original dock/tabs/console plan; leftover optional highlight phase.
[^source-plan]: Inspect JSON and dropdown contract; artifact fill shipped.
[^research]: Earlier DX research (src clobber, cover dock); shells have since landed.
[^preview-nav-js]: Overlay dock, tuple sync, R/B buttons, iframe 100% size.
[^inspector-rs]: Rust panel HTML, DOCUMENT_CSS, escaped pre, sibling server.
[^inspect-rs]: Views, capabilities, OKF unavailable strings, route normalize.
[^metrics-panel]: Scoped template CSS including `.inspector-panel` flex rules.
[^lower-rs]: `@scope ([data-rocci-css~=id])` wrapper around file CSS.
[^highlight-lib]: `highlight_source` entry.
[^highlight-composite]: Empty spans for Rocdown and Markdown.
[^highlight-token]: `tok-keyword` and related class names.
[^article-highlight]: Shared span-to-HTML walk for fenced code.
[^rocdown-highlight]: `highlight_rocdown` cannot be called from `rocci-cli`.
[^okf-inspect]: Concept routes plus `/` from `index.md` only.
[^okf-pages]: `pages.json` also has `/review/` and collection indexes.
[^rocdown-inspect]: Site snapshot fills four views when HTML is on disk.
[^standalone-rs]: Standalone pages start with `html: None`.
[^driver-rs]: HTML capture skipped for `--no-window`.
[^theme-rocdown]: `100vh` sidebar/outline heights.
[^okf-chrome-css]: `--rocci-chrome-top` used; right/bottom not.
[^desktop-readme]: Overlay docks; no compiler output in chrome.
[^cli-readme]: Tabs, GET form, Console runtime-only.
[^cli-cargo]: `rocci-highlight` already a `rocci-cli` dependency.
[^browser-adapter]: `inspector_url_for` appends `/__rocci/dev`.
[^preview-rs]: Navigate event updates overlay inspector URL.
[^serve-rs]: Sibling inspector for `rocci run`.
[^logs-rs]: Runtime log tee.
[^chrome-rs]: Initialization script and `setInspectorUrl`.
[^playground-css]: Existing `tok-*` colors to reuse.
