---
type: Implementation Plan
title: Preview inspector source views
description: "Extend the preview-window Dev panel beyond profiling so it can show original source, formatted AST, generated Roc, or generated HTML for the current page, selected with a dropdown. Artifact JSON and the dropdown shipped; remaining UX is the inspector repair plan."
tags: [domain/rocci, domain/desktop, domain/runtime, domain/rocdown, domain/rocci-okf, concern/ui, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T21:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
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
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-rs
    resource: ../../crates/rocci-cli/src/inspector.rs
    title: Preview inspector HTTP panel and profile JSON
    author: process:git
    last_modified: 2026-08-18
  - id: metrics-panel
    resource: ../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: Preview-origin profiling inspector template
    author: process:git
    last_modified: 2026-08-18
  - id: cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: rocci-cli contract
    author: process:git
    last_modified: 2026-08-18
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
    title: Preview window entry point
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: rocci run window plus sibling inspector
    author: process:git
    last_modified: 2026-08-19
  - id: run-rs
    resource: ../../crates/rocci-cli/src/run.rs
    title: rocci run compile and preview
    author: process:git
    last_modified: 2026-08-19
  - id: inspect-cli
    resource: ../../crates/rocci-cli/src/main.rs
    title: rocci inspect AST and generated Roc
    author: process:git
    last_modified: 2026-08-18
  - id: inspect-rocdown
    resource: ../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown inspect ast and roc
    author: process:git
    last_modified: 2026-08-19
  - id: pprint-template
    resource: ../../crates/rocci-template/src/pprint.rs
    title: Rocci format_ast
    author: process:git
    last_modified: 2026-08-15
  - id: pprint-rocdown
    resource: ../../crates/rocci-rocdown/src/pprint.rs
    title: Rocdown format_ast
    author: process:git
    last_modified: 2026-08-19
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci template crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Rocdown format and inspect contract
    author: process:git
    last_modified: 2026-08-19
  - id: playground-app
    resource: ../../playground/src/app.ts
    title: Playground source and output-mode dropdown
    author: process:git
    last_modified: 2026-08-18
  - id: playground-compile
    resource: ../../crates/rocci-cli/src/playground_compile.rs
    title: Local playground compile JSON
    author: process:git
    last_modified: 2026-08-18
  - id: playground-html
    resource: ../../crates/rocci-cli/src/playground_html.rs
    title: Html.render snapshot for playground local mode
    author: process:git
    last_modified: 2026-08-19
  - id: source-rs
    resource: ../../crates/rocci-desktop/src/source.rs
    title: Overlay reveal and copy of original source
    author: process:git
    last_modified: 2026-08-19
  - id: pages-json
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: pages.json route and source path index
    author: process:git
    last_modified: 2026-08-19
  - id: preview-goto
    resource: ../../crates/rocci-desktop/assets/preview-goto.js
    title: Go-to-file catalog from pages.json
    author: process:git
    last_modified: 2026-08-19
  - id: dev-server
    resource: ../../crates/rocci-cli/src/dev_server.rs
    title: Shared static preview origin and /__rocci/dev
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run preview options
    author: process:git
    last_modified: 2026-08-19
  - id: highlight-readme
    resource: ../../crates/rocci-highlight/README.md
    title: rocci-highlight span contract
    author: process:git
    last_modified: 2026-08-17
  - id: inspector-plan
    resource: preview-inspector.md
    title: Extended preview-window inspector
    author: process:cursor
    last_modified: 2026-08-19
  - id: repair-plan
    resource: preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-19
---

# Preview inspector source views

Inspect JSON, capabilities, the Source dropdown, overlay `?route=` /
`?view=` sync, and per-product artifact fill shipped on 2026-08-19
(`feat(preview): show source, AST, Roc, and HTML in the Dev inspector`).
Remaining Dev-panel UX — scrolling Source, dock chrome that does not cover
tabs, OKF route coverage, and syntax highlighting — lives in
[Investigate and repair the preview inspector](preview-inspector-repair.md).
The [extended inspector](preview-inspector.md) plan is the original dock/tabs
contract; those shells shipped. This record is the artifact-and-dropdown
contract those routes still follow.[^inspector-plan][^repair-plan]

## Goal and scope

Extend the preview-window Dev panel so a maintainer can read the current
page as original source, formatted AST, generated Roc, or generated HTML,
chosen with a dropdown, without leaving the preview window.[^preview-decision][^chrome-research]

This plan covers host JSON for those artifacts, a preview-origin Rocci
panel, overlay wiring that points the iframe at the current route, and
product-specific capability flags for `rocci run` / `view`, `rocdown run`,
and `rocci-okf run`. It does not replace `rocci inspect` / `rocdown inspect`,
the playground workbench, overlay Reveal/Copy, or the native web inspector.[^inspect-cli][^inspect-rocdown][^playground-app][^source-rs][^desktop-readme]

Artifact and dropdown work landed in tree. Treat this record as the
inspect-JSON contract; do not restart Phases 1–4. Exploratory remaining
UX is not this plan.

## Established baseline

The native window is the **preview window**. Overlay navigation is host
HTML/CSS/JS. Compiler-derived panels belong on the preview HTTP origin as
Rocci that consumes host JSON. Overlay chrome may open that panel; it must
not snapshot the panel into the initialization script.[^preview-decision][^chrome-research][^desktop-readme]

Shipped today:

| Surface | Behavior |
| --- | --- |
| Dev control | Hidden until `PreviewOptions.inspector_url` is set; toggles a host-owned iframe |
| Panel URL | `GET /__rocci/dev` (aliases `/__rocdown/dev`, `/__rocci_okf/dev`) |
| Panel body | Rust HTML plus CSS extracted from `MetricsPanel.rocci`; timings only |
| Profile JSON | `GET /__rocci/profile` from `ProfileSnapshot` |
| `rocci run` / `view` | Sibling loopback `InspectorServer` holding only a profile snapshot |
| Static sites | Same-origin DevServer serves the panel from the last rebuild profile |
| CLI inspect | `format_ast` S-expression plus generated Roc (and segments on Rocci) |
| Playground | Editable source plus output dropdown `roc` / `AST` / `html` |
| Overlay More | Reveal and copy original source when `source_root` is set |

The Dev iframe is a fixed 320px right column. Overlay already tracks the
visible path and can resolve a catalog row from `/pages.json` or
`/catalog.json`. The overlay markup is a Dev toggle only;
`PreviewOptions.inspector_url` supplies the iframe
URL.[^preview-nav-js][^preview-nav-html][^preview-rs][^inspector-rs][^metrics-panel][^cli-readme][^serve-rs][^dev-server][^inspect-cli][^playground-app][^pages-json][^preview-goto]

`rocci-desktop` stays free of `rocci-template` and `rocci-rocdown`. Do not
author this panel as overlay chrome.[^desktop-readme][^chrome-rs]

## Source-view contract

Four representations, one native `<select>` (not tabs, not overlay
menus):

| Value | Label | Meaning |
| --- | --- | --- |
| `source` | Original source | Bytes of the authored file for the current route |
| `ast` | AST | Existing `format_ast` S-expression from that file's compiler |
| `roc` | Generated Roc | `compiled.roc` (or the page/module equivalent) |
| `html` | Generated HTML | The HTML the compiler or site builder emitted for that route |

Default view is `source`. Profiling stays in the same panel as a compact
header, not as a fifth dropdown option. Missing representations use the
playground-style capability pattern: `available` plus a `reason`, empty
body, no fake tree. AST text is the same `format_ast` dump as `rocci
inspect --ast` and `rocdown inspect ast`.[^playground-compile][^pprint-template][^pprint-rocdown][^template-readme][^rocdown-readme]

**Generated HTML** is the emitted document string, not the live webview
DOM and not a Datastar SSE patch. For a static site that is the built
file. For `rocci run` that is the last successful document `Html.render`
for that GET route when captured, otherwise a fixture/default snapshot
with an explicit reason. For OKF that is the served page HTML.[^playground-html][^run-rs]

**Current page** is the webview pathname. Overlay JS may only append
`?route=` (and preserve `view`) on the iframe URL. Artifact resolution
stays on the preview origin. Reuse `/pages.json` `path` when present;
do not parse in `rocci-desktop`.[^pages-json][^preview-goto][^chrome-research]

No-JS in the iframe: the dropdown is a GET form to `/__rocci/dev`. Changing
the select submits. Datastar may later refresh without a full reload; it
must not be the only way to switch views.

## Ownership

| Change | Owner |
| --- | --- |
| Inspect JSON, snapshot store, panel HTML, `MetricsPanel` / inspector template | `rocci-cli` |
| Shared static `/__rocci/dev` and `/__rocci/inspect` | `rocci-cli` `dev_server` |
| Sibling inspector for `rocci run` / `view` | `rocci-cli` `inspector` + `serve` |
| Rocdown compile artifacts during site/document rebuild | `rocci-rocdown` (data) via `rocci-rocdown-cli` |
| OKF source path and served HTML | `rocci-okf` |
| Iframe query sync, panel width | `rocci-desktop` overlay assets |
| `format_ast` / `compiled.roc` | `rocci-template` and `rocci-rocdown` (unchanged public dump) |

Do not add a `rocci-cli` → `rocci-okf` edge. Do not interpret templates in
Rust merely to avoid compiling the panel CSS. Do not depend on
`rocci-playground` from the inspector; copy the capability idea, not the
workbench protocol.

## Phased implementation

### Phase 0 — freeze the contract

- Record the dropdown values, default, profiling placement, and HTML
  definition above.
- Name the JSON route `GET /__rocci/inspect?route=&view=` (profile stays
  on `/__rocci/profile`).
- Decide panel width (recommended: at least 28rem when open; `pre`
  scrolls horizontally). Today's 320px column is too narrow for Roc.
- List unavailable reasons per product (OKF has no Rocci/Rocdown AST;
  WASM-unrelated here; live apps may lack a static HTML snapshot).
- Confirm overlay may change iframe `src` query only.

**Exit:** This section plus decision gates 1–4 are answered. No pixel
hunting later.

### Phase 1 — inspect snapshot and JSON

Replace the profile-only store with an inspect snapshot that still
includes `ProfileSnapshot`:

```text
route, path, language,
source, ast, roc, html,
capabilities { source, ast, roc, html },
profile
```

- Serve JSON from `InspectorServer` and from `DevServer` (`ServeTarget`
  alongside `Dev` and `Profile`).
- Resolve `route` against the last rebuild's page index, the entry
  `.rocci` module, or a documented fallback (`/` → entry file).
- Escape nothing in JSON; escape in HTML later.
- Tests in `rocci-cli` with no window: JSON keys, capability reasons,
  404/empty route, HTML special characters in source.

**Exit:** `cargo test -p rocci-cli` covers the JSON. `curl` of a
`--no-window` static server returns inspect JSON after a rebuild.

### Phase 2 — panel dropdown and code pane

Extend `templates/dev/MetricsPanel.rocci` (or a sibling `InspectorPanel`)
with a labeled `<select>` and a `<pre><code>` of the selected text. Keep
extracting CSS the way the profiling panel already does; Rust fills the
body from the snapshot so the iframe stays a static GET, matching today's
panel.[^metrics-panel][^inspector-rs]

- Form GET: `view` and `route` query params.
- Selected option matches `view`; unknown `view` falls back to `source`.
- Unavailable view: reason paragraph, empty `pre`.
- Include the current file path as plain text above the code.
- Fixture for `rocci view` of the panel template.

**Exit:** Panel HTML contains the four options and the selected body.
Existing span-table assertions still pass. `rocci view` of the fixture
still compiles.

### Phase 3 — bind the iframe to the current route

In overlay JS, when the Dev panel is open, set the iframe URL to
`inspectorUrl` plus `route=<pathname>` and the last chosen `view` stored
in `sessionStorage` (alongside `rocci-dev-panel`). Update on path
changes the chrome already receives. Cross-origin sibling inspectors
(`rocci run`) still work because the overlay owns the iframe `src`.[^preview-nav-js][^serve-rs]

Widen the `rocci-preview-dev` rule per Phase 0. Do not put the dropdown
in `preview-nav.html`.

**Exit:** Navigating a Rocdown preview with Dev open reloads the iframe
for the new route. Desktop tests still assert the Dev iframe exists when
`inspector_url` is set. `rocci-desktop` still has no template dependency.

### Phase 4 — fill artifacts per product

Stash artifacts during the compile/rebuild the server already does; do
not re-parse on every inspect GET unless the cache missed.

| Product | `source` | `ast` | `roc` | `html` |
| --- | --- | --- | --- | --- |
| `rocci run` / `view` / standalone `.rocci` | Entry (and later, mapped) `.rocci` | `rocci_template::format_ast` | `compiled.roc` | Captured document GET or playground-style snapshot + reason |
| `rocdown run` document or site | `pages.json` `path` | `rocci_rocdown::format_ast` | document/site `compiled.roc` | Built HTML for that route |
| `rocci-okf run` | Knowledge `.md` | unavailable (not a Rocci/Rocdown tree) | unavailable or whole-bundle chrome Roc with a reason | Served page HTML |

Watch rebuilds replace the snapshot. Failed rebuilds keep the previous
inspect tree, same as failed static output. Directory `rocci run` with
`main.roc` may start as entry-file-only; a file picker is out of scope
until a route→module map exists.

Set `source_root` on `rocci-okf run` only if overlay Reveal should work
there; inspect itself must not require it.[^okf-main][^source-rs]

**Exit:** `--no-window` inspect JSON for `examples/counter`, a Rocdown
fixture, and a knowledge concept each matches the table. `cargo test -p
rocci-cli`, `cargo test -p rocci-rocdown-cli` (or rocdown inspect reuse),
and `cargo test -p rocci-okf` cover the new payload at the owning
boundary.

### Phase 5 — highlighting and copy (optional)

Only if Phase 2 is usable as plain `pre`:

- Color with `rocci-highlight` spans in the panel HTML, not CodeMirror.[^highlight-readme]
- Copy-selected-view button in the panel (HTTP-origin, not wry IPC).
- Cap or stream very large Roc (OKF chrome modules) rather than hanging
  the iframe.

Skip this phase if Phase 2 already satisfies the Dev workflow.

### Phase 6 — docs

- `rocci-cli` README: Dev panel shows profiling plus source views.
- `rocci-desktop` README: overlay still only toggles the iframe; mention
  `route` query and width.
- Public docs only if the preview-window help page describes Dev today.
- Do not claim playground parity (no editor, no WASM compile).

**Exit:** README sentences match shipped routes and views.

## Acceptance criteria

- Opening Dev in `rocci run`, `rocdown run`, and `rocci-okf run` shows
  profiling and a dropdown with Original source, AST, Generated Roc, and
  Generated HTML.
- Switching the dropdown changes the code pane without opening a second
  window or the native web inspector.
- The pane follows the current preview route after in-window navigation.
- Unavailable views show a reason, not an empty success or a guessed AST.
- AST and Roc text match `rocci inspect --ast` / `rocdown inspect ast|roc`
  for the same file, modulo trailing headings the CLI adds.
- Generated HTML matches the built or `Html.render` document for that
  route, not a live DOM dump.
- Overlay assets still do not embed compiler output. `rocci-desktop`
  still does not depend on language crates.
- Failed site/OKF rebuilds keep the previous inspect snapshot.
- Tests do not require a preview window. `cargo test -p rocci-cli` covers
  JSON and panel HTML; product crates cover their artifact filling.

## Decision gates

Human approval is required before treating these exploratory choices as
normative:

1. Keep profiling as a header above the code dropdown, or make Profiling
   a fifth select option.
2. Dev column width: `28rem`, `40vw`, or a drag handle (handle is overlay
   chrome if it outlives navigation).
3. `rocci run` HTML: captured response, fixture snapshot only, or
   unavailable until a later capture hook.
4. OKF `roc`: hide as unavailable, or show the bundle chrome module with
   a size warning.
5. Highlight in Phase 5 versus ship monospace `pre` only.

[^preview-decision]: Preview window versus preview chrome versus Dev panel naming.
[^chrome-research]: Overlay HTML versus preview-origin inspector Rocci.
[^desktop-readme]: Overlay assets; compiler panels on the preview origin.
[^inspector-rs]: Profile-only panel HTML, JSON, and sibling `InspectorServer`.
[^metrics-panel]: Profiling-only Rocci template whose CSS the inspector extracts.
[^cli-readme]: Dev panel served at `/__rocci/dev` from `ProfileSnapshot`.
[^preview-nav-js]: Dev iframe, 320px column, sessionStorage open flag, path display.
[^preview-nav-html]: Dev button only; no source dropdown in overlay markup.
[^chrome-rs]: Initialization script embeds overlay assets and `inspector_url`.
[^preview-rs]: `PreviewOptions.inspector_url` and `source_root`.
[^serve-rs]: `rocci run` sibling inspector from a profile snapshot.
[^run-rs]: Standalone and directory compile that already holds `compiled.roc`.
[^inspect-cli]: CLI dump of AST, generated Roc, and source-map segments.
[^inspect-rocdown]: `rocdown inspect ast` and `inspect roc`.
[^pprint-template]: Rocci `format_ast`.
[^pprint-rocdown]: Rocdown `format_ast`.
[^template-readme]: `inspect --ast` prints parse tree plus generated Roc.
[^rocdown-readme]: `rocdown inspect ast FILE.rocdown`.
[^playground-app]: Output-mode `<select>` for roc / AST / html.
[^playground-compile]: JSON capabilities for roc, ast, and html.
[^playground-html]: Html.render snapshot rules and unavailable reason.
[^source-rs]: Overlay reveal/copy; not an inspector pane.
[^pages-json]: `pages.json` entries include `route` and source `path`.
[^preview-goto]: Catalog rows used to map the current URL to a source path.
[^dev-server]: Same-origin `/__rocci/dev` for static Rocdown and OKF preview.
[^okf-main]: OKF preview sets `inspector_url` and does not set `source_root`.
[^highlight-readme]: Token spans for Roc, HTML, Rocci, and Rocdown.
[^inspector-plan]: Original dock/tabs/console contract; remaining repair is preview-inspector-repair.md.
[^repair-plan]: Investigation matrix, scroll fix, dock chrome, OKF routes, and tok-* highlighting.
