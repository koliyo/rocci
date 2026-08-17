---
type: Audit
title: Rocdown product-boundary refactor completion review
description: Evidence-based review of the completed Rocdown boundary refactor, including exit-gate coverage, residual coupling, stale automation and documentation, and prioritized follow-up work.
tags: [domain/rocci, domain/rocdown, domain/okf, concern/architecture, concern/migration, concern/tooling, concern/validation]
status: draft
generated: { by: process:codex, at: 2026-08-17T21:38:11Z }
stale_after: 2026-11-15
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/rocdown-boundary-refactor.md
    title: Rocdown product-boundary refactor plan
    author: process:codex
    last_modified: 2026-08-17
  - id: decision
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-17
  - id: workspace
    resource: ../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-17
  - id: dependency-check
    resource: ../../scripts/check-workspace-deps.py
    title: Workspace dependency-direction checker
    author: process:cursor
    last_modified: 2026-08-17
  - id: ci-workflow
    resource: ../../.github/workflows/ci.yml
    title: Main CI workflow
    author: process:git
    last_modified: 2026-08-17
  - id: knowledge-workflow
    resource: ../../.github/workflows/knowledge.yml
    title: Knowledge validation workflow
    author: process:git
    last_modified: 2026-08-17
  - id: ci-run
    resource: https://github.com/koliyo/rocci/actions/runs/32070977528
    title: Failed CI run for Phase 8 commit aa9b032
    author: process:github-actions
    last_modified: 2026-08-17
  - id: knowledge-run
    resource: https://github.com/koliyo/rocci/actions/runs/32070977526
    title: Failed Knowledge run for Phase 8 commit aa9b032
    author: process:github-actions
    last_modified: 2026-08-17
  - id: rocci-lsp
    resource: ../../crates/rocci-lsp/src/lib.rs
    title: Generic Rocci language-server core
    author: process:git
    last_modified: 2026-08-17
  - id: rocci-lsp-main
    resource: ../../crates/rocci-lsp/src/main.rs
    title: Shipped Rocci language-server binary
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-lsp
    resource: ../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown-owned language analyzer
    author: process:git
    last_modified: 2026-08-17
  - id: vscode
    resource: ../../editors/vscode/package.json
    title: VS Code extension manifest
    author: process:git
    last_modified: 2026-08-17
  - id: zed
    resource: ../../editors/zed/extension.toml
    title: Zed extension manifest
    author: process:git
    last_modified: 2026-08-17
  - id: okf-app
    resource: ../../crates/rocci-okf/Cargo.toml
    title: Rocci OKF application manifest
    author: process:git
    last_modified: 2026-08-17
  - id: ui-readme
    resource: ../../crates/rocci-ui/README.md
    title: Shared UI boundary contract
    author: process:git
    last_modified: 2026-08-17
  - id: ui-view
    resource: ../../crates/rocci-ui/src/view.rs
    title: Shared UI view records
    author: process:git
    last_modified: 2026-08-17
  - id: ui-html
    resource: ../../crates/rocci-ui/src/html.rs
    title: Shared UI HTML renderers
    author: process:git
    last_modified: 2026-08-17
  - id: ui-css
    resource: ../../crates/rocci-ui/src/themes/base.css
    title: Shared UI base styles
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-config
    resource: ../../crates/rocci-rocdown/src/config.rs
    title: Rocdown configuration loader
    author: process:git
    last_modified: 2026-08-17
  - id: theme
    resource: ../../crates/rocci-theme/Cargo.toml
    title: Rocci theme package manifest
    author: process:git
    last_modified: 2026-08-17
  - id: examples-doc
    resource: ../../docs/examples/index.rocdown
    title: Published examples index
    author: human:nils
    last_modified: 2026-08-17
  - id: language-knowledge
    resource: ../architecture/language-tooling.md
    title: Language-tooling architecture record
    author: process:codex
    last_modified: 2026-08-17
  - id: implementation-status
    resource: ../status/implementation.md
    title: Rocci implementation status record
    author: process:codex
    last_modified: 2026-08-17
  - id: knowledge-log
    resource: ../log.md
    title: Rocci knowledge log
    author: process:git
    last_modified: 2026-08-17
  - id: playground-plan
    resource: ../../ROCCI_PLAYGROUND_IMPLEMENTATION_PLAN.md
    title: Root playground implementation plan
    author: process:codex
    last_modified: 2026-08-17
