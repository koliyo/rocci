---
type: Implementation Plan
title: First-party Rocci chrome library and generation host
description: Extract demonstrated documentation chrome into base-Rocci components, keep Markdown and OKF governance in their owners, and host Roc through a cached native subprocess and an embedded Wasmtime runtime.
tags: [domain/rocci, domain/rocdown, domain/rocci-okf, integration/roc, concern/rendering, concern/performance, concern/architecture, concern/caching]
status: stable
generated: { by: process:cursor, at: 2026-08-18T13:50:00Z }
stale_after: 2026-11-18
authority: normative
owners: [human:nils]
sources:
  - id: research
    resource: ../research/rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell decision
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: generator
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
  - id: theming
    resource: ../architecture/theming.md
    title: Current Rocci theming surfaces
    author: process:okf-phase-4
    last_modified: 2026-08-18
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-18
  - id: ui-readme
    resource: ../../crates/rocci-ui/README.md
    title: rocci-ui view records
    author: process:git
    last_modified: 2026-08-18
  - id: ui-view
    resource: ../../crates/rocci-ui/src/view.rs
    title: Domain-neutral view records
    author: process:git
    last_modified: 2026-08-18
  - id: deps-check
    resource: ../../scripts/check-workspace-deps.py
    title: Workspace dependency-direction check
    author: process:git
    last_modified: 2026-08-18
  - id: docs-rocci
    resource: ../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Rocci documentation widgets
    author: process:git
    last_modified: 2026-08-17
  - id: theme-rocci
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-18
  - id: site-layouts
    resource: ../../site/theme/Layouts.rocci
    title: rocci.dev site layouts
    author: process:git
    last_modified: 2026-08-18
  - id: okf-presentation
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF review HTML renderer
    author: process:git
    last_modified: 2026-08-18
  - id: build-rs
    resource: ../../crates/rocci-rocdown/src/build.rs
    title: Rocdown Roc invocation and watch hash
    author: process:git
    last_modified: 2026-08-18
  - id: datastar-cache
    resource: ../../crates/rocci-cli/src/datastar_asset.rs
    title: Existing ~/.rocci/cache directory and SHA-256 integrity check
    author: process:git
    last_modified: 2026-08-16
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci template crate contract
    author: process:git
    last_modified: 2026-08-17
  - id: roc-wasi
    resource: https://github.com/ostcar/roc-wasi-platform
    title: Roc WASI platform and roc build --target wasm32
    author: human:ostcar
  - id: roc-glue
    resource: https://github.com/roc-lang/roc/blob/main/src/glue/README.md
    title: Roc glue ABI generator
    author: organization:roc-lang
  - id: priority-1
    resource: ../reference/priority-1-review.md
    title: Priority-1 knowledge review checklist
    author: process:okf-phase-6
    last_modified: 2026-08-18
---

# First-party Rocci chrome library and generation host

## Purpose and authority

This plan is exploratory. It does not amend the [Rust catalog / Rocci shell
decision](/decisions/rust-catalog-rocci-shell.md) until a human reviewer
accepts a scope. Evidence for the current pipeline, Roc hosts, caching, and
duplication map lives in the companion [research
record](/research/rocci-components-in-generation.md).[^research][^catalog-shell]

It is the merged conclusion of two parallel drafts. Native subprocess and
Wasmtime hosts are both in scope. Native glue is documented as future
potential. It does not add an architecture record until any of this
pipeline is implemented.

## Problem

Rust still concatenates HTML in places the project already knows how to
express as Rocci: documentation outlines, some navigation, and a leftover
`@docs` renderer that duplicates `DocsComponents.rocci`. The OKF viewer copies
standalone Rocdown's table of contents instead of calling a shared component.
At the same time, moving Markdown or OKF governance into Rocci would fight
existing ownership rules and make cheap preview depend on the Roc
compiler.[^generator][^cli-plan][^okf-presentation]

## Recommended contract

Keep data work in Rust. Author *demonstrated chrome* as Rocci. Compile a
renderer program once. Pass page data in at apply time.

