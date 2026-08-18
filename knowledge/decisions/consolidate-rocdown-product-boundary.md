---
type: Decision
title: Consolidate the Rocdown format and documentation generator
description: "Approved direction: Rocci owns the app framework and template language, while Rocdown owns its document format, static generator, interactive document runtime, CLI, themes, and document tooling."
tags: [domain/rocci, domain/rocdown, concern/architecture, concern/tooling, integration/okf]
status: draft
generated: { by: process:cursor, at: 2026-08-17T19:50:00Z }
authority: normative
owners: [human:nils]
sources:
  - id: current-system
    resource: ../architecture/system-overview.md
    title: Current Rocci system overview
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: current-format
    resource: ../architecture/rocdown-format.md
    title: Current Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: current-generator
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-17
  - id: workspace
    resource: ../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-17
  - id: static-okf
    resource: static-okf-boundary.md
    title: Strict OKF Markdown boundary
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-diagnostics
    resource: ../../crates/rocs/src/catalog.rs
    title: Current Rocdown identity, route, link, and navigation diagnostic codes
    author: process:git
    last_modified: 2026-08-16
  - id: docs-diagnostics
    resource: ../../crates/rocs/src/docs.rs
    title: Current documentation-component diagnostic codes
    author: process:git
    last_modified: 2026-08-16
  - id: site-diagnostics
    resource: ../../crates/rocs/src/site.rs
    title: Current parse and unsupported-feature diagnostic codes
    author: process:git
    last_modified: 2026-08-16
  - id: architecture-check
    resource: ../../scripts/check-workspace-deps.py
    title: Workspace dependency-direction check
    author: process:cursor
    last_modified: 2026-08-17
  - id: refactor-plan
    resource: ../plans/rocdown-boundary-refactor.md
    title: Rocdown product-boundary refactor plan
    author: process:codex
    last_modified: 2026-08-17
---

# Consolidate the Rocdown format and documentation generator

## Status

The maintainer approved this replacement direction on 2026-08-17 and froze the
remaining Phase 0 names, compatibility, diagnostic-code, and OKF-exception
choices the same day. The record remains `draft` pending evidence review; the
architecture is not yet implemented. The current repository still exposes
Rocdown through base Rocci commands and tooling and exposes the static
generator as Rocs.[^current-system][^workspace]

## Context

The current architecture treats the `.rocdown` format as a Rocci-level sibling
of `.rocci`, then places multi-page static generation in a separate Rocs
product. In practice, the format includes page metadata, Roc and Rocci regions,
documentation components, link semantics, assets, and rendering rules, while
the generator supplies the catalog, navigation, validation, article rendering,
artifacts, and shell.[^current-format][^current-generator]

The attempted analogy with generic Markdown plus interchangeable static-site
generators is therefore misleading. Rocdown documents are Markdown-readable,
but their complete executable and publication contract is specific to the
Rocdown toolchain. Keeping the format below Rocci while placing its principal
site semantics in Rocs splits one product contract across two public concepts.

## Decision

Rocci owns the `.rocci` template language, Roc and Datastar application model,
runtime configuration, generic build/run driver, and desktop/webview host. Base
Rocci libraries, commands, and language tooling do not recognize `.rocdown` and
do not depend on Rocdown packages.

Rocdown owns the `.rocdown` document language and its complete documentation
system: parsing and semantic document IR, standalone document compilation,
site catalog and routes, navigation, static article rendering, documentation
components, themes, assets, inspection, checking, build/run commands, and
document-specific editor support. The Rocs product name and public boundary are
retired in favor of Rocdown.

The command boundary mirrors the product boundary. `rocci run` runs a Rocci
application or `.rocci` entry point. `rocdown run` runs either one interactive
`.rocdown` document or a Rocdown documentation site. Base `rocci` commands do
not accept `.rocdown`; authors do not choose between Rocci and Rocs commands for
the same document.

This symmetry is intentional: Rocci is both the application runtime and its
component/template format, while Rocdown is both the document system and its
interactive document format. Rocdown uses Rocci to implement interactive
documents and visible site UI without becoming part of base Rocci.

This is one product boundary, not a requirement for one monolithic Rust crate.
Internal crates may remain split by compilation and test concerns, but they
must sit behind a Rocdown facade and follow one-way dependencies.

## Naming layers

Public product surface and Cargo package provenance are separate:

| Layer | Approved name |
| --- | --- |
| Document product, format, and configuration | Rocdown / `rocdown.toml` |
| User-facing executable | `rocdown` |
| Public Rust library/facade | `rocci-rocdown` |
| CLI Cargo package | `rocci-rocdown-cli` |
| Rocci-based OKF application | `rocci-okf` |
| Portable OKF engine | `okf` |

The `rocci-` Cargo prefix means that Rocdown is implemented in and belongs to
the Rocci ecosystem; it does not mean base Rocci depends on Rocdown. The
dependency rules below define that distinction mechanically. Keeping the
existing `rocci-rocdown` library name also avoids an unnecessary parser-crate
rename while its scope expands to the full product facade.

The historical `rocs-okf` label is retired with Rocs. `rocci-okf` names the
Rocci application for an external standard. The portable engine is `okf`
because it must be reusable without Rocci.

## Compatibility

