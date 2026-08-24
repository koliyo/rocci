---
type: Research Report
title: Extended preview-window inspector
description: "Code-backed DX investigation of the shipped Dev inspector. Dock/tabs/console shells landed; Source still does not scroll, overlay dock chrome covers tabs, and several OKF routes have no snapshot. Repair plan: investigate and repair the preview inspector."
tags: [domain/rocci, domain/desktop, domain/runtime, domain/rocdown, domain/rocci-okf, concern/ui, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-20T08:25:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: preview-decision
    resource: ../../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: chrome-research
    resource: ../desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: source-plan
    resource: ../../plans/rocci/inspector-source-views.md
    title: Preview inspector source views
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-js
    resource: ../../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview chrome host script and Dev iframe
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-html
    resource: ../../../crates/rocci-desktop/assets/preview-nav.html
    title: Preview chrome navigation markup
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-css
    resource: ../../../crates/rocci-desktop/assets/preview-nav.css
    title: Preview chrome navigation styles
    author: process:git
    last_modified: 2026-08-19
  - id: chrome-rs
    resource: ../../../crates/rocci-desktop/src/chrome.rs
    title: Preview chrome asset embedding
    author: process:git
    last_modified: 2026-08-19
  - id: preview-rs
    resource: ../../../crates/rocci-desktop/src/preview.rs
    title: Preview window entry point and chrome sync
    author: process:git
    last_modified: 2026-08-19
  - id: window-rs
    resource: ../../../crates/rocci-desktop/src/window.rs
    title: WebViewBuilder without a console handler
    author: process:git
    last_modified: 2026-08-19
  - id: history-rs
    resource: ../../../crates/rocci-desktop/src/history.rs
    title: Overlay path display and IPC commands
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-rs
    resource: ../../../crates/rocci-cli/src/inspector.rs
    title: Preview inspector HTTP panel and sibling InspectorServer
    author: process:git
    last_modified: 2026-08-19
  - id: inspect-rs
    resource: ../../../crates/rocci-cli/src/inspect.rs
    title: InspectSnapshot, views, and JSON
    author: process:git
    last_modified: 2026-08-19
  - id: metrics-panel
    resource: ../../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: Preview-origin profiling and source-view template
    author: process:git
    last_modified: 2026-08-19
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: rocci-cli Dev panel contract
    author: process:git
    last_modified: 2026-08-19
  - id: dev-server
    resource: ../../../crates/rocci-cli/src/dev_server.rs
    title: Same-origin /__rocci/dev, live reload, CSP
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: rocci run sibling inspector
    author: process:git
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../../crates/rocci-template/src/lower.rs
    title: File CSS wrapped in @scope
    author: process:git
    last_modified: 2026-08-18
  - id: okf-inspect
    resource: ../../../crates/rocci-okf/src/inspect.rs
    title: OKF inspect snapshot (no AST or Roc)
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-inspect
    resource: ../../../crates/rocci-rocdown/src/inspect_snapshot.rs
    title: Rocdown inspect snapshot from loaded site
    author: process:git
    last_modified: 2026-08-19
  - id: desktop-cargo
    resource: ../../../crates/rocci-desktop/Cargo.toml
    title: wry 0.55 with devtools feature
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-plan
    resource: ../../plans/rocci/preview-inspector.md
    title: Extended preview-window inspector implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: repair-plan
    resource: ../../plans/rocci/preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: console-scope
    resource: ../inspector-console-scope.md
    title: Preview inspector console scope
    author: process:cursor
    last_modified: 2026-08-20
---

# Extended preview-window inspector

## Scope and authority

This record is exploratory. It describes the shipped preview Dev panel,
records why the source-view dropdown and code pane feel broken, and
recommends a browser-inspired inspector shell (dock right or bottom, tabs
for Performance / Source / Console). It does not change a public language
contract.[^preview-decision][^chrome-research][^source-plan]

**App-level logging from Rocci apps is out of scope.** There is no designed
`log` API for `@on` handlers or components. v1 Console is runtime and
host-originated messages (compile, watch, serve, rebuild, preview errors)
plus, in a later phase, the inspected page's JavaScript `console.*`. It is
not a product logger. The [console-scope research](inspector-console-scope.md)
keeps that split and treats `rocci run` Roc stderr as the same runtime class,
not as component logging.[^repair-plan][^inspector-plan][^console-scope]

Implementation plan for remaining work: [Investigate and repair the
preview inspector](/plans/rocci/preview-inspector-repair.md). The original
[extended inspector](/plans/rocci/preview-inspector.md) specification is
historical relative to the shells already in tree.[^repair-plan][^inspector-plan]

## Shipped baseline

The [source-views plan](/plans/rocci/inspector-source-views.md) is no longer
"not started". Commit `feat(preview): show source, AST, Roc, and HTML in
the Dev inspector` (2026-08-19) landed the inspect snapshot, JSON, panel
dropdown, overlay `?route=` / `?view=` sync, and per-product artifact
fill. Tabs later split Performance / Source / Console, but the Source
code pane still does not scroll (scoped `.inspector-panel` rules miss the
scope root). Runtime findings and the remaining matrix are in the
[repair plan](/plans/rocci/preview-inspector-repair.md).[^source-plan][^inspector-rs][^inspect-rs][^metrics-panel][^preview-nav-js][^cli-readme][^rocdown-inspect][^repair-plan]

| Surface | Shipped behavior |
| --- | --- |
| Dev control | Overlay button; host-owned iframe to `PreviewOptions.inspector_url` |
| Panel URL | `GET /__rocci/dev` (aliases `/__rocdown/dev`, `/__rocci_okf/dev`) |
| Panel body | Rust HTML; CSS extracted from `MetricsPanel.rocci` and `@scope`d |
| Views | Native `<select>`: original source, AST, generated Roc, generated HTML |
| Inspect JSON | `GET /__rocci/inspect?route=&view=` |
| Static sites / OKF | Same-origin `DevServer` |
| `rocci run` / `view` | Cross-origin sibling `InspectorServer` on another loopback port |
| Native Web Inspector | Separate View-menu wry `open_devtools()`; not this panel |
| Overlay layout | Fixed `28rem` column, `position: fixed; right: 0; bottom: 0` |
| Page inset | `html` padding-top for the 48px nav only; **no padding-right when Dev is open** |
| Overlay Dev control markup | Button only; no tab strip or dock control |

The native window is the **preview window**. Overlay navigation is host
HTML/CSS/JS. Compiler-derived panel **content** belongs on the preview
HTTP origin as Rocci (or Rust HTML that consumes the same JSON). Overlay
may open, size, and dock that panel; it must not snapshot compiler output
into the initialization script.[^preview-decision][^chrome-research][^desktop-readme][^chrome-rs][^preview-nav-html]

`rocci-desktop` stays free of `rocci-template` and `rocci-rocdown`. Wry
0.55 is built with the `devtools` feature and has IPC, page-load, and
title handlers; it has no `with_console_handler`.[^window-rs][^desktop-cargo]

## Browser inspector patterns

Chrome DevTools, Firefox Developer Tools, and Safari Web Inspector share a
shell that Rocci can copy without copying Elements/Network/Debugger:

1. **The inspector is a docked split, not a cover.** Docking right or
   bottom **shrinks the inspected page**. Undock-to-window and left dock
   exist in all three; they are not required for a first Rocci inspector.
2. **A tab strip selects a tool.** Chrome: Elements, Console, Sources,
   Network, Performance, …. Firefox and Safari are the same idea with
   different names. Console is a first-class tab, not a footer on every
   other tool (Chrome also offers a drawer Console; Rocci does not need
   that split in v1).
3. **The tool chrome stays put; only the tool body scrolls.** Tab strip,
   dock controls, and (for Sources) the file/view toolbar stay visible.
   Long HTML or a minified bundle scrolls inside the body pane, with
   independent overflow-x and overflow-y.
4. **Dock and the last tab persist** for the window/session.
5. **Console is a multiplexed stream** with levels (verbose / info / warn /
   error), clear, and optional preserve-log across navigations. Page JS
   `console.*`, network failures, and some engine messages share one list.
   There is a separate terminal for the browser's own stdout.

Map onto Rocci:

| Browser piece | Rocci owner |
| --- | --- |
| Dock side, splitter, page inset | Overlay (`rocci-desktop` assets) — outlives navigation, talks to layout of the inspected page |
| Tab strip | Preview-origin inspector document (the iframe **is** DevTools) |
| Performance / Source bodies | Existing inspect JSON and profile snapshot |
| Console body | Preview-origin list over a host log hub |
| Native Web Inspector | Keep as the WKWebView/Chromium inspector; do not merge |

Putting tabs in the overlay would mimic Chrome's DevTools chrome more
literally, but it would grow host HTML that the chrome research says
should stay a thin HUD. Tabs inside the iframe keep compiler UI on the
preview origin. Dock **must** stay overlay because it changes `html`
padding and the iframe box.[^chrome-research][^preview-nav-js]

## DX investigation: source dropdown

The dropdown is a GET `<form>` whose `<select onchange="this.form.submit()">`
reloads the iframe. Overlay JS also writes `frame.src` whenever chrome
syncs the visible path. Those two writers fight.[^inspector-rs][^preview-nav-js][^preview-rs]

### Overlay clobbers the iframe `src`

On every `update({ path })` (page load and title sync), overlay:

1. Tries `frame.contentWindow.location.href` to remember `view`.
2. Builds `inspectorUrl?route=<pathname>&view=<stored>`.
3. Assigns `frame.src` if that string differs from `frame.src`.

Problems:

- **`iframe.src` is the last assigned attribute, not the current location
  after an in-frame form GET.** After the user picks AST, the document URL
  has `view=ast` while `frame.src` often still has `view=source`.
- **URLSearchParams encoding vs form encoding.** Overlay sets `route` via
  `URLSearchParams` (`/` may become `%2F`). The form posts the hidden
  `route` with ordinary slashes. `frame.src !== next` is then always true,
  so overlay **aborts the in-flight or completed form navigation** and
  reloads `view=source`. That is the leading "dropdown does nothing"
  failure.
- **Cross-origin sibling inspector** (`rocci run` / `view`): reading
  `contentWindow.location` throws. `rememberFrameView` is a silent
  `catch`. Stored view stays `"source"`. Every chrome sync resets the
  panel. Sibling inspectors are structurally unable to keep a chosen view
  with the current design.[^serve-rs][^inspector-rs]
- **Live reload** injects `/__rocci/reload.js` into the Dev HTML. A
  rebuild reloads **both** the page and the inspector iframe. Overlay then
  remounts and reapplies `frame.src` from sessionStorage, which may still
  be `"source"` if remember never ran.[^dev-server]

Rust panel HTML also **omits** `action="/__rocci/dev"` that
`MetricsPanel.rocci` declares. Default action is the current iframe URL,
which is usually fine, but the template and the served markup have already
drifted.[^inspector-rs][^metrics-panel]

### Product capabilities look like a broken dropdown

OKF pages have no Rocci/Rocdown tree. AST and Generated Roc are
unavailable by design, with a reason string and an empty `<pre>`. On a
knowledge preview, two of four options always "fail." That is correct
capability handling, but stacked under Profiling with no tab chrome it
reads as the control being broken.[^okf-inspect][^inspect-rs]

`rocci run` HTML is often `HTML snapshot was not captured for this
route.` unless a `@fixture` exists. Same UX trap.[^inspect-rs]

### What would make the dropdown reliable

- Overlay must **not** assign `frame.src` unless the canonical
  `(origin, path, route, tab)` tuple actually changed. Never use string
  inequality of `iframe.src` vs a freshly built URL.
- Persist `view` via `iframe → parent` `postMessage` (works cross-origin)
  rather than reading `contentWindow`.
- Keep `view` as a query param **inside** the Source tab, or switch the
  Source pane with a same-document GET/fetch that overlay does not observe.
- Always set `action="/__rocci/dev"` (or the current inspector path) so
  sibling `/` aliases cannot submit to the wrong place.
- Tests: URL canonicalization; overlay JS helpers for "should update
  src"; panel HTML includes `action`; cross-origin remember does not
  require `contentWindow`.

## DX investigation: scrollbars

The code pane is not a pane. It is a `<pre>` in a growing document.

`MetricsPanel.rocci` sets `html, body { min-height: 100%; }`,
`pre { overflow-x: auto; white-space: pre; }`, and no `overflow-y` or
`max-height`. The inspector crate extracts that CSS and wraps it in
`@scope ([data-rocci-css~="…"])`. `html, body` rules inside `@scope`
**do not match the document root** (the scope root is the
`<section data-rocci-css>`). Default body margin remains. Height is
unconstrained. The iframe document grows with the source; the iframe
window is the only vertical scroller. Profiling, the dropdown, and the
file path scroll away. Horizontal overflow lives at the bottom of a
possibly thousands-of-lines `<pre>`, which is the classic unusable
scrollbar.[^metrics-panel][^lower-rs][^inspector-rs]

The overlay iframe is `width: 100%; height: 100%` of a `28rem` column
that overlays the page. There is no `min-height: 0` flex child. Docking
to the bottom would make this worse: a short viewport plus a tall
profiling table plus source would nest scrollers incorrectly.[^preview-nav-js]

Required layout (browser Sources/Console body):

```text
html, body, .inspector { height: 100%; overflow: hidden; }
.inspector { display: flex; flex-direction: column; min-height: 0; }
.inspector-chrome { flex: 0 0 auto; }   /* tabs, view select, path */
.inspector-body { flex: 1 1 auto; min-height: 0; overflow: auto; }
```

Document-level `html, body` height/overflow **cannot** live in scoped
file CSS. Put that sheet unscoped in `render_panel_html`, or compile a
`MetricsPage` document (an `<html>` root) so file CSS is not scoped away
from `html`/`body`. Component-scoped rules may still style `.inspector`
internals.[^lower-rs][^metrics-panel]

`white-space: pre` is correct for source/AST/Roc/HTML. The **wrapper**
must scroll both axes (`overflow: auto`), not only `overflow-x` on an
unbounded `pre`.

## DX investigation: docking today

The panel is a **cover**, not a dock. `rocci-preview-dev` is
`position: fixed` over the right 28rem. `html` only gets
`--rocci-chrome-top` / `padding-top`. Page content, find-in-page, and
Cmd-K sit under the inspector. That is unlike Chrome/Firefox/Safari, and
it makes the 28rem column feel like a bug even when the dropdown
works.[^preview-nav-js][^preview-nav-css]

True dock:

- Right: `--rocci-chrome-right: <width>`; `html { padding-right }`;
  panel `top: var(--rocci-chrome-top); right: 0; bottom: 0; width: …`.
- Bottom: `--rocci-chrome-bottom: <height>`; `html { padding-bottom }`;
  panel `left: 0; right: 0; bottom: 0; height: …` (still below the 48px
  nav).
- Drag the inner edge (overlay). Persist side and size in
  `sessionStorage`.
- Shift find and go-to so they are not trapped under the panel.

Left dock and undock-to-a-second-window are browser-complete and
out of v1.

## Console

Three message classes, only two in scope for this work:

| Class | Origin | v1 |
| --- | --- | --- |
| Runtime | CLI, watch, Roc compile, static rebuild, inspector/server errors | Yes — tee stderr-style lines into a ring buffer |
| Page JS | Inspected document `console.*`, Datastar, `goto.js` | Later phase; overlay wrap + IPC; wry has no native console callback |
| App | Rocci `@on` / component `log` | **No.** Not designed |

Wry 0.55 does not expose a console-message builder method on
`WebViewBuilder`. Page JS capture has to be an initialization-script
wrap of `console.log/info/warn/error/debug` that `postMessage`s to the
existing IPC channel **without colliding** with `back` / `forward` /
`reveal:` / `copy-source:`. The wrap must skip the inspector iframe
(initialization scripts run in every frame).[^window-rs][^history-rs][^desktop-cargo]

Runtime capture does not need wry. `DevServer` already has a reload hub
and SSE `/__rocci/events`. A sibling `/__rocci/logs` SSE (plus a JSON
snapshot GET) on the same origin is the Console tab's data plane. `rocci
run`'s `InspectorServer` must grow the same routes or the Console tab is
empty for apps.[^dev-server][^serve-rs][^inspector-rs]

Console UI (preview-origin): level filter, clear, autoscroll, monotonic
timestamps, source badge (`runtime` vs later `page`). Preserve-log across
in-window navigation is a small overlay/session flag; default off like
Chrome.

Do not invent a Roc `Log` effect or Datastar `data-log` in this plan.

## Recommended architecture

Keep the existing three-layer split, with a sharper inspector-shell
boundary:[^chrome-research][^preview-decision]

```text
preview window
  overlay nav (48px)                    host HTML/JS
  inspected page                        preview origin
  overlay dock frame + splitter         host HTML/JS
    iframe /__rocci/dev?tab=&route=     preview origin Rocci/HTML
      tab strip: Performance | Source | Console
      Performance body                  ProfileSnapshot
      Source body                       view dropdown + code pane
      Console body                      EventSource /__rocci/logs
```

Query contract:

| Param | Owner | Meaning |
| --- | --- | --- |
| `tab` | iframe + overlay persist | `performance` \| `source` \| `console` |
| `route` | overlay copies pathname | current page; artifact resolution stays on origin |
| `view` | Source tab only | `source` \| `ast` \| `roc` \| `html` |

Overlay may change `tab` and `route` on the iframe URL. It must not
rewrite `view` except to restore a persisted value when **opening** the
Source tab, using postMessage as the source of truth after that.

Live reload: the inspector document should not use the same
`location.reload()` as the page, or it should reload while preserving
`tab` / `view` / scroll. Prefer a Datastar or fetch refresh of JSON
bodies in a later polish; v1 can exempt `/__rocci/dev` from
`inject_live_reload` and let overlay refresh the iframe on rebuild if
needed.

## Relationship to existing records

- [Source-views plan](/plans/rocci/inspector-source-views.md): artifact JSON
  and dropdown **shipped**.
- [Repair plan](/plans/rocci/preview-inspector-repair.md): investigation
  matrix, scroll, dock chrome, OKF routes, highlighting.
- [Extended inspector plan](/plans/rocci/preview-inspector.md): original
  dock/tabs/console specification; shells are in tree.
- [Chrome vs inspector research](desktop-host-chrome-and-inspector-ui.md):
  unchanged. Dock is overlay; tabs and tool bodies are preview-origin.
- [Preview-window naming](/decisions/preview-window.md): keep **dev
  panel**; "inspector" in this record means that panel, not wry native
  DevTools.
- Product CLIs (`rocci`, `rocdown`, `rocci-okf`) forward
  `PreviewOptions.inspector_url` into the same overlay.

## Open questions (decision gates)

Human approval before treating these as normative:

1. Tab strip inside the iframe (recommended) vs overlay-owned tabs.
2. v1 dock: right and bottom only, or also left / undock.
3. Console v1: runtime-only, or runtime plus page `console.*` in the
   same milestone. **Recommendation (2026-08-20):** runtime-only;
   [console scope](inspector-console-scope.md). Page JS stays later.
4. Default tab on first open: Performance (current primary) or Source.
5. Source view switching: keep no-JS GET form (fixed) vs fetch JSON
   into the code pane.
6. Inspector live reload: exempt, or reload preserving query.

## Disposition

Draft and exploratory. The source-view **capability** is in tree; the
source-view **UX** is not. Docking, tabs, scroll, and Console are new
work on top of that snapshot, not a rewrite of `format_ast` or inspect
JSON.

[^preview-decision]: Preview window versus preview chrome versus Dev panel naming.
[^chrome-research]: Overlay HTML versus preview-origin inspector Rocci.
[^source-plan]: Original dropdown-and-JSON plan; implementation landed, UX incomplete.
[^desktop-readme]: Overlay assets; compiler panels on the preview origin; 28rem column.
[^preview-nav-js]: Dev iframe, 28rem cover, sessionStorage, `frame.src` sync, no page inset.
[^preview-nav-html]: Dev button only; no tab strip or dock control.
[^preview-nav-css]: Nav bar rules; Dev box is in the overlay JS spacer string.
[^chrome-rs]: Initialization script embeds overlay assets and `inspector_url`.
[^preview-rs]: `sync_chrome` on loads; wry native DevTools is a separate menu item.
[^window-rs]: `WebViewBuilder` IPC and page-load hooks; no console handler.
[^history-rs]: IPC verbs `back`/`forward`/`home`/`reload`/`reveal:`/`copy-source:`.
[^inspector-rs]: Rust-rendered panel, missing form `action`, sibling server.
[^inspect-rs]: Views, capabilities, OKF unavailable reasons, HTML capture gaps.
[^metrics-panel]: Template dropdown, `pre { overflow-x: auto }`, stacked profiling.
[^cli-readme]: Documents profiling plus source views at `/__rocci/dev`.
[^dev-server]: Same-origin Dev, inspect JSON, live-reload injection, CSP.
[^serve-rs]: `rocci run` sibling inspector from a snapshot; other loopback port.
[^lower-rs]: File CSS wrapped in `@scope ([data-rocci-css~=id])`.
[^okf-inspect]: OKF pages: source and HTML only; AST/Roc unavailable.
[^rocdown-inspect]: Rocdown pages fill source, AST, Roc, and built HTML.
[^desktop-cargo]: wry 0.55 with `devtools`.
[^inspector-plan]: Original dock/tabs/console specification.
[^repair-plan]: Investigate-and-repair plan; includes 2026-08-19 findings.
[^console-scope]: Runtime-only Console; reject @component logs; feed rocci run stderr.
