---
type: Implementation Plan
title: Rocdown product-boundary refactor
description: Phased migration that removes Rocdown from base Rocci, consolidates the Rocdown format and static generator, retires Rocs, and separates portable OKF behavior.
tags: [domain/rocci, domain/rocdown, concern/architecture, concern/migration, concern/tooling, integration/okf]
status: draft
generated: { by: process:cursor, at: 2026-08-17T22:15:00Z }
stale_after: 2026-11-15
authority: exploratory
owners: [human:nils]
sources:
  - id: boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved consolidated Rocdown product direction
    author: process:codex
    last_modified: 2026-08-17
  - id: current-format
    resource: ../architecture/rocdown-format.md
    title: Current Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: current-generator
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Current Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-17
  - id: workspace
    resource: ../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-17
  - id: current-okf
    resource: ../../crates/okf/src/lib.rs
    title: Portable OKF engine implementation
    author: process:git
    last_modified: 2026-08-17
  - id: architecture-check
    resource: ../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace dependency-direction check
    author: process:cursor
    last_modified: 2026-08-17
  - id: okf-plan
    resource: rocci-okf-app.md
    title: Standalone Rocci OKF application plan
    author: process:codex
    last_modified: 2026-08-17
---

# Rocdown product-boundary refactor

## Purpose and authority

This plan implemented the approved consolidated direction across Phases 0–8.
All phases have landed and the remaining findings from the completion audit
have been resolved. Base Rocci has zero reverse dependencies, Rocdown owns the
document system and static generator, the portable `okf` engine and `rocci-okf`
application are separated, and `rocci-rocdown-lsp` provides the composition
language server.[^boundary]

The migration followed a continuously testable sequence: product naming and
dependency directions were changed behind testable facades; destructive removal
happened after replacement commands, configuration, tooling, and OKF paths
passed parity gates. There was no published compatibility window.

## Current coupling to unwind

The current `rocci-rocdown` crate owns parsing, the semantic Markdown tree,
standalone lowering, links, media, documentation-declaration fields, and theme
selection. Rocs consumes those types for its site catalog, article renderer,
documentation components, build plan, and OKF Markdown adapter.[^current-format][^current-generator][^current-okf]

The base workspace also has reverse dependencies that contradict the target:
`rocci-cli`, `rocci-highlight`, and `rocci-lsp` consume
`rocci-rocdown`; Rocs also consumes `rocci-highlight`, and `rocs-cli` is the
only consumer of Rocs.[^workspace]

The OKF module is not only a Markdown adapter. It currently imports the Rocs
article renderer, output commit utilities, catalog types, site configuration,
and loaded-site types, so moving the file alone would preserve the wrong
boundary.[^current-okf]

## Target package and command map

The approved package namespace uses `rocci-rocdown` for the public Rust facade
and `rocci-rocdown-cli` for the Cargo command package, while the binary,
product, format, and configuration use `rocdown`. The portable OKF engine is
`okf`; the Rocci application is `rocci-okf`.

| Current surface | Target owner | Migration disposition |
| --- | --- | --- |
| `rocci-rocdown` | `rocci-rocdown` facade | Expand the current parser library to own or expose the complete Rocdown product API |
| `rocs` | `rocci-rocdown` internals | Move catalog, site, article, docs components, planning, build, and dev modules behind the facade; delete the Rocs crate |
| `rocs-cli`, binary `rocs` | `rocci-rocdown-cli`, binary `rocdown` | Add the replacement first; delete `rocs-cli` after parity with no compatibility shim |
| `rocci-theme` | Rocdown | Rename or internalize; it is a Rocdown CSS contract rather than a base Rocci theme system |
| `.rocdown` branches in `rocci-cli` | Rocdown CLI | Call a generic Rocci driver with generated Roc instead of teaching base Rocci the format |
| Rocdown analysis in `rocci-lsp` | Rocdown language tooling | Compose reusable Rocci/Roc analyzers from a Rocdown-owned server or adapter |
| Rocdown composite logic in `rocci-highlight` | Rocdown highlighting | Keep language-neutral and Rocci primitives below; move document composition above |
| `rocs.toml` | `rocdown.toml` | Switch in-repo; do not parse both filenames |
| `RocsTheme.rocci`, `DocsComponents.rocci`, `RocsBuild.roc` | Rocdown templates/runtime | Rename after behavior parity so generated-module changes are isolated |
| `rocs knowledge` and `rocs::okf` | `okf` engine plus `rocci-okf` app | Preserve current commands through an adapter until `rocci-okf` matches fixtures, then remove from Rocdown |
| `rocci-wry` | `rocci-desktop` | Rename crate and module to reflect domain role as the native window/webview host rather than one backend dependency |