There is no compatibility period. The repository does not ship a dual
`rocs.toml` parser, a `rocs` compatibility binary, or a published deprecation
release. An in-repo cutover may overlap on a branch until the replacement
passes parity; then `rocs`, `rocs-cli`, and `rocs.toml` are removed in the same
series.[^refactor-plan]

## Dependency rules

1. Rocci has no Rocdown or OKF dependency.
2. Rocdown may depend on Rocci's template compiler, generic application driver,
   host, and language/highlighting extension points.
3. Rocdown does not depend on OKF and does not contain OKF policy.
4. The portable `okf` engine depends on neither Rocdown nor Rocci, and must not
   import Rocdown types. Cargo metadata can enforce package edges; type leakage
   remains a review rule when the engine is extracted.
5. `rocci-okf` depends on `okf` and Rocci. It may temporarily depend on Rocdown
   presentation (inert Markdown or shell) only if a tracking issue exists when
   that adapter is introduced. The adapter is not part of the target domain
   model.
6. Shared UI packages, if evidence later justifies them, contain only
   domain-neutral Rocci components and view data. Rocdown and OKF navigation,
   routing, graph, lifecycle, and validation rules remain with their domains.

These rules reverse the current base-tooling dependencies recorded by the
workspace, where `rocci-cli`, `rocci-highlight`, and `rocci-lsp` directly
consume `rocci-rocdown`.[^workspace] Direct workspace edges are checked by
`scripts/check-workspace-deps.py`; today's reverse edges are allowlisted until
Phase 3 removes them.[^architecture-check]

## Diagnostic codes

Existing `RDxxxx` allocations stay. New Rocdown diagnostics continue that
series. Do not introduce `ROCS` or `Rocs` codes. `OKFxxxx` stays with OKF.

Current families:[^catalog-diagnostics][^docs-diagnostics][^site-diagnostics]

- `RD1xxx` parse / source (`RD1001`, `RD1002`)
- `RD20xx` identity / routes
- `RD21xx` links / assets
- `RD22xx` navigation / drafts
- `RD23xx` unsupported static-page features
- `RD24xx`–`RD26xx` documentation components, includes, and examples

User-facing strings that say "Rocs" (CLI help, failed-build HTML, public docs)
change only at the Phase 4 product switch. Internal identifiers (`RocsTheme`,
`RocsBuild`, generated module names) stay until that same switch.

## OKF boundary

Canonical knowledge remains strict OKF Markdown with YAML metadata and inert
bodies under the existing decision.[^static-okf] Rocdown must not acquire an
OKF dependency merely because the current implementation reuses its Markdown
AST and documentation shell.

The planned Rocci OKF application may reuse the consolidated Rocdown
presentation path temporarily while extracting `okf` and measuring the actual
shared UI. That edge requires a tracking issue at introduction and must not
leak Rocdown types into `okf`. A speculative `rocci-layout` Rust library should
not own navigation or catalogs: those are domain transformations, not layout
primitives.

If canonical OKF records later move from standard Markdown into `.rocdown`,
that requires a separate decision covering OKF conformance, export and
round-trip behavior, unknown metadata, inertness, source spans, and which
representation is authoritative. It is not implied by this consolidation.

## Consequences

Rocci becomes a smaller and more coherent application framework. Rocdown gains
one command, one configuration story, one documentation contract, and one
place for format-plus-generator evolution. The migration must first extract
generic run, language-server, and highlighting seams so Rocdown can continue
to use Rocci capabilities without reverse dependencies.

The rename is broader than crate moves: commands, configuration, environment
variables, templates, examples, public documentation, diagnostics, CI, and
historical plans must be migrated deliberately. Current behavior must remain
characterized while those names and owners move.

Rocs is not retained as a product tier inside Rocdown. After the in-repo
cutover, active packages, commands, configuration, templates, and
documentation use Rocdown naming; Rocs remains only in historical records and
archived reports.

## Frozen Phase 0 contract

The product symmetry, separate CLIs, one-way ownership, package names, `okf`
engine name, no-compatibility window, `RDxxxx` diagnostic policy, and the
temporary `rocci-okf` presentation exception are approved. Current architecture
records remain descriptive of shipped behavior until implementation
changes.[^current-system][^refactor-plan]

[^current-system]: Shipped workspace and product split before this proposal.
[^current-format]: Shipped Rocdown syntax, metadata, executable regions, Markdown semantics, and standalone compilation.
[^current-generator]: Shipped Rocs catalog, navigation, article rendering, shell, artifacts, and isolated OKF path.
[^workspace]: Current workspace membership and package dependency declarations.
[^static-okf]: Approved canonical strict-Markdown, inert-content, and interoperability boundary for knowledge records.
[^catalog-diagnostics]: Current `RD20xx`–`RD22xx` identity, route, link, asset, and navigation codes.
[^docs-diagnostics]: Current `RD24xx`–`RD26xx` documentation-component, include, and example codes.
[^site-diagnostics]: Current `RD1xxx` parse codes and `RD23xx` unsupported-feature codes.
[^architecture-check]: Mechanical classification of workspace packages and allowlisted reverse edges.
[^refactor-plan]: Phased migration that records the frozen Phase 0 names, no-compat window, and OKF exception.
