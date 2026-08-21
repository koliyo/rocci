---
type: Research Report
title: Dedicated rocci-browser CLI and desktop host
description: "Exploratory research for a product-blind project browser that registers directories, two-stage fuzzy-picks a target then a document, and opens a persistent preview session through out-of-process adapters. Complements, and can later own the window of, rocci / rocdown / rocci-okf preview. Does not reverse the rejection of plugins on those product CLIs."
tags: [domain/rocci, domain/desktop, domain/rocci-okf, domain/rocdown, concern/architecture, concern/tooling, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-20T05:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: browser-plan
    resource: ../plans/rocci-browser.md
    title: Dedicated rocci-browser implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: macos-plan
    resource: ../plans/rocci-browser-macos-app.md
    title: rocci-browser macOS app and TUI removal plan
    author: process:cursor
    last_modified: 2026-08-20
  - id: cli-plan
    resource: ../plans/cli-entry-points.md
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
    resource: desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: system-overview
    resource: ../architecture/system-overview.md
    title: Current Rocci system overview
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: language-tooling
    resource: ../architecture/language-tooling.md
    title: Language-server composition boundary
    author: process:cursor
    last_modified: 2026-08-18
  - id: lsp-analyzer
    resource: ../../crates/rocci-lsp/src/analyzer.rs
    title: DocumentAnalyzer extension point
    author: process:git
    last_modified: 2026-08-17
  - id: deps-check
    resource: ../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Mechanical one-way workspace dependency check
    author: process:cursor
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
  - id: session
    resource: ../../crates/rocci-core/src/session.rs
    title: Window session store
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
  - id: cli-lib
    resource: ../../crates/rocci-cli/src/lib.rs
    title: Shared Rocci CLI driver library
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
    resource: ../plans/fuzzy-navigation.md
    title: Cmd-K fuzzy navigation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: inspector-plan
    resource: ../plans/inspector-source-views.md
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
    last_modified: 2026-08-18
  - id: agents
    resource: ../../AGENTS.md
    title: Workspace ownership and CLASSES rule
    author: process:git
    last_modified: 2026-08-19
  - id: compile-research
    resource: okf-compile-render-cost.md
    title: OKF preview compile and render cost
    author: process:cursor
    last_modified: 2026-08-19
  - id: lsp-spec
    resource: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/
    title: Language Server Protocol specification
    author: organization:microsoft
---

# Dedicated rocci-browser CLI and desktop host

## Scope and authority

This record is exploratory. It does not approve a fourth product CLI, a plugin
ABI, or a change to the shipped three-binary split. It asks whether a
**product-blind browser host** can replace or complement the shared preview
window used by `rocci`, `rocdown`, and `rocci-okf`, with a project registry and
a two-stage fuzzy picker, **without the host naming those products**.[^cli-plan][^system-overview]

It does **not** reverse the CLI-entry-points recommendation that `rocci` and
`rocdown` must not grow a plugin lifecycle for first-party format
dispatch.[^cli-plan] A new host whose job is session, window, and project
selection is a different question from teaching `rocci run` to compile Rocdown
or OKF.

The [implementation plan](../plans/rocci-browser.md) turns this recommendation
into a phased delivery track. Human review is still required before treating
that split as approved work (fourth CLI and out-of-process adapter
gates).[^browser-plan]

## Job to be done

Authors in this repository already hop among several previewable trees: the
`site/` rocci.dev tree, the standalone `docs/` tree, and the `knowledge/` OKF
bundle. Each hop is a different binary, a new process, and a new preview
window that dies when the window closes.[^root-readme][^okf-main][^rocdown-cli][^serve-rs]

The desired loop is closer to a launcher than to a compiler:

1. Register project directories once (for this repo: `site`, `docs`,
   `knowledge`).
2. Fuzzy-pick a **target** (one of those directories, or another registered
   root).
3. **Enter** opens that target as a whole (site home, bundle home, app root).
4. **Tab** drills into a second fuzzy list of **documents inside that target**,
   then Enter opens that document.

The host that owns that loop must not encode `rocci` / `rocdown` / `rocci-okf`
as a closed enum. First-party products participate as adapters. A later
third-party adapter should be able to claim a directory the same way.

## Established baseline

### Three product CLIs, one preview primitive

The workspace ships three user-facing binaries with one-way ownership. Base
Rocci must not depend on Rocdown or OKF. Rocdown must not depend on OKF.
`rocci-okf` must not depend on Rocdown. Those edges are checked
mechanically.[^product-boundary][^deps-check][^system-overview]

| Product | Command that opens a window | Target |
| --- | --- | --- |
| Rocci apps and `.rocci` | `rocci run`, `rocci view`, `rocci browse` | `.rocci` file, Roc app directory, or component gallery |
| Rocdown documents and sites | `rocdown run` | `.rocdown` / site directory / file inside a `rocdown.toml` site |
| OKF review | `rocci-okf run` | bundle directory, root `index.md`, or concept `.md` |

Each of those paths starts an HTTP origin, then calls
`rocci_desktop::preview` (directly or through `rocci_cli::serve`). The native
window is the **preview window**. Overlay navigation is host HTML/CSS/JS.
Compiler-derived panels belong on the preview HTTP origin. Closing the window
stops the child server.[^preview-decision][^chrome-research][^serve-rs][^okf-main][^desktop-readme]

`rocci-desktop` has no dependency on `rocci-template`, `rocci-rocdown`, `okf`,
or `rocci-okf`. That blindness is already the right host property; it is just
wired as a one-shot URL viewer rather than a project
browser.[^desktop-readme]

Shared runtime reuse is a **library**, not a plugin. `rocci-cli` exposes
driver, serve, and preview helpers that `rocdown` and `rocci-okf`
consume.[^cli-lib][^cli-plan]

### What is already named "browse" or "go to"

`rocci browse` is a **component gallery**. It walks `.rocci` files, infers
previewable props, and compiles a temporary Browser/Catalog/Preview Roc app. It
does not register projects, does not open Rocdown or OKF trees, and must not be
renamed into a generic launcher.[^browse-rs][^cli-readme]

Cmd/Ctrl-K `goto.js` is **in-page document navigation** after a site is already
being served. It ranks `pages.json` / `catalog.json` titles and paths and swaps
already-rendered HTML. It is not a target switcher, not a project registry, and
not full-text search.[^goto-js][^fuzzy-plan][^known-limitations]

File-aware `rocci-okf run path/to/concept.md` already resolves the enclosing
bundle and opens that concept URL. `rocdown run` on a file inside a site
previews the site at that page route. Those are the one-shot commands the
browser would wrap, not replace as compilers.[^okf-preview][^rocdown-cli]

### Two desktop shells, only one used for preview

`rocci_desktop::preview` is a blocking, single-window event loop around one
URL. The webview can `load_url` for Home, but the function returns when the
window closes, and there is no session manager for switching child
servers.[^preview-rs]

`rocci_desktop::run` is a persistent multi-window shell with a `RunningBackend`,
window ids, and `~/.rocci/state` geometry. Multi-window lifecycle is not
connected to authored Roc apps; native capabilities beyond the webview
boundary remain absent.[^desktop-lib][^backend][^session][^window-state][^known-limitations]

A project browser that stays open while targets change needs the persistent
shell (or an extension of `preview` that never returns until quit), not a
fresh `preview()` per hop.

### Plugins were rejected in a narrower setting

The CLI-entry-points plan considered and rejected:

- plugins on `rocdown` to absorb OKF Markdown;
- plugins on `rocci` as a universal `run` multiplexer;
- a fourth CLI whose purpose was format dispatch.

It allowed later **exec of sibling binaries** as UX sugar, and treated that as
a dispatcher, not a plugin system. It also recorded that a plugin host has only
three dependency-safe shapes: forbidden static link, unstable in-process native
modules, or exec of sibling binaries.[^cli-plan]

The language-server crate already uses the good in-process analog:
`DocumentAnalyzer` is a trait in generic `rocci-lsp`; `rocci-rocdown-lsp`
composes analyzers into one product binary. The generic core does not import
Rocdown types. The composition binary does know both products.[^language-tooling][^lsp-analyzer]

That composition pattern is **wrong** for rocci-browser if the requirement is
that the host binary itself does not know the products. A
`rocci-browser-all` that statically links three adapters would know them at
compile time, the way `rocci-rocdown-lsp` knows Rocdown.

## The actual problem

The shared pain is **session and selection**, not a missing compiler.

1. Switching `site` → `docs` → `knowledge` is three commands and three window
   lifetimes.
2. Opening a known document still requires picking the right binary
   (`rocdown run` vs `rocci-okf run`) even after cross-CLI hints exist.
3. In-page Cmd-K only helps after the right origin is already up.
4. First-open cost for OKF preview is dominated by Roc compile and render;
   killing the window on every hop repeats that cost.[^compile-research]
5. The preview window is product-blind, but every product CLI re-owns window
   creation, inspector URL, and state key.

A plugin host **on** `rocci` or `rocdown` would reopen the frozen format
boundary. A new host that only **execs** product servers and never parses
their files does not need that boundary to move.[^cli-plan][^product-boundary]

## Options

### A. Keep three `run` windows; improve hints and aliases

Ship shell aliases, editor tasks, and keep `rocci-okf run <file>` /
`rocdown run <file>`. Cheapest. Does not give a registry, a two-stage picker,
or a living session across targets.

**Keep as the one-shot path.** Insufficient as the only answer to the job
above.

### B. Plugin lifecycle on `rocci` or `rocdown`

Already rejected for format dispatch. Reopening it to add a picker still
forces either a forbidden package edge or an ABI inside a product
binary.[^cli-plan][^deps-check]

**Reject** as the browser architecture.

### C. Composition binary that statically links three adapters

A `rocci-browser` crate that depends on `rocci-cli`, `rocci-rocdown-cli`, and
`rocci-okf` libraries. Fast in-process `list`/`open`. The host **does** know
the products. It also needs a new dependency-checker class or an allowlist,
and it cannot accept a third-party adapter without a rebuild.

This is the LSP composition pattern applied to preview. It fails the "must not
know those explicitly" requirement.

**Reject** as the host shape. In-process traits remain allowed **inside** a
product adapter (for example Rocdown listing pages), not across the host
boundary.

### D. Product-blind host plus out-of-process adapters (recommended)

Add a dedicated `rocci-browser` executable and desktop app whose crates depend
only on base Rocci (`rocci-core`, `rocci-desktop`, maybe `rocci-cli` serve
helpers and `rocci-ui` for shared fuzzy chrome). Discover adapters by
manifest and PATH. Talk JSON over stdio, in the same family as LSP.
`probe` / `list` / `open` are adapter methods. `open` is a long-lived child
that already knows how to `run --no-window`.[^lsp-spec][^cli-plan][^deps-check]

The host's closed vocabulary is **target**, **document**, **session**,
**window**. It never matches on `.rocci`, `.rocdown`, or OKF YAML. Existing
`path_hint` sniffing stays inside the product CLIs for wrong-tool
errors.[^path-hint]

**Recommend** this option. It is the exec-sibling shape the CLI plan already
called out, promoted from a `rocci` multiplexer to a separate host so `rocci
run` keeps owning applications.

### E. dlopen / Wasm plugins in the host process

Unstable Rust ABI, or a Wasm sandbox that still needs a serve/watch capability
model the desktop host does not have. Overkill for three trusted first-party
CLIs that already exec.[^cli-plan][^known-limitations]

**Defer.** Revisit only if a third-party adapter cannot ship a binary.

## Recommended product split

| Layer | Owns | Must not own |
| --- | --- | --- |
| `rocci-browser` host | Project registry, two-stage picker chrome, persistent window(s), session table (adapter child → origin → webview URL), plugin discovery | File formats, Roc compile, OKF profiles, Rocdown catalogs, product-specific flags |
| Adapter (first-party: extra subcommand on `rocci`, `rocdown`, `rocci-okf`) | `probe` a path, `list` documents cheaply, `open` by exec of existing `run --no-window` | Windowing, project persistence, fuzzy UI |
| Existing product `run` / `view` / `browse` | Compile, watch, serve, one-shot preview when invoked directly | Knowledge of other products |

Complement first: `rocci-okf run knowledge/plans/….md` still opens its own
preview window. The browser is another way to reach the same servers.

Replace later, and only the **window ownership**: when launched by the
browser, adapters always `--no-window` and return `{ url, title,
inspector_url }`. Direct `run` keeps today's preview. Overlay chrome, Dev
panel, and in-page Cmd-K stay where they are.[^inspector-plan][^chrome-research][^fuzzy-plan]

Do not merge `rocci browse` into this host. A future Rocci adapter may expose
a "components" target that execs `rocci browse --no-window` as one document
list, but the gallery remains a Rocci product feature.[^browse-rs]

## Plugin architecture

### Discovery

The host starts with an empty adapter set. It loads:

1. Manifests from `~/.rocci/browser/plugins/*.toml` and, if present, a
   repo-local `.rocci/browser.toml` `plugins` list.
2. Optional `ROCCI_BROWSER_PLUGINS` (comma-separated executable names).
3. Nothing that encodes `rocdown` or `rocci-okf` in host source.

A manifest names an executable and an identifier. Example shape (illustrative,
not shipped):

```toml
id = "rocdown"
bin = "rocdown"
argv = ["browser-adapter"]
```

First-party CLIs grow a dedicated adapter subcommand so the host does not learn
`run --no-window --port auto`. The adapter translates host JSON into that
product's existing `run`.

Handshake is LSP-like: the host spawns the adapter, sends `initialize`, and
reads capabilities (`probe`, `listDocuments`, `open`, maybe `shutdown`).
Unknown methods are ignored. A missing adapter is a registry warning, not a
host compile error.[^lsp-spec]

### Protocol (host-blind)

JSON-RPC or JSON-lines over stdio. One long-lived adapter process per
installed plugin is enough for `probe`/`list` caching; `open` may spawn a
**second** child (the existing watch server) so adapter RPC does not share a
process with Roc compile.

Illustrative methods:

```text
initialize  -> { protocolVersion, adapterId, capabilities }
probe       { path } -> { claimed, label, detail? } | { claimed: false }
listDocuments { root } -> { documents: [{ id, title, path, route? }] }
open        { root, document?, port? } -> { url, title, inspectorUrl? }
shutdown
```

`document` is an opaque adapter id (OKF concept id, Rocdown route, Rocci
entry path). The host displays `title` / `path` and sends the id back on
open. It does not parse the id.

`probe` must be cheap: existence of `rocdown.toml`, `index.md` with
`okf_version`, `rocci.toml` / `.rocci` / `main.roc`, without compiling Roc
and without importing other products' crates. Listing documents must also be
cheap: reuse catalog/page indexes or a file walk, not a full `run`
rebuild.[^pages-json][^okf-preview][^compile-research]

If two adapters claim the same path, the host shows both claims as distinct
targets (label plus adapter id) rather than guessing. Do not implement
"more specific wins" in v1; overlapping `docs/` vs `site/` is a real case
in this repo, not a bug.[^site-config][^docs-config]

### What stays out of the host

- YAML frontmatter sniffing (`path_hint` stays in product CLIs).[^path-hint]
- `pages.json` schema, OKF concept ids, `.rocci` module names.
- Theme, profile, host-runtime (`native`/`wasm`), provenance flags. If an
  adapter needs them, they are adapter config, not host flags.
- In-process `dlopen`.

### Tests without product crates

Host tests spawn a fixture adapter script that implements the protocol and
serves a static `hello` origin. `cargo test -p rocci-browser` must not start
`rocci-okf` or compile Rocdown. Adapter crates test `probe`/`list` at their
own boundary.

## Project registry

Persist registered roots in `~/.rocci/browser/projects.json`, next to existing
window state under `~/.rocci/`. Honor `ROCCI_HOME` / `ROCCI_STATE_DIR` the
same way the desktop crate already does.[^window-state]

A repo-local `.rocci/browser.toml` (or `[browser]` table later) lists
**this workspace's** targets as relative paths, without product kinds:

```toml
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

Worked example for this repository:

| Target id | Path | Likely adapter after probe | Enter | Tab then Enter |
| --- | --- | --- | --- | --- |
| site | `site/` | Rocdown (`site/rocdown.toml`, mounts `docs/`) | `rocdown run site` home | a `pages.json` row such as `/docs/guides/desktop-app/` |
| docs | `docs/` | Rocdown (`docs/rocdown.toml`) | docs home | a docs page without the site chrome |
| knowledge | `knowledge/` | OKF (`knowledge/index.md`) | review home | a concept such as `plans/cli-entry-points` |

`site` including mounted docs and `docs` as a standalone site is intentional
duplication of content, not a reason to special-case Rocdown in the
host.[^site-config][^docs-config]

Registration UI: add/remove directories, show last probe label, show last
error if no adapter claimed the path. The host may offer a folder picker
only if native dialogs exist; they do not today, so v1 can be path-typed
plus CLI `rocci-browser add <path>`.[^known-limitations]

## Two-stage picker

### Stage 1: targets

Input filters registered targets by `id`, relative path, and adapter label
using the same subsequence / substring scoring already used in
`goto.js`.[^goto-js]

- **Enter** (or click): `open { root }` with no document. Adapter chooses its
  home URL (`/`, bundle home, app `/`).
- **Tab**: do not open. Call `listDocuments` for the highlighted target and
  switch to stage 2. If the list is empty, stay on stage 1 with a reason
  from the adapter.
- **Escape**: close picker without launching.

This is the Raycast / shell-completion shape (Tab completes, Enter runs),
not fzf's Tab-toggle. Do not steal in-page Cmd-K: once a session is showing
a Rocdown or OKF origin, Cmd-K remains `goto.js` inside that origin.
Recommend **Cmd-P** (or Cmd-Shift-P) for the host picker so it can switch
targets without leaving the window.[^fuzzy-plan]

### Stage 2: documents

Rows come only from the adapter. Display title plus path. Score like
`goto.js` (`title`, `path`, optional `route`).[^goto-js][^pages-json]

- **Enter**: `open { root, document }`. Adapter maps that to the same URL
  `rocci-okf run knowledge/plans/cli-entry-points.md` or `rocdown run
  docs/guides/desktop-app.rocdown` would have opened.[^okf-preview][^rocdown-cli]
- **Shift-Tab** or **Escape**: back to stage 1, keep the target query if
  possible.
- Further Tab in stage 2 does nothing in v1 (no third stage).

Headless / agent form:

```text
rocci-browser open knowledge --document plans/cli-entry-points --no-window --json
```

prints `{ url, title }` on stdout. That is the same job as today's
`rocci-okf run … --no-window` with a product-blind entry.

### Where the picker is authored

The picker must exist before any product origin and survive origin changes.
That is host overlay lifecycle, not a Rocci `@component` on a child
server.[^chrome-research][^preview-decision]

Author it as HTML/CSS/JS under the browser crate (sibling of
`preview-nav` / `goto.js`), mounted in the initialization script. Reuse the
`goto.js` scoring function rather than a second algorithm. Do not snapshot
the picker from a `.rocci` template; the desktop crate's existing reason
stands.[^desktop-readme][^goto-js]

The **registry** screen (add/remove roots) can be the same overlay in a
different mode, or a small host page at a host-owned origin. It should not
be an OKF or Rocdown site.

Tab inside a webview must `preventDefault` on the picker input so it does
not move focus. Native menus can alias Cmd-P onto the same overlay.

## CLI and desktop app

Same library, two fronts.

| Front | Behavior |
| --- | --- |
| `rocci-browser` (no args, graphical) | Persistent window: registry + picker overlay + webview that `load_url`s adapter origins |
| `rocci-browser add\|remove\|list` | Registry CRUD |
| `rocci-browser open <query>` | Non-interactive fuzzy over targets; `--document` for stage 2 |

Do not ship a TUI. A Phase 1 `tui` command existed as a bootstrap; it is
withdrawn. Headless work is `open --no-window`.[^macos-plan]

v1 remains `cargo run -p rocci-browser`. An ad-hoc Finder `.app` is the
[macOS app plan](../plans/rocci-browser-macos-app.md): wrap the existing
`preview()` host, do not `rocci bundle` a gallery, do not switch to
`run()`. Production signing stays a known limitation.[^macos-plan][^known-limitations][^root-readme]

Executable name: **`rocci-browser`**. Cargo package: **`rocci-browser`**. Do
not call it `rocci browse`, `rocci-okf-cli`, or `rocci` subcommand
`browser`. A `rocci browser` subcommand would turn the Rocci binary into the
multiplexer the CLI plan rejected.[^cli-plan][^cli-readme]

Classify the package as **base Rocci** in `tools/rocci-ops/src/rocci_ops/workspace_deps.py`
so it cannot depend on Rocdown or OKF. First-party adapter code lives in
`rocci-cli`, `rocci-rocdown-cli`, and `rocci-okf` (their classes already
allow depending on base Rocci). Adding the workspace member must update
`CLASSES` in the same change.[^deps-check][^agents]

## Window and session model

Keep one long-lived native window by default.

1. Launch: webview shows a host-owned empty/launcher surface; picker opens
   focused.
2. `open` succeeds: host records a **session** `{ target, document?, child,
   origin, inspector_url }`, then `webview.load_url(url)`. Overlay chrome
   (back/forward/home/reload, Dev iframe) remains preview chrome, now
   pointed at the new origin.[^preview-rs][^inspector-plan]
3. Switch target: start the new adapter child (or reuse a still-warm session
   for that root), navigate, then stop the previous child after a grace
   period so OKF/Rocdown watch servers are not rebuilt on every hop.
4. Quit: stop children, persist registry and window geometry.

Reuse of a warm knowledge session is the main performance win versus today's
one-shot `preview()`. First-open compile cost stays an adapter problem;
the browser must not try to cache Roc artifacts itself.[^compile-research]

`ExternalBackend` already models "webview this origin". The missing piece is
**changing** the origin without exiting the event loop, plus owning child
lifetimes. That belongs in `rocci-desktop` or the browser crate sitting on
top of `LiveWindow`, not in product CLIs.[^backend][^preview-rs]

Multiple windows (one per target) can wait. The persistent shell already
allocates window ids; connecting that to adapter sessions is extra product
scope.[^desktop-lib][^known-limitations]

`--no-window` on the browser CLI is for agents and for users who want the
picker to print a URL the existing previewer or a system browser can open.

## Relationship to current previewer

| Keep | Change only if browser owns the window |
| --- | --- |
| Product `run` / `view` / `browse` one-shot preview | Adapters called from the browser pass `--no-window` |
| Overlay nav, find, Cmd-K, Dev iframe | Host picker is an additional overlay (Cmd-P) |
| `PreviewOptions.inspector_url` / `source_root` | Browser forwards adapter-provided inspector URL into the same overlay |
| Three product binaries | Fourth binary for host only |

The inspector source-views plan still applies to whatever origin is
visible. The browser must not interpret AST/Roc/HTML; it only keeps the
iframe pointed at the current session's inspector URL plus `?route=` if
overlay sync already does that.[^inspector-plan]

## Naming

Call the product **rocci-browser** (CLI and app). Call the native window the
**preview window** still, per the existing naming decision; the browser is
the process that owns it for multi-target sessions.[^preview-decision]

Avoid: *hub*, *studio*, *workbench* (imply an IDE), *browse* (collides with
`rocci browse`), *plugin host* as the user-facing name (too much ABI
baggage).

## Decision gates

Human approval is required before:

1. Adding a `rocci-browser` workspace member and CLASSES entry (fourth
   user-facing CLI).
2. Treating out-of-process adapters as the approved plugin shape versus a
   static composition binary.
3. Moving window ownership so product `run` defaults to `--no-window` when
   a browser session exists.
4. Authoring picker UI in Rocci instead of host HTML.
5. Native folder dialogs or a third-party plugin marketplace.
6. Encoding `site` / `docs` / `knowledge` as built-in targets in host
   source (they belong in a repo-local registry file).

Until those gates open, do not implement *this* record's host. Phases 1–5 of
the [implementation plan](../plans/rocci-browser.md) already exist in the
tree. TUI removal and an ad-hoc Finder `.app` live in the [macOS app
plan](../plans/rocci-browser-macos-app.md).[^browser-plan][^macos-plan]

## Disposition

Draft and exploratory. The recommended architecture is option D: a base-Rocci
`rocci-browser` host that does not know Rocci apps, Rocdown, or OKF by name;
process adapters; registry of directories; two-stage fuzzy picker (Enter
opens a target, Tab lists documents). It complements today's previewer
immediately and can later own the preview window without collapsing the three
product CLIs. It does not authorize plugins on `rocci run` or `rocdown run`.
The [implementation plan](../plans/rocci-browser.md) sequences that split.
Do not keep a TUI. An ad-hoc **Rocci Browser.app** is a follow-on, not
`rocci bundle`.[^browser-plan][^macos-plan]

[^browser-plan]: Phased host crate, fixture adapter, persistent window, first-party adapters, repo-local registry, warm sessions.
[^macos-plan]: Withdraw TUI; wrap preview() in an ad-hoc Finder .app; do not embed adapters.
[^cli-plan]: Three-CLI split, rejection of plugin hosts on rocci/rocdown, exec-sibling dispatcher deferred.
[^product-boundary]: Approved one-way package edges and product ownership.
[^preview-decision]: Preview window versus overlay chrome versus Dev panel naming.
[^chrome-research]: Overlay HTML/JS versus preview-origin Rocci inspector UI.
[^system-overview]: Current workspace and preview-window contract.
[^language-tooling]: Generic lsp core versus product composition binary.
[^lsp-analyzer]: `DocumentAnalyzer` as an in-process extension point.
[^deps-check]: Mechanical CLASSES forbidding base Rocci → Rocdown/OKF.
[^desktop-readme]: Desktop host has no language-crate dependencies; chrome is assets.
[^preview-rs]: Blocking `preview()`, Home uses `webview.load_url`.
[^desktop-lib]: Persistent `run()` shell with multiple windows.
[^backend]: `RunningBackend` / `ExternalBackend` attach a window to an origin.
[^session]: WindowId and session tokens in rocci-core.
[^window-state]: `~/.rocci/state` geometry persistence.
[^serve-rs]: Shared `preview()` helper; window close stops the child.
[^browse-rs]: Component gallery over `.rocci` files, not a project launcher.
[^cli-readme]: Documented `rocci browse` / `view` / `run` surface.
[^cli-lib]: Shared driver library consumed by product CLIs.
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
[^agents]: New workspace members must be classified in `tools/rocci-ops/src/rocci_ops/workspace_deps.py`.
[^compile-research]: OKF first-open dominated by Roc compile and render.
[^lsp-spec]: JSON-RPC process model for a generic host plus adapters.
