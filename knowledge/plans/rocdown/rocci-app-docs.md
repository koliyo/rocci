---
type: Implementation Plan
title: Documentation generator for Rocci applications
description: "Own application documentation generation in crates/rocci-docs: stage colocated Rocdown plus a full highlighted source tree, mount it on rocci.dev, and host only selected apps as separate live origins. Skip Rocdown examples. Align published source with the semantic handler syntax."
tags: [domain/rocci, domain/rocdown, concern/publication, concern/developer-experience, concern/rendering, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-24T11:03:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: handlers
    resource: ../action-handler-syntax.md
    title: Semantic view, patch, command, and live handlers
    author: process:cursor
    last_modified: 2026-08-21
  - id: generator
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator architecture
    author: process:cursor
    last_modified: 2026-08-19
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: site-plan
    resource: ../rocci-dev-site.md
    title: rocci.dev site architecture and Rocdown evolution
    author: process:codex
    last_modified: 2026-08-18
  - id: publish-plan
    resource: ../rocci-dev-publish.md
    title: Deploy rocci.dev with Cloudflare, a small VPS, and CI
    author: process:cursor
    last_modified: 2026-08-21
  - id: example-origins
    resource: ../site/publish-example-origins.md
    title: Publish live examples on id.examples.rocci.dev
    author: process:cursor
    last_modified: 2026-08-24
  - id: efficient
    resource: ../efficient-publishing.md
    title: Efficient publishing of pre-built sites and Rocci apps
    author: process:cursor
    last_modified: 2026-08-20
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: Unified rocci.dev site configuration
    author: process:git
    last_modified: 2026-08-21
  - id: examples-index
    resource: ../../../docs/examples/index.rocdown
    title: Current hand-maintained examples index
    author: process:git
    last_modified: 2026-08-20
  - id: article
    resource: ../../../crates/rocci-rocdown/src/article.rs
    title: Article renderer with tok-* fence highlighting
    author: process:git
    last_modified: 2026-08-21
  - id: include
    resource: ../../../crates/rocci-rocdown/src/docs.rs
    title: :include resolution, snippet roots, and code-literal includes
    author: process:git
    last_modified: 2026-08-21
  - id: rocdown-site-ref
    resource: ../../../docs/reference/rocdown-site.rocdown
    title: Published snippet-root and include contract
    author: process:git
    last_modified: 2026-08-21
  - id: highlight
    resource: ../../../crates/rocci-highlight/src/lib.rs
    title: rocci-highlight public highlight_source API
    author: process:git
    last_modified: 2026-08-20
  - id: highlight-lang
    resource: ../../../crates/rocci-highlight/src/language.rs
    title: LanguageId parse and is_highlighted set
    author: process:git
    last_modified: 2026-08-17
  - id: workspace
    resource: ../../../Cargo.toml
    title: Cargo workspace membership
    author: process:git
    last_modified: 2026-08-21
  - id: workspace-deps
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace package classes and dependency direction
    author: process:git
    last_modified: 2026-08-21
  - id: agents
    resource: ../../../AGENTS.md
    title: Owning-layer table for workspace changes
    author: process:git
    last_modified: 2026-08-21
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: rocci CLI including build --release
    author: process:git
    last_modified: 2026-08-21
  - id: bundle
    resource: ../../../crates/rocci-cli/src/bundle.rs
    title: Linux server package from a Rocci app
    author: process:git
    last_modified: 2026-08-20
  - id: docker
    resource: ../../../docker/README.md
    title: Static, hybrid, and Rocci-app Docker hosting
    author: process:cursor
    last_modified: 2026-08-21
  - id: compose-app
    resource: ../../../docker/compose.app.yml
    title: Pre-built Rocci app Compose file
    author: process:git
    last_modified: 2026-08-20
  - id: caddy
    resource: ../../../docker/cdn/Caddyfile
    title: Hybrid Caddy reverse proxy for site islands
    author: process:git
    last_modified: 2026-08-21
  - id: site-workflow
    resource: ../../../.github/workflows/site.yml
    title: Site package and origin deploy workflow
    author: process:git
    last_modified: 2026-08-21
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Standalone one-shot patch counter
    author: process:git
    last_modified: 2026-08-20
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Live command and stream counter
    author: process:git
    last_modified: 2026-08-21
  - id: datastar-readme
    resource: ../../../examples/rocci/custom/datastar/README.md
    title: Datastar gallery local-run README
    author: process:git
    last_modified: 2026-08-20
---

# Documentation generator for Rocci applications

## Purpose and authority

rocci.dev should publish the repository's Rocci example apps as first-class
pages: authored documentation, a complete highlighted source tree, and—only
where the catalog opts in—a running backend. Today the public examples page
is a table of local `rocci run` commands, READMEs live beside the apps, and
`:include` from `docs/` cannot see `examples/` because the site has no snippet
roots.[^examples-index][^site-config][^include][^rocdown-site-ref]

This is an exploratory implementation plan. Writing it does not approve a
fourth author-facing product CLI, a second site engine, or public hosting of
every example.

Published source follows the semantic handler syntax (`@view`, `@patch`,
`@command`, `@live`). Do not publish `@on` as current. That cutover is the
[semantic handler plan](/plans/rocci/action-handler-syntax.md); this plan consumes it and
does not re-decide the nouns.[^handlers]

## Goal

An author colocates Rocdown with a Rocci app. The `rocci-docs` crate produces
a documentation tree the existing Rocdown site can mount. Every published file
in that app appears as highlighted source. Some apps are docs-only; a small
explicit set is packaged and reverse-proxied as separate live origins.

A reader on rocci.dev can open `/examples/counter/`, read the tutorial, browse
the full `Counter.rocci` with `tok-*` highlighting, and—when the catalog says
`hosting = "live"`—follow **Open live demo** to a dedicated hostname that
serves that app.

## Recommendation

1. **Own generation in `crates/rocci-docs`.** Classify it with base Rocci in
   `workspace_deps.py` in the same change that adds the workspace member.
   The crate inventories apps, selects published files, and writes a staging
   tree of `.rocdown`. It does not render the site, compile Roc, or package
   servers. Binary name: `rocci-docs` (workspace tool, like `rocci-ungram`,
   not a fourth product CLI beside `rocci` / `rocdown` / `rocci-okf`).[^workspace][^workspace-deps][^agents]
2. **Do not fork the Rocdown site engine.** Rocdown owns catalog, routes,
   article HTML, and chrome. Generated pages are ordinary `.rocdown` that
   `:include` staged source, so highlighting stays on the existing article
   path.[^catalog-shell][^generator][^article]
3. **Colocate authored docs with the app.** `index.rocdown` sits next to the
   entry `.rocci` (and other pages under the same directory). GitHub READMEs
   remain short local-run notes; they are not the published tutorial.
4. **Always emit a full source tree.** Authors write prose and optional
   excerpt `:include`s. They do not paste whole files. The generator inventories
   published files and writes one source page per file plus a source index.
5. **Host live apps on their own origin, not under a path prefix.** Rocci apps
   own `/`, `/actions/`, and `/sse`. The site hybrid Caddy already uses those
   paths for the home-page island. Do not invent a base-path rewrite in this
   plan.[^caddy][^docker]
6. **Skip Rocdown examples.** `examples/rocdown/**` stay on the hand index as
   “run locally” until a later generator. The hybrid home counter is a site
   island, not an app-docs target.

## Out of bound

- A documentation generator for Rocdown sites, pages, or hybrid islands.
- A second article renderer, highlighter, or site catalog inside `rocci-docs`.
- `rocci-docs` depending on `rocci-rocdown` or `rocci-cli`; `rocci-rocdown`
  depending on `rocci-docs` or `rocci-cli`.
- A fourth author-facing product CLI or plugins on `rocci` / `rocdown`.
- Path-prefix mounting (`/examples/counter/app/`) or rewriting Datastar URLs.
- Hosting every example, or hosting snake on the public origin.
- Changing handler syntax, Datastar, SQLite, or live SSE policy.
- Making the whole of rocci.dev a dynamic Rocci application.
- Committing generated docs trees or treating `dist/` as source.
- JavaScript or TOML token highlighting as a publish gate.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Ownership | `rocci-docs` inventories apps and writes the staging tree; Rocdown mounts and renders pages. Base Rocci must not depend on Rocdown.[^catalog-shell][^workspace-deps] |
| Authored docs | Tutorial prose is `.rocdown`, not README-as-site. Attached Roc `## ` docs on top-level `@` declarations are staged onto source pages. |
| Highlighting | `.rocci` / `.roc` / HTML / CSS use `rocci-highlight` `tok-*` spans already used by fences and `:include`.[^article][^highlight] |
| Include safety | Generated includes never use `..`, NUL, or absolute paths; staged files sit inside allowed snippet roots.[^include][^rocdown-site-ref] |
| Syntax | Published example source uses `@view` / `@patch` / `@command` / `@live`; `@on` remains only in removal diagnostics and historical records.[^handlers] |
| Hosting default | `docs` unless the catalog explicitly sets `live`. |
| Live isolation | A live example is its own process and hostname; it does not share the site island `/actions/` or `/sse`.[^caddy] |
| Artifacts | Failed example docs or app builds must not replace a previously valid `dist/` or origin release.[^efficient] |
| Tests | Inventory, staging, and catalog checks do not require Roc or a server. |

## Decision: catalog, staging tree, and live hostnames

### Catalog

One workspace catalog, `examples/rocci/apps.toml`, lists every Rocci app that
may be published. Discovery is not “every directory under `examples/rocci`”.

```toml
[[app]]
id = "counter"
path = "standalone/counter"
title = "Counter"
summary = "SQLite document plus one-shot HTML patches"
entry = "Counter.rocci"
hosting = "docs"

[[app]]
id = "live-counter"
path = "standalone/live-counter"
title = "Live counter"
summary = "Commands plus a shared @live stream"
entry = "LiveCounter.rocci"
hosting = "live"

[[app]]
id = "styling"
path = "standalone/styling"
title = "Styling"
summary = "File-level and component-scoped CSS"
entry = "Styling.rocci"
hosting = "docs"

[[app]]
id = "datastar"
path = "custom/datastar"
title = "Datastar gallery"
summary = "Search, edit, todos, tabs, and validation"
entry = "."
hosting = "live"

[[app]]
id = "snake"
path = "custom/snake"
title = "Snake"
summary = "Custom main.roc multiplayer stress demo"
entry = "."
hosting = "docs"
```

`handler-matrix` is added when that example exists in the handler plan. `id`
is the URL slug. `entry` is a `.rocci` file or `.` for an app directory.
`hosting` is `docs` or `live` only.

### Staging tree

`rocci-docs --catalog examples/rocci/apps.toml --output <tmp>/example-docs`
writes a Rocdown content tree, not HTML:

```text
example-docs/
  index.rocdown                 # generated app index
  counter/
    index.rocdown               # copy of authored docs
    source/
      index.rocdown             # generated file list
      Counter.rocci.rocdown     # :include of staged source
    snippets/
      Counter.rocci             # copy used by :include
  live-counter/
    ...
```

Authored pages copy from the app directory. The generator also copies every
published source file into that app's `snippets/` so includes stay inside the
output tree. Site config mounts this tree once:

```toml
[[mount]]
source = "../dist/example-docs"
prefix = "examples"
layout = "docs"
```

Canonical routes are `/examples/<id>/` and `/examples/<id>/source/<file>/`.
Keep `/docs/examples/` as an alias of `/examples/` during the move.[^site-plan]

### Published file inventory

Include:

- `*.rocci`, authored `*.roc`, `rocci.toml`
- `assets/**` files that are not empty keepers
- other explicit `[[app.files]]` entries if a later app needs them

Exclude:

- `generated/`, `target/`, `dist/`, `.git/`
- `*.db`, `.gitkeep`, editor swap files
- authored `*.rocdown` (those are pages, not source listings)
- `README.md` (local run notes, not the published tutorial)

A `publish = true` app with no `index.rocdown` is an error. Extra Rocdown
pages in the app directory become extra docs routes under `/examples/<id>/`.

### Highlighting

`:include[path: "Counter.rocci"]` already becomes a Markdown code block whose
info string is the extension; the article renderer highlights `rocci` and
`roc`.[^include][^article] The generator's job is inventory and page emission,
not a parallel highlighter.

`.js` and `.toml` currently parse as languages that `is_highlighted()`
rejects, so they render as escaped plaintext with a language class. That is
acceptable for v1 (snake's `snake-input.js`, gallery `rocci.toml`). Do not
block publish on new highlighters.[^highlight-lang]

### Live origins

| App | Hosting | Reason |
| --- | --- | --- |
| counter | docs | Tutorial source; home already has a live island counter.[^counter] |
| live-counter | live | Two-tab shared stream is the public CQRS proof.[^live-counter] |
| styling | docs | CSS isolation; no shared backend story. |
| datastar | live | Multi-page gallery is the interaction showcase.[^datastar-readme] |
| snake | docs | Stress demo with custom JS; not a public origin tenant. |
| handler-matrix | docs | Method matrix is for local `curl`; do not expose it publicly. |

Live hostname pattern: `<id>.examples.rocci.dev` (staging:
`<id>.examples.staging.rocci.dev`). Caddy on the VPS routes by `Host` to that
app container. Docs pages link out; they do not reverse-proxy the app through
`rocci.dev/examples/<id>/`. Serving those names, the site Launch control, and
catalog `site` inclusion are the follow-on
[publish example origins](/plans/site/publish-example-origins.md) plan. This plan packaged
binaries and Host Caddy; it does not flip public advertising.[^example-origins]

Each live app is `rocci build --release --target x64musl` plus the existing
slim app image, with its own SQLite volume. Cap live apps at the catalog; the
Cost-Optimized VPS is not a fleet.[^compose-app][^publish-plan][^cli-readme]

## Phase 0 — Freeze catalog and inventory

**Bound**

- No site routes, no `rocci-docs` crate, no origin hostnames.
- Use the current example tree plus the planned handler-matrix id.

**Work**

1. Check in `examples/rocci/apps.toml` with the table above (`handler-matrix`
   omitted until that directory exists).
2. Freeze inventory include/exclude rules and the staging-tree shape in a
   fixture expectation (tests may come in Phase 1).
3. Record that published source waits on the handler-syntax cutover; this
   phase does not convert `.rocci` files.

**Exit**

- The catalog file is valid TOML with unique `id`s, repo-relative `path`s, and
  only `docs` | `live` hosting values.
- `cargo run -q -p rocci-okf -- check knowledge --profile rocci` still passes
  after this plan lands.

## Phase 1 — `rocci-docs` crate and staging command

**Bound**

- Owning crate: `crates/rocci-docs` (library plus `rocci-docs` binary).
- Add the workspace member and classify it under `base-rocci` in
  `workspace_deps.py` in the same change.[^workspace][^workspace-deps][^agents]
- Allowed workspace deps: `rocci-core` (optional), `rocci-highlight` only if
  a test needs spans; default staging does not highlight. Forbidden: any
  Rocdown or `rocci-cli` dependency.
- Do not mount the tree on `site/` yet.
- Do not compile Roc or package servers.

**Work**

1. Scaffold `crates/rocci-docs` with `rocci-docs --catalog PATH --output DIR`.
2. Parse the catalog; error on duplicate ids, missing paths, unknown hosting,
   or `publish`-implied apps without `index.rocdown`.
3. Inventory published files; copy authored `.rocdown` and snippets into the
   staging tree; emit per-file source pages and a per-app source index.
4. Emit the generated `/examples/` index from catalog metadata (title,
   summary, hosting badge, link to docs and source).
5. Tests cover include/exclude, missing docs, duplicate ids, and stable
   relative include paths with no `..`.

**Exit**

```sh
cargo test -p rocci-docs
cargo fmt --all -- --check
```

- A fixture app with one `.rocci` and `index.rocdown` produces the expected
  staging tree.
- Output is deterministic given the same catalog and sources.
- Workspace dep check classifies `rocci-docs` as base Rocci.

## Phase 2 — Highlighted source through Rocdown

**Bound**

- Consume the staging tree with `rocdown check` / `rocdown build` in a
  temporary site fixture. Do not change rocci.dev nav yet.

**Work**

1. Point a test `rocdown.toml` at the Phase 1 output with a snippets root
   inside that tree if needed; default includes of staged `snippets/` should
   already resolve.
2. Prove a built source page for `Counter.rocci` (or the fixture) contains
   `tok-*` spans for `@component` / `@view` (or `@on` until the handler
   cutover lands, then `@view`).
3. Prove a docs-only catalog app is present and a non-catalog directory is
   absent.
4. Keep these tests Roc-free where `rocdown check` is enough; full `build` of
   the tiny fixture may use the renderer cache.

**Exit**

```sh
cargo test -p rocci-docs -p rocci-rocdown
```

- Built HTML for a generated source page includes highlighted Rocci, not a
  plain escaped dump of a `.rocci` file.

## Phase 3 — Mount on rocci.dev

**Bound**

- Static docs only. No live hostnames, no extra app containers.
- Do not generate Rocdown-example pages.

**Work**

1. Mount `dist/example-docs` (or the package-time equivalent) at prefix
   `examples` with the docs layout.[^site-config]
2. Replace the hand table in `docs/examples/index.rocdown` with a short
   pointer to `/examples/` plus the still-uncovered Rocdown examples as local
   run commands.[^examples-index]
3. Add `/examples/` to site nav. Alias `/docs/examples/` to `/examples/`.
4. Document that `rocdown package site` / local site preview requires
   `rocci-docs` first (or a `rocci-ops` wrapper that runs both). `rocci-rocdown`
   must not import `rocci-docs` or `rocci-cli`.
5. Add `examples/**` to `site.yml` path filters so catalog and app-doc edits
   package.[^site-workflow]
6. Add `rocci-docs` to the AGENTS.md owning-layer table: application
   documentation staging.

**Exit**

```sh
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
cargo run -q -p rocci-rocdown-cli -- check site
```

- `check site` sees `/examples/counter/` (and siblings) after docs generation.
- Nav contains Examples. No generated files are committed.

## Phase 4 — Author the first app docs

**Bound**

- Authored Rocdown for Rocci apps only.
- Published snippets and full source must use semantic handlers. If that
  cutover is not on the branch, this phase waits or lands with it.

**Work**

1. Write `index.rocdown` for counter, live-counter, and styling (and
   handler-matrix when it exists). Cover what the app demonstrates, how to
   run it locally, and—for live-catalog apps—what the hosted demo will do.
2. Optional excerpt `:include`s of named regions in the tutorial; the
   generator still emits the full file under `source/`.
3. Shrink each README to local `rocci run` / smoke `curl` and a link to the
   published path.
4. Author datastar and snake docs in the same shape; snake remains docs-only.
5. Inventory: no published example page teaches `@on`.

**Exit**

- Each catalog app has `index.rocdown`.
- `rocci-docs` plus `rocdown check site` succeeds.
- `rg` over `examples/rocci/**/*.rocdown` finds no `@on:` used as current
  syntax.

## Phase 5 — Live example origins

**Bound**

- Only catalog rows with `hosting = "live"` (live-counter, datastar).
- Reuse `rocci build --release --target x64musl` and the slim app image.
  Do not put these processes behind the site island `/actions/` proxy.[^docker][^bundle]

**Work**

1. Origin Compose grows one app service per live catalog entry, each with its
   own SQLite volume and health check.
2. Caddy routes `<id>.examples.rocci.dev` (and staging) by `Host` to that
   service. Cloudflare DNS/Tunnel for those hostnames is operator work listed
   here, not a product feature.[^publish-plan]
3. Generated overview pages include an **Open live demo** link when
   `hosting = "live"`, omitted for `docs`.
4. Docs-only apps are absent from the live package list even if someone runs
   `rocci build --release` locally.
5. Smoke: live-counter two-tab update through the example hostname; datastar
   gallery `/search` 200; site `/actions/counter/*` still hits the home
   island, not the gallery.

**Exit**

- Live catalog ids have musl binaries in the site package artifact set.
- Docs-only ids do not.
- A focused Compose/Caddy fixture (or documented origin file) shows Host
  routing without stealing `/actions/` from the hybrid site.

## Phase 6 — Package, CI, and public contract

**Bound**

- Temporary output directories. Failed package must not publish.

**Work and exit commands**

1. `rocci-ops` / `site.yml` sequence:

   ```text
   rocci-docs --catalog examples/rocci/apps.toml --output dist/example-docs
   rocci build --release <each live app> --target x64musl
   rocdown package site --target x64musl
   ```

2. Language and site checks:

   ```sh
   cargo test -p rocci-docs -p rocci-rocdown -p rocci-rocdown-cli
   cargo fmt --all -- --check
   cargo run -q -p rocci-okf -- check knowledge --profile rocci
   ```

3. Update the `rocci-docs` crate README, Rocdown site reference (mount of
   generated examples), examples index, and docker origin README. Mark live
   hostnames as planned until a staging deploy has served them.

4. Confirm `site.yml` path filters include `examples/**`.

The phase exits when a local package produces the examples mount, highlighted
source pages, and live binaries only for `hosting = "live"`, and when
knowledge check reports no new errors. Report lifecycle warnings separately.

## Roll-forward and rollback

Land catalog, `rocci-docs`, site mount, and authored pages before live
hostnames. If highlighting or mount integration fails, keep the hand examples
index and drop the mount; do not ship unhighlighted dumped source as the
public contract. Live origins roll forward independently: a live-container
failure must not take down `rocci.dev` HTML.

Once handler syntax has cut over, regenerate the staging tree; do not keep a
mixed `@on` / `@view` published examples section.

[^workspace]: New members must land in the root workspace list in the same change.
[^workspace-deps]: CI classifies every workspace package; base Rocci must not depend on Rocdown.
[^agents]: Owning-layer table must name `rocci-docs` when the crate exists.
[^handlers]: Semantic declarations replace `@on`; example conversion is Phase 4 of that plan.
[^counter]: Standalone one-shot counter is the first-app tutorial; keep it docs-only on the public origin.
[^live-counter]: Live-counter is the two-window shared-stream example.
[^datastar-readme]: The gallery README already lists the five Datastar page ports and local run commands.
[^generator]: Rocdown already discovers pages, renders articles, and applies one shell.
[^catalog-shell]: Rust owns catalog and artifacts; Rocci owns visible chrome.
[^site-plan]: Examples were sketched as `/docs/examples/`; this plan promotes them to `/examples/`.
[^publish-plan]: Origin is hybrid Caddy plus packaged artifacts on a small VPS.
[^example-origins]: Follow-on for serving `<id>.examples.rocci.dev`, Launch, and catalog `site`.
[^efficient]: Build once; failed output must not replace a good tree.
[^site-config]: `site/` mounts `../docs`; it does not yet mount example apps.
[^examples-index]: Current public examples page is a run-command table.
[^article]: Fences and code includes already emit `tok-*` for Rocci and Roc.
[^include]: `:include` of non-Rocdown files becomes a code block; paths cannot escape snippet roots.
[^rocdown-site-ref]: Snippet roots and the no-`..` include rule are published.
[^highlight]: `highlight_source` is the shared span API.
[^highlight-lang]: `is_highlighted` is Roc, HTML, CSS, Rocci, Rocdown, Markdown only.
[^cli-readme]: `rocci build --release` already packages a Linux server.
[^bundle]: App packaging is distinct from macOS `.app` bundles.
[^docker]: Slim app Compose already exists; it publishes a whole origin, not a path.
[^compose-app]: One app process plus SQLite volume per Compose project today.
[^caddy]: Site hybrid Caddy sends `/actions/*`, `/sse`, and `/health` to islands.
[^site-workflow]: Site workflow path filters currently omit `examples/**`.
