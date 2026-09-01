---
type: Research Report
title: Rocci template diagnostics should use stable RCxxxx IDs
description: "Template diagnostics are message-only while Rocdown and OKF already ship stable codes. Recommend RCxxxx on rocci-template, rustc-style frames, and pass-through instead of collapsing to RD1001/RD1002."
tags: [domain/rocci, domain/rocdown, concern/developer-experience, concern/tooling, concern/docs]
status: draft
generated: { by: process:cursor, at: 2026-08-31T13:14:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/diagnostic-ids.md
    title: Add RCxxxx diagnostic IDs to rocci-template
    author: process:cursor
    last_modified: 2026-08-31
  - id: diagnostic-rs
    resource: ../../../crates/rocci-template/src/diagnostic.rs
    title: Template Diagnostic is span, severity, and message only
    author: process:git
    last_modified: 2026-08-16
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Closed set of module, handler, fixture, and order errors
    author: process:git
    last_modified: 2026-08-25
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Parser recovery pushes Diagnostic::error with English only
    author: process:git
    last_modified: 2026-08-25
  - id: compile-rs
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: compile_err collects messages; tests match English substrings
    author: process:git
    last_modified: 2026-08-30
  - id: lsp
    resource: ../../../crates/rocci-lsp/src/analysis.rs
    title: LSP map_diagnostics sets code to None
    author: process:git
    last_modified: 2026-08-25
  - id: playground
    resource: ../../../crates/rocci-cli/src/playground_compile.rs
    title: Playground JSON has severity, message, and spans only
    author: process:git
    last_modified: 2026-08-18
  - id: error-page
    resource: ../../../crates/rocci-cli/src/error_page.rs
    title: Failed-build HTML renders DiagnosticFrame without a code
    author: process:git
    last_modified: 2026-08-22
  - id: docs-diag
    resource: ../../../docs/reference/diagnostics.rocdown
    title: Public page says no stable template ID catalog yet
    author: process:git
    last_modified: 2026-08-22
  - id: troubleshooting
    resource: ../../../docs/troubleshooting/index.rocdown
    title: Quote the exact error; RD codes belong to Rocdown
    author: process:git
    last_modified: 2026-08-22
  - id: catalog-types
    resource: ../../../crates/rocci-rocdown/src/catalog/types.rs
    title: CatalogDiagnostic requires a &'static str code
    author: process:git
    last_modified: 2026-08-31
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: Site compile wraps every template diagnostic as RD1001 or RD1002
    author: process:git
    last_modified: 2026-08-31
  - id: tree-rs
    resource: ../../../crates/rocci-rocdown/src/docs/tree.rs
    title: Article parse wrap also buckets to RD1001 or RD1002
    author: process:git
    last_modified: 2026-08-31
  - id: page-rs
    resource: ../../../crates/rocci-rocdown/src/page.rs
    title: Rocdown @page parse uses template Diagnostic without a code
    author: process:git
    last_modified: 2026-08-21
  - id: parse-rs
    resource: ../../../crates/rocci-rocdown/src/parse.rs
    title: Rocdown document parse emits uncoded template diagnostics
    author: process:git
    last_modified: 2026-08-25
  - id: decision
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Frozen RDxxxx families; no ROCS codes; OKFxxxx stays with OKF
    author: process:cursor
    last_modified: 2026-08-31
  - id: priority-1
    resource: ../../reference/priority-1-review.md
    title: OKF4004–OKF4008 lifecycle and provenance codes
    author: process:okf-phase-6
    last_modified: 2026-08-31
---

# Rocci template diagnostics should use stable RCxxxx IDs

## Verdict

**Yes.** The `.rocci` template compiler should publish stable diagnostic
IDs. Rocdown and OKF already do. The template type, CLI frames, LSP,
playground JSON, and public docs do not.[^diagnostic-rs][^docs-diag][^decision]

This is not a claim that every product failure needs a code. Roc type
errors stay Roc's. HTTP 404/500 stay status pages. The missing catalog is
the template parse/validate surface that authors already see as `error:
<message>`.[^docs-diag][^error-page]

Implementation: [add RCxxxx IDs](/plans/rocci/diagnostic-ids.md).[^plan]

## What ships today

Three products, three contracts.

| Surface | ID? | Shape |
| --- | --- | --- |
| `rocci-template::Diagnostic` | No | `span`, `severity`, `message`[^diagnostic-rs] |
| Rocdown `CatalogDiagnostic` | Yes | `RDxxxx` `&'static str`[^catalog-types][^decision] |
| OKF / okmate | Yes | `OKFxxxx` families (`OKF4004`–`OKF4008` and others)[^priority-1] |

Template emit sites are English-only. The parser has on the order of
eighty recovery messages; validate has a closed set of about seventeen
conditions (duplicate `@context` / `@init`, illegal handler pairs,
unknown fixtures, `@let` / `@css` order). Lowering does not push
diagnostics.[^parser][^validate]

Consumers drop identity as well:

- CLI `DiagnosticFrame` renders rustc-like `error: …` plus caret, with
  no `[CODE]`.[^diagnostic-rs]
- LSP `map_diagnostics` sets `code: None` and `source: "rocci"`.[^lsp]
- Playground compile JSON has severity, message, and spans only.[^playground]
- Failed-build HTML reprints the same frames.[^error-page]
- Compile tests collect `message` and assert English
  substrings.[^compile-rs]

The public diagnostics page already states the gap: no stable template
ID catalog; do not invent codes in prose; quote the leading message.
Rocdown `RD2101` and friends are documented as a different
product.[^docs-diag][^troubleshooting]

## Rocdown already has IDs — and a bucket hole

The product-boundary decision froze `RDxxxx` and forbade `ROCS`
codes.[^decision]

Allocated families in tree today include `RD1xxx` parse/source,
`RD20xx` identity/routes, `RD21xx` links/assets, `RD22xx` navigation,
`RD23xx` unsupported static features, `RD24xx`–`RD26xx` documentation
components, and `RD2701`. About thirty-seven distinct `RDxxxx` strings
appear in `rocci-rocdown`.[^catalog-types][^decision]

`RD1001` / `RD1002` are not a parse taxonomy. Site compile and article
tree walk take every `rocci_template::Diagnostic` and assign
`RD1001` for errors and `RD1002` for warnings, regardless of
cause.[^site-rs][^tree-rs]

Rocdown document parse (`@page`, scan, parse) also uses the template
`Diagnostic` type and therefore has no per-condition code until that
wrap.[^page-rs][^parse-rs]

So Rocdown's catalog path is coded and searchable; the shared parse
type is not. Adding template IDs without changing the wrap would still
leave `.rocdown` site reports saying `RD1001` for a duplicate
`@context` inside a template island.

## Why IDs, not better sentences

A stable ID is a public key. The message can be polished; the ID
cannot change meaning.

Jobs that English-only diagnostics fail:

1. **Docs and issues.** The published contract today is "quote the
   leading message." That breaks when a sentence is edited.[^docs-diag]
2. **Tests.** `compile_err` matching `"did you mean \`@if\`"` couples
   CI to copy.[^compile-rs]
3. **Editors and playground.** LSP `Diagnostic.code` and
   `codeDescription` are empty; machines cannot link
   `RC2009` to an explain page.[^lsp][^playground]
4. **Search and allowlists.** Rocdown tests already assert `d.code ==
   "RD2007"`. Template tests cannot.[^catalog-types][^compile-rs]
5. **Cross-product reports.** A site build that collapses every
   template failure to `RD1001` hides the author-facing
   condition.[^site-rs]

This matches how rustc (`error[E0308]:`) and TypeScript (`error TS2345:`)
treat codes: identity for explain indexes and issue search; prose for
humans. Rocdown's `{code} {kind} {path}: {message}` line is the same
idea with a product prefix.[^catalog-types]

## What should not get a Rocci code

- **Roc compiler diagnostics.** Those belong to `roc`. Map spans
  through source maps; do not invent `RC` numbers for `TYPE MISMATCH`.
- **HTTP 404 / 500 and handler dispatch.** Status pages and
  `--log-handlers` stay runtime, not the compiler catalog.[^error-page]
- **Rocdown catalog conditions.** Keep allocating `RDxxxx`. Do not
  put template IDs on broken links or `RD2205`.[^decision]
- **OKF lifecycle / provenance.** `OKFxxxx` stays with OKF.[^priority-1]
- **One code per parser call site.** Eighty recovery strings are not
  eighty public IDs. Group by author-facing condition.

## Recommended contract

**Prefix `RCxxxx`.** Parallel to `RDxxxx` and `OKFxxxx`. Short, not
`ROCS`, and not rustc `E0xxx` (those collide in mixed editor
surfaces).[^decision]

**Families**

| Series | Owner | Typical conditions |
| --- | --- | --- |
| `RC1xxx` | parse / recovery | expected token, unterminated, unknown form, removed syntax, HTML mismatch |
| `RC2xxx` | validate | duplicate context/init, illegal pair, fixture/test names, `@let`/`@css` order |
| `RC3xxx` | reserved | lowering or resolve if those later emit |

Validation is the better first catalog: a closed set, already tested by
message, and the conditions authors file issues about (`@on` was
removed, illegal `@get:command`, unknown fixture).[^validate]

Parse should use a **small family set**, not one ID per `self.error`
call. A workable first cut:

- `RC1001` expected structure / token
- `RC1002` unterminated construct
- `RC1003` unknown or unexpected form
- `RC1004` removed syntax (rewrite, not deprecation)
- `RC1005` HTML / tag mismatch
- `RC1006` Datastar action / attribute shape
- `RC1007` directive header / placement

Messages stay specific. Codes stay searchable.

**Render** rustc-style on template frames: `error[RC2009]: illegal
handler pair…`. Keep the current caret block. When `code` is absent
(Rocdown-owned parse that has not been numbered yet), keep today's
`error:` line so this change does not force every `page.rs` call
site in the same commit.[^diagnostic-rs][^page-rs]

**Type.** Add `code: Option<&'static str>` on `Diagnostic`, matching
Rocdown's string codes rather than a second enum species. Named
constants in a `codes` module (`RC2001`, …) keep allocations in one
place.[^diagnostic-rs][^catalog-types]

**Rocdown wrap.** If a wrapped diagnostic already has a code, use it
(`RC2001` or a future `RD1xxx`). Assign `RD1001` / `RD1002` only when
`code` is `None`. Do not retarget template IDs into the `RD` series,
and do not invent `ROCS` codes.[^site-rs][^decision]

**Rocdown document-parse taxonomy** (splitting `RD1001` into specific
`RD10xx` for `@page` / scan) is a follow-on. This research does not
require it for the template catalog to be useful.

## Recommendation

Ship `RCxxxx` on `rocci-template` parse and validate, thread the field
through CLI frames, LSP, playground JSON, and error pages, then stop
collapsing coded diagnostics to `RD1001`/`RD1002`. Publish the catalog
on the existing diagnostics page instead of "quote the leading
message."[^docs-diag][^plan]

Do not wait for a perfect parse taxonomy, an `--explain` CLI, or
Rocdown parse IDs. Those can follow once the field and prefix exist.

[^plan]: Phased type, validate, parse families, consumers, wrap, and catalog.
[^diagnostic-rs]: `Diagnostic` has no `code`; frames render `error: {message}`.
[^validate]: Closed module, handler, fixture, test, and `@let`/`@css` order errors.
[^parser]: `Parser::error` pushes English-only `Diagnostic::error`.
[^compile-rs]: `compile_err` maps diagnostics to `message` strings.
[^lsp]: `map_diagnostics` sets `code: None` and `source: "rocci"`.
[^playground]: JSON objects are severity, message, and byte/UTF-16 spans.
[^error-page]: Failed-build HTML reprints `DiagnosticFrame` text.
[^docs-diag]: Status line: no stable template ID catalog; do not invent codes.
[^troubleshooting]: Quote the exact error; `RDxxxx` belong on Rocdown pages.
[^catalog-types]: `CatalogDiagnostic.code` is a required `&'static str`.
[^site-rs]: Every compiled-page diagnostic becomes `RD1001` or `RD1002`.
[^tree-rs]: Article-tree wrap uses the same two bucket codes.
[^page-rs]: `@page` field errors use uncoded template `Diagnostic`.
[^parse-rs]: Document parse pushes uncoded template diagnostics.
[^decision]: Frozen `RDxxxx` families; no `ROCS` codes; `OKFxxxx` stays with OKF.
[^priority-1]: `OKF4004`–`OKF4008` are stable lifecycle and provenance codes.