| Layer | Owner | Rendered by |
| --- | --- | --- |
| Catalog, routes, outline headings, nav items | Rust (`rocci-rocdown`, `okf`) | Not HTML |
| Markdown / footnote / code-block HTML | Rust | Fragments or article strings |
| `@docs` widgets | Rocci (`DocsComponents` or extracted twins) | Compiled renderer |
| Shared chrome: outline, breadcrumbs, nav list | Rocci in base Rocci | Compiled renderer |
| Product shell: `<html>`, CSP, reload, metadata | Product theme or product Rust | Product |
| OKF governance (badges, review queue, sources) | `rocci-okf` | Stay in that product; Rocci only if the same product wants it |

This preserves catalog checks without Roc, keeps knowledge records inert, and
avoids a `rocci-okf` → Rocdown edge. Shared files live where both products may
depend: base Rocci.[^deps-check][^cli-plan][^catalog-shell]

## Component library shape

Do not start with a repo-root `components/docs/` tree until a crate owns it.
Put templates next to the view records they consume:

```text
crates/rocci-ui/templates/chrome/
  PageOutline.rocci
  NavList.rocci
  Breadcrumbs.rocci
```

Names follow `PageView` fields rather than `DocumentationNavigationPanel`.
`PageOutline` is the shared "On this page" control. `NavList` is a list of
`NavItemView`. Product layouts compose those parts; they do not share one
monolithic documentation frame. A Rocdown-only `DocumentationShell` that
assembles header, `NavList`, `PageOutline`, and breadcrumbs can live in the
Rocdown theme after the primitives exist.[^ui-view][^theme-rocci]

`DocsComponents.rocci` stays Rocdown-owned until a second consumer needs the
same widgets. Extracting it into `rocci-ui` without an OKF or site caller
repeats the Phase 7 over-extraction that was pruned.[^ui-readme][^docs-rocci]

CSS stays product-owned. Shared components emit stable class hooks
(`rd-toc`, `rd-nav-list`, `rd-breadcrumbs`) or take a `class_name` prefix.
OKF can keep its dark review palette; documentation sites can keep `--ink`
tokens. Sharing markup is not sharing a theme.[^theming]

`rocci-ui` currently has no template compiler dependency. Adding `.rocci`
files there is source ownership only. `rocci-template` already lives in base
Rocci; product CLIs compile the templates. If `rocci-ui` must not grow a
template story, a sibling `rocci-components` crate is the alternative, still
classified as base Rocci in `scripts/check-workspace-deps.py`.[^deps-check]

Do not embed templates with `include_str!` as the v1 integration. Product
builders already stage `.rocci` sources into a Roc workspace. Embedding is
useful later if a release binary must carry the templates without a source
tree.

## Roc hosts

Both in-scope hosts compile the same renderer program. Neither embeds the Roc
compiler. Identity of a renderer is the whole program, not a dynamically
linked `.rocci` module.[^research][^build-rs]

Shared Roc API, independent of host:

```text
render : Str, Str -> Str
```

The first `Str` is serialized `PageView` (or a smaller chrome view). The
second is already-rendered article HTML. The result is HTML. `Html` from
`basic-cli` is not the FFI type; the platform owns encoding.[^research]

### Host A — native subprocess (required)

Rocdown already shells out to `roc build` with `basic-cli` and reruns the
`apply` binary when generated Roc is unchanged, but that binary lives in a
temp workspace and dies with the process.[^build-rs]

Keep this host. Persist the binary through the compiled-artifact cache.
Move `PageView` out of `RocdownPages.roc` so content edits apply without
recompile. stdin/files remain acceptable for the native applicator.

### Host B — embedded Wasmtime (required)

Compile the same renderer with `roc build --target wasm32` to a WASI
module and evaluate `render` in-process through Wasmtime.[^roc-wasi]

This host exists so preview and OKF can apply without spawning `roc` or a
native `apply` child, and so release binaries can ship a prebuilt
`components.wasm` for the first-party chrome library. It still needs `roc`
on the machine that *compiles* the module (or a CI job that seeds the
cache / release artifact).

