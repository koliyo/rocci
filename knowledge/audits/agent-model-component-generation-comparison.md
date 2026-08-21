---
type: Audit
title: Agent-model comparison for Rocci component-generation research
description: Objective comparison of Gemini 3.7 Flash and Grok 4.6 results for the same Rocci component-generation research and planning task.
tags: [domain/rocci, domain/rocdown, domain/rocci-okf, integration/roc, concern/agents, concern/research, concern/architecture, concern/validation]
status: draft
generated: { by: process:codex, at: 2026-08-18T18:05:02Z }
stale_after: 2026-11-18
authority: exploratory
owners: [human:nils]
sources:
  - id: result-a
    resource: https://github.com/koliyo/rocci/commit/ff49f01bc4e19c8b41bdfbdf4299d806e82fc8d3
    title: Gemini 3.7 Flash component-generation result
    author: model:gemini-3.7-flash
    last_modified: 2026-08-18
  - id: result-b
    resource: https://github.com/koliyo/rocci/commit/c3d6fbe78d70ba15e5d334f43da15d8daa26977a
    title: Grok 4.6 component-generation result
    author: model:grok-4.6
    last_modified: 2026-08-18
  - id: synthesis
    resource: ../research/rocci-components-in-generation.md
    title: Accepted synthesis of the parallel component-generation drafts
    author: process:cursor
    last_modified: 2026-08-18
  - id: generator
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
  - id: build
    resource: ../../crates/rocci-rocdown/src/build.rs
    title: Rocdown Roc invocation and watch hash
    author: process:git
    last_modified: 2026-08-18
  - id: ui
    resource: ../../crates/rocci-ui/README.md
    title: Shared UI boundary contract
    author: process:git
    last_modified: 2026-08-18
  - id: dependency-check
    resource: ../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace dependency-direction checker
    author: process:git
    last_modified: 2026-08-18
  - id: review-policy
    resource: ../reference/priority-1-review.md
    title: Knowledge review and verification policy
    author: process:okf-phase-6
    last_modified: 2026-08-16
  - id: roc-platforms
    resource: https://www.roc-lang.org/platforms
    title: Roc platforms and applications
    author: organization:roc-programming-language-foundation
  - id: roc-faq
    resource: https://www.roc-lang.org/faq
    title: Roc FAQ
    author: organization:roc-programming-language-foundation
---

# Agent-model comparison for Rocci component-generation research

## Verdict

**Grok 4.6 result B is the stronger result.** It is more accurate,
repository-aware, appropriately exploratory, and useful as an implementation
plan. Gemini 3.7 Flash result A is readable and covers the requested option
space, but it repeatedly turns unmeasured hypotheses into architectural facts
and prematurely extracts product-specific UI into a shared library.[^result-a][^result-b]

The model attribution was supplied by the maintainer for this audit. Git
records the commits under the maintainer's identity, so the attribution is
context for the comparison rather than a claim inferred from commit metadata.

| Criterion | Weight | A — Gemini 3.7 Flash | B — Grok 4.6 |
| --- | ---: | ---: | ---: |
| Technical accuracy and evidence | 30 | 10 | 24 |
| Repository grounding | 20 | 11 | 19 |
| Task coverage | 20 | 17 | 18 |
| Architectural judgment | 15 | 6 | 14 |
| Actionable plan | 10 | 7 | 9 |
| Knowledge and provenance hygiene | 5 | 1 | 5 |
| **Total** | **100** | **52** | **89** |

The scores compare the single top commit on each branch from their shared
parent `ae904c9`. A added 310 lines across an architecture record and plan; B
added 637 lines across a research report, plan, indexes, and the knowledge log.
Size did not contribute directly to the score.[^result-a][^result-b]

## Why result B is stronger

B models the shipped pipeline before proposing change. It distinguishes Rust
Markdown rendering, duplicated article-block wrappers, reusable documentation
chrome, and product-specific OKF governance instead of treating all Rust HTML
as one migration opportunity.[^result-b][^generator]

Its duplication inventory is tied to concrete consumers:

- `RocdownTheme.rocci` and `site/theme/Layouts.rocci` both render navigation,
  breadcrumbs, and outlines;
- standalone Rocdown and the OKF viewer duplicate the “On this page” surface;
- `DocsComponents.rocci` overlaps `docs.rs::render_docs`.

It also identifies non-duplication: OKF governance cards and review queues do
not have Rocdown consumers, and full document shells carry product-owned CSP,
metadata, scripts, and styling. This leads to a conservative initial library
of `PageOutline`, `NavList`, and `Breadcrumbs` rather than a speculative
cross-product widget catalog.[^result-b][^synthesis][^ui]

