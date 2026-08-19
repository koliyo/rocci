---
type: Implementation Plan
title: Dedicated rocci-browser CLI and desktop host
description: "Phased delivery of a product-blind project browser: registry of directories, two-stage fuzzy picker (Enter opens a target, Tab lists documents), persistent preview window, and out-of-process adapters that exec existing run --no-window servers. Complements the three product CLIs; does not add plugins on rocci or rocdown."
tags: [domain/rocci, domain/desktop, domain/rocci-okf, domain/rocdown, concern/architecture, concern/tooling, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-19T22:30:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/rocci-browser.md
    title: Dedicated rocci-browser CLI and desktop host research
    author: process:cursor
    last_modified: 2026-08-19
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points for Rocci, Rocdown, and OKF preview
    author: process:cursor
    last_modified: 2026-08-18
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-18
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
  - id: system-overview
    resource: ../architecture/system-overview.md
    title: Current Rocci system overview
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: deps-check
    resource: ../../scripts/check-workspace-deps.py
    title: Mechanical one-way workspace dependency check
    author: process:cursor
    last_modified: 2026-08-19
  - id: agents
    resource: ../../AGENTS.md
    title: Workspace ownership and CLASSES rule
    author: process:git
    last_modified: 2026-08-18
  - id: cargo-toml
    resource: ../../Cargo.toml
    title: Workspace members
    author: process:git
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: preview-rs
    resource: ../../crates/rocci-desktop/src/preview.rs
    title: Preview window entry point and webview load_url
    author: process:git
    last_modified: 2026-08-19
  - id: desktop-lib
    resource: ../../crates/rocci-desktop/src/lib.rs
    title: Persistent multi-window desktop shell
    author: process:git
    last_modified: 2026-08-19
  - id: backend
    resource: ../../crates/rocci-core/src/backend.rs
    title: RunningBackend and ExternalBackend
    author: process:git
    last_modified: 2026-08-13
  - id: window-state
    resource: ../../crates/rocci-desktop/src/state.rs
    title: Persistent window geometry under ~/.rocci/state
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: Shared preview-window helper used by product CLIs
    author: process:git
    last_modified: 2026-08-19
  - id: browse-rs
    resource: ../../crates/rocci-cli/src/browse.rs
    title: rocci browse component gallery
    author: process:git
    last_modified: 2026-08-17
  - id: cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: rocci CLI contract including browse and view
    author: process:git
    last_modified: 2026-08-19
  - id: path-hint
    resource: ../../crates/rocci-cli/src/path_hint.rs
    title: Boundary-safe OKF Markdown sniff
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run preview options
    author: process:git
    last_modified: 2026-08-19
  - id: okf-preview
    resource: ../../crates/okf/src/preview.rs
    title: OKF preview path resolution
    author: process:cursor
    last_modified: 2026-08-18
  - id: rocdown-cli
    resource: ../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown run file and site dispatch
    author: process:git
    last_modified: 2026-08-19
  - id: pages-json
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: pages.json route and source path index
    author: process:git
    last_modified: 2026-08-19
  - id: goto-js
    resource: ../../crates/rocci-ui/assets/goto.js
    title: Shared go-to-page palette and fuzzy scoring
    author: process:cursor
    last_modified: 2026-08-19
  - id: fuzzy-plan
    resource: fuzzy-navigation.md
    title: Cmd-K fuzzy navigation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: inspector-plan
    resource: inspector-source-views.md
    title: Preview inspector source views
    author: process:cursor
    last_modified: 2026-08-19
  - id: known-limitations
    resource: ../status/known-limitations.md
    title: Known Rocci limitations
    author: process:okf-phase-6
    last_modified: 2026-08-19
  - id: site-config
    resource: ../../site/rocdown.toml
    title: rocci.dev site configuration mounting docs
    author: process:git
    last_modified: 2026-08-19
  - id: docs-config
    resource: ../../docs/rocdown.toml
    title: Standalone docs site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: root-readme
    resource: ../../README.md
    title: Rocci workspace overview and CLI surface
    author: human:nils
    last_modified: 2026-08-19
  - id: compile-research
    resource: ../research/okf-compile-render-cost.md
    title: OKF preview compile and render cost
    author: process:cursor
    last_modified: 2026-08-19
  - id: lsp-spec
    resource: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/
    title: Language Server Protocol specification
    author: organization:microsoft
---

# Dedicated rocci-browser CLI and desktop host

## Purpose and authority

This is the implementation plan for the [rocci-browser
research](../research/rocci-browser.md). Gates 1–2 were accepted with an
explicit request to implement from Phase 1. Architecture records and the three
product CLIs remain the current one-shot preview contract until later
phases.[^research][^system-overview][^root-readme]

Phase 0 is this freeze. Phases 1–5 are implemented in this revision; they are
not logged complete until required GitHub workflows succeed.

This plan does **not** reverse the CLI-entry-points recommendation that `rocci`
and `rocdown` must not grow a plugin lifecycle for first-party format
dispatch.[^cli-plan] A host whose job is session, window, and project selection
is a different product from teaching `rocci run` to compile Rocdown or OKF.

## Goal

Give authors one persistent process that:

1. Registers project directories once.
2. Fuzzy-picks a **target**.
3. **Enter** opens that target as a whole.
4. **Tab** drills into adapter-supplied **documents**, then Enter opens one.

The host never names Rocci apps, Rocdown, or OKF. First-party products
participate as PATH adapters. Direct `rocci run` / `rocdown run` /
`rocci-okf run` keep today's one-shot preview windows.[^research][^cli-plan]

## Frozen architecture (option D)

The research compared five shapes. This plan implements only **D**: a
product-blind host plus out-of-process adapters.[^research]

| Option | Disposition |
| --- | --- |
| A. Keep three `run` windows; hints and aliases | Keep as the one-shot path. Insufficient as the only answer. |
| B. Plugin lifecycle on `rocci` or `rocdown` | Reject. Reopens the frozen format boundary.[^cli-plan][^deps-check] |
| C. Composition binary that statically links three adapters | Reject. The host would know the products at compile time. |
| D. Product-blind host plus stdio adapters | **This plan.** |
| E. dlopen / Wasm plugins in the host | Defer. Revisit only if a third-party adapter cannot ship a binary. |

### Layer split

| Layer | Owns | Must not own |
| --- | --- | --- |
| `rocci-browser` host | Project registry, two-stage picker chrome, persistent window(s), session table, plugin discovery | File formats, Roc compile, OKF profiles, Rocdown catalogs, product-specific flags |
| Adapter (first-party: extra subcommand on `rocci`, `rocdown`, `rocci-okf`) | `probe` a path, `list` documents cheaply, `open` by exec of existing `run --no-window` | Windowing, project persistence, fuzzy UI |
| Existing product `run` / `view` / `browse` | Compile, watch, serve, one-shot preview when invoked directly | Knowledge of other products |

Complement first. Replace later, and only **window ownership**: when launched
by the browser, adapters always `--no-window` and return `{ url, title,
inspectorUrl }`. Overlay chrome, Dev panel, and in-page Cmd-K stay where they
are.[^inspector-plan][^chrome-research][^fuzzy-plan][^serve-rs]

Do not merge `rocci browse` into this host. A future Rocci adapter may expose a
"components" target that execs `rocci browse --no-window` as one document list,
but the gallery remains a Rocci product feature.[^browse-rs][^cli-readme]

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Three product CLIs | `rocci`, `rocdown`, and `rocci-okf` stay the compilers and one-shot previewers.[^cli-plan][^product-boundary] |
| One-way package edges | Base Rocci must not depend on Rocdown or OKF. Checked mechanically.[^deps-check][^product-boundary] |
| Preview window name | The native Tao/Wry shell stays the **preview window**. rocci-browser is the process that can own it for multi-target sessions.[^preview-decision] |
| Overlay vs origin | Host picker and nav chrome are HTML/CSS/JS. Compiler-derived panels stay on the preview HTTP origin.[^chrome-research][^desktop-readme] |
| Cmd-K | In-page `goto.js` after an origin is showing. Host picker is **Cmd-P** (Ctrl-P).[^fuzzy-plan][^goto-js] |
| `path_hint` | YAML/extension sniff stays inside product CLIs for wrong-tool errors.[^path-hint] |
| CLASSES | New workspace member classified **base Rocci** in the same change.[^deps-check][^agents] |

## Non-goals (all phases)

- Plugins on `rocci run` or `rocdown run`.
- A `rocci browser` subcommand (that would turn Rocci into the rejected multiplexer).
- Renaming `rocci browse`.
- Encoding `site` / `docs` / `knowledge` as built-in targets in host Rust.
- Host parsing `.rocci`, `.rocdown`, OKF YAML, `pages.json` schema, or concept ids.
- In-process `dlopen`, Wasm adapter sandbox, or a third-party marketplace.
- Native folder dialogs (they do not exist today).[^known-limitations]
- Desktop `.app` packaging and production signing.
- Multiple native windows (one per target). The persistent shell already
  allocates window ids; connecting them is extra product scope.[^desktop-lib][^known-limitations]
- Caching Roc compile artifacts in the browser. First-open cost stays an
  adapter problem.[^compile-research]
- Authoring the picker as a `.rocci` template.

## Naming

| Surface | Name |
| --- | --- |
| Product, CLI, Cargo package | `rocci-browser` |
| Native window | preview window (existing decision) |
| Closed host vocabulary | target, document, session, window |
| Adapter executable role | plugin (manifests), never the user-facing product name |

Avoid: hub, studio, workbench, `browse`, plugin host as the app name.

## Workspace membership

Package: `crates/rocci-browser`. Classify in `BASE_ROCCI`. Add to
`[workspace].members` and `[workspace.dependencies]` in the same
change.[^cargo-toml][^deps-check][^agents]

Allowed host dependencies: `rocci-core`, `rocci-desktop` (binary only),
`rocci-ui` (shared `goto.js` scoring/embed). **Do not** depend on
`rocci-cli`, `rocci-template`, `rocci-rocdown`, `okf`, or `rocci-okf`.

Product adapters live in `rocci-cli`, `rocci-rocdown-cli`, and `rocci-okf`.
Those classes may depend on the `rocci-browser` **library** for protocol types
only. If that pulls `tao`/`wry` into adapters, keep protocol and stdio client
in the lib without `rocci-desktop`; the binary crate owns the window.

Update the AGENTS.md ownership table in the same change: CLI/desktop host
behavior for this product lives in `crates/rocci-browser`.

## Protocol (v1 freeze)

JSON-RPC 2.0, **one JSON object per line** over stdio (no LSP
`Content-Length` framing in v1). Unknown methods are ignored. A missing adapter
is a registry warning, not a host compile error.[^lsp-spec][^research]

`protocolVersion` is integer `1`. The host sends `initialize` first. The
adapter replies with `{ protocolVersion, adapterId, capabilities }`.

| Method | Params | Result |
| --- | --- | --- |
| `initialize` | `{ protocolVersion }` | `{ protocolVersion, adapterId, capabilities }` |
| `probe` | `{ path }` | `{ claimed: true, label, detail? }` or `{ claimed: false }` |
| `listDocuments` | `{ root }` | `{ documents: [{ id, title, path, route? }] }` |
| `open` | `{ root, document?, port? }` | `{ url, title, inspectorUrl? }` |
| `shutdown` | `{}` | `{}` |

Capabilities are a string list subset of `probe`, `listDocuments`, `open`,
`shutdown`.

`document` is an opaque adapter id (OKF concept id, Rocdown route, Rocci entry
path). The host displays `title` / `path` and sends the id back on `open`. It
does not parse the id.

`probe` must be cheap: existence of `rocdown.toml`, `index.md` with
`okf_version`, `rocci.toml` / `.rocci` / `main.roc`, without compiling Roc and
without importing other products' crates. `listDocuments` must also be cheap:
reuse catalog/page indexes or a file walk, not a full `run` rebuild.[^pages-json][^okf-preview][^compile-research]

One long-lived adapter RPC process per installed plugin is enough for
`probe`/`list` caching. `open` spawns a **second** child (the existing watch
server) so adapter RPC does not share a process with Roc compile. The RPC
process translates host JSON into that product's `run --no-window` and returns
when the origin is ready. The watch child stays until the host stops the
session.

If two adapters claim the same path, the host shows both claims as distinct
targets (label plus adapter id). Do not implement "more specific wins" in v1;
overlapping `docs/` vs `site/` is a real case in this repo.[^site-config][^docs-config]

## Discovery

The host starts with an empty adapter set. It loads, in order:

1. Manifests from `~/.rocci/browser/plugins/*.toml` (see path rules below).
2. Repo-local `.rocci/browser.toml` `[[plugin]]` rows, when launching from a
   directory that has that file.
3. `ROCCI_BROWSER_PLUGINS` (comma-separated `id=bin` or executable names).

Nothing in host source encodes `rocdown` or `rocci-okf`.

Illustrative manifest (not a shipped schema until Phase 1 tests lock it):

```toml
id = "rocdown"
bin = "rocdown"
argv = ["browser-adapter"]
```

`bin` is looked up on `PATH`. Workspace development documents that
`target/debug` must be on `PATH` (or tests pass absolute bins). A missing
binary is a warning next to the plugin id.

Handshake: spawn `bin` + `argv`, send `initialize`, read capabilities.

## Project registry

Persist registered roots next to existing window state under `~/.rocci/`.
Honor `ROCCI_HOME` the same way `state.rs` does. `ROCCI_STATE_DIR` continues
to control geometry only.[^window-state]

| Override | Browser directory |
| --- | --- |
| `ROCCI_BROWSER_DIR` | that directory |
| `ROCCI_HOME` | `$ROCCI_HOME/.rocci/browser` |
| default | `$HOME/.rocci/browser` |

Files: `projects.json` (user registry) and `plugins/*.toml`. Window geometry
stays in the existing `state/windows.json` with key `browser`.

Repo-local `.rocci/browser.toml` lists **this workspace's** targets as relative
paths, without product kinds:

```toml
[[plugin]]
id = "rocdown"
bin = "rocdown"
argv = ["browser-adapter"]

[[plugin]]
id = "okf"
bin = "rocci-okf"
argv = ["browser-adapter"]

[[plugin]]
id = "rocci"
bin = "rocci"
argv = ["browser-adapter"]

[[target]]
id = "site"
path = "site"

[[target]]
id = "docs"
path = "docs"

[[target]]
id = "knowledge"
path = "knowledge"
```

On launch from a repo that has that file, the host unions those targets with
the user registry. `id` is the fuzzy label; `path` is what adapters `probe`.
Plugin rows in the repo file are data, not host knowledge of products.

Worked example after probe (adapters, not host logic):

| Target id | Path | Likely adapter | Enter | Tab then Enter |
| --- | --- | --- | --- | --- |
| site | `site/` | Rocdown (`site/rocdown.toml`, mounts `docs/`) | site home | a `pages.json` row such as `/docs/guides/desktop-app/` |
| docs | `docs/` | Rocdown (`docs/rocdown.toml`) | docs home | a docs page without the site chrome |
| knowledge | `knowledge/` | OKF (`knowledge/index.md`) | review home | a concept such as `plans/cli-entry-points` |

`site` including mounted docs and `docs` as a standalone site is intentional
duplication, not a reason to special-case Rocdown in the host.[^site-config][^docs-config]

Registration UI: add/remove directories, show last probe label, show last error
if no adapter claimed the path. v1 is path-typed plus CLI
`rocci-browser add <path>`. No native folder picker.[^known-limitations]

## Two-stage picker

Reuse the `goto.js` subsequence / substring scoring (`fuzzy` then `scoreEntry`
over `title`, `path`, optional `route`). Do not invent a second
algorithm.[^goto-js]

### Stage 1: targets

Filter registered targets by `id`, relative path, and adapter label.

- **Enter** (or click): `open { root }` with no document. Adapter chooses its
  home URL.
- **Tab**: do not open. Call `listDocuments` for the highlighted target and
  switch to stage 2. If the list is empty, stay on stage 1 with a reason from
  the adapter.
- **Escape**: close picker (desktop) or quit TUI without launching.

This is Raycast / shell-completion (Tab completes, Enter runs), not fzf
Tab-toggle.

### Stage 2: documents

Rows come only from the adapter. Display title plus path.

- **Enter**: `open { root, document }`. Adapter maps that to the same URL
  `rocci-okf run knowledge/plans/cli-entry-points.md` or `rocdown run
  docs/guides/desktop-app.rocdown` would have opened.[^okf-preview][^rocdown-cli]
- **Shift-Tab** or **Escape**: back to stage 1, keep the target query if
  possible.
- Further Tab in stage 2 does nothing in v1.

### Hosting the picker

The picker must exist before any product origin and survive origin changes.
That is host overlay lifecycle, not a Rocci `@component` on a child
server.[^chrome-research][^preview-decision]

Author it as HTML/CSS/JS under the browser crate (sibling of `preview-nav` /
`goto.js`), mounted in the initialization script. Reuse `goto.js` scoring
rather than a second algorithm. Do not snapshot the picker from a `.rocci`
template.[^desktop-readme][^goto-js]

The registry screen (add/remove roots) can be the same overlay in a different
mode, or a small host page at a host-owned origin. It must not be an OKF or
Rocdown site.

Tab inside a webview must `preventDefault` on the picker input so it does not
move focus. Native menus alias Cmd-P onto the same overlay. Do not steal
in-page Cmd-K.

Headless / agent form:

```text
rocci-browser open knowledge --document plans/cli-entry-points --no-window --json
```

prints `{ url, title }` on stdout.

## CLI and desktop fronts

Same library, two fronts. Executable name **`rocci-browser`**. Not a `rocci`
subcommand.[^cli-plan][^cli-readme]

| Front | Behavior |
| --- | --- |
| `rocci-browser` (no args, graphical) | Persistent window: registry + picker overlay + webview that `load_url`s adapter origins |
| `rocci-browser tui` | Terminal two-stage picker, then either exec with a preview window or `--no-window` print URL |
| `rocci-browser add\|remove\|list` | Registry CRUD |
| `rocci-browser open <query>` | Non-interactive fuzzy over targets; `--document` for stage 2 |

Desktop packaging waits. v1 is `cargo run -p rocci-browser`. A later `.app`
would use the persistent `run` shell, not `rocci bundle` of a Datastar
gallery.[^known-limitations][^root-readme]

## Window and session model

Keep one long-lived native window by default.

1. Launch: webview shows a host-owned empty/launcher surface; picker opens
   focused.
2. `open` succeeds: host records a **session** `{ target, document?, child,
   origin, inspectorUrl }`, then `webview.load_url(url)`. Overlay chrome
   (back/forward/home/reload, Dev iframe) remains preview chrome, now pointed
   at the new origin.[^preview-rs][^inspector-plan]
3. Switch target: start the new adapter child (or reuse a still-warm session
   for that root), navigate, then stop the previous child after a grace period
   so OKF/Rocdown watch servers are not rebuilt on every hop.
4. Quit: stop children, persist registry and window geometry.

`preview()` today returns when the window closes. A project browser needs the
persistent `run()` shell, or an extension of `preview` that never returns until
quit, plus **changing** the origin without exiting the event loop. That belongs
in `rocci-desktop` or the browser crate sitting on `LiveWindow`, not in product
CLIs.[^preview-rs][^desktop-lib][^backend]

`ExternalBackend` already models "webview this origin". The missing piece is
swapping origin and owning child lifetimes. Prefer a small `rocci-desktop` API
(`load_url` + session key) used by the browser crate over forking a third event
loop.

Warm-session reuse is the main performance win versus today's one-shot
`preview()`. First-open compile cost stays an adapter problem.[^compile-research]

`--no-window` on the browser CLI is for agents and for users who want the
picker to print a URL the existing previewer or a system browser can open.

## Relationship to current previewer

| Keep | Change only if browser owns the window |
| --- | --- |
| Product `run` / `view` / `browse` one-shot preview | Adapters called from the browser pass `--no-window` |
| Overlay nav, find, Cmd-K, Dev iframe | Host picker is an additional overlay (Cmd-P) |
| `PreviewOptions.inspector_url` / `source_root` | Browser forwards adapter-provided inspector URL into the same overlay |
| Three product binaries | Fourth binary for host only |

The inspector source-views plan still applies to whatever origin is visible.
The browser must not interpret AST/Roc/HTML; it only keeps the iframe pointed
at the current session's inspector URL plus `?route=` if overlay sync already
does that.[^inspector-plan]

## Tests without product crates

Host tests spawn a fixture adapter script that implements the protocol and
serves a static `hello` origin. `cargo test -p rocci-browser` must not start
`rocci-okf` or compile Rocdown. Adapter crates test `probe`/`list` at their
own boundary. Cursor and token loops in any new scanners must make monotonic
forward progress; this crate should not need a scanner.

Workspace-deps check must fail if `rocci-browser` grows a Rocdown or OKF
dependency.

## Delivery phases

Do not start a phase until the user asks. Phase 1 also requires human
acceptance of decision gates 1–2.

### 0. Freeze the host contract

This record is the freeze:

- Protocol version `1`, newline-delimited JSON-RPC, methods above.
- Enter opens a target; Tab lists documents; Cmd-P vs Cmd-K.
- Host never sniffs formats.
- Option D; options B, C, and E stay rejected or deferred.
- Package name `rocci-browser`, class base Rocci.

Exit when a reviewer treats this plan as the cited owner for the browser
question, including gates 1–2.

### 1. Host crate, fixture adapter, TUI / headless open

- Add `crates/rocci-browser` to the workspace, `CLASSES`, AGENTS.md table, and
  crate README in the same change.
- Library: protocol types, discovery, registry CRUD, stdio client, Rust port of
  `goto.js` `fuzzy` / `scoreEntry` with fixture strings that match the JS
  function.
- Fixture adapter under `crates/rocci-browser/tests/fixtures/` (script or tiny
  binary) that claims a temp directory, lists two documents, and on `open`
  serves `hello` over loopback.
- CLI: `add` / `remove` / `list` / `open` / `tui`. `open --no-window --json`
  prints `{ url, title }`.
- Tests: protocol round-trip against the fixture; registry file under a temp
  `ROCCI_BROWSER_DIR`; two adapters claiming one path appear as two targets;
  `cargo test -p rocci-browser` does not invoke `rocci-okf` or `rocdown`.
- `scripts/check-workspace-deps.py` stays green.

Exit when `rocci-browser open fixture --no-window --json` prints the fixture
URL, and Tab-then-Enter via `tui` or `--document` prints a document URL, with
no product crates in the host test process.

### 2. Persistent preview window and host picker overlay

- Graphical `rocci-browser` with one preview window that does not return until
  quit.
- Host-owned launcher surface; Cmd-P overlay authored as crate HTML/CSS/JS;
  Tab `preventDefault` on the picker input.
- On fixture `open`, `webview.load_url` the fixture origin; overlay chrome
  (back/home/reload) still works. Forward `inspectorUrl` when present.
- Session table: start fixture child, navigate, stop child on quit.
- Extend `rocci-desktop` only as needed to `load_url` without exiting
  `preview()`'s process model. Prefer reusing `LiveWindow` over a third shell.
- Tests: window/session unit tests without opening a display when possible;
  protocol tests remain headless. If a display test is required, gate it
  `#[ignore]` or skip without a display.

Exit when the window stays up across two fixture `open`s (home then document)
and the picker can switch targets without restarting the host process.

### 3. First-party Rocdown and OKF adapters

- `rocdown browser-adapter` and `rocci-okf browser-adapter` implement the
  protocol.
- `probe`: `rocdown.toml` vs bundle-root `okf_version` / concept file inside a
  bundle. Cheap. No Roc compile.
- `listDocuments`: Rocdown from planned `pages.json` fields (title, path,
  route) without a full site rebuild if an index already exists; otherwise a
  config-aware file walk. OKF from the bundle catalog / concept paths, not
  `run`.[^pages-json][^okf-preview]
- `open`: exec the existing `run --no-window` path with the same targeting as
  `rocdown run <site|file>` and `rocci-okf run <bundle|concept.md>`.[^rocdown-cli][^okf-main][^okf-preview]
- Adapter tests in those crates: probe true/false fixtures, list ids, open
  `--no-window` URL shape. Host tests still use the fixture adapter only.
- Product `run` without the browser still opens today's preview window.

Exit when, with adapters on `PATH` and a temp registry, `rocci-browser open`
can print a Rocdown site home URL and an OKF concept URL, matching direct
`run --no-window`.

### 4. Repo-local registry and Rocci adapter

- Commit `.rocci/browser.toml` for this workspace with `site`, `docs`,
  `knowledge` targets and plugin rows as data.
- `rocci browser-adapter`: `probe` app directories / `.rocci` / `rocci.toml` /
  `main.roc`; `listDocuments` of previewable entries; `open` via `rocci run
  --no-window`. Optional later: a "components" document that execs `rocci
  browse --no-window` without renaming `browse`.[^browse-rs]
- Union repo-local targets with the user registry when cwd (or an explicit
  `--root`) contains `.rocci/browser.toml`.
- Docs in the crate README for PATH/`target/debug` during workspace dev.

Exit when `cargo run -q -p rocci-browser -- open knowledge --document
plans/cli-entry-points --no-window --json` from this repo reaches the same
concept URL as `rocci-okf run knowledge/plans/cli-entry-points.md --no-window`,
and `open site` / `open docs` reach the two Rocdown trees.

### 5. Warm sessions, overlay inspect, and public docs

- Reuse a still-warm session for the same target root instead of rebuilding on
  every hop. Grace-stop the previous child after the new origin is up (default
  grace on the order of 30s, configurable later).
- Point preview overlay Dev iframe at the session `inspectorUrl`; do not parse
  inspect JSON in the host.[^inspector-plan]
- Root README and a public docs page: what rocci-browser is, Cmd-P vs Cmd-K,
  registry file, that `rocci browse` is unrelated. Mark planned desktop
  packaging as planned.
- Product CLIs keep one-shot preview. Do not default `run` to `--no-window`
  when a browser exists (gate 3).

Exit when hopping `knowledge` → `docs` → `knowledge` in one window reuses the
warm OKF child on the way back, and docs describe the host without encoding
adapter ids in Rust.

## Later, gated work

Not in Phases 0–5:

| Gate | Work |
| --- | --- |
| 3 | Product `run` defaults to `--no-window` when a browser session exists |
| 4 | Author picker UI in Rocci instead of host HTML |
| 5 | Native folder dialogs or a third-party plugin marketplace |
| 6 | Built-in `site` / `docs` / `knowledge` in host source (forbidden; belongs in the repo-local file) |
| — | Multi-window (one native window per target) |
| — | Content-Length LSP framing, dlopen/Wasm adapters |
| — | Signed `.app` packaging |

## Acceptance criteria (through Phase 5)

- Fourth user-facing binary `rocci-browser` is classified base Rocci and has
  no Rocdown/OKF package edge.
- Host source does not mention `.rocdown`, OKF YAML keys, or `rocci-okf` except
  in docs/fixtures/comments that describe *other* products' adapters.
- Enter opens a target home; Tab then Enter opens an adapter document.
- Cmd-P opens the host picker; Cmd-K remains in-page `goto.js`.
- `rocci`, `rocdown`, and `rocci-okf` still open one-shot preview windows when
  invoked directly.
- `rocci browse` is still the component gallery.
- Overlapping `site/` and `docs/` appear as two targets.
- `cargo test -p rocci-browser` does not compile Rocdown or start `rocci-okf`.

## Decision gates

Human approval is required before:

1. Adding a `rocci-browser` workspace member and CLASSES entry (fourth
   user-facing CLI).
2. Treating out-of-process adapters as the approved plugin shape versus a
   static composition binary.
3. Moving window ownership so product `run` defaults to `--no-window` when a
   browser session exists.
4. Authoring picker UI in Rocci instead of host HTML.
5. Native folder dialogs or a third-party plugin marketplace.
6. Encoding `site` / `docs` / `knowledge` as built-in targets in host source.

Until gates 1–2 open, do not implement Phases 1–5. This plan is the delivery
track for the research recommendation, not an approved schedule.

## Status

Exploratory; Phase 0 freeze plus Phases 1–5 in this revision. Not CI-complete.

[^research]: Recommended split: product-blind host, stdio adapters, registry, two-stage picker.
[^cli-plan]: Three-CLI split, rejection of plugin hosts on rocci/rocdown, exec-sibling dispatcher deferred.
[^product-boundary]: Approved one-way package edges and product ownership.
[^preview-decision]: Preview window versus overlay chrome versus Dev panel naming.
[^chrome-research]: Overlay HTML/JS versus preview-origin Rocci inspector UI.
[^system-overview]: Current workspace and preview-window contract.
[^deps-check]: Mechanical CLASSES forbidding base Rocci → Rocdown/OKF.
[^agents]: New workspace members must be classified in check-workspace-deps.py.
[^cargo-toml]: Current workspace member list; rocci-browser is not a member yet.
[^desktop-readme]: Desktop host has no language-crate dependencies; chrome is assets.
[^preview-rs]: Blocking `preview()`, Home uses `webview.load_url`.
[^desktop-lib]: Persistent `run()` shell with multiple windows.
[^backend]: `RunningBackend` / `ExternalBackend` attach a window to an origin.
[^window-state]: `~/.rocci/state` geometry persistence and `ROCCI_HOME` / `ROCCI_STATE_DIR`.
[^serve-rs]: Shared `preview()` helper; `--no-window`; window close stops the child.
[^browse-rs]: Component gallery over `.rocci` files, not a project launcher.
[^cli-readme]: Documented `rocci browse` / `view` / `run` surface.
[^path-hint]: Extension and YAML-prefix sniff without an okf dependency.
[^okf-main]: `rocci-okf run` serves then opens a preview window.
[^okf-preview]: Bundle, index, and concept-file preview targeting.
[^rocdown-cli]: `rocdown run` on a site, document, or in-site file.
[^pages-json]: `pages.json` entries include title, route, path, kind.
[^goto-js]: Shared fuzzy scoring and in-page palette.
[^fuzzy-plan]: Cmd-K is document navigation, not full-text search.
[^inspector-plan]: Dev iframe stays preview-origin; overlay only syncs URL.
[^known-limitations]: No general native dialogs; packaging and multi-window app lifecycle gaps.
[^site-config]: `site/` mounts `../docs` as the docs prefix.
[^docs-config]: Standalone `docs/rocdown.toml` site.
[^root-readme]: Public commands for rocci, rocdown, and rocci-okf.
[^compile-research]: OKF first-open dominated by Roc compile and render.
[^lsp-spec]: JSON-RPC process model for a generic host plus adapters.
