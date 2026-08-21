---
type: Implementation Plan
title: CLI entry points for Rocci, Rocdown, and OKF preview
description: Choose how authors preview OKF Markdown and other documents without collapsing the three product CLIs into a plugin host.
tags: [domain/rocci, domain/rocdown, domain/okf, domain/rocci-okf, concern/architecture, concern/tooling, concern/rendering]
status: draft
generated: { by: process:cursor, at: 2026-08-19T19:50:00Z }
stale_after: 2026-11-18
authority: exploratory
owners: [human:nils]
sources:
  - id: browser-plan
    resource: rocci-browser.md
    title: Dedicated rocci-browser implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-18
  - id: static-okf
    resource: ../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: okf-app-plan
    resource: rocci-okf-app.md
    title: Standalone Rocci OKF application plan
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
  - id: root-readme
    resource: ../../README.md
    title: Rocci workspace overview and CLI surface
    author: human:nils
    last_modified: 2026-08-18
  - id: rocci-cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: Base Rocci CLI contract
    author: process:git
    last_modified: 2026-08-17
  - id: rocci-run
    resource: ../../crates/rocci-cli/src/run.rs
    title: rocci run entry resolution and Markdown hint
    author: process:git
    last_modified: 2026-08-17
  - id: rocci-cli-lib
    resource: ../../crates/rocci-cli/src/lib.rs
    title: Shared Rocci CLI driver library
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-cli-readme
    resource: ../../crates/rocci-rocdown-cli/README.md
    title: Rocdown CLI contract
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-cli
    resource: ../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown command dispatch and Markdown file acceptance
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-parse
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: Rocdown Markdown parse options without YAML frontmatter
    author: process:git
    last_modified: 2026-08-17
  - id: okf-cli
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf commands and bundle-root run
    author: process:git
    last_modified: 2026-08-17
  - id: okf-readme
    resource: ../../crates/rocci-okf/README.md
    title: rocci-okf usage and review-server contract
    author: process:git
    last_modified: 2026-08-17
  - id: okf-presentation
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF concept metadata and review HTML
    author: process:git
    last_modified: 2026-08-18
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: OKF preview server requiring a bundle directory
    author: process:git
    last_modified: 2026-08-17
  - id: okf-engine
    resource: ../../crates/okf/README.md
    title: Portable OKF engine boundary
    author: process:git
    last_modified: 2026-08-17
  - id: deps-check
    resource: ../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Mechanical one-way workspace dependency check
    author: process:cursor
    last_modified: 2026-08-18
  - id: site-plan
    resource: rocci-dev-site.md
    title: rocci.dev site architecture plan
    author: process:codex
    last_modified: 2026-08-18
  - id: preview
    resource: ../../crates/okf/src/preview.rs
    title: OKF preview path resolution
    author: process:cursor
    last_modified: 2026-08-18
  - id: path-hint
    resource: ../../crates/rocci-cli/src/path_hint.rs
    title: Boundary-safe OKF Markdown sniff used by rocci and rocdown
    author: process:cursor
    last_modified: 2026-08-18
---

# CLI entry points for Rocci, Rocdown, and OKF preview

## Goal and scope

Authors currently preview knowledge Markdown with `rocdown run`, which treats
YAML frontmatter as document body and dumps lifecycle, sources, and tags as
plain text. This plan chooses the CLI architecture for that preview path:
whether to add `rocci-okf-cli`, teach `rocdown` or `rocci` plugins, or keep
three product binaries and fix dispatch plus OKF presentation.

The plan covers command ownership, dependency direction, preview UX, and
metadata rendering. It does not redesign the portable OKF engine, the Rocdown
site generator, or a general third-party plugin marketplace.

This is an exploratory recommendation. The three-CLI product split is already
approved and implemented; a plugin host or a fourth CLI would reopen that
boundary.[^product-boundary][^system-overview]

## Established baseline

