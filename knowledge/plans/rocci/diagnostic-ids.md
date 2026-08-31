---
type: Implementation Plan
title: Add RCxxxx diagnostic IDs to rocci-template
description: "Give template parse and validate stable RCxxxx codes, rustc-style frames, and consumer pass-through. Do not collapse coded diagnostics to RD1001/RD1002. Do not number Roc or HTTP failures."
tags: [domain/rocci, domain/rocdown, concern/developer-experience, concern/tooling, concern/docs]
status: draft
generated: { by: process:cursor, at: 2026-08-31T13:14:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/diagnostic-ids.md
    title: Rocci template diagnostics should use stable RCxxxx IDs
    author: process:cursor
    last_modified: 2026-08-31
  - id: diagnostic-rs
    resource: ../../../crates/rocci-template/src/diagnostic.rs
    title: Diagnostic and DiagnosticFrame
    author: process:git
    last_modified: 2026-08-16
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Validation error sites
    author: process:git
    last_modified: 2026-08-25
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Parser error helper and recovery sites
    author: process:git
    last_modified: 2026-08-25
  - id: compile-rs
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: compile_err and message substring tests
    author: process:git
    last_modified: 2026-08-30
  - id: lib-rs
    resource: ../../../crates/rocci-template/src/lib.rs
    title: format_diagnostic entry
    author: process:git
    last_modified: 2026-08-25
  - id: lsp
    resource: ../../../crates/rocci-lsp/src/analysis.rs
    title: map_diagnostics code is None
    author: process:git
    last_modified: 2026-08-25
  - id: playground
    resource: ../../../crates/rocci-cli/src/playground_compile.rs
    title: Playground diagnostic JSON
    author: process:git
    last_modified: 2026-08-18
  - id: error-page
    resource: ../../../crates/rocci-cli/src/error_page.rs
    title: Failed-build frames
    author: process:git
    last_modified: 2026-08-22
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: RD1001/RD1002 wrap
    author: process:git
    last_modified: 2026-08-31
  - id: tree-rs
    resource: ../../../crates/rocci-rocdown/src/docs/tree.rs
    title: Article wrap to RD1001/RD1002
    author: process:git
    last_modified: 2026-08-31
  - id: catalog-types
    resource: ../../../crates/rocci-rocdown/src/catalog/types.rs
    title: CatalogDiagnostic.code
    author: process:git
    last_modified: 2026-08-31
  - id: docs-diag
    resource: ../../../docs/reference/diagnostics.rocdown
    title: Public diagnostics stub
    author: process:git
    last_modified: 2026-08-22
  - id: troubleshooting
    resource: ../../../docs/troubleshooting/compile.rocdown
    title: Compile troubleshooting
    author: process:git
    last_modified: 2026-08-22
  - id: decision
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: RDxxxx stay; no ROCS codes
    author: process:cursor
    last_modified: 2026-08-31
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Parser and consumer test order
    author: process:git
    last_modified: 2026-08-22
---

# Add RCxxxx diagnostic IDs to rocci-template

Exploratory. Do not start a phase until the user asks.

## Goal

Authors, tests, LSP, and the playground can cite a stable **`RCxxxx`**
for every template parse and validate diagnostic. Frames look like
`error[RC2009]: …`. Rocdown site reports pass that code through instead
of rewriting it to `RD1001`. The public diagnostics page lists the
catalog.[^research][^docs-diag]

## Out of bound

- Numbering Roc compiler diagnostics or mapping them to `RC*`
- HTTP 404/500, handler dispatch, or `--log-handlers` as compiler IDs
- New `RD*` allocations except keeping `RD1001`/`RD1002` as the uncoded
  fallback
- `ROCS` / `Rocs` codes, rustc `E0xxx`, or `OKFxxxx` on templates[^decision]
- One public ID per parser `self.error` call site
- `rocci explain RC2009` / rustc-style `--explain` CLI
- Splitting Rocdown `@page` / scan / parse into specific `RD10xx`
- Changing diagnostic *messages* except where a test must mention the
  new code
- Ungram, highlighter, or grammar changes[^language-dev]

## Constraints that do not move

1. **Prefix `RC`.** Families: `RC1xxx` parse, `RC2xxx` validate,
   `RC3xxx` reserved. Do not put template IDs in the `RD` series.[^research][^decision]
2. **`code` is optional on the shared type.** Rocdown parse sites may
   keep `None` until a later plan. Template parse and validate always
   set a code.[^diagnostic-rs][^research]
3. **String codes, not a second enum species.** Named `&'static str`
   constants in `rocci_template::codes`, same shape as
   `CatalogDiagnostic.code`.[^catalog-types]
4. **One ID per author-facing condition.** Parse uses the family table
   in Phase 0; validate uses one code per closed condition.[^research]
5. **Messages remain the human text.** Tests may keep a substring
   assert *and* must assert `code` once the site is coded.[^compile-rs]
6. **No reverse crate edges.** `rocci-template` does not learn Rocdown
   codes. Rocdown reads `diagnostic.code` and falls back to
   `RD1001`/`RD1002`.[^site-rs][^decision]
7. **Parser/lowering tests do not invoke Roc.**[^language-dev]

## Phase 0 — Freeze the catalog contract

Bound: record the prefix, families, and first allocations in this
plan (table below). No Rust change required if the table is complete
enough for Phase 1 constructors. If a later phase needs a new
condition, append a code; do not reuse a published meaning.

Parse families (`RC1xxx`):

| Code | Condition |
| --- | --- |
| `RC1001` | expected structure or token |
| `RC1002` | unterminated construct |
| `RC1003` | unknown or unexpected form |
| `RC1004` | removed syntax (rewrite) |
| `RC1005` | HTML / tag mismatch |
| `RC1006` | Datastar action or attribute shape |
| `RC1007` | directive header or placement |

Validate codes (`RC2xxx`):

| Code | Condition |
| --- | --- |
| `RC2001` | duplicate `@context` |
| `RC2002` | duplicate `@init` |
| `RC2003` | `@init` without `@context` |
| `RC2004` | `@context` without `@init` |
| `RC2005` | record handler without `@context` |
| `RC2006` | defaulted field needs a type |
| `RC2007` | handler arity over two |
| `RC2008` | unknown HTTP method |
| `RC2009` | illegal handler pair |
| `RC2010` | empty handler path |
| `RC2011` | duplicate method+path handler |
| `RC2012` | generated Roc handler name collision |
| `RC2013` | unknown fixture target |
| `RC2014` | unknown `@test` fixture name |
| `RC2015` | `@let` after render-producing items |
| `RC2016` | `@css` outside a component body start |
| `RC2017` | `@css` after render-producing items |
| `RC2018` | illegal fixture target name (from resolve) |

Exit: this table is the allocation source; `cargo fmt --all -- --check`
if the phase only touches Markdown.

## Phase 1 — Type, constants, and frames

Bound: add `code: Option<&'static str>` to `Diagnostic` and
`DiagnosticFrame`. Add `rocci_template::codes` with the Phase 0
strings. `Diagnostic::error` / `warning` stay message-only (`code:
None`). Add `Diagnostic::error_code(code, span, message)` and
`warning_code`. Render `error[RC2001]:` when `code` is `Some`; keep
`error:` when `None`. Update frame unit tests.[^diagnostic-rs][^lib-rs]

Out of bound: parser/validate call-site conversion (Phases 2–3).

Exit:

```sh
cargo test -p rocci-template --lib diagnostic
cargo test -p rocci-template
cargo fmt --all -- --check
```

## Phase 2 — Validate emits `RC2xxx`

Bound: every `Diagnostic::error` in `validate.rs` uses
`error_code` and the Phase 0 table. `compile_err` (or a sibling)
exposes codes. Existing message tests also assert the matching
`RCxxxx`.[^validate][^compile-rs]

Exit:

```sh
cargo test -p rocci-template --test compile
cargo test -p rocci-template
cargo fmt --all -- --check
```

## Phase 3 — Parse emits `RC1xxx` families

Bound: `Parser::error` takes a code (or a thin wrapper per family).
Map each recovery site to `RC1001`–`RC1007` from Phase 0. Do not mint
a new public ID per site. Update parse-oriented compile tests to
assert family codes where they already match a message.[^parser][^compile-rs]

Exit: same commands as Phase 2.

## Phase 4 — LSP, playground, error page

Bound: `map_diagnostics` sets `code` to the string when present
(`NumberOrString::String`). Playground JSON includes `"code"`. Error
page / `DiagnosticFrame` already show the code from Phase 1; add a
test that a coded diagnostic appears in the HTML or JSON. No
`codeDescription` URL required in this phase.[^lsp][^playground][^error-page]

Exit:

```sh
cargo test -p rocci-lsp
cargo test -p rocci-cli --lib
cargo test -p rocci-template
cargo fmt --all -- --check
```

## Phase 5 — Rocdown wrap pass-through

Bound: `site.rs` and `docs/tree.rs` (and any other `RD1001`/`RD1002`
wrap of `rocci_template::Diagnostic`) use `diagnostic.code` when
`Some`, otherwise the current bucket. Add one test that a template
validate error inside a compiled page reports `RC2001` (or another
Phase 0 code), not `RD1001`. Uncoded Rocdown `@page` / parse errors
stay `RD1001`/`RD1002`.[^site-rs][^tree-rs][^catalog-types]

Exit:

```sh
cargo test -p rocci-rocdown
cargo fmt --all -- --check
```

## Phase 6 — Public catalog

Bound: replace the "no stable codes" stub on
`docs/reference/diagnostics.rocdown` with the Phase 0 tables and the
rustc-style example line. Point troubleshooting compile at codes
instead of "quote the leading message" as the only handle. Crate
README notes that diagnostics have IDs. Do not invent codes in other
prose.[^docs-diag][^troubleshooting]

Exit:

```sh
cargo test -p rocci-template
cargo fmt --all -- --check
```

Inspect the generated diagnostics page after `cargo run -q -p
rocci-rocdown-cli -- build docs` if that command is used for the docs
change.

## Tests

- Frame render includes `[RC2001]` only when coded.
- Validate fixtures assert both message intent and `RC2xxx`.
- At least one parse fixture per `RC1001`–`RC1007`.
- LSP / playground / wrap tests in Phases 4–5.
- AllSyntax still has no error diagnostics.

## Exit (plan)

Phases 0–6 green on `diagnostic-ids`. Template diagnostics always
carry `RC*`. Rocdown wrap preserves them. Public catalog matches
Phase 0. Do not log complete until CI and Knowledge succeed.

[^research]: Verdict, prefix, families, and wrap pass-through.
[^diagnostic-rs]: Shared `Diagnostic` and rustc-like frames.
[^validate]: Closed validate conditions mapped in Phase 0.
[^parser]: Recovery sites mapped to `RC1xxx` families.
[^compile-rs]: Tests must assert `code` once a site is numbered.
[^lib-rs]: `format_diagnostic` uses `DiagnosticFrame`.
[^lsp]: LSP `code` field is currently `None`.
[^playground]: Playground JSON omits `code`.
[^error-page]: Failed-build HTML reprints frames.
[^site-rs]: Site compile currently buckets to `RD1001`/`RD1002`.
[^tree-rs]: Article wrap uses the same buckets.
[^catalog-types]: Catalog codes are `&'static str`.
[^docs-diag]: Public stub to replace with the Phase 0 tables.
[^troubleshooting]: Compile page still teaches message-only recovery.
[^decision]: Keep `RDxxxx`; do not introduce `ROCS` codes.
[^language-dev]: Parser tests stay Roc-free; consumers after the type change.