B correctly notices that `RocdownPages.roc` currently embeds `PageView` data.
Titles, navigation, breadcrumbs, and outlines therefore affect the generated
Roc hash. Moving page data to apply time is a prerequisite for effective
cross-build renderer caching; A's claim that such changes never invalidate the
cache does not describe the current builder.[^result-b][^build]

B also makes the important distinction between invoking the compiler, hosting
a compiled Roc application through a platform and generated glue, and
embedding the compiler itself. Roc's platform model gives the non-Roc host
control of startup and calls into compiled application functions; glue is a
host ABI mechanism, not a supported Rust compiler API.[^result-b][^roc-platforms][^roc-faq]

Finally, B keeps the proposal exploratory, preserves the enforced
`rocci-okf`-to-Rocdown dependency boundary, and exposes product decisions as
open questions instead of silently deciding them.[^result-b][^dependency-check]

## Problems in result A

A's main weakness is unsupported precision:

1. Its latency table gives cold, warm, Wasm, and FFI timings without repository
   benchmarks that support those values.
2. It calls the subprocess path “zero-risk,” treats Wasm as a committed v2
   backend, and uses `<1ms`, `0ms`, and `100% test coverage` as exit criteria
   before proving the platform or packaging path.
3. It discusses embedding old Rust compiler crates such as `roc_load`,
   `roc_mono`, and `roc_gen_llvm`, while Rocci's pinned toolchain is on Roc's
   newer Zig compiler line.
4. It assumes deterministic shared templates create one machine-wide cache
   key. Roc compiles a whole renderer program; different shells, platforms,
   CSS, or baked page data create different artifacts.
5. It proposes `ConceptMetadataCard`, `ReviewDashboard`, `StatGrid`, and other
   OKF-specific components as phase-one shared-library work without a second
   product consumer.
6. It labels an unimplemented future pipeline `authority: descriptive` and
   copies an older `human:nils` verification event onto both newly generated
   records.[^result-a][^synthesis][^review-policy]

The last issue is mechanically visible: OKF validation accepted A's document
shape but introduced 13 lifecycle and source-provenance warnings for the two
new records. B introduced no new lifecycle warnings. These warnings are not
syntax errors, but they expose a material difference in knowledge hygiene.

A's useful contributions are its readable execution-model taxonomy, explicit
content-addressed cache layout, recognition that separately loadable Rocci
modules are not the current compilation model, and clear phased presentation.
Those strengths were retained where supported in the later synthesis.[^result-a][^synthesis]

## Remaining weaknesses in result B

B should not be treated as error-free:

1. The original result says `roc build --lib` produces a shared library. That
   option was absent from the pinned nightly used during evaluation, so the
   exact current compiler/platform artifact flow required a spike.
2. It proposes a Rocci renderer cache without first measuring how much of the
   same work Roc's own compilation and executable cache already avoids.
3. Its performance analysis is qualitative. A decision should measure cold
   compile time, warm compiler-cache time, applicator startup, per-page apply
   time, cache size, and invalidation behavior.
4. Glue and host references should be pinned to a compiler revision because
   Roc is pre-1.0 and the integration surface changes.
5. Release-time precompilation into `rocci-okf` or `rocdown` needs explicit
   cross-target packaging and fallback behavior.

These are correctable research gaps rather than errors in the core ownership
model. They do not overturn the comparison.[^result-b][^synthesis]

## Validation and disposition

Both diffs passed `git diff --check`. Both knowledge bundles passed structural
validation with no errors when checked with the existing `rocci-okf`
validator. Rebuilding the validator independently from the two historical
branch tips was blocked equally by missing generated `playground/dist` assets,
so that infrastructure failure was not scored against either result.

Use B as the base result. Before treating its plan as implementation-ready,
verify the pinned compiler's host artifact flow, measure the existing Roc
cache, and establish benchmark-driven cache and execution budgets. Do not use
A as the primary plan without reworking its lifecycle metadata, cache model,
performance claims, and shared-component scope.

The current component-generation research record is a later synthesis rather
than either raw result: it adopts B's ownership analysis, retains supported
parts of A's host/cache taxonomy, and explicitly records the rejected
overreach.[^synthesis]

[^result-a]: Gemini 3.7 Flash's single top commit from the shared comparison base.
[^result-b]: Grok 4.6's single top commit from the shared comparison base.
[^synthesis]: Current evidence synthesis, including its comparison of the two parallel drafts.
[^generator]: Shipped Rust-catalog, Rust-article, and Rocci-shell boundary.
[^build]: Current generated-page data, session hash, compiler subprocess, and applicator reuse.
[^ui]: Current domain-neutral shared UI contract.
[^dependency-check]: Mechanically enforced package ownership and dependency directions.
[^review-policy]: Human verification and generated-revision lifecycle rules.
[^roc-platforms]: Host ownership of startup and calls into compiled Roc application code.
[^roc-faq]: Current compiler language and platform/host model.