## Architectural end state

```text
Rocci libraries and tools
    ^
    | depends on
Rocdown format + generator + CLI + document tooling

portable `okf` engine        domain-neutral Rocci UI (only if justified)
    ^                         ^                 ^
    |                         |                 |
`rocci-okf` application -----+           Rocdown product
```

No arrow points from Rocci to Rocdown or OKF, or from Rocdown to OKF. The
`okf` engine has no Rocci, Rocdown, theme, HTTP, webview, or git-host
dependency. A temporary `rocci-okf`-to-Rocdown presentation edge is allowed
only if it has a removal issue and does not leak Rocdown types into the
engine.

## Phase 0 — approve and freeze the contract

The maintainer approved the product symmetry, separate CLI ownership, and
one-way boundary on 2026-08-17, then froze the remaining contract the same
day.[^boundary]

1. Record the companion decision as approved direction while retaining its
   `draft` lifecycle until evidence review.
2. Approved names: `rocci-rocdown` facade, `rocci-rocdown-cli` with binary
   `rocdown`, `rocci-okf` application, and portable engine `okf`. The
   historical `rocs-okf` label is retired.
3. No compatibility period: no dual `rocs.toml` parser, no `rocs` shim, and no
   published deprecation release. Branch-local overlap is allowed only until
   parity, then `rocs` names are deleted in the same series.
4. Encode dependency rules in `tools/rocci-ops/src/rocci_ops/workspace_deps.py` over
   `cargo metadata`, with today's reverse edges allowlisted until Phase 3.[^architecture-check]
5. Freeze diagnostic-code policy: keep existing `RDxxxx` allocations; change
   user-facing "Rocs" strings only at the Phase 4 product switch.

Exit gate: the target names, no-compat window, remaining dependency details,
and OKF exception are approved; current architecture records remain explicitly
descriptive until implementation changes.

## Phase 1 — characterize behavior and extract generic Rocci seams

1. Add golden/fixture coverage for Rocdown parse trees, generated Roc, source
   maps, single-document run routes, site catalog JSON, article HTML, build
   artifacts, last-good rebuild behavior, LSP diagnostics/tokens, and OKF
   normalized output.
2. Extract a reusable Rocci driver from private `rocci-cli` orchestration. It
   accepts `.rocci` or already-generated Roc modules and owns generic compile,
   runtime staging, HTTP dispatch, preview, and host concerns; it does not
   inspect `.rocdown`.
3. Split highlighter primitives from Rocdown document composition. Roc/Rocci,
   HTML, CSS, region, and token APIs remain reusable below Rocdown.
4. Introduce language-tooling extension points for a document analyzer to
   supply diagnostics, regions, symbols, hovers, completion, definitions, and
   semantic tokens without the Rocci server importing Rocdown AST types.
5. Make the current OKF characterization fixtures cover base/Rocci profiles,
   graph and chunks, filters, retrieval benchmark, rendered review pages, and
   deterministic artifacts.

Exit gate: existing commands and filenames behave unchanged; the generic
driver and tooling APIs have tests proving they contain no Rocdown-specific
extension or type checks.

## Phase 2 — establish the unified Rocdown product

1. Add the `rocdown` public facade and `rocdown` binary. Initially they may
   delegate to current internal crates so the product boundary moves before
   physical modules do.
2. Expose `rocdown build`, `run`, `check`, `inspect`, and `test`. Decide whether
   single-file compile/AST inspection are subcommands or library-only APIs.
3. Route standalone interactive documents through Rocdown parsing/lowering and
   the generic Rocci driver extracted in Phase 1.
4. Make one Rocdown configuration loader own site metadata, navigation, build,
   assets, themes, and development behavior. Do not parse `rocs.toml`.
5. Move first-party document examples and integration tests to invoke the new
   binary. Compare against the old path only on the cutover branch until the
   old path is deleted.

Exit gate: every shipped `.rocdown` workflow has a Rocdown-owned command; a
static site and an interactive document both pass through one public Rocdown
facade; `rocdown run FILE.rocdown` is the sole supported single-document run
path; failed builds still preserve the previous output tree.

## Phase 3 — remove Rocdown from base Rocci