---

# Rocdown product-boundary refactor completion review

## Executive verdict

The refactor successfully moved the principal product boundary. The old
`rocs`, `rocs-cli`, and `rocci-wry` packages are gone; `rocdown` owns document
and site commands; base `rocci-cli` has no Rocdown dependency; the generic
Rocci driver and language-tooling extension points exist; the portable `okf`
engine is independent; and the default Rust workspace tests pass on the clean
Phase 8 tree.[^workspace][^plan]

The refactor is nevertheless **not complete against its own exit gates**. The
Phase 8 commit `aa9b032` was red in both GitHub workflows, Rocdown editor
support is no longer wired into a shipped server or extension, the temporary
`rocci-okf -> rocci-rocdown` dependency is unnecessary, the extracted
`rocci-ui` package contains OKF-specific and unused surface, and several
canonical records and active documents still describe the pre-refactor
architecture.[^ci-run][^knowledge-run][^rocdown-lsp][^okf-app][^ui-html][^language-knowledge]

The recommended disposition is to treat Phases 1, 2, 4, 5, and the portable
engine portion of Phase 6 as complete; treat Phases 3, 7, and 8 as requiring a
focused closure pass; and keep the plan and decision in `draft` until the
blocking integrations below are closed and independently reviewed.[^plan][^decision]

## Scope and method

This review compared every phase and exit gate in the plan with package
manifests, dependency enforcement, CLI ownership, source modules, tests,
editor manifests, workflows, public documentation, and canonical knowledge.
It reviewed the clean Phase 8 commit `aa9b032` separately from later work on
`main`, because `rocci-datastar` landed after the refactor and temporarily
invalidated the dependency classification.[^plan][^workspace][^dependency-check]

Validation used a clean exported tree for `aa9b032`, not generated `dist/`
output. The clean tree passed the default workspace tests, Rust formatting,
the dependency-direction script, Rocdown AST inspection, Rocdown documentation
checking, and OKF validation with seven lifecycle warnings. The full docs build
could not be used as evidence because the local Roc toolchain failed to start
its FSEvents watcher in the audit environment. GitHub Actions is authoritative
for the two workflow failures described below.[^ci-run][^knowledge-run]

## Exit-gate assessment

| Phase | Assessment | Evidence and remaining gap |
| --- | --- | --- |
| 0 — freeze contract | Mostly complete | Names and dependency rules are encoded, but the temporary OKF presentation exception has no linked removal issue and the checker comments still describe migration-era state.[^decision][^dependency-check] |
| 1 — characterize and extract seams | Substantially complete | Golden parser/generator/LSP coverage, a generic driver, reusable highlighting primitives, and `DocumentAnalyzer` extension points exist.[^rocci-lsp][^rocdown-lsp] |
| 2 — unified Rocdown product | Complete for CLI and facade | `rocci-rocdown-cli` owns `rocdown` commands and the facade owns language, catalog, article, build, and development behavior.[^workspace] |
| 3 — remove Rocdown from base Rocci | Incomplete at the product edge | Cargo direction is correct, but the Rocdown analyzer is not instantiated by any shipped language-server binary and neither editor currently registers Rocdown.[^rocci-lsp-main][^rocdown-lsp][^vscode][^zed] |
| 4 — retire Rocs | Core complete, cleanup incomplete | Old packages, binary, configuration, and generated module names are gone, but active plans, knowledge identifiers, and public copy retain current-tense Rocs claims.[^plan][^playground-plan][^language-knowledge] |
| 5 — desktop rename | Complete | Active packages and imports use `rocci-desktop`; remaining old-name mentions are historical plan text.[^workspace] |
| 6 — separate OKF | Engine complete; adapter cleanup incomplete | `okf` is portable and Rocdown has no OKF edge, but `rocci-okf` still declares a Rocdown dependency that its source does not use.[^okf-app][^dependency-check] |
| 7 — extract shared UI only from demonstrated duplication | Needs redesign | Shared page-view records are used by Rocdown, but most HTML helpers, the Rocci UI template, badge/alert records, and OKF-prefixed styles are unused or OKF-specific rather than demonstrated cross-product primitives.[^ui-readme][^ui-view][^ui-html][^ui-css] |
| 8 — docs, knowledge, and release cleanup | Incomplete | Both workflows failed at the completion commit, public examples and knowledge are stale, and the knowledge log claims verification that the live runs disprove.[^ci-run][^knowledge-run][^examples-doc][^knowledge-log] |