The workspace already ships three user-facing binaries with one-way
ownership:[^root-readme][^product-boundary][^deps-check]

| Product | Cargo package | Executable | `run` target |
| --- | --- | --- | --- |
| Rocci apps and `.rocci` | `rocci-cli` | `rocci` | `.rocci` file or Roc app directory |
| Rocdown documents and sites | `rocci-rocdown-cli` | `rocdown` | `.rocdown` / `.md` file or site directory |
| OKF review and query | `rocci-okf` | `rocci-okf` | knowledge bundle directory |

There is no `rocci-okf-cli` package. `rocci-okf` is already the application
binary, the way `rocci-cli` is the Rocci binary and `rocci-rocdown-cli` is the
Rocdown binary.[^okf-readme][^okf-app-plan]

Base Rocci must not depend on Rocdown or OKF. Rocdown must not depend on OKF.
`okf` depends on neither Rocdown nor Rocci. `rocci-okf` may depend on `okf` and
Rocci, and must not depend on Rocdown. Those edges are checked
mechanically.[^product-boundary][^deps-check]

Shared runtime is already a library, not a plugin. `rocci-cli` exposes a driver,
serve, and preview surface that `rocdown` and `rocci-okf` consume. The language
server uses the same composition pattern: `rocci-lsp` stays generic, and
`rocci-rocdown-lsp` composes analyzers into one product binary.[^rocci-cli-lib][^language-tooling]

`rocci run` already inspects the file extension and, for `.md` / `.markdown` /
`.rocdown`, tells the author to use `rocdown run`. It does not import Rocdown
types. Before this plan was implemented, `rocdown run` accepted those Markdown
extensions as ordinary documents and `rocci-okf run` required a bundle
directory, so authors who opened a knowledge record in Rocdown saw YAML as
body prose.[^rocci-run][^rocdown-cli][^okf-dev]

Rocdown's Markdown parser does not enable YAML frontmatter. A knowledge record
opened with `rocdown run knowledge/plans/cli-entry-points.md` therefore renders
the `---` delimiters and YAML mapping as body prose. The OKF application
already has a structured metadata surface: type, status, authority, trust,
staleness, owners, verification, generated provenance, sources, and
tags.[^rocdown-parse][^okf-presentation][^static-okf]

Canonical knowledge remains inert Markdown with OKF YAML. Previewing it must
not execute Rocdown declarations or teach Rocdown OKF policy.[^static-okf]

## The actual problem

The metadata dump is a **wrong-tool** problem first, and a presentation-polish
problem second.

1. Knowledge records are `.md`, so `rocci run` and muscle memory send authors to
   `rocdown run`.[^rocci-run][^rocdown-cli-readme]
2. `rocci-okf run` cannot open a single record path, so the dedicated viewer is
   harder to reach than the generic Markdown viewer.[^okf-cli][^okf-dev]
3. Rocdown has no YAML page-metadata contract today, so OKF frontmatter becomes
   visible noise instead of chrome.[^rocdown-parse]
4. The structured OKF metadata UI already exists on the `rocci-okf` review
   path; it is unused when the file is opened as a Rocdown document.[^okf-presentation]

A fourth CLI or a plugin host does not fix (1)–(3) by itself. A file-aware
`rocci-okf run`, plus the same class of extension/content hint already used by
`rocci run`, does.

## Options

### A. Introduce `rocci-okf-cli`

Rename or split the current `rocci-okf` package so the Cargo name matches
`rocci-cli` / `rocci-rocdown-cli`.

This is packaging symmetry, not a product. The executable is already
`rocci-okf`. A second crate would add a workspace class, a release artifact,
and a rename without changing `run` semantics or metadata HTML.[^okf-readme][^okf-app-plan]

**Reject** unless a later crate-split is needed for library-versus-binary
layering. Do not introduce the name as a new user-facing command.

### B. Plugins on `rocdown`

Keep `rocdown run` as the Markdown entry point and load an OKF plugin when the
file looks like a knowledge record.

