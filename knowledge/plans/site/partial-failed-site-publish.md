---
type: Implementation Plan
title: Partial failed site publish
description: "After catalog or compile errors, generate and commit HTML for pages that did resolve, and surface remaining diagnostics in the preview build-error dialog."
tags: [domain/rocci, domain/rocdown, domain/runtime, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-22T13:30:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: Rocdown site build and commit_output
    author: process:git
    last_modified: 2026-08-22
  - id: dev-server
    resource: ../../../crates/rocci-cli/src/dev_server.rs
    title: Static dev server and build-error preview
    author: process:git
    last_modified: 2026-08-22
  - id: catalog-rs
    resource: ../../../crates/rocci-rocdown/src/catalog.rs
    title: Catalog resolve and RD2101
    author: process:git
    last_modified: 2026-08-22
---

# Partial failed site publish

## Goal

When a Rocdown site rebuild fails partway through catalog resolve or page
generation, still publish HTML for pages that **did** resolve, and list the
remaining diagnostics in the preview build-error dialog.[^build-rs][^dev-server]

Today `prepare_plan` bails before `commit_output`, so the output tree keeps
only the last fully successful build. The shipped preview dialog (native
`<dialog>` over stale HTML) makes that last-good tree visible; this plan
extends the build contract so a failed rebuild can refresh the pages that
succeeded.[^build-rs]

[^build-rs]: `crates/rocci-rocdown/src/build.rs`
[^dev-server]: `crates/rocci-cli/src/dev_server.rs`
[^catalog-rs]: `crates/rocci-rocdown/src/catalog.rs`

## Out of bound

Weakening or demoting RD2101 and other catalog errors. Interpreting themes in
Rust to skip broken pages. Changing `rocci-desktop` or `RocdownTheme.rocci` for
this behavior. Partial publish for CDN-only release artifacts without an
explicit maintainer gate.

## Constraints that do not move

- Catalog diagnostics remain authoritative; partial output must not imply a
  clean `rocdown check` or `rocdown build`.
- `commit_output` stays atomic per publish: either the staged tree swaps in, or
  the previous tree remains. Partial publish stages only resolved pages inside
  that single swap.
- Preview and production share the same diagnostic codes (for example
  RD2101).[^catalog-rs]

## Phase 1 — Resolve and plan without all-or-nothing bail

**Bound:** `crates/rocci-rocdown` only. Split `prepare_plan` so catalog
resolve collects errors per page without aborting the whole `BuildPlan` when
some pages are valid. Record which page ids are excluded and why.

**Exit:** Unit tests: a site with one broken internal link still plans and
builds the other pages; `error_summary` lists RD2101 for the broken page.

## Phase 2 — Stage and commit partial trees

**Bound:** `build.rs` `rebuild_loaded` path used by `rocdown view` and
`rocdown build`. On failure after partial generation, call `commit_output` with
the staged subset and attach a structured failure report (diagnostics +
excluded page ids) to the dev server.

**Exit:** Integration test: fix one link, watch rebuild updates only affected
pages; broken page routes 404 or show last-good copy with a per-page notice
(TBD in Phase 3).

## Phase 3 — Preview contract

**Bound:** `rocci-cli` dev server. Dialog lists diagnostics from the partial
report; stale pages without fresh HTML remain served from disk. First-build
failure with zero HTML still uses the minimal shell.

**Exit:** `cargo test -p rocci-cli` covers partial-failure dialog copy; manual
`rocdown view site` with one RD2101 shows good pages plus dialog.

## Phase 4 — Authoring and check alignment

**Bound:** `rocdown check` documents that errors block release builds; preview
may show partial output. Update `docs/rocdown/sites.rocdown` and crate READMEs.

**Exit:** `cargo run -q -p rocci-rocdown-cli -- check site` on a fixture with
a deliberate broken link still fails; `view` on the same tree serves partial
HTML.