## Blocking findings

### F-01 — Phase 8 was declared complete while both required workflows were red

**Severity:** P0 release blocker.

The main CI run for `aa9b032` failed in two places. The fixture job invoked
`rocci-cli -- inspect --ast` on `.rocdown` files after the base CLI had been
changed to reject every non-`.rocci` input. The lint job also failed on
`okf::markdown::walk_node` under `-D warnings` because it had eleven
parameters.[^ci-workflow][^ci-run]

The Knowledge workflow invoked the removed `rocdown knowledge` compatibility
surface for validation, graph inspection, benchmarking, and deterministic
builds. It failed immediately with `unrecognized subcommand 'knowledge'`.[^knowledge-workflow][^knowledge-run]

Follow-up commit `b353895` corrected both workflows and the Clippy finding
after the evidence pass. The fixes passed targeted local validation, but their
existence does not change the conclusion that the Phase 8 exit gate was not
met on the revision that declared completion. Add a release rule that a phase
cannot be marked complete in the knowledge log until required GitHub workflows
have completed successfully for that revision.[^knowledge-log]

### F-02 — Rocdown LSP implementation exists, but no product wiring ships it

**Severity:** P0 functional regression.

The refactor correctly introduced a generic `DocumentAnalyzer` interface in
`rocci-lsp` and moved `RocdownAnalyzer` into `rocci-rocdown`. The shipped
`rocci-language-server` binary, however, constructs `LanguageServer::new()`,
which registers only `RocciAnalyzer`. No binary constructs a server with
`RocdownAnalyzer`.[^rocci-lsp][^rocci-lsp-main][^rocdown-lsp]

The VS Code manifest registers only the `rocci` language and selects only
`.rocci` documents. The Zed manifest likewise attaches the server only to
`Rocci`. Their READMEs and a dormant VS Code integration test still claim
Rocdown support, so documentation and tests obscure the regression rather
than detect it.[^vscode][^zed]

Choose and implement one explicit product design:

1. add a Rocdown-owned server binary that composes the generic core with
   `RocdownAnalyzer`, and wire `.rocdown` to that binary; or
2. add a thin product composition binary that registers both analyzers while
   keeping the generic server crate free of Rocdown types.

Then run editor-host integration tests for both file types in CI. Merely
testing `RocdownAnalyzer` through `LanguageServer::with_analyzers` in a Rust
integration test is insufficient evidence for shipped editor support.

### F-03 — The dependency audit was invalidated by the first post-refactor crate

**Severity:** P0 continuous-enforcement blocker; separate from the refactor
implementation itself.

The post-refactor `rocci-datastar` commit added a workspace package without
classifying it in `scripts/check-workspace-deps.py`, causing the dependency
check to fail with `unclassified workspace package rocci-datastar`.[^workspace][^dependency-check]

Follow-up commit `b353895` contains the obvious classification fix. Add a small
regression test or contributor instruction requiring every new workspace
member to be classified in the same change. This incident is useful evidence
that the checker is enforcing the rule, but also that package addition and
boundary classification are not yet one atomic workflow.

## High-priority architectural findings

### F-04 — The temporary OKF-to-Rocdown edge is unused and should be deleted

**Severity:** P1 boundary debt.

`rocci-okf` declares `rocci-rocdown`, and the dependency checker explicitly
allowlists that temporary presentation edge. Direct source inspection finds no
`rocci_rocdown` use in the application; the portable engine has its own inert
Markdown path and the application renders through its own presentation module
plus `rocci-ui`.[^okf-app][^dependency-check]

Remove the dependency and the temporary allowlist now rather than creating a
tracking issue for an adapter that is no longer present. Also remove the
unused direct `rocci-core` dependency from `rocci-okf`; it reaches shared
serving behavior through `rocci-cli`, and no application source imports
`rocci_core`.