That either gives Rocdown an OKF dependency or invents a plugin ABI for one
in-tree consumer. Both contradict the frozen rule that Rocdown does not depend
on OKF and does not contain OKF policy. It also invites OKF semantics into
ordinary `.md` documents and site builds.[^product-boundary][^static-okf]

Rocdown may later grow **generic** YAML page metadata for sites and reports.
That is a Rocdown format change, not an OKF plugin. It must not interpret
`authority`, `verified`, source drift, or profile diagnostics.[^site-plan][^static-okf]

**Reject** as the OKF preview architecture.

### C. Plugins on `rocci` as the universal entry point

Make `rocci run` dispatch through plugins so one binary covers apps, Rocdown
documents, and OKF bundles.

A plugin host has only three implementations that preserve the frozen
dependency rules:

1. **Static link.** `rocci` depends on Rocdown and OKF. Forbidden.[^deps-check]
2. **In-process native plugins.** Unstable Rust ABI, capability model, and
   fingerprinting for three first-party products. The documentation-generator
   research treated this as a later, trusted-build concern, not a CLI
   identity.[^site-plan]
3. **Exec sibling binaries.** That is a dispatcher, not a plugin system. It can
   be considered later as UX sugar, but the product binaries remain the owners.

`rocci` is already a library used by the other CLIs. Extending that library
with serve/window helpers is the approved reuse path. Turning the `rocci`
binary into a product multiplexer reverses the symmetry that `rocci run` owns
applications and `rocdown run` owns documents.[^product-boundary][^rocci-cli-readme]

**Reject** a plugin host. Defer even a thin `rocci` dispatcher until file-aware
`rocci-okf run` and cross-CLI hints are in use.

### D. Keep three CLIs; fix dispatch and OKF preview (recommended)

Keep `rocci`, `rocdown`, and `rocci-okf` as the public commands. Improve the
preview path inside `rocci-okf`, and add the same kind of boundary-safe hint
already used for `.md` files.

This matches the implemented product split, the language-server composition
pattern, and the existing shared-driver library.[^product-boundary][^language-tooling][^rocci-cli-lib]

## Recommendation

1. **Do not add `rocci-okf-cli`.** The application binary already exists.
2. **Do not add a plugin lifecycle** to `rocci` or `rocdown` for first-party
   format dispatch.
3. **Keep three `run` commands**, one per product.
4. **Make `rocci-okf run` the OKF viewer**, including a single record path.
5. **Hint, do not absorb.** `rocci` and `rocdown` may detect an OKF-looking
   `.md` file by string inspection at the CLI boundary and point at
   `rocci-okf run`, without importing `okf` types.
6. **Render OKF metadata only in `rocci-okf`.** Polish badges, provenance, and
   sources there. Rocdown may later strip or promote generic YAML as page
   metadata, but it must not grow an OKF profile.

The intended author commands become:

```text
rocci run examples/rocci/standalone/counter/Counter.rocci
rocdown run examples/rocdown/pages/Guide.rocdown
rocdown run docs
rocci-okf run knowledge
rocci-okf run knowledge/plans/cli-entry-points.md
```

## Current disposition

Phases 1–3 are implemented. `rocci-okf run` resolves a bundle directory, root
`index.md`, or concept file to a preview URL; `rocci run` and `rocdown run` /
`build` detect OKF-looking Markdown by leading-byte inspection and point at
`rocci-okf run` without importing `okf`; concept headers render badges, a
description, compact provenance, sources, and unknown fields instead of a YAML
wall. Phases 4–5 remain deferred. This record stays exploratory until a human
reviewer accepts the implemented contract.[^preview][^path-hint]

## Target contract

### Command ownership

`rocci` continues to reject `.md` and `.rocdown`. The existing hint stays.
If the path is a knowledge-bundle directory or an OKF concept file, a second
sentence may mention `rocci-okf run`. Detection remains extension- and
prefix-based string inspection.[^rocci-run][^product-boundary]