Prefer a tiny custom wasm platform that imports host functions for string
in/out rather than giving the module a filesystem. Do not reuse
`basic-cli` as the wasm platform. Do not put Wasmtime in `rocci-core`
(that crate is configuration and session contracts). A new base-Rocci
crate such as `rocci-roc-host`, classified in
`scripts/check-workspace-deps.py`, owns cache lookup, native spawn, and an
optional `wasmtime` feature so catalog-only tests do not link the
engine.[^deps-check]

Both hosts are delivery, not a bake-off. Measure compile and apply cost
after they exist; do not gate the work on unmeasured sub-millisecond
budgets.

### Host C — native glue and `dlopen` (out of scope, document as future)

`roc glue` plus `roc build --lib` would load the same `render` function as
a native shared library with compiler-committed `RocStr` layout. That is a
third *compiled* artifact of the same generated Roc, not a compiler
embedding.[^roc-glue][^research]

Leave it out of this delivery. Record it as the native in-process follow-on
if Wasmtime apply cost or WASI limits become the bottleneck. It shares the
custom-platform requirement with host B and inherits ABI, allocator, and
macOS signing costs that wasm avoids.

### Rejected

- Embedding the Roc compiler as a Rust crate.
- Dynamically linking separately compiled Rocci modules.
- Interpreting `.rocci` in Rust to avoid Roc.
- Encoding Markdown prose as Roc constructors.
- Reopening a `rocci-okf` dependency on `rocci-rocdown` templates.
- Publishing a descriptive architecture record for this pipeline before any
  of it is implemented.
- Moving OKF governance HTML into `rocci-ui` as a first delivery.
- Byte-identical HTML as an exit gate. Shared components may change class
  names (`.outline` versus `.rd-toc`); tests should assert structure and
  accessibility, not frozen strings from the Rust concatenators.

## Two-tier renderer cache

Today's watch hash covers generated Roc bytes in memory for one process. It
does not persist generated Roc, does not persist the binary, and treats
page views as Roc source.[^build-rs]

Datastar already uses `~/.rocci/cache` with SHA-256 integrity files. Renderer
caching is a second named subtree of that directory, overridable with
`ROCCI_CACHE`.[^datastar-cache]

```text
~/.rocci/cache/
  roc/<gen-hash>/
    modules/*.roc
    maps/*.map
    fingerprints.json
    manifest.json
  renderers/<compile-hash>/
    apply                 # host A only: native executable
    components.wasm       # host B only: wasm32 module
    artifact.sha256
    fingerprints.json
    manifest.json
```

Each `compile-hash` includes the target, so a directory holds either `apply`
or `components.wasm`, never both as one identity.

Hash is identity. Timestamps are metadata and a hashing fast path. Never
key a cache entry on mtime, wall clock, or "file looks recent."

### Tier 1 — generated Roc

`rocci-template` already lowers `.rocci` to Roc in Rust and does not invoke
`roc`. That output is expensive enough to persist when the same theme and
chrome modules recur across watch runs and workspaces.[^template-readme]

`gen-hash` is SHA-256 of a canonical bundle:

- `rocci-template` version and lowering options
- each renderer `.rocci` / runtime `.roc` file, sorted by module name, as
  raw bytes
- platform Roc headers that are copied into the generated workspace

Do not include Markdown, YAML, catalog graphs, `PageView` JSON, or
product CSS unless that CSS is inlined into generated Roc.

On hit, stage the cached `.roc` files into the Roc workspace and skip
lowering. Host A and host B share this tier: wasm does not re-lower.

### Tier 2 — compiled artifacts

`compile-hash` is SHA-256 of:

- `gen-hash`
- full `roc version` output (nightlies can share a marketing version)
- target (`native:<triple>` or `wasm32`)
- opt level
- platform identity (hash of the platform host sources or a pinned name
  and version)
- `rocci-roc-host` crate version

Native and wasm are different `compile-hash` values and different artifacts
of the same `gen-hash`. A cache miss on wasm must not delete a native hit.

On hit, skip `roc build`. Host A executes `apply`; host B instantiates
`components.wasm`. Verify `artifact.sha256` before use; mismatch is a miss
and the directory is discarded.

Do not expect two products that merely import `PageOutline` to share a
compiled entry. They share a compiled entry only when the whole renderer
program, target, and platform match.