1. Delete `.rocdown`, `.md`, and `.markdown` format dispatch from `rocci-cli`.
   Keep generic generated-Roc execution only in the extracted driver.
2. Remove `rocci-rocdown` from base Rocci CLI dependencies and move the
   Rocdown-specific theme arguments, asset discovery, sibling-page planning,
   error-page cases, examples, and tests to Rocdown.
3. Remove Rocdown document compilation and registration from the base Rocci
   language server and editor activation. Add Rocdown-owned editor wiring that
   composes the reusable Rocci/Roc analyzers.
4. Remove the Rocdown AST dependency from the base highlighter. Rocdown owns
   Markdown-plus-embedded-region composition and may depend on the remaining
   Rocci highlight primitives.
5. Rename or internalize `rocci-theme` under Rocdown. Verify no base Rocci
   public API exposes `rd-*` CSS, page metadata, or document theme selection.

Exit gate: `cargo tree --workspace` shows no path from a base Rocci package to
Rocdown; `rocci --help` and the Rocci editor manifest contain no Rocdown
surface; Rocdown standalone run, site build, LSP, and highlighting still pass.

## Phase 4 — physically consolidate and retire Rocs

1. Move the parser/AST/lowering and generator/catalog modules into the chosen
   Rocdown internal crate layout. Prefer cohesive modules and explicit private
   seams over preserving historical crate names.
2. Rename `RocsTheme`, `RocsBuild`, generated modules, staging variables,
   templates, examples, and configuration to Rocdown equivalents.
3. Replace `docs/rocs.toml`, documentation commands, CI invocations, and
   generated metadata with their Rocdown names.
4. Remove `rocs` and `rocs-cli` packages after parity; there is no
   compatibility binary or config parser to expire.
5. Archive or supersede the old root Rocs implementation plan, and revise
   current architecture/status/public reference records only when code has
   actually crossed the boundary.

Exit gate: no active product, package, binary, config, template, or public docs
surface uses Rocs naming. Historical knowledge and archived reports may retain
it with explicit historical status. Full docs build output is byte-equivalent
apart from approved naming, metadata, and asset-path changes.

## Phase 5 — rename rocci-wry to rocci-desktop

1. Rename crate directory `crates/rocci-wry` to `crates/rocci-desktop` and update
   its package manifest name to `rocci-desktop`.
2. Update workspace manifests (`Cargo.toml`), dependencies in `rocci-cli`,
   `rocci-rocdown-cli`, and any in-flight tools to consume `rocci-desktop`.
3. Update module-level docs and Rust import paths (`use rocci_desktop::...`)
   across the codebase.
4. Update `tools/rocci-ops/src/rocci_ops/workspace_deps.py` so `BASE_ROCCI` classifies
   `rocci-desktop`.
5. Update repository documentation, contributor guides, and agent instructions
   (`AGENTS.md`) referencing `rocci-wry`.

Exit gate: no active code, workspace manifest, or test references `rocci-wry` or
`rocci_wry`; `rocci-desktop` builds and passes all windowing, webview, menu, and
state tests; workspace dependency assertions pass.

## Phase 6 — separate OKF without coupling Rocdown to it

1. During Phases 1–5, preserve `knowledge check`, inspect, search, benchmark,
   build, and run through a compatibility adapter. Do not add new OKF policy to
   Rocdown.
2. Extract the UI-neutral `okf` engine using the existing standalone plan: parsing
   with unknown metadata preservation, conformance and profile diagnostics,
   provenance/lifecycle, graph and backlinks, chunks, filters, search,
   benchmarks, and stable serializable types.[^okf-plan]
3. Separate Markdown input behind a narrow adapter. Initially it may call
   Rocdown's inert `parse_markdown_body` path to preserve exact spans and
   output. Extract a neutral Markdown layer only when the portable engine has a
   second consumer and the required AST contract is clear.
4. Build the Rocci OKF application on the portable engine and generic Rocci
   driver/host. It may temporarily reuse the Rocdown shell or renderer, but the
   dependency must stay in presentation wiring and have parity tests plus a
   removal gate.
5. Remove the compatibility OKF command and module from Rocdown after the new
   application covers current local review, query, build, and last-good preview
   behavior.

Exit gate: Rocdown has no OKF dependency or command; the OKF engine can be used
by a minimal non-Rocci, non-Rocdown consumer; the Rocci OKF application matches
the current deterministic outputs and governance behavior.