`rocdown` continues to own ordinary Markdown and Rocdown documents. When a
single `.md` file starts with YAML that looks like OKF (`type` plus
`authority`, or bundle-root `okf_version` only), `rocdown run` and
`rocdown build` should fail with a pointer to `rocci-okf run` rather than
render the YAML as prose. Ordinary Markdown without that signature stays on
the Rocdown path, including root reports that are not knowledge
records.[^rocdown-cli][^static-okf]

`rocci-okf run <path>` accepts:

- a bundle directory, as today;
- a concept file inside a bundle, loading the enclosing bundle and opening
  that concept URL;
- a bundle-root `index.md`, opening the review home.

A Markdown file outside any OKF bundle is not silently treated as a one-file
knowledge base. Report that it is not in a bundle and stop. Root
implementation plans that are not in `knowledge/` remain Rocdown or plain
Markdown documents.[^okf-engine][^okf-dev]

### Metadata presentation

`rocci-okf` remains the only renderer of OKF governance metadata. The current
concept header already exposes type, lifecycle, authority, trust, staleness,
owners, verification, generation, sources, and tags. Follow-up polish should
stay in that HTML, not in `RocdownTheme.rocci`.[^okf-presentation]

Do not reopen a `rocci-okf` → Rocdown presentation adapter to reuse the
documentation chrome. That edge was removed and is forbidden by the workspace
checker. Domain-neutral view records in `rocci-ui` remain available if both
consumers share a proven layout primitive.[^deps-check][^product-boundary]

### What is not a plugin

Allowed reuse:

- `rocci-cli` library helpers for ports, windows, and generic app plans;
- `rocci-desktop` preview;
- `rocci-ui` domain-neutral view records;
- product binaries that compose those libraries, as `rocci-rocdown-lsp`
  already composes analyzers.

Not allowed without a new normative decision:

- a `rocci` or `rocdown` plugin registry;
- dynamically loaded native modules;
- Rocdown interpreting OKF profile fields;
- base Rocci compiling or serving `.rocdown` or OKF bundles.

## Delivery phases

### 0. Freeze the CLI contract

- Record that `rocci-okf` is the OKF CLI; `rocci-okf-cli` is not a product.
- Record that plugins are out of scope for first-party format dispatch.
- Keep the three-binary dependency matrix unchanged.

Exit when this plan is the cited owner for the question.

### 1. File-aware `rocci-okf run`

- Accept a concept path, resolve the enclosing bundle, and serve the existing
  review site at that concept.
- Preserve `run knowledge` as the bundle home.
- Reject non-bundle Markdown with an explicit error.
- Add CLI and presentation tests for path resolution, missing bundle, and
  canonical concept URL.

Exit when `rocci-okf run knowledge/plans/cli-entry-points.md` opens the
structured metadata view rather than a Rocdown YAML dump.

### 2. Cross-CLI hints

- Keep the `rocci run` Markdown hint; add an OKF hint when the file or
  directory looks like a bundle or concept.
- Make `rocdown run` / `build` refuse OKF-looking concept files with a
  `rocci-okf run` pointer.
- Implement detection with extension and leading-byte inspection only.
- Do not add `okf` or `rocci-okf` to Rocdown or base Rocci package edges.

Exit when `rocdown run knowledge/plans/cli-entry-points.md` and
`rocci run knowledge/plans/cli-entry-points.md` both name `rocci-okf run`.

### 3. Metadata presentation polish

- Iterate the existing `render_concept_meta` header: scannable badges first,
  provenance and sources behind a compact summary, unknown keys recoverable
  but not dumped as a YAML wall.
- Keep unknown OKF keys visible to reviewers without making them body
  prose.
- Measure against the current review-site HTML, not against Rocdown article
  rendering.

Exit when a draft plan, a stable architecture record, and a record with
source drift each have a distinct, readable header.

### 4. Optional Rocdown YAML for ordinary documents

