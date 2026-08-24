---
type: Implementation Plan
title: Rocdown @{expr} follow-ons after v1
description: "After Markdown `@{expr}` on branch `rocdown-inline-interpolation`: fail closed on the Rust article path, align hydrate docs, lock splitter and placement tests, and stop LSP go-to from targeting the hole when no binding exists. Do not add heading or URL interpolation."
tags: [domain/rocdown, domain/rocci, concern/syntax, concern/authoring, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-23T14:37:00Z }
stale_after: 2026-11-23
authority: exploratory
owners: [human:nils]
sources:
  - id: parent
    resource: ../rocdown-inline-interpolation.md
    title: Rocdown Markdown @{expr} interpolation
    author: process:cursor
    last_modified: 2026-08-23
  - id: research
    resource: ../../research/rocdown/rocdown-inline-interpolation.md
    title: Inline interpolation in Rocdown Markdown
    author: process:cursor
    last_modified: 2026-08-22
  - id: article-rs
    resource: ../../../crates/rocci-rocdown/src/article.rs
    title: Page classification and Rust article HTML
    author: process:git
    last_modified: 2026-08-22
  - id: docs-rs
    resource: ../../../crates/rocci-rocdown/src/docs.rs
    title: Typed article forest for static widgets
    author: process:git
    last_modified: 2026-08-23
  - id: markdown-rs
    resource: ../../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak to MdNode conversion
    author: process:git
    last_modified: 2026-08-21
  - id: lsp-rs
    resource: ../../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown LSP hover, go-to, and diagnostics
    author: process:git
    last_modified: 2026-08-22
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-22
  - id: sites-ref
    resource: ../../../docs/rocdown/sites.rocdown
    title: Public Rocdown site and page-kind matrix
    author: process:git
    last_modified: 2026-08-22
  - id: hybrid-ref
    resource: ../../../docs/rocdown/hybrid.rocdown
    title: Public hybrid islands page-kind matrix
    author: process:git
    last_modified: 2026-08-22
  - id: lang-ref
    resource: ../../../docs/rocdown/language.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-22
  - id: all-syntax
    resource: ../../../test/AllSyntax.rocdown
    title: Comprehensive Rocdown syntax fixture
    author: process:git
    last_modified: 2026-08-22
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-22
---

# Rocdown `@{expr}` follow-ons after v1

## Purpose and authority

This is the implementation plan for gaps found after executing
[Rocdown Markdown `@{expr}` interpolation](rocdown-inline-interpolation.md).
It does not re-litigate the settled spelling. `{@expr}` and `{{expr}}` stay
rejected. Heading and destination interpolation stay deferred on the parent
Follow-ons list.[^parent][^research]

Do not start a phase until the user asks. Branch name:
`rocdown-inline-interpolation-follow-ons`. Prefer continuing from branch
`rocdown-inline-interpolation` (Phases 1–5 of the parent, not on `main` as of
this record) rather than re-implementing v1 on `main`.[^parent][^language-dev]

## Goal

After the last in-scope phase, v1 `@{expr}` fails closed on the Rust article
path, public hydrate matrices agree with the language page, focused tests lock
the splitter and placement contract, and LSP go-to does not treat the hole
span as a definition target when no binding exists.[^parent][^article-rs][^sites-ref][^lsp-rs]

## Depends on

Parent Phases 1–5: `MdNode::Interpolation`, source-aware `\@{` split,
`Html.text(expr)` with `OriginKind::TextExpression`, hydrate + `RD2303`,
heading/URL placement, docs/AllSyntax, highlight/LSP. Those commits live on
`rocdown-inline-interpolation`. This plan assumes that tree (or an equivalent
merge to `main`).[^parent]

## Out of bound

