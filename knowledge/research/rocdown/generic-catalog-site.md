---
type: Research Report
title: rocci.dev is a catalog instance; the planner still knows first-party layouts
description: "run_site_dev is already generic (any rocdown.toml root). Remaining coupling is a closed layout/news/playground planner, rocci-docs URL and /examples/ defaults, and first-party ops paths. Full generic marketing sites are feasible only after layouts leave Rust."
tags: [domain/rocdown, domain/site, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:58:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: cli-main
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: run_site_dev previews any catalog directory
    author: process:git
    last_modified: 2026-09-09
  - id: dev-rs
    resource: ../../../crates/rocci-rocdown/src/dev.rs
    title: run_with_host_at and ContentRoots
    author: process:git
    last_modified: 2026-09-09
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: load_site and closed VALID_LAYOUTS
    author: process:git
    last_modified: 2026-09-09
  - id: config-rs
    resource: ../../../crates/rocci-rocdown/src/config.rs
    title: Mount layout allowlist
    author: process:git
    last_modified: 2026-09-01
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan/mod.rs
    title: home, news-index, playground planner special cases
    author: process:git
    last_modified: 2026-09-09
  - id: playground-rs
    resource: ../../../crates/rocci-rocdown/src/plan/playground.rs
    title: Playground session assets in the generator
    author: process:git
    last_modified: 2026-09-01
  - id: stage-rs
    resource: ../../../crates/rocci-docs/src/stage.rs
    title: Hardcoded /examples/ routes and examples.rocci.dev URLs
    author: process:git
    last_modified: 2026-08-29
  - id: ops-site
    resource: ../../../rocci-ops/src/rocci_ops/site.py
    title: First-party package site pipeline
    author: process:git
    last_modified: 2026-09-09
  - id: example-site
    resource: ../../../examples/rocdown/site/rocdown.toml
    title: Generic tiny Rocdown catalog
    author: process:git
    last_modified: 2026-08-21
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog versus Rocci chrome
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: watch-plan
    resource: ../../plans/rocdown/preview-watch-content-roots.md
    title: Composed-tree preview watch
    author: process:cursor
    last_modified: 2026-09-09
  - id: plan
    resource: ../../plans/rocdown/generic-catalog-site.md
    title: Treat first-party site as one catalog instance
    author: process:cursor
    last_modified: 2026-09-09
---

# rocci.dev is a catalog instance; the planner still knows first-party layouts

## Claim

`run_site_dev` is not a rocci.dev pipeline. It is the directory preview entry
for **any** catalog (`rocdown view site`, `view docs`, `view examples/rocdown/site`).
The engine already composes trees with `[[mount]]` / `[[peer]]` and
`ContentRoots`. What is not generic is a **closed layout vocabulary and
planner special cases** (home, news, playground), **rocci-docs** defaults
(`/examples/`, `*.examples.rocci.dev`), and **rocci-ops** paths that rightly
name this repo’s `site/` directory.[^cli-main][^dev-rs][^watch-plan]

Making “site as a generic use case” fully true in the **engine** is feasible
only if named layouts and collections stop being Rust `match` arms. Making
**rocci-docs** reusable is a smaller config job and must not pull staging into
`rocci-rocdown`. First-party deploy (`package site`, `site.yml`) should stay
ops.[^stage-rs][^ops-site][^catalog-shell]

Implementation: [generic catalog versus first-party site](/plans/rocdown/generic-catalog-site.md).[^plan]

## Four layers

| Layer | What “site” means | Generic today? |
| --- | --- | --- |
| CLI name | `run_site_dev(root)` | Yes. `root` is a path. `view site` is this repo’s `site/` folder, same as `view docs`.[^cli-main] |
| Catalog engine | `load_site`, mounts, peers, islands, watch | Yes for composition. No for layout names and news/playground planning.[^site-rs][^plan-rs][^dev-rs] |
| App-docs stager | `rocci-docs` → tree to mount | No. Prefix, aliases, and live URL template are rocci.dev-shaped. Boundary: Rocdown must not depend on this crate.[^stage-rs] |
| Operator pipeline | `rocci-ops package site`, CI | Intentionally first-party. Not a Rocdown product surface.[^ops-site] |

`examples/rocdown/site` is already a generic catalog: title, `base_url`, nav,
no mounts, default theme. That path uses the same `run_site_dev` as
rocci.dev.[^example-site][^cli-main]

## What `run_site_dev` actually requires

The function binds a port, calls `run_with_host_at`, tees logs, opens the
preview window with `state_key: "rocdown"`. Nothing inspects the folder name
`site`. Required inputs are a directory (optional `rocdown.toml`), not a
product hostname.[^cli-main][^dev-rs]

Renaming it to `run_catalog_preview` would only clarify. It is not required
for correctness.

## Engine coupling that blocks a second marketing site

Rust rejects unknown `@page.layout` / mount `layout` against a closed list:
`home`, `faq`, `product`, `section`, `docs`, `news-index`, `news-post`,
`plain`, `not-found`, `playground`. Root `index` defaults to `home`, other
pages to `docs`. The planner then:

- hashes playground JS/Wasm when any page uses layout or `:playground`
- fills `collection_items` from pages with `collection == "news"` or
  `/news/` + `news-post` when layout is `news-index` or `home` (first three)

That is first-party chrome policy inside the catalog crate. The
Rust-catalog / Rocci-shell decision says chrome belongs in the theme;
a closed layout enum in `site.rs` is the leak.[^site-rs][^plan-rs][^playground-rs][^catalog-shell]

A third-party catalog can still ship docs-like pages (`layout = "docs"` /
default theme). It cannot introduce `layout = "landing"` without a Rocdown
code change. Playground is an engine feature, not a mount.

## rocci-docs

Staging copies cataloged `.rocci` apps into a Rocdown tree with routes
`/examples/<id>/`, an index table, optional Launch URLs
`https://{id}.examples.rocci.dev`, and alias `/docs/examples/`. The `site`
boolean on an app row means “include on rocci.dev,” not “this is a Rocdown
catalog.”[^stage-rs]

Feasible generalization: `StageOptions` already has `include_all` and
`advertise_live`. Add route prefix, live-URL template, and index copy as
config (CLI or a stager toml). Keep emitting ordinary `.rocdown` so any
catalog can `[[mount]]` the output. Do not teach `load_site` about
`apps.toml`.[^stage-rs][^ops-site]

## What should stay first-party

`rocci-ops` staging `dist/example-docs`, building live musl binaries, and
`rocdown package site` is this repository’s publish kit. Docker, icons, and
`site.yml` belong there. Generalizing Rocdown does not mean a `rocci-ops`
that takes an arbitrary path instead of `site/` unless a second product site
appears.[^ops-site]

Two catalogs in this repo (`docs/` and `site/`) remain a **composition
choice**, not an engine requirement. Watch already treats both as generic
roots.[^watch-plan]

## Feasibility

| Goal | Feasible? | Cost |
| --- | --- | --- |
| Treat `view site` as `view <dir>` (docs) | Already true | Optional rename |
| Second docs-only catalog with mounts/peers | Already true | Author `rocdown.toml` |
| Second rocci.dev-like site (home, news, playground, FAQ) without forking Rocdown | No | Open layout/collection contract or copy planner arms |
| Reuse `rocci-docs` for another prefix/host | Yes | Config on the stager |
| Generic `package <any-catalog>` in the product CLI | Partial | `rocdown package DIR` already exists; ops `package site` is extra live-app work |

[^cli-main]: Directory `view` always calls `run_site_dev`; path is the argument.
[^dev-rs]: Catalog preview and `ContentRoots` take `root: &Path`.
[^site-rs]: `VALID_LAYOUTS` and default `home` / `docs`.
[^config-rs]: Mount `layout` must be in the same closed list.
[^plan-rs]: `news-index` / `home` collection_items; playground asset gating.
[^playground-rs]: `page_uses_playground`; baked runtime bytes.
[^stage-rs]: `live_demo_url`, `/examples/{id}/` index rows, `/docs/examples/` alias.
[^ops-site]: Hardcoded `action, "site"` and `examples/rocci/apps.toml`.
[^example-site]: Minimal `[site]` + `[[nav]]` example catalog.
[^catalog-shell]: Chrome in Rocci theme; Rust owns catalog data.
[^watch-plan]: Watch follow-on complete for composed trees; not layout policy.
[^plan]: Follow-on phases: optional rename, stager config, layout contract.