Only after phases 1–2, consider generic YAML frontmatter on Rocdown `.md`
pages as ordinary title/description metadata. That work belongs to the
Rocdown site-metadata track and must stay OKF-unaware.[^site-plan]

Exit only if a Rocdown fixture with non-OKF YAML renders as page chrome and a
knowledge fixture is still rejected with the phase-2 hint.

### 5. Deferred dispatcher

A later `rocci` convenience that `exec`s `rocdown` or `rocci-okf` may be
considered if authors still type `rocci run` for everything after the hints
land. It requires a separate decision: the `rocci` binary would learn sibling
command names without importing their packages. It is not a plugin system and
is not required to close the metadata-preview gap.

A dedicated `rocci-browser` host that execs sibling adapters is a different
question: session and selection, not format dispatch. It must not become a
`rocci browser` subcommand. See [rocci-browser](rocci-browser.md).[^browser-plan]

## Acceptance criteria

- `rocci-okf run <concept.md>` opens the enclosing bundle on that concept with
  structured metadata, not YAML-as-prose.
- `rocdown run` on an OKF concept file does not silently render governance
  YAML as article body.
- No new workspace package named `rocci-okf-cli`.
- No Rocdown or base-Rocci dependency on `okf` or `rocci-okf`.
- `rocci-okf` still has no Rocdown dependency.
- Canonical knowledge records remain inert Markdown.

## Decision gates

Human review is required before:

- adding a plugin registry or dynamically loaded CLI modules;
- making `rocci run` execute Rocdown or OKF work itself;
- introducing `rocci-okf-cli` as a rename;
- teaching Rocdown OKF profile fields;
- treating Markdown outside a bundle as a one-file OKF application.

Until those gates open, implement option D.

[^browser-plan]: Dedicated product-blind host with out-of-process adapters; not a `rocci` multiplexer.
[^product-boundary]: Approved three-CLI split, one-way dependencies, and the `rocci run` extension-hint exception.
[^static-okf]: Canonical records are inert OKF Markdown; Rocdown must not own OKF policy.
[^okf-app-plan]: `rocci-okf` is the approved application and Cargo namespace; `okf` is the portable engine.
[^system-overview]: Current workspace split across Rocci, Rocdown, and OKF crates.
[^language-tooling]: Product composition lives in a dedicated binary, not in base Rocci.
[^root-readme]: Current public commands for `rocci`, `rocdown`, and `rocci-okf`.
[^rocci-cli-readme]: Base `rocci` owns `.rocci` and Roc apps, not `.rocdown`.
[^rocci-run]: Implemented Markdown/Rocdown hint from `rocci run`.
[^rocci-cli-lib]: Shared driver library consumed by product CLIs.
[^rocdown-cli-readme]: `rocdown run` accepts `.rocdown`, `.md`, and `.markdown`.
[^rocdown-cli]: Implemented single-file Markdown dispatch in the Rocdown CLI.
[^rocdown-parse]: Comrak options enable tables, footnotes, and wikilinks, not YAML frontmatter.
[^okf-cli]: Current `rocci-okf` subcommands; `run` takes a bundle root.
[^okf-readme]: Documented bundle-oriented `run`, `check`, `inspect`, `search`, and `build`.
[^okf-presentation]: Structured concept metadata HTML already implemented in the OKF application.
[^okf-dev]: Preview server requires a directory knowledge root.
[^okf-engine]: Portable engine parses bundles, not arbitrary standalone Markdown as a product.
[^deps-check]: Mechanical classification forbidding Rocci→Rocdown/OKF, Rocdown→OKF, and rocci-okf→Rocdown.
[^site-plan]: Plugin lifecycles and typed page metadata are Rocdown site questions, not OKF CLI questions.
[^preview]: Implemented bundle, root-index, and concept-file preview targeting.
[^path-hint]: Implemented extension- and YAML-prefix sniff shared by `rocci` and `rocdown` without an `okf` dependency.