- Heading or link/image **destination** interpolation (parent follow-on).
- `{@expr}`, `{{expr}}`, bare `{expr}` in Markdown, MDX, a Rust evaluator.
- Changing Rocci `{expr}` in templates.
- Go-to for non-ident exprs (`count.to_str()`, `if …`).
- Rewriting the Text splitter to scan Comrak `value` instead of source.
- Fixing unrelated catalog `RD2101` (`/project/status/` in
  `docs/reference/compatibility.rocdown`).
- Executing fences; `@island`; Datastar signals as the hole.[^parent][^research]

## Constraints that do not move

1. **Markdown owns prose.** Comrak first; do not scan the raw file for `@{`
   before block/inline parse.[^parent][^markdown-rs]
2. **Rust does not run Roc.** Static catalog HTML must not evaluate holes.
   A hole on a Rust-only path is a gate failure, not empty success
   HTML.[^catalog-shell][^article-rs][^docs-rs]
3. **Static `docs/` cannot ship `@{`.** Keep examples in fences or code
   spans. `RD2303` stays the catalog error.[^parent][^lang-ref]
4. **Hover and diagnostics** keep the interp span (`@` through `}`). Only
   go-to targeting changes in Phase 4.[^parent][^lsp-rs]

## Current gaps (on the v1 branch)

These are the review findings this plan closes. They are not claims about
`main`.[^parent]

| Gap | Today on `rocdown-inline-interpolation` | Target |
| --- | --- | --- |
| Rust article HTML | `render_md` emits `text("")` for `Interpolation` | Diagnostic or skip; never silent empty as if the hole were prose |
| Docs forest / search text | `docs.rs` falls through to `text_content()` (`""`) | Same gate; do not evaluate |
| Hydrate matrix | `language.rocdown` lists `@{expr}`; `sites.rocdown` / `hybrid.rocdown` still say only `@component` / `@render` / `@css` / `@roc` | All three matrices name Markdown `@{expr}` |
| Splitter tests | Odd `\@{` and code spans exist; even `\\@{` and `:note` / table-cell rows are thin | Focused tests plus AllSyntax coverage |
| Image alt | Image convert does not split children; alt keeps Comrak glyphs | Lock that v1: no `Html.text(x)` for `![alt @{x}](url)` |
| LSP go-to | Simple ident jumps to `@roc` / `@let`; missing binding jumps to the hole | No location when there is no binding; hover range stays the hole |

[^article-rs][^docs-rs][^sites-ref][^hybrid-ref][^lang-ref][^lsp-rs]

## Phase 1: Loud Rust-path gate

**Bound.** When `render_md` or the docs forest / `md_to_markdown` search path
meets `MdNode::Interpolation`, do not emit unevaluated `@{…}` glyphs and do
not succeed as empty prose. Prefer a diagnostic at the interp span, or omit
the node from static HTML while recording the error. Catalog `RD2303` stays
the site-check failure; this phase is the in-process renderer
contract.[^article-rs][^docs-rs][^catalog-shell]

**Files.** `crates/rocci-rocdown/src/article.rs`,
`crates/rocci-rocdown/src/docs.rs`, focused tests next to existing
interpolation tests on the v1 branch.

**Out of this phase.** Public docs matrices, AllSyntax rows, LSP.

**Tests.** A hydrate page still lowers `Html.text(expr)` on the Roc path. A
static-path `render_document` / docs forest call on a tree that still contains
an `Interpolation` node does not contain `"@{date}"` as HTML **and** reports
the gate (diagnostic or `Err`). Empty `""` alone is not enough.

**Exit.** `cargo test -p rocci-rocdown`. `cargo fmt --all -- --check`.

## Phase 2: Hydrate matrices and knowledge pointers

**Bound.** Update `docs/rocdown/sites.rocdown` and
`docs/rocdown/hybrid.rocdown` so `hydrate` includes Markdown `@{expr}` (same
claim as `language.rocdown`). Keep heading/URL holes **planned**. Do not put
live `@{` into static `docs/` catalog prose. Point the parent plan and
research index at this follow-on; do not mark the parent `stable` or
CI-complete. The crate README stays the implemented contract if it already
names `@{expr}`; only touch it if the site matrices were the last
drift.[^sites-ref][^hybrid-ref][^lang-ref][^parent][^language-dev][^rocdown-readme]