## Phase 7 — extract shared UI only from demonstrated duplication

1. Implement Rocdown and Rocci OKF navigation models independently first.
   Compare their actual data and component contracts after both work.
2. Keep route derivation, catalog resolution, graph/backlinks, lifecycle,
   review queues, and authorization in their owning domains.
3. If stable duplication remains, extract small Rocci view components such as
   shell frame, navigation groups, breadcrumbs, outline, previous/next links,
   empty states, and responsive chrome. Prefer a name such as `rocci-ui` or
   `rocci-shell` over `rocci-layout` unless the package truly owns only layout.
4. Keep components driven by plain, domain-neutral view records. The shared
   package must not parse files, resolve routes, query graphs, or know Rocdown
   and OKF metadata.

Exit gate: either no shared package is created and the divergence is recorded,
or both products consume a small dependency-neutral component package without
moving domain behavior into it.

## Phase 8 — documentation, knowledge, and release cleanup

1. Update the root README, roadmap, owning crate READMEs, public Rocdown/CLI
   references, contributing guide, examples, and editor documentation.
2. Revise the system overview, Rocdown architecture, former Rocs compiler
   record, implementation status, known limitations, static OKF decision, and
   standalone OKF plan according to what actually shipped. Retain historical
   verification events and return substantively revised records to draft.
3. Update the knowledge log and collection indexes; run the Rocci knowledge
   profile and report lifecycle/provenance warnings separately.
4. Run focused crate tests while moving each boundary, then formatting,
   workspace tests, Rocdown AST inspection, the documentation site build,
   editor integration tests, OKF deterministic checks, and dependency audits.
5. Search active source and docs for obsolete `rocs`, `Rocs`,
   `rocci-rocdown`, `rocci_theme`, `rocci-wry`, and old command/config names. Classify every
   remaining match as compatibility, generated, or historical before release.

Exit gate: dependency rules are mechanically enforced; public documentation
matches commands and packages; the knowledge bundle distinguishes shipped,
approved, proposed, and historical states; the workspace and both product
vertical slices pass. A phase cannot be logged as complete until the required
GitHub CI and Knowledge workflows have succeeded on the declaring revision;
cite those run IDs in the knowledge log.

## Cross-phase validation matrix

Every phase should retain these independent gates:

- Rocci: `.rocci` parse/lower, build/run, Datastar dispatch, desktop preview,
  bundle, LSP, and highlighting without Rocdown packages.
- Rocdown: syntax fixture AST, static and interactive documents, links/assets,
  catalog/navigation, docs components, themes, atomic build, run/live reload,
  LSP, highlighting, and deterministic artifacts.
- OKF: base and Rocci profiles, unknown fields, inert bodies, citations,
  provenance/lifecycle, graph/chunks/search, benchmark thresholds, review
  rendering, last-good preview, and deterministic outputs.
- Architecture: `cargo metadata` dependency assertions and an active-name
  search that excludes historical/archive paths.

## Principal risks and mitigations

- **Big-bang rename hides behavior regressions.** Move the public facade before
  physical modules and compare old/new fixtures until parity.
- **Generic Rocci seams become Rocdown APIs with neutral names.** Require their
  tests and types to work without Markdown, page metadata, or document routes.
- **One product becomes one oversized crate.** Preserve private crate/module
  seams where they improve compile time and tests, but expose one product.
- **OKF becomes accidentally Rocdown-specific.** Keep the engine serializable
  and UI-neutral, isolate the temporary Markdown/presentation adapters, and
  prove a minimal independent consumer.
- **Shared layout absorbs domain logic.** Extract presentation components only
  after two working consumers reveal stable common view records.
- **Future Rocdown-backed OKF silently breaks interchange.** Require a separate
  decision and round-trip/export contract before changing canonical storage.

[^boundary]: Approved product ownership, frozen names, no-compat window, diagnostic policy, and OKF exception.
[^current-format]: Current parser, AST, lowering, declarations, metadata, link, media, theme, and standalone document behavior.
[^current-generator]: Current catalog, article, documentation-component, shell, artifact, and development behavior.
[^workspace]: Current workspace package set and declared dependencies that must be reversed or moved.
[^current-okf]: Current OKF imports from Rocdown and Rocs plus its implemented validation, retrieval, rendering, and build behavior.
[^architecture-check]: Mechanical classification of workspace packages and allowlisted reverse edges.
[^okf-plan]: Existing exploratory portable-engine and Rocci-application extraction plan.
