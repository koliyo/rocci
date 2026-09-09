---
type: Implementation Plan
title: Treat rocci.dev as one Rocdown catalog instance
description: "Optional rename of run_site_dev; configurable rocci-docs prefix and live URLs; do not put ops package site into the engine. Opening the closed layout/news/playground planner is the real generic-site gate."
tags: [domain/rocdown, domain/site, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:58:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocdown/generic-catalog-site.md
    title: rocci.dev is a catalog instance; planner still knows first-party layouts
    author: process:cursor
    last_modified: 2026-09-09
  - id: watch-plan
    resource: preview-watch-content-roots.md
    title: Composed-tree preview watch
    author: process:cursor
    last_modified: 2026-09-09
  - id: cli-main
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: run_site_dev
    author: process:git
    last_modified: 2026-09-09
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: VALID_LAYOUTS
    author: process:git
    last_modified: 2026-09-09
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan/mod.rs
    title: News and home collection_items
    author: process:git
    last_modified: 2026-09-09
  - id: stage-rs
    resource: ../../../crates/rocci-docs/src/stage.rs
    title: rocci-docs staging defaults
    author: process:git
    last_modified: 2026-08-29
  - id: ops-site
    resource: ../../../rocci-ops/src/rocci_ops/site.py
    title: rocci-ops package site
    author: process:git
    last_modified: 2026-09-09
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog versus Rocci chrome
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: app-docs
    resource: rocci-app-docs.md
    title: rocci-docs ownership
    author: process:cursor
    last_modified: 2026-08-29
---

# Treat rocci.dev as one Rocdown catalog instance

Exploratory. Do not start a phase until the user asks. Research:
[generic catalog versus first-party site](/research/rocdown/generic-catalog-site.md).[^research]

Watch for mounts/peers is a separate, implemented plan:
[preview watch content roots](preview-watch-content-roots.md).[^watch-plan]

## Goal

Document and, if started, reduce **engine** special cases so a Rocdown catalog
is not rocci.dev-shaped unless the author opts into those layouts. Keep
`rocci-ops package site` first-party. Keep `docs/` and `site/` as two catalogs
in this repo.[^research][^ops-site]

## Out of bound

- Collapsing `docs/` into `site/` or removing `rocdown view docs`.
- `rocci-rocdown` depending on `rocci-docs`.
- Moving Docker, Cloudflare, or `site.yml` into the product CLI.
- Replacing `rocci-docs` with a mount of `examples/rocci/`.
- Reopening watch/`ContentRoots` (done).[^watch-plan]
- Inventing a second product site in this repository.

## Constraints that do not move

1. **Directory preview is generic.** `run_site_dev` takes a path. Do not add a
   `site`-only code path beside it.[^cli-main]
2. **Chrome vs catalog.** New layout names must not require a Rocdown parser
   change if the theme can own them; planner data (collections) may stay Rust
   if they are catalog facts, not rocci.dev copy.[^catalog-shell][^plan-rs]
3. **Stager emits Rocdown.** Any catalog may mount the output. Live hostnames
   are stager/ops config, not `load_site`.[^stage-rs][^app-docs]
4. **Two catalogs stay** in this workspace until a human changes IA.[^watch-plan]

## Phase 1 — Name the generic preview (optional)

Bound: rename `run_site_dev` to something like `run_catalog_preview` in
`rocci-rocdown-cli` only; comments/README that `view DIR` is the product
site when `DIR` is `site`. No behavior change.

Out of bound: layout list, rocci-docs, ops.

Exit: `cargo test -p rocci-rocdown-cli --offline` or the crate’s default
offline tests; `cargo fmt --all -- --check`.

## Phase 2 — Configurable `rocci-docs` mount shape

Bound:

- Stage options or a small config for route prefix (default `/examples/`),
  live URL template (default `{id}.examples.rocci.dev`), and optional index
  alias. Defaults keep current rocci.dev tests green.
- Unit tests with a non-default prefix.
- Crate README: this tool is a stager, not a site builder.

Out of bound: `load_site` reading `apps.toml`; changing `site = true` meaning
without a rename in apps.toml (that flag is first-party inclusion).

Exit: `cargo test -p rocci-docs`.

## Phase 3 — Layout and collection contract (review gate)

Bound: written contract, not necessarily code in the same revision:

- Which layout names are **engine** (playground assets, news collection
  fill) versus **theme-only** strings.
- Whether `VALID_LAYOUTS` opens to “theme must define the component” or
  stays closed.

A human accepts or rejects opening the list. If rejected, stop; rocci.dev-like
sites stay first-party themes on the closed set.[^site-rs][^plan-rs][^catalog-shell]

Out of bound: implementing an open layout registry in the same phase as the
write-up unless the user asks.

Exit: knowledge record or README section the reviewer can accept; `okmate check knowledge --profile base --format terminal`.

[^research]: Four layers; planner leak is the real generic-site gate.
[^watch-plan]: ContentRoots watch; not this plan.
[^cli-main]: `run_site_dev` is path-generic.
[^site-rs]: Closed `VALID_LAYOUTS`.
[^plan-rs]: home/news/playground special cases.
[^stage-rs]: Hardcoded examples routes and demo URLs.
[^ops-site]: First-party `package site`.
[^catalog-shell]: Theme owns chrome.
[^app-docs]: `rocci-docs` ownership; Rocdown does not import it.
