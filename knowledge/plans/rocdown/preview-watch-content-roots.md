---
type: Implementation Plan
title: Watch composed catalog trees for Rocdown preview
description: "Introduce ContentRoots from mount, peer, theme, assets, and service paths so view site rebuilds mounted docs and view docs rebuilds on peer and in-root edits. Do not collapse the two catalogs."
tags: [domain/rocdown, concern/tooling, concern/architecture, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:26:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocdown/preview-watch-content-roots.md
    title: Preview watch ignores mounted catalog trees
    author: process:cursor
    last_modified: 2026-09-09
  - id: dev-rs
    resource: ../../../crates/rocci-rocdown/src/dev.rs
    title: path_is_relevant and watch_paths
    author: process:git
    last_modified: 2026-08-31
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: load_site mount and peer discovery
    author: process:git
    last_modified: 2026-09-01
  - id: config-rs
    resource: ../../../crates/rocci-rocdown/src/config.rs
    title: SiteConfig mounts and peers
    author: process:git
    last_modified: 2026-09-01
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build/mod.rs
    title: BuildSession rebuild_loaded
    author: process:git
    last_modified: 2026-09-01
  - id: watch-loop
    resource: ../../../crates/rocci-cli/src/dev_server/mod.rs
    title: Shared static preview watcher
    author: process:git
    last_modified: 2026-08-31
  - id: cli-main
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: view site and view docs
    author: process:git
    last_modified: 2026-09-02
  - id: site-toml
    resource: ../../../site/rocdown.toml
    title: rocci.dev mounts
    author: process:git
    last_modified: 2026-09-01
  - id: docs-toml
    resource: ../../../docs/rocdown.toml
    title: Standalone docs peers
    author: process:git
    last_modified: 2026-09-01
  - id: readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Composable content mounts
    author: process:git
    last_modified: 2026-09-01
  - id: preview-doc
    resource: ../../../docs/troubleshooting/preview.rocdown
    title: Preview troubleshooting
    author: process:git
    last_modified: 2026-09-01
  - id: limitations
    resource: ../../status/known-limitations.md
    title: Known limitations
    author: process:cursor
    last_modified: 2026-09-09
---

# Watch composed catalog trees for Rocdown preview

Exploratory. Do not start a phase until the user asks. Research:
[preview watch ignores mounted catalog trees](/research/rocdown/preview-watch-content-roots.md).[^research]

## Goal

`rocdown view site` and `rocdown view docs` both live-reload when **composed**
sources change: the catalog root, `[[mount]]` trees, `[[peer]]` trees, theme,
assets, snippet roots, and out-of-tree `http.service`. Not a new site-builder
product. `[[peer]]` still does not emit pages on `build`; preview rebuilds so
peer routes stay in the link graph.[^research][^readme]

## Out of bound

- Collapsing `docs/` into `site/` or removing `rocdown view docs`.
- Replacing `rocci-docs` staging with a raw mount of `examples/rocci/`.
- Dirty-page apply or skipping whole-catalog `load_site` on one file edit.
- Changing `reload.js`, SSE paths, `--no-live-reload`, or inspector pause.
- FSEvents/CI browser e2e (notify is not reliable in the audit environment).
- Editing `rocci-cli` watch_loop semantics beyond using a better filter from
  Rocdown.

## Constraints that do not move

1. **Two catalogs.** `site/` is rocci.dev; `docs/` is the standalone manual.
   Both remain `rocdown view` / `check` / `build` roots.[^site-toml][^docs-toml][^cli-main]
2. **Mounts emit; peers do not emit.** Both are watch roots. `check`/`load_site`
   already resolve peer routes; missing peer dirs stay skipped until they
   exist.[^readme][^site-rs][^config-rs]
3. **No third builder.** `ContentRoots` is derived from `SiteConfig` the
   loader already understands.[^research]
4. **Offline tests prove relevance.** Encode site-shaped mounts and
   docs-shaped peers with temp paths. Do not require `notify` in default
   tests.[^dev-rs]
5. **Keep apply-without-recompile.** Body-only edits (including peer-only
   rebuilds that do not change generated Roc) must still hit the existing
   `roc_hash` session path once watch fires.[^build-rs]
6. **Skip preview output and VCS noise.** `watch_loop` already ignores the
   output dir; extra roots still ignore `.git`, `target`, and hidden
   segments.[^watch-loop][^dev-rs]

## Phase 1 — `ContentRoots` from site config

Bound:

- In `rocci-rocdown` (likely `dev.rs`, using `SiteConfig` + canonical paths),
  collect:
  - catalog root
  - each mount source directory that exists
  - each peer source directory that exists
  - theme directory (`build.theme` or `theme/`)
  - assets directory
  - snippet roots from config
  - parent directory of `http.service` when that file exists outside the root
- Unit tests with temp trees:
  - site-shaped: root `site/`, mount `../docs` → `docs/index.rocdown` is a
    watch root member; `docs/.git/index` is not.
  - docs-shaped: root `docs/`, peer `../site/project` → peer `.rocdown` files
    **are** watch members; `docs/index.rocdown` is; missing peer dir omitted.
- Do not change `notify` wiring yet if that keeps the phase small; exporting
  the collector and `path_is_relevant(path, &ContentRoots)` with failing
  mount assertions is enough as long as Phase 2 lands immediately after.
  Prefer implementing the matcher in this phase so tests are green.

Out of bound: CLI, README, live-reload JS, `watch_loop`.

Exit:

```text
cargo test -p rocci-rocdown --lib dev::
cargo fmt --all -- --check
```

## Phase 2 — Wire watcher to `ContentRoots`

Bound:

- `run_with_host_at` sets `watch_paths` from `ContentRoots` (not ad-hoc mount
  loops that the filter then ignores). Include peer dirs.
- `custom_filter` uses the same matcher; canonicalize the candidate path.
- Drop the pre-rebuild `snippet_paths` clone as the primary extra-root
  mechanism.
- Existing in-root `view docs` files remain relevant.

Out of bound: public docs (Phase 3); island fingerprint algorithm.

Exit: Phase 1 tests plus any new wiring tests. Same cargo commands.

Manual (not CI): `rocdown view site --no-window`, edit
`docs/templates/index.rocdown`, see `rocdown: rebuilding` / `rebuilt` and a
`reload` on `/__rocci/events`. `rocdown view docs --no-window`, edit
`docs/index.rocdown`, same. Peer edit (`site/project` or staged
`dist/example-docs`) while `view docs` must rebuild.

## Phase 3 — Public contract and status

Bound:

- Preview troubleshooting (`docs/troubleshooting/preview.rocdown`):
  mounted sources rebuild `view site`; peer sources rebuild `view docs`;
  `--no-live-reload` is pause only.
- Rocdown crate README composable-mounts section: watch follows mounts and peers.
- Keep [`known-limitations`](/status/known-limitations.md) aligned if the
  mount-watch gap is still listed until this phase lands.[^limitations][^preview-doc][^readme]

Out of bound: architecture record rewrite beyond a one-line shipped-state
correction if a reviewer asks; collapsing catalogs.

Exit: `okmate check knowledge --profile base --format terminal` after the
doc/knowledge edits; `cargo fmt --all -- --check` if Rust comments changed
(they should not).

[^research]: Filter drops mount events; peers are not registered; both catalogs stay.
[^dev-rs]: Current `path_is_relevant` and mount `watch_paths`.
[^site-rs]: Discovery order: root, mounts, peers.
[^config-rs]: Mount versus peer config.
[^build-rs]: Session apply without recompile.
[^watch-loop]: Shared debounce and broadcast.
[^cli-main]: Both directory previews use `run_site_dev`.
[^site-toml]: `site/` mounts `../docs`.
[^docs-toml]: `docs/` peers only.
[^readme]: Documented mount/peer split.
[^preview-doc]: Troubleshooting does not mention mounts.
[^limitations]: Status currently over-claims watch for composed catalogs.