### Fingerprints and timestamps

Each cache directory stores per-input fingerprints:

```text
{ "path": "PageOutline.rocci", "len": 4120, "mtime_ns": 1755510000000000000, "sha256": "…" }
```

Lookup procedure:

1. If every stored `(path, len, mtime_ns)` still matches the filesystem,
   reuse the stored file SHA-256 values and do not re-read file bodies.
2. If any fingerprint mismatches, re-read those files, recompute SHA-256,
   and rewrite fingerprints.
3. Recompute `gen-hash` / `compile-hash` from the file hashes and toolchain
   strings. Hit or miss follows that content hash, never mtime alone.
4. After a hit or successful fill, set `last_used_at` to now. `created_at`
   is written once when the directory is filled. Both are RFC 3339 in
   `manifest.json` for LRU eviction and diagnostics, not for identity.
5. Write artifacts to `*.tmp` and rename; write `manifest.json` last so a
   crash cannot publish a half-written renderer. Readers treat a directory
   without a valid manifest as a miss.

mtime is a hint that hashing can be skipped. It is not a freshness signal.
Git checkouts, copy tools, and coarse filesystems can leave mtime unchanged
or lie; the SHA-256 path is the authority.

Optional eviction: delete unused `roc/` and `renderers/` entries after a
configurable idle age using `last_used_at`. Never expire because
`created_at` is old while the content hash still matches.

### What must move out of generated Roc

`RocdownPages.roc` currently embeds page views, so title, sidebar, and
outline edits change the Roc hash and force both tiers to miss.[^build-rs]
Pass views and article fragments as apply-time data (JSON, stdin, or
staging files that the wasm host reads through imports). Until that
happens, a persistent cache only helps theme-source stability, not watch
edits.

## Sharing with the OKF viewer

Without Roc, lift `toc.js` and the `.rd-toc` class contract into `rocci-ui`
or a tiny chrome asset, and have both standalone Rocdown and OKF include
them. That removes the copied script and CSS while leaving OKF Roc-free.

With a Rocci renderer, compile `PageOutline.rocci` into the Rocdown
applicator and into an OKF renderer only if OKF is allowed a Rocci
build-time dependency. Preview stays Roc-free at run time if host B's
`components.wasm` was precompiled into the `rocci-okf` binary or loaded
from `~/.rocci/cache`.

OKF-specific concept headers stay in `rocci-okf`. If they later become Rocci,
they are `crates/rocci-okf/templates/`, not shared docs chrome.[^cli-plan][^okf-presentation]

## Delivery phases

### 0. Freeze the layer map

Record that Markdown fragments, catalog data, and OKF governance are not in
scope. Record that shared chrome is outline, nav list, and breadcrumbs.
Leave product shells product-owned.

### 1. Delete the leftover Rust `@docs` painter for site output

Keep Rust Markdown fragment rendering. Stop treating `docs.rs::render_docs` as
a production HTML path once tests assert through `plan_segments` plus generated
Roc (or a golden HTML fixture from the Rocci path). `article_html` used for
inspection can remain a Markdown-only projection.

### 2. Deduplicate the two Rocci documentation layouts

Make `RocdownTheme.rocci` and `site/theme/Layouts.rocci` call shared
`PageOutline`, `NavList`, and `Breadcrumbs` modules. No OKF change yet. This
is the first real library consumer pair.[^theme-rocci][^site-layouts]

Do not require custom themes to override individual chrome files in v1. A
theme that replaces the whole shell already works. Per-component override is
a later theming question, not a cache prerequisite.

### 3. Share standalone and OKF table-of-contents assets

Move the duplicated `toc.js` and the `.rd-toc` class contract to a base-Rocci
asset. Point standalone lowering and `rocci-okf` at it. Optionally replace
OKF's string-built `<nav>` with the Rocci `PageOutline` once a renderer host
exists.

### 4. Stop baking page views into generated Roc and persist the two-tier cache

Pass `PageView` and article fragments as apply-time data. Implement the
`roc/` and `renderers/` cache layout, content hashes, fingerprint fast
path, artifact integrity, and atomic manifest writes in `rocci-roc-host`.
Point host A at the native `apply` artifact.