### F-05 — `rocci-ui` does not satisfy the Phase 7 extraction rule

**Severity:** P1 architecture revisit.

The package contract says it is domain-neutral, but its renderer and CSS emit
`okf-*` classes, its badge tones encode OKF lifecycle, trust, authority, and
review-action vocabulary, and its base variables use the Rocdown-specific
`--rd-*` prefix.[^ui-readme][^ui-view][^ui-html][^ui-css]

The actual cross-product reuse is narrower than the package surface. Rocdown
uses the generic page, navigation, breadcrumb, outline, and resource view
records. `rocci-okf` uses the stat-card renderer. The shell renderer,
`BadgeView`, `AlertView`, most badge and alert renderers, `ROCCI_UI_TEMPLATE`,
and `BASE_CSS` have no non-test consumers. This is speculative extraction,
which Phase 7 explicitly prohibited.[^plan][^ui-view][^ui-html]

Revisit the package from observed consumers:

- retain only genuinely shared, neutral view records and primitives;
- move OKF lifecycle/review tones and `okf-*` presentation back to
  `rocci-okf`;
- either rename shared CSS classes and variables neutrally or keep product CSS
  in each product until identical use is demonstrated; and
- delete unused shell/template/rendering surface unless a second real consumer
  is introduced with parity tests.

### F-06 — Canonical knowledge still describes mutually incompatible states

**Severity:** P1 governance and retrieval accuracy.

The refactor plan and approved decision still say the architecture is not
implemented. The language-tooling record cites a deleted
`crates/rocci-lsp/src/rocdown.rs`, describes a removed custom request, and
claims both editors register Rocdown. The implementation status and roadmap
also claim shipped Rocdown editor registration. The current architecture
record named `rocs-documentation-compiler.md` describes the shipped Rocdown
generator under an obsolete concept identifier rather than as an explicitly
historical record.[^plan][^decision][^language-knowledge][^implementation-status]

The Phase 8 knowledge-log entry says the obsolete-name audit, workspace
dependencies, and test matrix were verified, but the corresponding GitHub
runs failed. Preserve that entry as historical provenance, add a correction,
and revise current records from source evidence. Do not mark them stable or
add human verification as part of the mechanical cleanup.[^knowledge-log][^ci-run][^knowledge-run]

Recommended record work:

- mark the refactor plan completed or historical only after the open closure
  items are resolved;
- revise the decision's status section to distinguish implemented direction
  from remaining deviations;
- rewrite `architecture/language-tooling.md` for the generic-core plus
  product-analyzer reality and current editor gap;
- correct `status/implementation.md`, `status/known-limitations.md`, the system
  overview, theming, and the priority review checklist;
- rename the current Rocdown generator concept away from the `rocs-*`
  identifier, updating graph references and the hard-coded featured concept in
  `rocci-okf`; and
- rename or explicitly historicalize the `rocs-okf` plan identifier.

### F-07 — Active documentation and plans retain executable pre-refactor advice

**Severity:** P1 user and maintainer friction.

The published examples page still tells users to run `.rocdown` files with
`rocci run`, which now rejects them. Root `AGENTS.md` tells agents to inspect
Rocdown syntax through the base CLI, and the DevOps skill retained the deleted
knowledge commands until an uncommitted audit-time correction. The root
playground and language-server plans, branding launch material, and agent-skill
plan contain current-tense references to `rocs`, `rocci-wry`, or the old shared
server architecture.[^examples-doc][^playground-plan]

Update commands that remain operational guidance. For large dated research
reports, prefer moving them under `archive/` or adding an unmistakable
superseded banner rather than mechanically rewriting historical analysis.
Active future plans must be rebased onto the new ownership model before anyone
implements them.

## Medium-priority cleanup findings

### F-08 — The theme seam remains public and split across the facade

**Severity:** P2 boundary simplification.

The target allowed `rocci-theme` to be renamed or internalized under Rocdown.
It remains a workspace package, `rocci-rocdown` publicly re-exports its types,
and `rocci-rocdown-cli` depends on it directly only for integration tests.
The user-facing environment variable remains `ROCCI_THEME` even though theme
selection is now a Rocdown product contract.[^plan][^theme]

