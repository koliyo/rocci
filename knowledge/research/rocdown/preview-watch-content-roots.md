---
type: Research Report
title: Preview watch ignores mounted catalog trees
description: "rocdown view site watches docs/ but the rebuild filter only accepts paths under site/, so mounted Rocdown never rebuilds. view docs stays a first-class catalog. Watch mounts and peers; peers stay link inventory for emit."
tags: [domain/rocdown, concern/tooling, concern/architecture, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:26:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: dev-rs
    resource: ../../../crates/rocci-rocdown/src/dev.rs
    title: Site preview watcher, path_is_relevant, rebuild closure
    author: process:git
    last_modified: 2026-08-31
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: load_site discovers root pages, mounts, and peers
    author: process:git
    last_modified: 2026-09-01
  - id: config-rs
    resource: ../../../crates/rocci-rocdown/src/config.rs
    title: MountConfig versus PeerConfig
    author: process:git
    last_modified: 2026-09-01
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build/mod.rs
    title: BuildSession rebuild_loaded and apply-without-recompile
    author: process:git
    last_modified: 2026-09-01
  - id: watch-loop
    resource: ../../../crates/rocci-cli/src/dev_server/mod.rs
    title: Shared notify watch_loop, output skip, live-reload SSE
    author: process:git
    last_modified: 2026-08-31
  - id: cli-main
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: view dispatches run_site_dev for directories
    author: process:git
    last_modified: 2026-09-02
  - id: site-toml
    resource: ../../../site/rocdown.toml
    title: rocci.dev catalog mounts docs and example-docs
    author: process:git
    last_modified: 2026-09-01
  - id: docs-toml
    resource: ../../../docs/rocdown.toml
    title: Standalone docs catalog peers example-docs and site/project
    author: process:git
    last_modified: 2026-09-01
  - id: readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Mounts emit; peers are link inventory
    author: process:git
    last_modified: 2026-09-01
  - id: preview-doc
    resource: ../../../docs/troubleshooting/preview.rocdown
    title: Live-reload troubleshooting
    author: process:git
    last_modified: 2026-09-01
  - id: rocci-docs
    resource: ../../../crates/rocci-docs/README.md
    title: Example staging tree is not a Rocdown catalog
    author: process:git
    last_modified: 2026-08-29
  - id: pages-hash
    resource: ../../../crates/rocci-rocdown/src/plan/tests.rs
    title: pages_roc stable for body-only edits
    author: process:git
    last_modified: 2026-09-04
  - id: compiler-arch
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Architecture claims watch/serve and live reload shipped
    author: process:cursor
    last_modified: 2026-08-31
  - id: limitations
    resource: ../../status/known-limitations.md
    title: Known limitations on watch and live reload
    author: process:cursor
    last_modified: 2026-09-09
  - id: plan
    resource: ../../plans/rocdown/preview-watch-content-roots.md
    title: Content-root preview watch plan
    author: process:cursor
    last_modified: 2026-09-09
---

# Preview watch ignores mounted catalog trees

## Claim

`rocdown view site` and `rocdown view docs` are two catalogs, not one builder
plus a stub. Composition already exists: `[[mount]]` emits another tree into
this catalog; `[[peer]]` is link inventory only for **emit**. Preview watch
must follow every configured composition tree (mounts **and** peers). Today it
registers mount directories with `notify` and then drops those events in
`path_is_relevant`, so editing mounted `docs/*.rocdown` during `view site`
never rebuilds or live-reloads. Peers are not even added to `watch_paths`.[^dev-rs][^readme][^site-toml]

There is no missing third “site builder.” `rocci-docs` stages example source
into `dist/example-docs`; Rocdown then mounts or peers that tree. Replacing
staging with a mount of `examples/rocci/` would skip the snippet `:include`
shape and is out of bound here.[^rocci-docs][^readme]

Implementation: [preview watch content roots](/plans/rocdown/preview-watch-content-roots.md).[^plan]

## Two catalogs stay

| Command | Root | Emits | Link-only |
| --- | --- | --- | --- |
| `rocdown view site` | `site/` | `site/**`, mounted `../docs`, mounted `../dist/example-docs`, `site/theme` | none |
| `rocdown view docs` | `docs/` | `docs/**` (default crate theme) | `[[peer]]` `../dist/example-docs`, `../site/project` |

`load_site` discovers root pages, then mount pages with a prefix, then indexes
peer pages for routes without adding them to the emit set. `check`/`build`/`view`
all use that loader. Standalone `docs/` exists so the manual can be checked and
previewed without the hybrid rocci.dev chrome, playground, or island
service.[^site-rs][^config-rs][^cli-main]

Nav lists are duplicated on purpose: each catalog owns its navigation. Do not
collapse `docs/rocdown.toml` into `site/` as part of making watch work.

## What watch does today

`view DIR` always goes through `run_site_dev` → `run_with_host_at` →
`serve_static_site`. The shared loop watches configured paths, debounce 200ms,
skips the preview output directory, then calls `load_site` +
`BuildSession::rebuild_loaded`. Successful rebuilds `broadcast` on
`/__rocci/events`; `reload.js` does `location.reload()`.[^dev-rs][^watch-loop][^cli-main]

`run_with_host_at` already appends canonical mount directories (and snippet
roots outside the site root) to `watch_paths`. Peers are not added.[^dev-rs]

The rebuild filter then requires `path.strip_prefix(site_root)`:

- A change under `site/index.rocdown` is relevant.
- A change under `/…/docs/templates/index.rocdown` is watched, then discarded.
- `.git` / `target` / hidden path segments are ignored inside the site root
  only; outside the root everything is false except a stale empty
  `snippet_paths` set cloned before the first rebuild.[^dev-rs]

A `rocdown view site` session that served rocci.dev logged the initial load and
island proxy, then never printed `rocdown: rebuilding` after edits to mounted
docs files. Live-reload SSE never ran because rebuild never ran. That is not a
Datastar island-proxy bug for ordinary `/docs/` HTML: `should_proxy` leaves
CDN GETs on the static tree, and HTML responses inject `reload.js`.[^watch-loop]

`view docs` watches `docs/` itself, so in-root `.rocdown` edits should already
pass the filter. Peers are not registered, so restaging `dist/example-docs` or
editing `site/project` while `view docs` is running never reloads. That is a
gap: `load_site` reindexes peer routes on every rebuild (missing peer dirs are
skipped; links fail `RD2101`). Peer body text is not emitted, but the link
graph and inspector diagnostics are. A full HTML reload when only a peer
changed is cheap once `roc_hash` matches (apply without recompile). That is
not a strong reason to skip peer watch.[^docs-toml][^readme][^site-rs][^build-rs]

## Rebuild cost once events pass

`rebuild_loaded` reloads the whole catalog and re-applies every page. Body-only
edits keep `pages_roc` stable, so a warm `BuildSession` can apply without
`roc` when the renderer hash matches. First native compile of this repo’s site
is on the order of two seconds; subsequent prose edits should skip compile if
watch actually fires.[^build-rs][^pages-hash]

Dirty-page apply (rewrite one HTML file) is not required to make refresh
correct. It is a later performance follow-on, same class as skipped OKF
dirty-page apply.

`[http] service` for `site/` points at `../examples/rocci/standalone/live-counter/LiveCounter.rocci`.
That path is outside `site/`. Island restart uses a fingerprint inside
`sync_island_backend` on each rebuild; without watching the service file,
handler edits do not trigger a rebuild.[^site-toml][^dev-rs]

## What to watch (composed trees)

For a catalog root `R`:

1. `R` recursively (already).
2. Each `[[mount]]` source directory, even when it does not `starts_with(R)`.
3. Each `[[peer]]` source directory that exists (same matcher; skip if missing,
   same as `load_site`).
4. Theme directory when `build.theme` or `R/theme` exists (usually already
   under `R`).
5. Assets directory named by `build.assets`.
6. Configured snippet roots.
7. Parent of `http.service` when that `.rocci` sits outside `R`.

`build` still does not emit peer pages. Preview rebuilds so peer route
inventory stays current. Apply the same extension allow-list (`.rocdown`,
`.md`, `.rocci`, `.roc`, `.css`, images, `rocdown.toml`) and the same `.git` /
`target` / hidden-segment skips to every composed root. Canonicalize notify
paths before `strip_prefix`. Preview `--output` stays excluded by
`watch_loop`.[^readme][^config-rs][^dev-rs]

## Status wording

Architecture and known-limitations currently say watch/serve and live reload
are implemented. The shared server, SSE, and in-root watch are real; mount-aware
relevance is not. Public troubleshooting only mentions `--no-live-reload`.[^compiler-arch][^limitations][^preview-doc]

[^dev-rs]: `watch_paths` includes mounts; `path_is_relevant` returns false unless `strip_prefix(root)` succeeds.
[^site-rs]: Root discovery, mount prefixing, peer indexing without emit.
[^config-rs]: `[[mount]]` versus `[[peer]]` records.
[^build-rs]: Session reuse; `content changed, applying without recompile` when `roc_hash` matches.
[^watch-loop]: `notify` debounce, output-dir skip, `ReloadHub::broadcast`, `reload.js`.
[^cli-main]: Directory `view` is `run_site_dev` for both `site` and `docs`.
[^site-toml]: Mounts `../docs` and `../dist/example-docs`; `http.service` live-counter.
[^docs-toml]: No mounts; peers `../dist/example-docs` and `../site/project`.
[^readme]: Mounts remain a standalone `docs/` plus `build site`; peers are link inventory.
[^preview-doc]: Live reload section is pause-flag only.
[^rocci-docs]: Staging writes `dist/example-docs`; Rocdown does not depend on this crate.
[^pages-hash]: `pages_roc_is_stable_for_body_only_edits` and docs-body variant.
[^compiler-arch]: Shipped-state sentence lists watch/serve and live reload.
[^limitations]: Watch/serve and live reload described as already implemented.
[^plan]: Content-root watch; both catalogs remain; mounts and peers are watch roots.