### 5. Ship host B (Wasmtime)

Add a tiny wasm platform around the same `render` function. Compile with
`roc build --target wasm32`, store `components.wasm` under the matching
`compile-hash`, and apply through Wasmtime. Seed CI with a first-party
chrome wasm so `rocdown` / `rocci-okf` release builds can run host B
without a local `roc` for that program. Measure native vs wasm apply; do
not delete host A.

### 6. Decide whether OKF gets a Rocci renderer

Only after phases 2–5. Options: remain Rust HTML with shared assets (phase
3), or apply `PageOutline` through host A or host B. Accepting Roc at
`rocci-okf` *build* time (to compile or to vendor wasm) is a product
decision; accepting Roc on PATH for every `rocci-okf run` is a steeper
one. Host B is the path that keeps preview Roc-free at run time.

### Future potential — native glue

After hosts A and B exist, a `roc glue` + `roc build --lib` host can reuse
tier-1 generated Roc and add a third compiled artifact. Do not start that
work in this plan.

## Open questions that change the plan

1. May `rocci-okf` depend on Rocci templates at application build time, or
   must preview remain compilable on a machine without `roc`?
2. Should shared chrome look the same (shared CSS) or only share structure
   (shared markup, product CSS)?
3. Is `rocci-ui` the right owner for `.rocci` files, or should a new
   `rocci-components` crate hold them so view records stay Rust-only?
4. Should standalone Rocdown keep emitting TOC during lowering, or should
   single-page `rocdown run` use the same compiled shell as sites?
5. Should host B use WASI filesystem APIs or only host-imported string
   functions for `render`?

## Validation

- Crate tests for `PageView` construction stay Roc-free.
- Template compile tests cover the new `.rocci` modules.
- Cache tests cover hash identity, fingerprint skip, mtime-lie rehash,
  artifact checksum mismatch, and atomic manifest publication without
  invoking Roc.
- `scripts/check-workspace-deps.py` still forbids OKF → Rocdown.
- `rocci-okf check knowledge --profile rocci` after knowledge edits.
- Roc-gated tests for host A and host B apply the same fixture view and
  compare structure; they do not require byte-identical HTML with the
  current Rust concatenators.
- Do not log this plan complete until CI and Knowledge workflows succeed on
  the landing revision.[^priority-1]

## Knowledge follow-through when accepted

If a reviewer accepts the recommended contract, amend
[rust-catalog-rocci-shell](/decisions/rust-catalog-rocci-shell.md) to say
visible *chrome components* may live in base Rocci while product shells stay
product-owned; update
[rocdown-documentation-compiler](/architecture/rocdown-documentation-compiler.md)
and [theming](/architecture/theming.md) for the shared outline/nav modules;
and leave OKF governance in the CLI plan until phase 5 lands. Do not add a
descriptive architecture record before implementation.

[^research]: Evidence for feasibility, glue versus compiler, caching, and duplication.
[^catalog-shell]: Current normative catalog-versus-shell split.
[^generator]: Fragment bridge and once-compiled theme.
[^theming]: Product-owned CSS surfaces.
[^cli-plan]: OKF presentation stays in `rocci-okf`; no Rocdown adapter.
[^ui-readme]: View records are the present shared primitive.
[^ui-view]: Shared primitives should match shipped `PageView` fields.
[^deps-check]: Frozen package-class edges.
[^docs-rocci]: Existing Rocci `@docs` widgets.
[^theme-rocci]: First documentation shell.
[^site-layouts]: Second documentation shell over the same view records.
[^okf-presentation]: Rust TOC and governance HTML.
[^build-rs]: Current subprocess Roc compile and in-process watch hash.
[^datastar-cache]: Existing `~/.rocci/cache` root, `ROCCI_CACHE`, and SHA-256 sidecar.
[^template-readme]: Template crate lowers `.rocci` to Roc without invoking `roc`.
[^roc-wasi]: `roc build --target wasm32` produces a WASI module runnable in Wasmtime.
[^roc-glue]: Glue generates ABI bindings for a compiled app; it is not in this delivery.
[^priority-1]: Lifecycle and verification rules for later record amendments.