**Files.** `docs/rocdown/sites.rocdown`, `docs/rocdown/hybrid.rocdown`;
knowledge index lines for the parent and this record if not already current.
Optional crate README one-liner if the site matrix is the only drift.

**Out of this phase.** Splitter tests, LSP.

**Exit.** `cargo test -p rocci-rocdown`.
`cargo run -q -p rocci-rocdown-cli -- build docs` when those pages change.
`cargo run -q -p rocci-okf -- check knowledge --profile rocci --format
terminal` if knowledge indexes moved.

## Phase 3: Splitter and placement tests

**Bound.** Lock the source-aware splitter and v1 placement without changing
the grammar.[^markdown-rs][^parent][^all-syntax]

- `\\@{ident}` (even backslashes) is a real hole; `\@{ident}` stays literal.
- `:note {{ Hello @{name}. }}` interpolates; a table cell with `@{x}`
  interpolates.
- `![alt @{x}](./a.png)` does not lower `Html.text(x)`; alt remains the
  Comrak string (glyphs allowed).
- Optional: one entity-adjacent case (`&amp;` next to `@{x}`) so Text
  reconstruction from source cannot silently diverge.

Refresh AllSyntax only for rows that belong in the kitchen sink; prefer
focused tests for the rest. Review fixture diffs.

**Files.** Interpolation tests; optionally `test/AllSyntax.rocdown` and
inspect/Roc fixtures.

**Out of this phase.** LSP go-to.

**Exit.** `cargo test -p rocci-rocdown`. If AllSyntax changed:
`cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown`
reviewed. `cargo fmt --all -- --check`.

## Phase 4: LSP go-to without a hole target

**Bound.** Keep hover range and heading-hole diagnostics on the interp span
(`@` through `}`). If go-to finds an `@roc` / document `@let` binding for a
simple ident, keep that jump. If it does not, return no location — do not
emit `GotoDefinition` at the hole itself. Do not add method/expr
resolution.[^lsp-rs][^parent]

**Files.** `crates/rocci-rocdown/src/lsp.rs`,
`crates/rocci-rocdown/tests/lsp.rs`.

**Exit.** `cargo test -p rocci-rocdown`. `cargo test -p rocci-lsp`.

## Validation (every phase)

```sh
cargo test -p rocci-rocdown
cargo fmt --all -- --check
```

After Phase 2 docs edits, build `docs`. After Phase 4, `cargo test -p
rocci-lsp`. Do not set `ROCCI_REQUIRE_ROC=1` unless a phase proves generated
Roc against the pinned toolchain.[^language-dev]

## Still not this plan

Parent Follow-ons remain: heading and destination interpolation; no `{@` /
`{{` aliases; no static Rust substitution of `@page.meta` under `@{`.[^parent]

[^parent]: v1 contract, five phases, and deferred heading/URL holes.
[^research]: Settled `@{expr}`; rejected MDX / `{{ }}` / Rust evaluator.
[^article-rs]: Rust `render_md` must not evaluate Roc.
[^docs-rs]: Static widget forest must not evaluate Roc exprs.
[^markdown-rs]: Comrak first; Text split is source-aware.
[^lsp-rs]: Hover/go-to/diagnostics on Rocdown documents.
[^rocdown-readme]: Crate README is the implemented language contract.
[^sites-ref]: Site page-kind matrix used by authors of hybrid catalogs.
[^hybrid-ref]: Hybrid page-kind matrix; hydrate row omits `@{expr}` today.
[^lang-ref]: Language page already states current `@{expr}` and planned heading/URL.
[^all-syntax]: Kitchen-sink fixture and inspect lock.
[^catalog-shell]: Rust owns static article HTML; Roc owns authored dynamic regions.
[^language-dev]: Ungram/docs/AllSyntax workflow; build docs when public pages change.