Decide explicitly whether `rocci-theme` is a supported ecosystem crate. If it
is internal, consume it only through `rocci-rocdown`, remove the CLI's direct
dependency, and consider a Rocdown-owned package/module name plus
`ROCDOWN_THEME`. If it is intentionally public, amend the plan and decision so
the retained seam is no longer an unexplained migration exception.

### F-09 — Migration aliases remain in the Rocdown configuration API

**Severity:** P2 dead public surface.

`CONFIG_FILE` and `ROCDOWN_CONFIG_FILE` are identical, while `load_config` and
`load_rocdown_config` are equivalent. `load_rocdown_config` and
`ROCDOWN_CONFIG_FILE` have no callers outside their definitions and re-exports.
`load_config_named` is also public despite having no workspace consumer.[^rocdown-config]

Keep one canonical constant and loader. Make filename-parametric loading
private unless external callers are deliberately supported. This reduces the
facade to the single-configuration contract promised by Phase 2.

### F-10 — Base CLI still special-cases Rocdown extensions after dispatch removal

**Severity:** P2 plan deviation, low coupling risk.

`rocci run` no longer parses Rocdown, but its entry resolver still recognizes
`.rocdown`, `.md`, and `.markdown` to produce a `rocdown run` suggestion. This
violates the plan's literal instruction to delete format dispatch, although it
does not create a package dependency and is arguably useful error guidance.

Record an explicit exception if this user-facing hint is intentional. If the
goal is complete base-product ignorance, replace the special case with a
generic unsupported-extension error.

### F-11 — Release packaging does not match the documented product set

**Severity:** P2 release-contract gap.

The release workflow packages `rocci`, `rocdown`, and
`rocci-language-server`, but omits the documented `rocci-okf` application. It
also packages a language server that the root README describes as supporting
Rocdown even though the binary registers only Rocci.[^ci-workflow][^rocci-lsp-main][^implementation-status]

After the editor composition decision, make the release archive reflect the
actual supported binaries. Either include `rocci-okf` or document it as a
source-only/internal tool. Do not advertise the current server as a Rocdown
server.

## What is solid and should be preserved

- The Cargo dependency direction at the Phase 8 commit is structurally sound:
  base Rocci does not depend on Rocdown or OKF, Rocdown does not depend on OKF,
  and `okf` has no workspace dependencies.[^workspace][^dependency-check]
- Standalone Rocdown uses the extracted generic Rocci driver rather than
  reintroducing document parsing into `rocci-cli`.[^plan]
- Rocdown owns parsing, lowering, cataloging, article rendering, planning,
  building, live preview, highlighting composition, and its analyzer behind
  one facade.[^workspace][^rocdown-lsp]
- The Rocs packages, command, configuration filename, theme module, build
  module, and staging environment were physically retired or renamed.
- Default workspace tests provide strong coverage for parser recovery,
  generated Roc and source maps, catalog determinism, atomic last-good builds,
  static article rendering, CLI parity, OKF determinism, and analyzer behavior.
- The dependency checker fails closed on unknown packages, as demonstrated by
  `rocci-datastar`; preserve that property when improving its maintenance
  workflow.[^dependency-check]

## Recommended closure sequence

1. Verify follow-up commit `b353895` with green CI and Knowledge runs.
2. Restore shipped Rocdown language-server and editor integration at the
   Rocdown product boundary, with editor-host tests enabled in CI.
3. Delete the unused `rocci-okf -> rocci-rocdown` edge and its temporary
   allowlist; remove other manifest-only direct dependencies found alongside
   it.
4. Reduce `rocci-ui` to demonstrated neutral reuse and return OKF-specific
   presentation to `rocci-okf`.
5. Reconcile canonical knowledge and current public docs with the actual
   implementation; archive or supersede dated plans and reports.
6. Decide and document the retained `rocci-theme` seam, then remove redundant
   configuration aliases and direct theme dependencies.
7. Align release artifacts with the supported CLI and language-server product
   set.
8. Re-run the complete closure gate: dependency audit, formatting, Clippy,
   workspace and doc tests, both syntax inspectors, editor integration, docs
   check and build, OKF profile/benchmark/determinism checks, and both GitHub
   workflows.

