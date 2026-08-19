---
type: Implementation Plan
title: Extended preview-window inspector
description: "Original dock/tabs/console specification for the preview Dev inspector. Those shells are in the tree; remaining investigation and repair is the inspector-repair plan. App-level Rocci logging is out of scope."
tags: [domain/rocci, domain/desktop, domain/runtime, domain/rocdown, domain/rocci-okf, concern/ui, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T21:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/preview-inspector.md
    title: Extended preview-window inspector research
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-decision
    resource: ../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: chrome-research
    resource: ../research/desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: source-plan
    resource: inspector-source-views.md
    title: Preview inspector source views
    author: process:cursor
    last_modified: 2026-08-19
  - id: repair-plan
    resource: preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-js
    resource: ../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview chrome host script and Dev iframe
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-html
    resource: ../../crates/rocci-desktop/assets/preview-nav.html
    title: Preview chrome navigation markup
    author: process:git
    last_modified: 2026-08-19
  - id: chrome-rs
    resource: ../../crates/rocci-desktop/src/chrome.rs
    title: Preview chrome asset embedding
    author: process:git
    last_modified: 2026-08-19
  - id: preview-rs
    resource: ../../crates/rocci-desktop/src/preview.rs
    title: Preview window chrome sync and native DevTools
    author: process:git
    last_modified: 2026-08-19
  - id: window-rs
    resource: ../../crates/rocci-desktop/src/window.rs
    title: WebViewBuilder without a console handler
    author: process:git
    last_modified: 2026-08-19
  - id: history-rs
    resource: ../../crates/rocci-desktop/src/history.rs
    title: Overlay IPC command vocabulary
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-rs
    resource: ../../crates/rocci-cli/src/inspector.rs
    title: Preview inspector HTTP panel and sibling InspectorServer
    author: process:git
    last_modified: 2026-08-19
  - id: inspect-rs
    resource: ../../crates/rocci-cli/src/inspect.rs
    title: InspectSnapshot, views, and JSON
    author: process:git
    last_modified: 2026-08-19
  - id: metrics-panel
    resource: ../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: Preview-origin profiling and source-view template
    author: process:git
    last_modified: 2026-08-19
  - id: cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: rocci-cli Dev panel contract
    author: process:git
    last_modified: 2026-08-19
  - id: dev-server
    resource: ../../crates/rocci-cli/src/dev_server.rs
    title: Same-origin /__rocci/dev, live reload, CSP
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: rocci run sibling inspector
    author: process:git
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../crates/rocci-template/src/lower.rs
    title: File CSS wrapped in @scope
    author: process:git
    last_modified: 2026-08-18
  - id: okf-inspect
    resource: ../../crates/rocci-okf/src/inspect.rs
    title: OKF inspect snapshot
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-inspect
    resource: ../../crates/rocci-rocdown/src/inspect_snapshot.rs
    title: Rocdown inspect snapshot
    author: process:git
    last_modified: 2026-08-19
---

# Extended preview-window inspector

## Goal and scope

Turn the preview-window Dev panel into a **browser-like inspector
shell**: dock right or bottom (true split, not a cover), tabs for
**Performance**, **Source**, and **Console**, a Source pane whose
dropdown and scrollbars actually work, and a Console of **runtime**
messages. Do not design logging from Rocci apps.[^research][^preview-decision][^chrome-research]

This plan **supersedes remaining work** from [Preview inspector source
views](inspector-source-views.md). Inspect JSON, capabilities, overlay
`?route=` / `?view=`, and per-product artifact fill already shipped.
This plan does not replace `rocci inspect` / `rocdown inspect`, the
playground, overlay Reveal/Copy, or wry native Web Inspector.[^source-plan][^cli-readme][^preview-rs]

Dock, tabs, Source dropdown, overlay tuple sync, and the runtime log hub
are in the tree. They are not yet usable: Source does not scroll, overlay
dock buttons cover tabs, and several OKF routes have no snapshot. Remaining
investigation, repair, and syntax highlighting live in [Investigate and
repair the preview inspector](preview-inspector-repair.md). Do not start
the original Phases 1–7 on this record.[^repair-plan]

## Established baseline

Compiler-derived **content** stays on the preview HTTP origin. Overlay
chrome may open, dock, and size that panel. `rocci-desktop` stays free of
language crates.[^chrome-research][^desktop-readme]

Shipped today (see the [research record](../research/preview-inspector.md)
for the defect list):[^research][^inspector-rs][^inspect-rs][^metrics-panel][^preview-nav-js][^dev-server][^serve-rs]

- One iframe, 28rem, `position: fixed` over the right of the page (cover).
- Profiling table stacked above a GET `<select>` for source / AST / Roc /
  HTML.
- Overlay rewrites `iframe.src` on every chrome path sync; sibling
  `InspectorServer` is cross-origin so `view` cannot be read back.
- `@scope`d CSS leaves `html, body` unconstrained; `<pre>` only has
  `overflow-x`.
- No log hub. Native DevTools is a separate menu item.

## Inspector contract

### Dock (overlay)

| Side | Page inset | Panel box |
| --- | --- | --- |
| `right` (default) | `html { padding-right: var(--rocci-chrome-right) }` | `top: var(--rocci-chrome-top); right: 0; bottom: 0; width` |
| `bottom` | `html { padding-bottom: var(--rocci-chrome-bottom) }` | `left: 0; right: 0; bottom: 0; height` (below the 48px nav) |

- Drag the inner edge. Minimums: right `20rem`, bottom `8rem`. Defaults:
  right `28rem`, bottom `36vh`.
- Persist `rocci-dev-dock` (`right` \| `bottom`) and size in
  `sessionStorage`.
- Closed panel: no inset, no splitter.
- v1 does **not** include left dock or undock-to-window.
- Find-in-page and Cmd-K must not render under the panel.

### Tabs (preview-origin)

Native tablist in the inspector document, not overlay menus:

| `tab` | Label | Body |
| --- | --- | --- |
| `performance` | Performance | Existing profile total + span table |
| `source` | Source | Path, view `<select>`, capability reason, code pane |
| `console` | Console | Runtime log list (Phase 4+) |

Default first open: last used tab, else `performance`. Query:
`GET /__rocci/dev?tab=&route=&view=`. No-JS: tab links are GET anchors
or a GET form. Datastar may later refresh a tab without reload; it must
not be the only switcher.

Profiling is **not** a header on Source. Source's view dropdown is
**not** a fifth tab.

### Source views (already named)

Unchanged values: `source`, `ast`, `roc`, `html`. Unavailable views keep
the playground-style `available` + `reason`. AST text remains
`format_ast`. Generated HTML remains the emitted document, not the live
DOM.[^inspect-rs][^source-plan][^okf-inspect][^rocdown-inspect]

### Console (v1 runtime only)

A ring buffer of `{ t, level, source, text }` served as
`GET /__rocci/logs` (JSON snapshot) and `GET /__rocci/logs/events` (SSE).
`source` for v1 is `runtime`. Levels: `debug`, `info`, `warn`, `error`.
Tee existing CLI/watch/compile lines; do not format a new product log
language.

**Out of bound:** Rocci app `log` APIs, Datastar `data-log`, structured
spans beyond today's `ProfileSnapshot`, network waterfall, DOM picker.

### Query and persistence

| Key | Storage | Who writes |
| --- | --- | --- |
| panel open | `sessionStorage rocci-dev-panel` | overlay (already) |
| dock side / size | `sessionStorage rocci-dev-dock` | overlay |
| `tab` | `sessionStorage rocci-dev-tab` | overlay + iframe postMessage |
| `view` | `sessionStorage rocci-dev-view` | iframe postMessage only |
| `route` | iframe query from overlay pathname | overlay |

Overlay assigns `iframe.src` only when the canonical tuple
`(inspectorOrigin, inspectorPath, tab, route)` changes. **Never**
compare raw `iframe.src` strings. **Never** put `view` back onto `src`
except when first opening the Source tab with no postMessage yet.

Iframe posts `{ type: "rocci-inspector", tab, view }` to `parent` on
load and on user change (works for same-origin OKF/Rocdown and
cross-origin `rocci run`).[^serve-rs]

## Ownership

| Change | Owner |
| --- | --- |
| Dock, splitter, page inset, src-sync fix, find/goto offset | `rocci-desktop` overlay assets |
| Inspect JSON (unchanged shape plus optional `tab`) | `rocci-cli` `inspect` |
| Panel HTML/CSS, tabs, source pane layout, console UI | `rocci-cli` `inspector` + `templates/dev` |
| Log hub, `/__rocci/logs`, exempt or preserve inspector live reload | `rocci-cli` `dev_server` + `InspectorServer` |
| Feed runtime lines into the hub | `rocci-cli` / `rocci-rocdown-cli` / `rocci-okf` at their existing eprintln sites |
| Rocdown / OKF inspect artifacts | already owned; no fill rewrite unless a route misses |
| `format_ast` / `compiled.roc` | unchanged language crates |
| Page `console.*` wrap (Phase 5) | overlay init script + IPC prefix `log:` (or JSON) in `rocci-desktop`, forwarded to the hub |

Do not add `rocci-cli` → `rocci-okf`. Do not interpret templates in Rust
merely to avoid compiling panel CSS. Unscoped **document** CSS for
`html, body { height: 100%; overflow: hidden }` may be a static string
in `inspector.rs` because `@scope` cannot style the document
root.[^lower-rs][^inspector-rs]

## Phased implementation

### Phase 0 — freeze the shell contract

- Record dock sides, tab ids, query params, persistence keys, and the
  overlay-must-not-clobber-`view` rule above.
- Confirm Console v1 is runtime-only.
- Confirm tabs live in the iframe (research recommendation) unless a
  reviewer picks overlay tabs.
- List inspector live-reload choice (exempt vs preserve query).
- Name CSS custom properties: `--rocci-chrome-top` (exists),
  `--rocci-chrome-right`, `--rocci-chrome-bottom`.

**Exit:** This section plus decision gates 1–6 are answered. No pixel
hunting later.

### Phase 1 — Source DX: dropdown and scroll

Fix the shipped Source pane **before** adding dock or Console. Otherwise
new chrome sits on a broken tool.

Overlay (`preview-nav.js`):

- Canonicalize inspector URLs (decode `route`, stable query order:
  `tab`, `route`, `view` only when required).
- `syncFrame` compares parsed tuples, not `frame.src !== next`.
- Listen for `message` events from the iframe; update stored `tab` /
  `view`.
- Do not assign `src` when only `view` changed inside the iframe.
- Keep `route` updates on in-window navigation.

Panel (`inspector.rs` + `MetricsPanel.rocci`):

- Form `method="get"` **and** `action="/__rocci/dev"` (or the served
  path). Hidden fields for `route` and `tab`.
- After submit, postMessage the chosen `view`.
- Document CSS (unscoped): `html, body { height: 100%; margin: 0;
  overflow: hidden; }`.
- Flex column: chrome (path + select) `flex: 0 0 auto`; `.code-pane`
  `flex: 1 1 auto; min-height: 0; overflow: auto`.
- `<pre>` inside `.code-pane` with `white-space: pre; overflow: visible`
  (the pane scrolls both axes).
- Unavailable view: reason in chrome, empty pane, **no** giant blank
  `pre` that still looks like a failed switch.
- Fixture still compiles under `rocci view`.

Tests (no preview window):

- `cargo test -p rocci-cli`: form has `action`, selected option, empty
  unavailable pane, panel HTML contains `.code-pane` / height 100%
  unscoped rules.
- `cargo test -p rocci-desktop`: overlay script contains postMessage
  listener, tuple compare, and does **not** contain `frame.src !== next`.
- Existing inspect JSON tests unchanged.

**Exit:** Changing the dropdown on same-origin and on a sibling inspector
keeps the chosen view across chrome title/path sync. Long generated HTML
scrolls inside `.code-pane`; Performance/header (still stacked until
Phase 3) does not leave the iframe. `cargo test -p rocci-cli` and
`cargo test -p rocci-desktop`.

### Phase 2 — True dock right / bottom

Overlay only.

- `rocci-preview-dev` classes `dock-right` | `dock-bottom`.
- Set `--rocci-chrome-right` / `--rocci-chrome-bottom` on `html` when
  open; clear when closed.
- Splitter: overlay handle, pointer capture, clamp to mins/maxes
  (`max-width: 80vw`, `max-height: 80vh`).
- Dock toggle: two buttons on the overlay frame (not in
  `preview-nav.html` nav bar) — or a small menu on the Dev
  button.[^preview-nav-html]
  Persist side and size.
- Offset `rocci-preview-find` (`right` currently `12px` from the window
  edge) by `--rocci-chrome-right` when docked right; when docked bottom,
  keep find in the page area.
- Desktop tests: spacer CSS contains padding-right/bottom variables,
  both dock classes, splitter.

**Exit:** Opening Dev on a Rocdown or OKF preview **shrinks** the page.
Switching to bottom dock moves the iframe under the page and restores
the right inset. Reload of the preview window restores last dock.
`rocci-desktop` still has no template dependency.

### Phase 3 — Tab strip

Split the stacked Profiling+Source document.

- Inspector chrome: `role="tablist"` with three tabs; selected tab
  matches `tab` query; unknown `tab` → `performance`.
- Bodies: Performance uses today's span table; Source is Phase 1 pane
  without the profile block; Console is a placeholder
  ("Runtime log stream is not attached yet.") until Phase 4.
- Overlay persists `tab` via postMessage and includes `tab` when it
  must rebuild `src` (route change, dock does **not** rebuild src).
- `MetricsPanel.rocci` (or `InspectorPanel.rocci`) authors the tablist;
  Rust fill stays a static GET.
- Title of the iframe document: `Inspector` (not always `Profiling`).
- Tests: HTML contains three tabs; `?tab=source` omits the span table
  from the source body (or hides it); `?tab=performance` omits the
  `<pre>`; fixture compiles.

**Exit:** Switching tabs does not reset dock or `view`. Navigating the
preview with Dev open keeps the current tab and updates Source `route`.

### Phase 4 — Runtime console hub

Data plane first, then the Console body.

- `LogHub` (ring, e.g. 1000 lines) next to `ReloadHub`.
- `GET /__rocci/logs` → JSON array. `GET /__rocci/logs/events` → SSE
  `event: log`. Aliases `/__rocdown/` and `/__rocci_okf/` like Dev.
- Same routes on `InspectorServer` for `rocci run`.
- Push helper used from rebuild start/finish, compile diagnostics
  already printed, serve bind line, watch errors. Stderr remains;
  this is a tee, not a redirect.
- Console tab: no-JS snapshot table from the GET; with JS (or Datastar),
  append SSE lines. Filter chips for level. Clear empties the visible
  list (and a `POST /__rocci/logs/clear` or query flag — pick one in
  Phase 0; prefer POST on the preview origin).
- Autoscroll when the last line is visible.
- Tests: hub JSON shape; SSE content-type; InspectorServer serves logs;
  panel Console tab contains a row after a recorded line. No window.

**Exit:** `rocci-okf run` / `rocdown run` / `rocci run` with Dev →
Console shows the same class of lines the terminal already printed for
that session after the hub existed. `--no-window` `curl` of `/__rocci/logs`
works.

### Phase 5 — Page JavaScript console (optional, after 4)

Only if gate 3 says yes.

- Overlay initialization script wraps `console.*` in the **top** window,
  not in the inspector iframe (`window.frameElement` / host check).
  Wry 0.55 has no console-message builder; this wrap is the capture
  path.[^chrome-rs][^window-rs]
- IPC payload `log:` + JSON `{ level, text, args }` so it cannot collide
  with `reveal:` / `copy-source:`.[^history-rs]
- Desktop forwards to the log hub HTTP POST on the inspector origin, or
  overlay `fetch`es same-origin `/__rocci/logs` when the inspector is
  same-origin (OKF/Rocdown). Sibling inspector: desktop must POST across
  loopback (allowlist 127.0.0.1).
- Badge `page` in the Console list.
- Do not wrap `console` in a way that breaks pages that replace
  `console.log`. Keep the original functions.

Skip this phase if runtime-only Console is enough for the Dev workflow.

**Exit:** A `console.warn` from page JS (or Datastar) appears in the
Console tab with `source: page`. Overlay IPC tests parse `log:` without
breaking Reveal.

### Phase 6 — Live reload, products, docs

- Inspector HTML: exempt from `inject_live_reload`, **or** reload
  preserving `tab`/`route`/`view`. Overlay may refresh the iframe on
  rebuild SSE if exempted (same-origin EventSource in overlay is a
  bigger change; prefer exemption + existing page reload already
  remounts overlay).
- Confirm OKF AST/Roc reasons still show only on Source, not as a
  broken tab.
- `rocci-cli` README: tabs, dock, Console runtime-only.
- `rocci-desktop` README: overlay docks and insets; still does not embed
  compiler output; `tab`/`route` query; postMessage.
- Public docs only if the preview-window help page describes Dev today.
- Do not claim Chrome Elements/Network parity.

**Exit:** README sentences match shipped routes, tabs, and dock sides.

### Phase 7 — Polish (optional)

Only after Phases 1–4 are usable:

- `rocci-highlight` spans in the Source pane (old source-views Phase 5).
- Copy current view from the Source chrome (HTTP-origin, not wry IPC).
- Cap very large Roc/HTML in the pane with a "truncated" marker.
- Preserve-log checkbox on Console.
- Keyboard: overlay shortcut to toggle Dev (do not steal wry native
  DevTools).

Skip if Phase 3 already satisfies the workflow.

## Acceptance criteria

- Dev docks **right or bottom** and **insets** the inspected page. The
  page is not covered. Size and side survive reload of the preview
  window.
- Dev has tabs Performance, Source, Console. Profiling is not glued to
  the source dropdown.
- Source dropdown switches original source, AST, Roc, and HTML and
  **keeps** the selection through chrome path/title sync, live rebuilds
  of the page, and `rocci run`'s sibling inspector.
- Unavailable views show a reason in the Source chrome, not an empty
  success and not a guessed AST.
- Source/HTML/AST/Roc text scrolls inside a bounded pane; tab strip and
  view select stay visible; both overflow axes work on long lines and
  long files.
- Console lists runtime messages teed from the session (Phase 4+). No
  Rocci app log API is implied.
- Overlay assets still do not embed compiler output. `rocci-desktop`
  still does not depend on language crates.
- Native Web Inspector remains a separate menu command.
- Tests do not require a preview window for JSON, panel HTML, log hub,
  or overlay string contracts.

## Decision gates

Human approval is required before treating these exploratory choices as
normative:

1. Tab strip in the iframe (recommended) vs overlay-owned tabs.
2. v1 dock right+bottom only vs also left / undock.
3. Console v1 runtime-only vs runtime plus page `console.*` in the same
   milestone (Phase 5).
4. First-open default tab: `performance` vs `source`.
5. Source switching: keep no-JS GET form (Phase 1) vs fetch JSON into
   the pane.
6. Inspector live reload: exempt `/__rocci/dev` vs reload preserving
   query.
7. Console clear: `POST /__rocci/logs/clear` vs client-only hide.

[^research]: DX findings: src clobber, @scope scroll, cover-not-dock, console sources.
[^preview-decision]: Preview window versus preview chrome versus Dev panel naming.
[^chrome-research]: Overlay HTML versus preview-origin inspector Rocci.
[^source-plan]: Shipped inspect JSON and dropdown; remaining UX moved to the repair plan.
[^repair-plan]: Investigate-and-repair follow-on: scroll, dock chrome, OKF routes, highlighting.
[^desktop-readme]: Overlay assets; 28rem column; compiler panels on the preview origin.
[^preview-nav-js]: Dev iframe, cover layout, `frame.src` sync.
[^preview-nav-html]: Dev button only.
[^chrome-rs]: Initialization script embeds overlay assets and `inspector_url`.
[^preview-rs]: Chrome sync on load; separate wry DevTools menu.
[^window-rs]: No wry console handler.
[^history-rs]: IPC vocabulary that Phase 5 must not collide with.
[^inspector-rs]: Rust panel HTML, sibling server, form without action.
[^inspect-rs]: Views and capabilities.
[^metrics-panel]: Template for profiling plus dropdown; scoped CSS.
[^cli-readme]: Current Dev panel documentation.
[^dev-server]: Same-origin Dev, live reload, CSP.
[^serve-rs]: Cross-origin sibling inspector for `rocci run`.
[^lower-rs]: File CSS `@scope` cannot style document `html, body`.
[^okf-inspect]: OKF AST/Roc unavailable by design.
[^rocdown-inspect]: Rocdown fills all four views when HTML is built.
