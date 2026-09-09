---
type: Implementation Plan
title: Measure Html emit, then fuse static chunks or unify Html.roc
description: "Phase 0 recorded: constructor-call compile and render are noise next to basic-cli main wrap. Phase 1 chose E (status quo). No fusion, no Html.roc unify, no debug/release emit. Closed after documenting why."
tags: [domain/rocci, domain/rocdown, domain/runtime, integration/roc, concern/performance, concern/rendering, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-09-09T10:30:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/html-node-lowering.md
    title: Emit shape versus Html runtime
    author: process:cursor
    last_modified: 2026-09-09
  - id: lower-html
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Html.element / fragment / text emit
    author: process:git
    last_modified: 2026-08-31
  - id: lower-mod
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: LowerOptions.html_type
    author: process:git
    last_modified: 2026-09-09
  - id: platform-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: HtmlNode render walk
    author: process:git
    last_modified: 2026-09-03
  - id: cli-html
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Product fragment serializes to Raw
    author: process:git
    last_modified: 2026-08-30
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: String Html for theme, tests, playground
    author: process:git
    last_modified: 2026-08-18
  - id: golden
    resource: ../../../crates/rocci-template/tests/fixtures/all_syntax.roc
    title: Verbose constructor golden
    author: process:git
    last_modified: 2026-09-09
  - id: compile-tests
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Goldens and Html.render fixture tests
    author: process:git
    last_modified: 2026-09-09
  - id: source-map
    resource: ../../../crates/rocci-template/src/source_map.rs
    title: StaticMarkup and TextExpression segments
    author: process:git
    last_modified: 2026-08-25
  - id: datastar-roc
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Serialize then strip style tags
    author: process:git
    last_modified: 2026-08-30
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Public component lowering contract
    author: process:git
    last_modified: 2026-09-09
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: "@component returns Html"
    author: process:okf-migration
    last_modified: 2026-08-31
  - id: rust-catalog
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Static article HTML stays in Rust
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: inspector-plan
    resource: inspector-source-views.md
    title: Inspector source / AST / Roc / HTML views
    author: process:cursor
    last_modified: 2026-08-19
  - id: article-render
    resource: ../../../crates/rocci-rocdown/src/docs/render.rs
    title: render_article String path
    author: process:git
    last_modified: 2026-08-31
  - id: theme-plan
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: Painters compile with html_type Str
    author: process:git
    last_modified: 2026-08-31
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Parser tests stay off the server
    author: process:git
    last_modified: 2026-08-31
---

# Measure Html emit, then fuse static chunks or unify Html.roc

Exploratory. Research:
[emit shape versus Html runtime](/research/rocci/html-node-lowering.md).[^research]

Phase 0 recorded 2026-09-09. Phase 1 chose **E (status quo)** from those numbers: both `roc check` and one-shot render are noise next to basic-cli `main` wrap through NavList. Phases 2–4 are skipped. Phase 5 documents the unchanged public emit.

## Goal

Know whether constructor-call lowering is the compile or runtime problem, then implement **one** chosen fix: unify the two `Html.roc` backends, fuse static chunks in the lowerer, or keep the status quo. Do not ship debug/release lowering modes unless Phase 1 explicitly chooses them.[^research][^lower-html]

## Out of bound

- A second product lowerer (`--html-nodes` / `--html-strings`) unless Phase 1 records that choice.
- Interpreting `.rocci` templates in Rust to skip a Rocci theme.
- Lowering static catalog prose to Roc `Html.element` (Rust `render_article` stays).[^rust-catalog][^article-render]
- Changing `@component` into an instance with state or effects.[^pure-render]
- Roc-native template compiler cutover, hosted parser glue, or Datastar CQRS policy.
- Requiring `ROCCI_REQUIRE_ROC=1` or `cargo test --workspace` for parser-only goldens.[^language-dev]
- A `roc-lang` PR for Html performance.

## Constraints that do not move

1. **One composition API.** `@component` remains a pure function from explicit values to Html. Handlers still `Html.render` or `Datastar.patch_elements`. Authors do not write markup strings in Roc to “go faster.”[^pure-render][^template-readme]
2. **Escape holes.** Fused static chunks are trusted compiler output. Dynamic text and attribute expressions keep the same escaping as `Html.text` / `Html.attribute` today.[^platform-html][^ui-html]
3. **Source maps at holes.** Interpolation and attribute expressions keep `TextExpression` / `AttributeExpression` segments. Fused literals may coarsen `StaticMarkup`.[^source-map]
4. **Static docs stay Rust.** Theme painters may keep `html_type: Str`; that is a signature, not a second emit.[^theme-plan][^rust-catalog]
5. **Inspector debug is views, not a second compiler.** Prefer source / AST / generated HTML over a debug-only lowerer.[^inspector-plan]
6. **Default tests stay in-process.** Roc compile benchmarks in Phase 0 are explicit, numbered, and not the crate default suite.[^language-dev]

## Phase 0 — Measure constructor emit against both Html backends

Bound:

- Pick a small fixed set: AllSyntax golden (or `hello` + one CSS-wrapped component from it), standalone Counter, and one theme painter (`NavList` or equivalent). Record generated Roc bytes and `Html.element` / `Html.fragment` counts.[^golden][^lower-html]
- For the same generated Roc, time `roc check` / a one-shot `Html.render` (or playground snapshot apply) once with platform/node Html and once with `rocci-ui` string Html. Do not change the lowerer.[^platform-html][^ui-html][^cli-html]
- Hand-write one fused-string twin of a tiny `Hello` component (static markup literal + `Html.text(name)` hole). Time `roc check` and render against the constructor-emitted `hello`. This is evidence for fusion, not a product path.
- Write the numbers and the suspected bottleneck (compile AST vs node walk vs `fragment` serialize vs escape `fold`) into the research record. Do not implement a fix in this phase.

**Exit:** numbers in [the research record](/research/rocci/html-node-lowering.md); `okmate check knowledge --profile base`. No production code change required.

**Outcome (2026-09-09):** sizes and timings are in the research. Suspected bottleneck is Roc process + basic-cli `main` wrap, not constructor AST or Html backend. NavList (99 constructors, 24 kB) `roc check` ~75–78 ms versus empty main 70 ms; one-shot render ~3 ms on both backends. AllSyntax counted only (incomplete Roc). Fused Hello twin did not move `roc check`. Dual-runtime HTML still diverges (doctype; quote escape in `<style>` text).

## Phase 1 — Human decision gate

Bound: **human decision only.** Record the choice in this plan or the research. Do not implement until chosen.

Options (from the research):[^research]

- **A.** Unify Html runtimes (string on the product path, or nodes everywhere including theme/tests). Lowerer unchanged.
- **B.** Static chunk fusion in `lower/html.rs` as the **only** emit. Goldens rewrite once.
- **C.** Dual lowering modes (debug nodes, release strings). Allowed only if Phase 0 shows a tool that cannot use AST/HTML inspector views.
- **E.** Status quo. Close the plan after documenting why.

Default recommendation if numbers are unclear: **do not choose C.** Prefer B when `roc check` dominates; prefer A when only CSS-wrapped `fragment` render dominates; prefer E when both are noise next to Roc compile of `main` wrap.

**Exit:** written choice. No code required.

**Outcome (2026-09-09):** **E. Status quo.** Phase 0 `roc check` (~75–78 ms NavList vs 70 ms empty main) and one-shot render (~3 ms on both backends) are noise next to the basic-cli `main` wrap. Do not choose **C**. **B** would rewrite goldens without moving `roc check`. **A** would still unify doctype / quote-escape semantics; it is not this plan’s performance fix. Phases 2–4 skipped. Dual-runtime drift remains documented in the research, not a second emit.

## Phase 2 — Runtime unify (only if Phase 1 chose A)

Bound:

- One Html semantics for void tags, boolean attributes, doctype, `fragment`, and `render`. Product `rocci run` and theme/`rocci test`/playground snapshot must agree on serialized HTML for the same constructor tree.[^platform-html][^ui-html][^cli-html]
- If string Html wins: platform `Html.element` may concatenate; keep the constructor names the lowerer already emits. If nodes win: `rocci-ui` / `rocci-rocdown` Html must become the node module or a wrap, and painters’ `html_type: Str` must be updated or justified.
- `Html.fragment` must not silently disagree (eager `Raw` versus `join`). Record whether eager serialize stays.
- Tests: existing `Html.roc` expects plus one shared serialization fixture used from platform and ui if both files remain.
- Do not fuse static chunks in this phase.

**Exit:** `cargo test -p rocci-platform` (if platform Html changed), `cargo test -p rocci-template --test compile`, `cargo fmt --all -- --check`. A focused Roc render comparison is allowed but not the default crate suite.

## Phase 3 — Static chunk fusion (only if Phase 1 chose B)

Bound:

- In `crates/rocci-template/src/lower/html.rs`, adjacent static elements and text become one trusted literal (via `Html.dangerously_include_unescaped_html` or equivalent) around dynamic holes. `@if` / `@for` / `@match`, interpolations, attribute expressions, and component calls stay Roc.[^lower-html]
- CSS inject may emit a style literal plus content rather than a node list that `fragment` immediately serializes. Do not change Datastar patch policy except that omitted style siblings need no later string-strip for that case.[^datastar-roc]
- Update AllSyntax golden and compile tests that grep `Html.element` only where the static prefix disappeared. `Html.text(expr)` holes remain.[^golden][^compile-tests]
- Source-map: fused literal is `StaticMarkup` or `Scaffolding`; holes keep expression kinds.[^source-map]
- Rocdown Markdown lowering on the hydrate path may fuse in a follow-on; this phase may stop at `rocci-template` if that is enough to prove the contract.
- Do not add a debug emit beside fusion.

**Exit:** `cargo test -p rocci-template`; `cargo test -p rocci-rocdown` if Markdown fusion landed; `cargo fmt --all -- --check`; `okmate check knowledge --profile base` if public README/docs changed.

## Phase 4 — Inspector presentation (optional; not a second emit)

Bound:

- If fusion made generated Roc harder to read, document that the inspector HTML and AST tabs are the debug structure. Do not add `--html-nodes` here.[^inspector-plan]
- Optional: pretty-print template AST as a tree in inspect output. No new lowerer.

**Exit:** existing inspector/inspect CLI tests if that surface changed; otherwise a README sentence. `okmate check knowledge --profile base` if knowledge or public docs changed.

**Skipped:** fusion did not land, so generated Roc is still constructor trees. No inspector pretty-printer.

## Phase 5 — Public contract

Bound:

- `crates/rocci-template/README.md` states the emit shape (constructors, fused literals, or both at holes) and that Html runtime may be `Node` or `Str` without a second language.[^template-readme]
- Do not copy this plan into `docs/` unless the public rendering model page already discusses generated Roc.

**Exit:** `okmate check knowledge --profile base`.

**Outcome (2026-09-09):** README Generated Roc section states constructor emit, `html_type` as a linked-runtime signature (`Html.Node` vs `Str`), and that static chunks are not fused. No `docs/` copy of this plan.

## Tests

Phase 0 is measurement notes. Phase 2–3 are the code floor: `cargo test -p rocci-template --test compile` and fmt. Roc-gated compile timing stays out of the default suite unless a later phase adds an ignored benchmark.

[^research]: Options A–E and why dual emit is expensive.
[^lower-html]: Current constructor emit and CSS `fragment` wrap.
[^lower-mod]: `html_type` is a signature, not an emit mode.
[^platform-html]: Node tree and escape walk.
[^cli-html]: Product `fragment` flattens to `Raw`.
[^ui-html]: String constructors.
[^golden]: AllSyntax lowered Roc size and shape.
[^compile-tests]: Fixture `Html.render` comparisons and `Html.element` greps.
[^source-map]: Segment kinds the fusion must preserve at holes.
[^datastar-roc]: Style siblings are stripped after serialize today.
[^template-readme]: Public “lowers to Html functions” contract.
[^pure-render]: Pure render decision.
[^rust-catalog]: Catalog HTML stays in Rust.
[^inspector-plan]: Existing debug views.
[^article-render]: `render_article` returns a Rust string.
[^theme-plan]: Painters already annotate `Str`.
[^language-dev]: No server in parser tests; Roc compile is explicit.