## Validation record

### Clean Phase 8 commit `aa9b032`

- `python3 scripts/check-workspace-deps.py`: passed with 12 classified packages.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace`: passed after allowing loopback socket tests; ignored
  exhaustive fuzz and performance suites remained ignored by design.
- `cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown`:
  passed.
- `cargo run -q -p rocci-rocdown-cli -- check docs`: passed.
- `cargo run -q -p rocci-okf -- check knowledge --profile rocci --format
  terminal`: no errors; seven `OKF4005` warnings for records revised after
  human verification.
- `cargo clippy -p okf -- -D warnings` in a fresh target directory: failed on
  `walk_node` having eleven arguments, matching GitHub run `32070977528`.
- Main CI run `32070977528`: failed in lint and Rocdown fixture inspection.[^ci-run]
- Knowledge run `32070977526`: failed on the removed `rocdown knowledge`
  command.[^knowledge-run]
- Full docs build: inconclusive in the audit environment because Roc failed to
  start FSEvents; this was not classified as a product regression.

### Follow-up commit `b353895`

The concurrently authored follow-up committed correct Rocdown fixture
commands, `rocci-okf` knowledge workflow commands, Clippy context structs,
DevOps-skill command updates, and `rocci-datastar` classification. Those files
were not edited by this audit. On `b353895`, the 13-package dependency check,
formatting, workspace Clippy, and Rocdown `EmbeddedLanguages.rocdown`
inspection passed locally. No live GitHub run for the commit was visible when
the audit report was finalized.

The git-aware knowledge check passed with no errors but reported the seven
expected `OKF4005` stale-verification warnings plus numerous `OKF4006` warnings
because cited implementation and documentation sources have changed since the
retained human verification events. These warnings are governance work, not a
reason to falsify verification metadata.

## Closure criteria

This audit can be closed when all P0 and P1 findings are resolved or explicitly
accepted by a reviewed decision, both GitHub workflows are green on the same
revision, the knowledge bundle passes with warnings separately accounted for,
and a maintainer verifies the corrected current-state records. P2 findings may
remain only when their public compatibility and ownership consequences are
documented.

[^plan]: Planned phases, architectural exit gates, prohibited reverse dependencies, and cleanup criteria.
[^decision]: Approved names, ownership rules, compatibility policy, diagnostic policy, and temporary OKF exception.
[^workspace]: Current workspace packages and direct dependency declarations.
[^dependency-check]: Mechanical package classification, forbidden-edge policy, and temporary OKF presentation allowlist.
[^ci-workflow]: Main formatting, lint, test, fixture, docs, and editor workflow commands.
[^knowledge-workflow]: Knowledge implementation, validation, retrieval, and deterministic-build workflow commands.
[^ci-run]: Live GitHub evidence for the Phase 8 main CI failures.
[^knowledge-run]: Live GitHub evidence for the Phase 8 Knowledge workflow failure.
[^rocci-lsp]: Generic analyzer extension point and default Rocci-only analyzer registration.
[^rocci-lsp-main]: Shipped binary construction of the default language server.
[^rocdown-lsp]: Rocdown-owned analyzer implementation and compilation-backed editor features.
[^vscode]: VS Code language registration and document selector contract.
[^zed]: Zed language-server registration and attached language list.
[^okf-app]: Direct dependencies declared by the Rocci OKF application.
[^ui-readme]: Claimed domain-neutral shared UI contract and dependency rules.
[^ui-view]: Shared view records and product-specific badge tone vocabulary.
[^ui-html]: Shared HTML helpers and their emitted CSS classes.
[^ui-css]: Shared base variables and OKF-specific presentation selectors.
[^rocdown-config]: Public configuration constants and equivalent loader functions.
[^theme]: Retained standalone theme package identity.
[^examples-doc]: Public commands for running the Rocdown examples.
[^language-knowledge]: Current canonical claims and citations for language-server and editor behavior.
[^implementation-status]: Current shipped-feature and editor-registration claims.
[^knowledge-log]: Phase completion and validation claims recorded on 2026-08-17.
[^playground-plan]: Active proposed work still based on pre-refactor package and product names.
